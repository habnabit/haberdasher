//use actix::{self, Actor, Addr, Arbiter, AsyncContext, Context, StreamHandler, System};
use actix::prelude::*;
use clacks_mtproto::mtproto;
use clacks_rpc::client::{self, CallFunction, RpcClientActor, SendMessage};
use failure::Error;
use futures::{Future, IntoFuture, future};
use slog::Logger;
use std::collections::BTreeMap;
use std::io;
use std::sync::Arc;
use tokio_codec::{FramedRead, LinesCodec};

use super::config::{Entry, TelegramDatacenter, TelegramServersV1, UserAuthKey, UserAuthKeyV1, UserData, UserDataV1};

type Result<T> = std::result::Result<T, failure::Error>;

fn wrap_async<A, T, E, F>(f: F) -> impl ActorFuture<Item = T, Error = E, Actor = A>
    where A: Actor,
          F: std::future::Future<Output = std::result::Result<T, E>>,
{
    actix::fut::wrap_future(tokio_async_await::compat::backward::Compat::new(f))
}

#[derive(Debug, Clone, Fail)]
#[fail(display = "")]
pub struct NoServers;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct UserDc {
    phone_number: String,
    dc: u32,
    primary: bool,
}

pub struct TelegramManagerActor {
    log: Logger,
    tree: Arc<sled::Tree>,
    app_id: clacks_transport::AppId,
    connections: BTreeMap<UserDc, Addr<RpcClientActor>>,
}

impl TelegramManagerActor {
    pub fn new(log: Logger, tree: Arc<sled::Tree>, app_id: clacks_transport::AppId) -> Self {
        TelegramManagerActor {
            log, tree, app_id,
            connections: BTreeMap::new(),
        }
    }

    fn servers_for(&self, req: &Connect) -> Result<TelegramServersV1> {
        let dc = match req.dc_id {
            Some(dc) => dc,
            None => match Entry::new(UserData { phone_number: (&req.phone_number).into() }).get(&self.tree)? {
                Some(data) => data.native_dc,
                None => 2,
            },
        };
        Entry::new(TelegramDatacenter { number: dc, test_mode: req.test_mode })
            .get_or_default(&self.tree)
    }
}

impl Actor for TelegramManagerActor {
    type Context = Context<Self>;
}

pub struct Connect {
    pub phone_number: String,
    pub test_mode: bool,
    pub dc_id: Option<u32>,
    pub read_auth_code: Option<Recipient<client::ReadAuthCode>>,
}

impl Message for Connect {
    type Result = Result<Addr<RpcClientActor>>;
}

async_handler!(fn handle()(this: TelegramManagerActor, req: Connect, ctx) -> Addr<RpcClientActor> {
    let servers = this.servers_for(&req)?;
    let manager = ctx.address();
    let log = this.log.new(o!());
    let tree = this.tree.clone();
    let app_id = this.app_id.clone();
    Ok(async move {
        let client = {
            let server_addr = servers.iter_addresses()
                .next()
                .ok_or(NoServers)?;
            let connection_fut = tokio::net::TcpStream::connect(&server_addr);
            let connection: super::RealShutdown<tokio::net::TcpStream> = await!(connection_fut)?.into();
            RpcClientActor::create({
                let log = log.clone();
                let app_id = app_id.clone();
                move |ctx| RpcClientActor::from_context(ctx, log, app_id, connection)
            })
        };
        let delegate = Delegate {
            log: log.new(o!("subsystem" => "delegate")),
        }.start();
        await!(client.send(client::SetDelegates {
            delegates: client::EventDelegates {
                unhandled: Some(delegate.recipient()),
                read_auth_code: req.read_auth_code.clone(),
            },
        }))?;
        let user_auth = Entry::new(
            UserAuthKey { phone_number: req.phone_number.to_owned().into() });
        let to_persist = match user_auth.get(&tree)? {
            Some(key) => {
                let perm_key = clacks_crypto::symm::AuthKey::new(&key.auth_key)?;
                await!(clacks_rpc::kex::adopt_auth_key(
                    client.clone(), futures_cpupool::CpuPool::new(1), chrono::Duration::hours(24),
                    perm_key))?;
                None
            }
            None => {
                let perm_key = await!(clacks_rpc::kex::new_auth_key(
                    client.clone(), futures_cpupool::CpuPool::new(1), chrono::Duration::hours(24)))?;
                Some(perm_key)
            }
        };
        let config = await!(client.send(<client::InitConnection as Default>::default()))??.only();
        let native_dc = config.this_dc as u32;
        save_config(config, &tree)?;
        info!(log, "config saved");
        if let Some(perm_key) = to_persist {
            let authed_user = match await!(client.send(client::SendAuthCode {
                phone_number: req.phone_number.clone().into(),
            }))? {
                Ok(mtproto::auth::Authorization::Authorization(mtproto::auth::authorization::Authorization {
                    user: mtproto::User::User(user), ..
                })) => user,
                Ok(auth) => bail!("weird authorization response {:?}", auth),
                Err(e) => {
                    drop(client);
                    let client::AuthRedirectTo(dc_id) = e.downcast()?;
                    if let Connect { dc_id: Some(prev_dc_id), .. } = req {
                        bail!("attempted double-redirect from {} to {}", prev_dc_id, dc_id);
                    }
                    let req = Connect { dc_id: Some(dc_id), ..req };
                    return Ok(await!(manager.send(req))??);
                },
            };
            Entry::new(user_auth.as_user_data()).set(&tree, &UserDataV1 { native_dc, authed_user })?;
            user_auth.set(&tree, &UserAuthKeyV1 { auth_key: (&perm_key.into_inner()[..]).into() })?;
        }
        Ok(client)
    })
});

fn save_config(config: mtproto::config::Config, save_to: &sled::Tree) -> Result<()> {
    let test_mode: bool = config.test_mode.into();
    let mut dcs: BTreeMap<TelegramDatacenter, TelegramServersV1> = BTreeMap::new();
    for mtproto::DcOption::DcOption(dc) in config.dc_options.0 {
        dcs.entry(TelegramDatacenter { test_mode, number: dc.id as u32 })
            .or_insert_with(|| TelegramServersV1 { servers: vec![] })
            .servers.push(dc);
    }
    for (key, value) in dcs {
        Entry::new(key).set(&save_to, &value)?;
    }
    Ok(())
}

impl Handler<SendMessage> for TelegramManagerActor {
    type Result = ResponseFuture<mtproto::TLObject, Error>;

    fn handle(&mut self, req: SendMessage, ctx: &mut Self::Context) -> Self::Result {
        Box::new({
            self.connections.values().next()
                .ok_or_else(|| format_err!("no connection"))
                .map(|c| {
                    c.send(req).map_err(|e| -> Error { e.into() })
                })
                .into_future()
                .and_then(|f| f)
                .and_then(|r| r)
        })
    }
}

struct Delegate {
    log: Logger,
}

impl Handler<client::Unhandled> for Delegate {
    type Result = ();

    fn handle(&mut self, unhandled: client::Unhandled, ctx: &mut Self::Context) {
        info!(self.log, "unhandled {:?}", unhandled.0);
        info!(self.log, "---json---\n{}\n---", ::serde_json::to_string_pretty(&unhandled.0).expect("not serialized"));
        //self.0.start_send(unhandled);
    }
}

impl Actor for Delegate {
    type Context = Context<Self>;
}
