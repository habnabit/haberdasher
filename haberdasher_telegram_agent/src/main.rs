#![feature(await_macro, async_await, futures_api)]
#![deny(private_in_public, unused_extern_crates)]

#[macro_use(async_handler)] extern crate clacks_rpc;
#[macro_use] extern crate delegate;
#[macro_use] extern crate failure;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate slog;
#[macro_use] extern crate tokio;

use actix::prelude::*;
use clacks_mtproto::{BoxedDeserialize, BoxedSerialize, IntoBoxed, mtproto};
use futures::{Future, Sink, Stream, future};
use futures::prelude::*;
use slog::Logger;
use structopt::StructOpt;

mod config;
mod real_shutdown;
pub use self::real_shutdown::RealShutdown;

mod tg_manager;


struct Delegate {
    log: Logger,
}

impl Handler<clacks_rpc::client::Unhandled> for Delegate {
    type Result = ();

    fn handle(&mut self, unhandled: clacks_rpc::client::Unhandled, _: &mut Self::Context) {
        info!(self.log, "unhandled {:?}", unhandled.0);
        info!(self.log, "---json---\n{}\n---", serde_json::to_string_pretty(&unhandled.0).expect("not serialized"));
    }
}

impl Actor for Delegate {
    type Context = Context<Self>;
}

#[derive(Debug, StructOpt)]
#[structopt(name = "haberdasher_telegram_agent")]
struct Opt {
    #[structopt(short = "f", long = "config", default_value = "config.toml", parse(from_os_str))]
    config: std::path::PathBuf,
    #[structopt(short = "t", long = "test-mode")]
    test_mode: bool,
    #[structopt(subcommand)]
    cmd: Command
}

#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(name = "login")]
    Login {
        phone_number: String,
    },
    #[structopt(name = "run")]
    Run {

    },
}

fn main() -> Result<(), failure::Error> {
    use futures::{Future, Stream};
    use slog::Drain;

    let opts = Opt::from_args();
    println!("opts: {:?}", opts);
    let self::config::AgentConfig { haberdasher, db, telegram } = config::load_config_file(&opts.config)?;
    let db_config = sled::ConfigBuilder::default()
        .path(&db.path)
        .build();
    let tree = std::sync::Arc::new(sled::Tree::start(db_config)?);

    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_envlogger::new(drain);
    let drain = slog_async::Async::new(drain).build().fuse();
    let log = slog::Logger::root(drain, o!());
    let _scoped = slog_scope::set_global_logger(log.new(o!("subsystem" => "implicit logger")));

    System::run(move || {
        let tg_manager = {
            let log = log.new(o!("subsystem" => "tg manager"));
            tg_manager::TelegramManagerActor::new(log, tree, telegram.as_app_id()).start()
        };
        match opts.cmd {
            Command::Login { phone_number } => {
                let connect = tg_manager::Connect {
                    phone_number: phone_number.into(),
                    test_mode: opts.test_mode,
                    dc_id: None,
                };
                let tg_manager = tg_manager.clone();
                Arbiter::spawn_fn(move || {
                    tg_manager.send(connect)
                        .then(move |r| {
                            info!(log, "login result: {:?}", r.map(|_| ()));
                            Ok(())
                        })
                })
            }
            Command::Run {} => {}
        }
        // let jsonrpc = {
        //     let log = log.new(o!("subsystem" => "jsonrpc handler"));
        //     jsonrpc_handler::JsonRpcHandlerActor::new(log, tg_manager).start()
        // };
        // Arbiter::spawn({
        //     let log = log.clone();
        //     tokio_uds::UnixListener::bind("socket")
        //         .into_future()
        //         .and_then(|l| l.incoming().for_each(move |conn| {
        //             let log = log.new(o!("connection" => format!("{:?}", conn)));
        //             let jsonrpc = jsonrpc.clone();
        //             info!(log, "spawning");
        //             let addr = agent_connection::AgentActor::create(|ctx| {
        //                 agent_connection::AgentActor::from_context(ctx, log, conn, jsonrpc)
        //             });
        //             Ok(())
        //         }))
        //         .map_err(|e| panic!("fatal {:?}", e))
        // });
    });

    Ok(())
}
