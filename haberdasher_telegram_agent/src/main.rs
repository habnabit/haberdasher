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
mod console;
mod real_shutdown;
pub use self::real_shutdown::RealShutdown;

mod tg_manager;


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

fn mailbox_lift<T>(r: Result<Result<T, failure::Error>, actix::MailboxError>) -> Result<T, failure::Error> {
    r?
}

struct FullTokio {
    thread: std::thread::JoinHandle<Result<(), failure::Error>>,
    executor: tokio::runtime::TaskExecutor,
    shutdown_tx: futures::sync::oneshot::Sender<()>,
}

impl FullTokio {
    fn spawn() -> Result<Self, failure::Error> {
        let mut runtime = tokio::runtime::Runtime::new()?;
        let executor = runtime.executor();
        let (shutdown_tx, shutdown_rx) = futures::sync::oneshot::channel();
        let thread = std::thread::spawn(move || Ok(runtime.block_on(shutdown_rx)?));
        Ok(FullTokio { thread, executor, shutdown_tx })
    }
}

fn main() -> Result<(), failure::Error> {
    use futures::{Future, Stream};
    use slog::Drain;

    let opts = Opt::from_args();
    let self::config::AgentConfig { haberdasher, db, telegram } = config::load_config_file(&opts.config)?;
    let db_config = sled::ConfigBuilder::default()
        .path(&db.path)
        .build();
    let tree = std::sync::Arc::new(sled::Tree::start(db_config)?);

    let decorator = slog_term::TermDecorator::new().stderr().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_envlogger::new(drain);
    let drain = slog_async::Async::new(drain).build().fuse();
    let log = slog::Logger::root(drain, o!());
    let _scoped = slog_scope::set_global_logger(log.new(o!("subsystem" => "implicit logger")));

    let full_tokio = FullTokio::spawn()?;

    System::run(move || {
        let tg_manager = {
            let log = log.new(o!("subsystem" => "tg manager"));
            tg_manager::TelegramManagerActor::new(log, tree, telegram.as_app_id()).start()
        };
        match opts.cmd {
            Command::Login { phone_number } => {
                let code_reader = self::console::AuthCodeReader::create({
                    let lines = tokio::io::lines(std::io::BufReader::new(tokio::io::stdin()));
                    let lines = futures::sync::mpsc::spawn(lines, &full_tokio.executor, 5);
                    let log = log.new(o!("subsystem" => "auth code reader"));
                    |ctx| self::console::AuthCodeReader::from_context(ctx, log, lines)
                });
                let connect = tg_manager::Connect {
                    phone_number: phone_number.into(),
                    test_mode: opts.test_mode,
                    dc_id: None,
                    read_auth_code: Some(code_reader.recipient()),
                };
                let tg_manager = tg_manager.clone();
                Arbiter::spawn_fn(move || {
                    tg_manager.send(connect)
                        .then(mailbox_lift)
                        .and_then(move |client| {
                            client.send(clacks_rpc::client::CallFunction::encrypted(
                                clacks_mtproto::mtproto::rpc::users::GetFullUser {
                                    id: clacks_mtproto::mtproto::InputUser::Self_,
                                }))
                                .then(mailbox_lift)
                        })
                        .then(move |r| {
                            info!(log, "login complete"; "result" => format!("{:#?}", r));
                            Ok(())
                        })
                })
            }
            Command::Run {} => {}
        }
    });

    Ok(())
}
