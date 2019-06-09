use actix::{self, Actor, AsyncContext, Context, StreamHandler};
use actix::prelude::*;
use chrono::Duration;
use clacks_crypto::symm::AuthKey;
use clacks_mtproto::{AnyBoxedSerialize, BoxedDeserialize, ConstructorNumber, IntoBoxed, mtproto};
use clacks_transport::{AppId, Session, TelegramCodec, session};
use failure::Error;
use futures::{self, IntoFuture};
use futures::sync::oneshot;
use slog::Logger;
use std::io;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::marker::PhantomData;

use super::Result;

#[macro_export]
macro_rules! async_handler {
    (fn handle ($($generics:tt)*) ($this:ident: $actor_ty:ty, $input:ident: $input_ty:ty, $ctx:ident) -> $output_ty:ty $output:block) => {
        impl <$($generics)*> Handler<$input_ty> for $actor_ty {
            type Result = Box<dyn futures::Future<Item = $output_ty, Error = failure::Error>>;

            fn handle(&mut self, $input: $input_ty, $ctx: &mut Self::Context) -> Self::Result {
                let fut = match (move || {
                    let $this = self;
                    $output
                })() {
                    Ok(f) => tokio_async_await::compat::backward::Compat::new(f),
                    Err(e) => return Box::new(futures::future::err(e)),
                };
                Box::new(fut)
            }
        }
    };
}

type Responder = oneshot::Sender<Result<mtproto::TLObject>>;

pub struct RpcClientActor {
    log: Logger,
    session: Session,
    tg_tx: Option<actix::io::FramedWrite<Box<tokio_io::AsyncWrite>, TelegramCodec>>,
    delegates: EventDelegates,
    pending_rpcs: BTreeMap<i64, (ConstructorNumber, Responder)>,
}

#[derive(Debug, Fail)]
pub enum RpcError {
    #[fail(display = "")]
    ConnectionClosed,
    #[fail(display = "")]
    DuplicateMessageId,
    #[fail(display = "")]
    BadReplyType,
}

#[derive(Debug, Fail)]
#[fail(display = "rpc error")]
pub struct UpstreamRpcError(mtproto::RpcError);

impl RpcClientActor {
    pub fn from_context<S>(ctx: &mut Context<Self>, log: Logger, app_id: AppId, stream: S) -> Self
        where S: tokio_io::AsyncRead + tokio_io::AsyncWrite + 'static,
    {
        let session = Session::new(log.new(o!("subsystem" => "session")), app_id);
        let (tg_rx, tg_tx) = stream.split();
        ctx.add_stream(tokio_codec::FramedRead::new(tg_rx, TelegramCodec::new()));
        let tg_tx: Box<tokio_io::AsyncWrite> = Box::new(tg_tx);
        let tg_tx = actix::io::FramedWrite::new(tg_tx, TelegramCodec::new(), ctx);
        RpcClientActor {
            log, session,
            tg_tx: Some(tg_tx),
            delegates: Default::default(),
            pending_rpcs: BTreeMap::new(),
        }
    }

    fn maybe_ack(&mut self, seq_no: Option<i32>, message_id: i64) {
        match seq_no {
            Some(s) if s & 1 != 0 => self.session.ack_id(message_id),
            _ => (),
        }
    }

    fn get_replier(&mut self, msg_id: i64) -> Result<(ConstructorNumber, Responder)> {
        self.pending_rpcs.remove(&msg_id)
            .ok_or_else(|| format_err!("no matching rpc for {}", msg_id))
    }

    fn handle_bytes(&mut self, vec: Vec<u8>, ctx: &mut Context<Self>) -> Result<()> {
        let message = self.session.process_message(&vec)?;
        if message.was_encrypted {
            self.maybe_ack(message.seq_no, message.message_id);
            self.scan_replies(ctx, message.payload)?;
        } else if self.pending_rpcs.len() != 1 {
            // XXX: can't dispatch this message
        } else {
            let (_, sender) = self.pending_rpcs.keys()
                .next().cloned()
                .and_then(|key| self.pending_rpcs.remove(&key))
                .ok_or_else(|| format_err!("can't figure out who to reply to"))?;
            // XXX: figure out how to plumb through RPC errors
            let _ = sender.send(Ok(message.payload));
        }
        Ok(())
    }
}

impl Actor for RpcClientActor {
    type Context = Context<Self>;
}

impl actix::io::WriteHandler<io::Error> for RpcClientActor {

}

impl StreamHandler<Vec<u8>, io::Error> for RpcClientActor {
    fn handle(&mut self, vec: Vec<u8>, ctx: &mut Self::Context) {
        match self.handle_bytes(vec, ctx) {
            Ok(()) => {}
            Err(e) => {
                crit!(self.log, "error handling wire data"; "err" => ?e);
            }
        }
    }
}

