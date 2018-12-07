use actix::{self, Actor, Addr, Arbiter, AsyncContext, Context, StreamHandler, System};
use actix::prelude::*;
use chrono::{Duration, Utc};
use clacks_crypto::symm::AuthKey;
use clacks_mtproto::{AnyBoxedSerialize, BoxedDeserialize, ConstructorNumber, mtproto};
use clacks_transport::{AppId, Session, TelegramCodec, session};
use failure::Error;
use futures::{self, Future, IntoFuture, Sink, Stream, future, stream};
use futures::sync::oneshot;
use slog::Logger;
use std::io;
use std::collections::BTreeMap;
use std::marker::PhantomData;

use super::Result;


macro_rules! async_handler {
    (impl ($($generics:tt)*) $input_ty:ty => $actor_ty:ty |$this:ident, $input:ident, $ctx:ident| -> $output_ty:ty $output:block) => {
        impl <$($generics)*> Handler<$input_ty> for $actor_ty {
            type Result = Box<Future<Item = $output_ty, Error = Error>>;

            fn handle(&mut self, $input: $input_ty, $ctx: &mut Self::Context) -> Self::Result {
                let fut = match (move || {
                    let $this = self;
                    $output
                })() {
                    Ok(f) => tokio_async_await::compat::backward::Compat::new(f),
                    Err(e) => return Box::new(future::err(e)),
                };
                Box::new(fut)
            }
        }
    };
}

type Responder = oneshot::Sender<Result<mtproto::TLObject>>;

pub struct RpcClientActor {
    session: Session,
    tg_tx: Option<actix::io::FramedWrite<Box<tokio_io::AsyncWrite>, TelegramCodec>>,
    tg_rx: actix::SpawnHandle,
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
    #[fail(display = "rpc error")]
    UpstreamRpcError(mtproto::RpcError),
}

impl RpcClientActor {
    pub fn from_context<S>(ctx: &mut Context<Self>, log: Logger, app_id: AppId, stream: S) -> Self
        where S: tokio_io::AsyncRead + tokio_io::AsyncWrite + 'static,
    {
        let session = Session::new(app_id);
        let (tg_rx, tg_tx) = stream.split();
        let tg_rx = ctx.add_stream(tokio_codec::FramedRead::new(tg_rx, TelegramCodec::new()));
        let tg_tx: Box<tokio_io::AsyncWrite> = Box::new(tg_tx);
        let tg_tx = actix::io::FramedWrite::new(tg_tx, TelegramCodec::new(), ctx);
        RpcClientActor {
            session, tg_rx,
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
}

impl Actor for RpcClientActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {

    }
}

impl actix::io::WriteHandler<io::Error> for RpcClientActor {

}

impl StreamHandler<Vec<u8>, io::Error> for RpcClientActor {
    fn handle(&mut self, vec: Vec<u8>, ctx: &mut Self::Context) {
        let message = match self.session.process_message(&vec) {
            Ok(m) => m,
            Err(e) => return,
        };
        let payload = mtproto::TLObject::boxed_deserialized_from_bytes(&message.payload);
        if message.was_encrypted {
            self.maybe_ack(message.seq_no, message.message_id);
            match payload.and_then(|o| self.scan_replies(ctx, o)) {
                Ok(()) => (),
                Err(e) => {

                },
            }
        } else if self.pending_rpcs.len() != 1 {
            // XXX: can't dispatch this message
        } else {
            let key = *self.pending_rpcs.keys().next().unwrap();
            let (_, sender) = self.pending_rpcs.remove(&key).unwrap();
            let _ = sender.send(payload);
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

async_handler!(impl() SendMessage => RpcClientActor |this, message, ctx| -> mtproto::TLObject {
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
        where M: ::clacks_mtproto::Function<Reply = R>,
    {
        CallFunction {
            inner: SendMessage::encrypted(query),
            _dummy: PhantomData,
        }
    }

    pub fn plain<M>(query: M) -> Self
        where M: ::clacks_mtproto::Function<Reply = R>,
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

async_handler!(impl(R: BoxedDeserialize + AnyBoxedSerialize) CallFunction<R> => RpcClientActor |this, message, ctx| -> R {
    let fut = <RpcClientActor as Handler<SendMessage>>::handle(this, message.inner, ctx);
    Ok(async move {
        let reply: mtproto::TLObject = await!(fut)?;
        match reply.downcast::<R>() {
            Ok(r) => Ok(r),
            Err(e) => {
                println!("got: {:?}", e);
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

async_handler!(impl() BindAuthKey => RpcClientActor |this, bind, ctx| -> () {
    let BindAuthKey { perm_key, temp_key, temp_key_duration, salt } = bind;
    this.session.adopt_key(temp_key);
    this.session.add_server_salts(::std::iter::once(salt));
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
        let mtproto::manual::MessageContainer::MsgContainer(mc) = mc;
        for msg in mc.messages.0 {
            this.maybe_ack(Some(msg.seqno), msg.msg_id);
            this.scan_replies(ctx, msg.body.0)?;
        }
        Ok(())
    }

    (this, ctx, rpc: mtproto::manual::RpcResult) {
        let mtproto::manual::RpcResult::RpcResult(rpc) = rpc;
        let (_, replier) = this.get_replier(rpc.req_msg_id)?;
        let result = match rpc.result.downcast::<mtproto::RpcError>() {
            Ok(err) => Err(RpcError::UpstreamRpcError(err).into()),
            Err(obj) => Ok(obj),
        };
        let _ = replier.send(result);
        Ok(())
    }

    (this, ctx, pong: mtproto::Pong) {
        let (_, replier) = this.get_replier(*pong.msg_id())?;
        let _ = replier.send(Ok(mtproto::TLObject::new(pong)));
        Ok(())
    }

    (this, ctx, salts: mtproto::FutureSalts) {
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

pub struct Unhandled(pub mtproto::TLObject);
impl Message for Unhandled {
    type Result = ();
}

#[derive(Default)]
pub struct EventDelegates {
    pub unhandled: Option<Recipient<Unhandled>>,
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
