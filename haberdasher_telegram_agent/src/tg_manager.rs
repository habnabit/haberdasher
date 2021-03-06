//use actix::{self, Actor, Addr, Arbiter, AsyncContext, Context, StreamHandler, System};
use actix::prelude::*;
use clacks_mtproto::mtproto;
use clacks_rpc::client::{self, RpcClientActor};
use futures::compat::*;
use futures::prelude::*;
use slog::Logger;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;
use haberdasher_rpc::haberdasher as protos;

use crate::config::{Entry, TelegramDatacenter, TelegramServersV1, UserAuthKey, UserAuthKeyV1, UserData, UserDataV1};
use crate::{FullTokioSpawn, Result};

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
    spawn: FullTokioSpawn,
    tree: Arc<sled::Tree>,
    app_id: clacks_transport::AppId,
    connections: BTreeMap<UserDc, Addr<RpcClientActor>>,
}

impl TelegramManagerActor {
    pub fn new(log: Logger, spawn: FullTokioSpawn, tree: Arc<sled::Tree>, app_id: clacks_transport::AppId) -> Self {
        TelegramManagerActor {
            log, spawn, tree, app_id,
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
    pub venue_tx: Option<futures::channel::mpsc::Sender<protos::Venue>>,
    pub read_auth_code: Option<Recipient<client::ReadAuthCode>>,
}

impl Message for Connect {
    type Result = Result<(UserDc, Addr<RpcClientActor>)>;
}

async_handler!(fn handle()(this: TelegramManagerActor, req: Connect, ctx) -> (UserDc, Addr<RpcClientActor>) {
    let servers = this.servers_for(&req)?;
    let manager = ctx.address();
    let log = this.log.new(o!());
    let spawn = this.spawn.clone();
    let tree = this.tree.clone();
    let app_id = this.app_id.clone();
    Ok(async move {
        let client = {
            let mut addr_futures = servers.iter_addresses()
                .map(|addr| tokio::net::TcpStream::connect(&addr).compat())
                .collect::<stream::FuturesUnordered<_>>()
                .filter_map(|r| match r {
                    Ok(s) => future::ready(Some(s)),
                    Err(e) => {
                        info!(log, "ignored failure to connect"; "error" => ?e);
                        future::ready(None)
                    },
                });
            let connection = addr_futures
                .next()
                .await
                .ok_or(NoServers)?;
            RpcClientActor::create({
                let log = log.clone();
                let app_id = app_id.clone();
                move |ctx| RpcClientActor::from_context(ctx, log, app_id, connection)
            })
        };
        let delegate = Delegate {
            log: log.new(o!("subsystem" => "delegate")),
            venue_tx: req.venue_tx.clone(),
        }.start();
        client.send(client::SetDelegates {
            delegates: client::EventDelegates {
                unhandled: Some(delegate.recipient()),
                read_auth_code: req.read_auth_code.clone(),
            },
        }).compat().await?;
        let user_auth = Entry::new(
            UserAuthKey { phone_number: req.phone_number.to_owned().into() });
        let to_persist = match user_auth.get(&tree)? {
            Some(key) => {
                let perm_key = clacks_crypto::symm::AuthKey::new(&key.auth_key)?;
                clacks_rpc::kex::adopt_auth_key(
                    client.clone(), chrono::Duration::hours(24),
                    perm_key).await?;
                None
            }
            None => {
                let perm_key = clacks_rpc::kex::new_auth_key(
                    client.clone(), chrono::Duration::hours(24)).await?;
                Some(perm_key)
            }
        };
        let config = client.send(<client::InitConnection as Default>::default()).compat().await??.only();
        let native_dc = config.this_dc as u32;
        save_config(config, &tree)?;
        info!(log, "config saved");
        if let Some(perm_key) = to_persist {
            let authed_user = match client.send(client::SendAuthCode {
                phone_number: req.phone_number.clone(),
            }).compat().await? {
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
                    return Ok(manager.send(req).compat().await??);
                },
            };
            Entry::new(user_auth.as_user_data()).set(&tree, &UserDataV1 { native_dc, authed_user })?;
            user_auth.set(&tree, &UserAuthKeyV1 { auth_key: (&perm_key.into_inner()[..]).into() })?;
        }
        let user_dc = UserDc {
            phone_number: req.phone_number,
            dc: native_dc,
            primary: true,
        };
        Ok((user_dc, client))
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

pub struct SpawnClient {
    pub phone_number: String,
    pub test_mode: bool,
    pub venue_tx: Option<futures::channel::mpsc::Sender<protos::Venue>>,
}

impl Message for SpawnClient {
    type Result = Result<()>;
}

async_handler!(fn handle()(this: TelegramManagerActor, req: SpawnClient, ctx) -> () {
    let connect = Connect {
        phone_number: req.phone_number,
        test_mode: req.test_mode,
        venue_tx: req.venue_tx,
        dc_id: None,
        read_auth_code: None,
    };
    let log = this.log.clone();
    let manager = ctx.address();
    Ok(async move {
        let (user_dc, client) = manager.send(connect).compat().await??;
        let saved_client = client.clone();
        manager.send(Trampoline::new(move |this: &mut Self, _ctx| {
            this.connections.insert(user_dc, saved_client);
        })).compat().await?;
        let dialogs = client.send(client::CallFunction::encrypted(mtproto::rpc::messages::GetDialogs {
            exclude_pinned: false,
            folder_id: None,
            offset_date: 0,
            offset_id: 0,
            offset_peer: mtproto::InputPeer::Empty,
            limit: 25,
            hash: 0,
        })).compat().await??;
        info!(log, "spawn complete"; "dialogs" => format!("{:#?}", dialogs));
        Ok(())
    })
});

struct Delegate {
    log: Logger,
    venue_tx: Option<futures::channel::mpsc::Sender<protos::Venue>>,
}

impl Handler<client::Unhandled> for Delegate {
    type Result = ();

    fn handle(&mut self, unhandled: client::Unhandled, _: &mut Self::Context) {
        info!(self.log, "unhandled {:?}", unhandled.0);
        info!(self.log, "---json---\n{}\n---", ::serde_json::to_string_pretty(&unhandled.0).expect("not serialized"));
    }
}

impl Actor for Delegate {
    type Context = Context<Self>;
}

pub struct Trampoline<A, F> {
    func: F,
    phantom: PhantomData<fn(A)>,
}


impl<A, F> Trampoline<A, F>
    where A: Actor,
          F: FnOnce(&mut A, &mut A::Context),
{
    pub fn new(func: F) -> Self {
        Self { func, phantom: PhantomData }
    }
}

impl<A, F> Message for Trampoline<A, F>
    where A: Actor,
          F: FnOnce(&mut A, &mut A::Context),
{
    type Result = ();
}

impl<F> Handler<Trampoline<Self, F>> for TelegramManagerActor
    where F: FnOnce(&mut Self, &mut <Self as Actor>::Context),
{
    type Result = ();

    fn handle(&mut self, t: Trampoline<Self, F>, ctx: &mut Self::Context) {
        (t.func)(self, ctx);
    }
}