pub struct SendMessage {
    builder: session::EitherMessageBuilder,
}

impl SendMessage {
    pub fn encrypted<M>(query: M) -> Self
        where M: AnyBoxedSerialize,
    {
        SendMessage {
            builder: session::EitherMessageBuilder::encrypted(query),
        }
    }

    pub fn plain<M>(query: M) -> Self
        where M: AnyBoxedSerialize,
    {
        SendMessage {
            builder: session::EitherMessageBuilder::plain(query),
        }
    }
}

impl Message for SendMessage {
    type Result = Result<mtproto::TLObject>;
}

async_handler!(fn handle()(this: RpcClientActor, message: SendMessage, _ctx) -> mtproto::TLObject {
    use std::collections::btree_map::Entry::*;
    let tg_tx = this.tg_tx.as_mut().ok_or(RpcError::ConnectionClosed)?;
    let message = message.builder;
    let (tx, rx) = oneshot::channel();
    match this.pending_rpcs.entry(message.message_id()) {
        Vacant(e) => {
            e.insert((message.constructor(), tx));
        },
        Occupied(_) => bail!(RpcError::DuplicateMessageId),
    }
    let serialized = this.session.serialize_message(message)?;
    tg_tx.write(serialized);
    Ok(async { Ok(await!(rx)??) })
});

pub struct CallFunction<R> {
    inner: SendMessage,
    _dummy: PhantomData<fn() -> R>,
}

impl<R> CallFunction<R>
    where R: BoxedDeserialize + AnyBoxedSerialize,
{
    pub fn encrypted<M>(query: M) -> Self
        where M: clacks_mtproto::Function<Reply = R>,
    {
        CallFunction {
            inner: SendMessage::encrypted(query),
            _dummy: PhantomData,
        }
    }

    pub fn plain<M>(query: M) -> Self
        where M: clacks_mtproto::Function<Reply = R>,
    {
        CallFunction {
            inner: SendMessage::plain(query),
            _dummy: PhantomData,
        }
    }
}

impl<R: 'static> Message for CallFunction<R> {
    type Result = Result<R>;
}

async_handler!(fn handle(R: BoxedDeserialize + AnyBoxedSerialize)(this: RpcClientActor, message: CallFunction<R>, ctx) -> R {
    let log = this.log.clone();
    let fut = <RpcClientActor as Handler<SendMessage>>::handle(this, message.inner, ctx);
    Ok(async move {
        let reply: mtproto::TLObject = await!(fut)?;
        match reply.downcast::<R>() {
            Ok(r) => Ok(r),
            Err(e) => {
                error!(log, "got: {:?}", e);
                Err(RpcError::BadReplyType.into())
            },
        }
    })
});

pub(crate) struct BindAuthKey {
    pub perm_key: AuthKey,
    pub temp_key: AuthKey,
    pub temp_key_duration: Duration,
    pub salt: mtproto::FutureSalt,
}

impl Message for BindAuthKey {
    type Result = Result<()>;
}

async_handler!(fn handle()(this: RpcClientActor, bind: BindAuthKey, ctx) -> () {
    let BindAuthKey { perm_key, temp_key, temp_key_duration, salt } = bind;
    this.session.adopt_key(temp_key);
    this.session.add_server_salts(std::iter::once(salt));
    let addr = ctx.address();
    let message = this.session.bind_auth_key(perm_key, temp_key_duration)?;
    let reply_fut = addr.send(CallFunction::<mtproto::Bool> {
        inner: SendMessage { builder: message.lift() },
        _dummy: PhantomData,
    });
    Ok(async {
        let bound: bool = await!(reply_fut)??.into();
        ensure!(bound, format_err!("confusing Ok(false) from bind_auth_key"));
        Ok(())
    })
});

struct Scan<'a, S>(&'a mut RpcClientActor, &'a mut Context<RpcClientActor>, S);

macro_rules! scan_type_impl {
    (@block_phase(($self:ident, $ctx:ident, $name:ident: $ty:ty) $block:block $($rin:tt)*) $($rout:tt)*) => {
        impl<'a> Scan<'a, $ty> {
            fn scan(self) -> Result<()> {
                let Scan($self, $ctx, $name) = self;
                $block
            }
        }

        scan_type_impl! { @block_phase($($rin)*) @out($ty) $($rout)* }
    };

    (@block_phase() $($rest:tt)*) => {
        impl RpcClientActor {
            fn scan_replies(&mut self, ctx: &mut Context<Self>, mut obj: mtproto::TLObject) -> Result<()> {
                scan_type_impl! { @obj(self, ctx, obj) $($rest)* }
            }
        }
    };

    (@obj($self:ident, $ctx:ident, $obj:ident) @out($ty:ty) $($rest:tt)*) => {
        $obj = match $obj.downcast::<$ty>() {
            Ok(d) => return Scan::<$ty>($self, $ctx, d).scan(),
            Err(o) => o,
        };
        scan_type_impl! { @obj($self, $ctx, $obj) $($rest)* }
    };

    (@obj($self:ident, $ctx:ident, $obj:ident)) => {
        if let Some(ref recipient) = $self.delegates.unhandled {
            recipient.do_send(Unhandled($obj)).map_err(clear_send_error)?;
        }
        Ok(())
    };
}

macro_rules! scan_type {
    ($($everything:tt)*) => {
        scan_type_impl! { @block_phase($($everything)*) }
    }
}

scan_type! {
    (this, ctx, mc: mtproto::manual::MessageContainer) {
        for msg in mc.only().messages.0 {
            this.maybe_ack(Some(msg.seqno), msg.msg_id);
            this.scan_replies(ctx, msg.body.0)?;
        }
        Ok(())
    }

    (this, _ctx, rpc: mtproto::manual::RpcResult) {
        let rpc = rpc.only();
        let (_, replier) = this.get_replier(rpc.req_msg_id)?;
        let result = match rpc.result.downcast::<mtproto::RpcError>() {
            Ok(err) => Err(UpstreamRpcError(err).into()),
            Err(obj) => Ok(obj),
        };
        let _ = replier.send(result);
        Ok(())
    }

    (this, _ctx, pong: mtproto::Pong) {
        let (_, replier) = this.get_replier(*pong.msg_id())?;
        let _ = replier.send(Ok(mtproto::TLObject::new(pong)));
        Ok(())
    }

    (this, _ctx, salts: mtproto::FutureSalts) {
        let (_, replier) = this.get_replier(*salts.req_msg_id())?;
        let _ = replier.send(Ok(mtproto::TLObject::new(salts)));
        Ok(())
    }
}

fn clear_send_error<T>(err: SendError<T>) -> Error {
    match err {
        SendError::Full(_) => format_err!("inbox full"),
        SendError::Closed(_) => format_err!("inbox closed"),
    }
}

#[derive(Debug, Clone, Default)]
pub struct InitConnection {
    pub device_model: Option<Cow<'static, str>>,
    pub system_version: Option<Cow<'static, str>>,
    pub app_version: Option<Cow<'static, str>>,
    pub lang_code: Option<Cow<'static, str>>,
    pub system_lang_code: Option<Cow<'static, str>>,
    pub lang_pack: Option<Cow<'static, str>>,
}

impl Message for InitConnection {
    type Result = Result<mtproto::Config>;
}

fn unwrap_cow_option(cow_opt: &Option<Cow<'static, str>>, default: &str) -> String {
    cow_opt.as_ref()
        .map(|c| c.as_ref())
        .unwrap_or(default)
        .to_owned()
}

async_handler!(fn handle()(this: RpcClientActor, req: InitConnection, ctx) -> mtproto::Config {
    let call = mtproto::rpc::InvokeWithLayer {
        layer: mtproto::LAYER,
        query: mtproto::rpc::InitConnection {
            api_id: this.session.app_id().api_id,
            device_model: unwrap_cow_option(&req.device_model, "test"),
            system_version: unwrap_cow_option(&req.system_version, "test"),
            app_version: unwrap_cow_option(&req.app_version, "test"),
            lang_code: unwrap_cow_option(&req.lang_code, "en"),
            system_lang_code: unwrap_cow_option(&req.system_lang_code, "en"),
            lang_pack: unwrap_cow_option(&req.lang_pack, ""),
            proxy: None,
            query: mtproto::rpc::help::GetConfig,
        },
    };
    let addr = ctx.address();
    Ok(async move {
        let res = await!(addr.send(CallFunction::encrypted(call)))??;
        Ok(res)
    })
});

#[derive(Debug, Clone)]
pub struct Unhandled(pub mtproto::TLObject);
impl Message for Unhandled {
    type Result = ();
}

#[derive(Debug, Clone)]
pub struct ReadAuthCode {
    pub phone_number: String,
    pub type_: mtproto::auth::SentCodeType,
    pub last_error: Option<mtproto::rpc_error::RpcError>,
}
#[derive(Debug, Clone)]
pub enum AuthCodeReply {
    Code(String),
    Resend,
    Cancel,
}
impl Message for ReadAuthCode {
    type Result = Result<AuthCodeReply>;
}

#[derive(Default)]
pub struct EventDelegates {
    pub unhandled: Option<Recipient<Unhandled>>,
    pub read_auth_code: Option<Recipient<ReadAuthCode>>,
}

pub struct SetDelegates {
    pub delegates: EventDelegates,
}

impl Message for SetDelegates {
    type Result = ();
}

impl Handler<SetDelegates> for RpcClientActor {
    type Result = ();

    fn handle(&mut self, delegates: SetDelegates, _: &mut Self::Context) {
        self.delegates = delegates.delegates;
    }
}

#[derive(Debug, Clone)]
struct SendToDelegate<T>(T);

impl Message for SendToDelegate<ReadAuthCode> {
    type Result = Result<AuthCodeReply>;
}

impl Handler<SendToDelegate<ReadAuthCode>> for RpcClientActor {
    type Result = ResponseActFuture<Self, AuthCodeReply, failure::Error>;

    fn handle(&mut self, to_send: SendToDelegate<ReadAuthCode>, _: &mut Self::Context) -> Self::Result {
        match &self.delegates.read_auth_code {
            Some(addr) => Box::new({
                actix::fut::wrap_future(addr.send(to_send.0))
                    .then(|rr, _, _| actix::fut::wrap_future(rr.unwrap_or_else(|e| Err(e)?).into_future()))
            }),
            None => Box::new({
                actix::fut::wrap_future(Err(format_err!("no delegate")).into_future())
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SendAuthCode {
    pub phone_number: String,
}

impl Message for SendAuthCode {
    type Result = Result<mtproto::auth::Authorization>;
}

#[derive(Debug, Fail)]
pub enum AuthError {
    #[fail(display = "")]
    Canceled(bool),
}

#[derive(Debug, Fail)]
#[fail(display = "")]
pub struct AuthRedirectTo(pub u32);

async_handler!(fn handle()(this: RpcClientActor, req: SendAuthCode, ctx) -> mtproto::auth::Authorization {
    let SendAuthCode { phone_number } = req;
    let app_id = this.session.app_id();
    let send_code = mtproto::rpc::auth::SendCode {
        api_hash: app_id.api_hash.clone(),
        api_id: app_id.api_id,
        phone_number: phone_number.clone(),
        settings: mtproto::code_settings::CodeSettings {
            allow_flashcall: false,
            current_number: false,
            app_hash_persistent: false,
            app_hash: None,
        }.into_boxed(),
    };
    let addr = ctx.address();
    Ok(async move {
        let mut sent = match await!(addr.send(CallFunction::encrypted(send_code)))? {
            Ok(mtproto::auth::SentCode::SentCode(sent)) => sent,
            Err(e) => {
                let e = e.downcast::<UpstreamRpcError>()?;
                let mtproto::RpcError::RpcError(ref e_inner) = &e.0;
                let dc_opt = if e_inner.error_code == 303 { Some(e_inner.error_message.as_str()) } else { None }
                    .and_then(|s| s.chars().next_back())
                    .and_then(|c| c.to_digit(10));
                match dc_opt {
                    Some(dc)  => Err(AuthRedirectTo(dc))?,
                    _ => Err(e)?,
                }
            },
        };
        let mut last_error = None;
        loop {
            use self::AuthCodeReply::*;
            match await!(addr.send(SendToDelegate(ReadAuthCode {
                phone_number: phone_number.clone(),
                type_: sent.type_.clone(),
                last_error: last_error.take(),
            })))?? {
                Code(phone_code) => {
                    match await!(addr.send(CallFunction::encrypted(mtproto::rpc::auth::SignIn {
                        phone_code,
                        phone_number: phone_number.clone(),
                        phone_code_hash: sent.phone_code_hash.clone(),
                    })))? {
                        Ok(auth) => return Ok(auth),
                        Err(e) => {
                            last_error = Some(e.downcast::<UpstreamRpcError>()?.0.only());
                        }
                    }
                }
                Resend => {
                    let reply = await!(addr.send(CallFunction::encrypted(mtproto::rpc::auth::ResendCode {
                        phone_number: phone_number.clone(),
                        phone_code_hash: sent.phone_code_hash.clone(),
                    })))??.only();
                    sent = reply;
                }
                Cancel => {
                    let canceled = await!(addr.send(CallFunction::encrypted(mtproto::rpc::auth::CancelCode {
                        phone_number: phone_number.clone(),
                        phone_code_hash: sent.phone_code_hash.clone(),
                    })))??;
                    Err(AuthError::Canceled(canceled.into()))?;
                }
            }
        }
    })
});
