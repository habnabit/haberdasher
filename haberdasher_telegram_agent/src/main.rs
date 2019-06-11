#![feature(async_await, never_type)]
#![deny(private_in_public, unused_extern_crates)]

#[macro_use(async_handler)] extern crate clacks_rpc;
//#[macro_use] extern crate delegate;
#[macro_use] extern crate failure;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate slog;

use actix::prelude::*;
use futures::compat::*;
use futures::prelude::*;
use futures::task::Spawn;
use structopt::StructOpt;

mod config;
mod console;
//mod real_shutdown;
//pub use self::real_shutdown::RealShutdown;

mod tg_manager;
//mod publisher;

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

pub type Result<T> = std::result::Result<T, failure::Error>;

fn mailbox_lift<T>(r: std::result::Result<Result<T>, actix::MailboxError>) -> Result<T> {
    r?
}

struct FullTokio {
    thread: std::thread::JoinHandle<Result<()>>,
    executor: tokio::runtime::TaskExecutor,
    shutdown_tx: Option<futures::channel::oneshot::Sender<()>>,
}

impl FullTokio {
    fn spawn() -> Result<Self> {
        let mut runtime = tokio::runtime::Runtime::new()?;
        let executor = runtime.executor();
        let (shutdown_tx, shutdown_rx) = futures::channel::oneshot::channel();
        let thread = std::thread::spawn(move || {
            runtime.block_on(Compat::new(shutdown_rx))?;
            Ok(())
        });
        Ok(FullTokio { thread, executor, shutdown_tx: Some(shutdown_tx) })
    }

    fn shutdown(&mut self) {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
        }
    }

    fn join(mut self) -> Result<()> {
        self.shutdown();
        match self.thread.join() {
            Ok(r) => r?,
            Err(_) => Err(format_err!("failed to join"))?,
        }
        Ok(())
    }

    fn spawner(&self) -> FullTokioSpawn {
        FullTokioSpawn(self.executor.clone().compat())
    }
}

#[derive(Clone)]
pub struct FullTokioSpawn(Executor01As03<tokio::runtime::TaskExecutor>);

impl Spawn for FullTokioSpawn {
    fn spawn_obj(
        &mut self,
        future: future::FutureObj<'static, ()>
    ) -> std::result::Result<(), futures::task::SpawnError> {
        self.0.spawn_obj(future)
    }
}

fn main() -> Result<()> {
    use slog::Drain;

    let opts = Opt::from_args();
    let test_mode = opts.test_mode;
    let self::config::AgentConfig { db, telegram, .. } = config::load_config_file(&opts.config)?;
    let db = sled::Db::start_default(&db.path)?;
    let tree = db.open_tree("legacy")?;

    let decorator = slog_term::TermDecorator::new().stderr().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_envlogger::new(drain);
    let drain = slog_async::Async::new(drain).build().fuse();
    let log = slog::Logger::root(drain, o!());
    let _scoped = slog_scope::set_global_logger(log.new(o!("subsystem" => "implicit logger")));

    let full_tokio = FullTokio::spawn()?;
    let mut tokio_spawn = full_tokio.spawner();

    System::run(move || {
        let tg_manager = {
            let log = log.new(o!("subsystem" => "tg manager"));
            tg_manager::TelegramManagerActor::new(log, tokio_spawn.clone(), tree, telegram.as_app_id()).start()
        };
        match opts.cmd {
            Command::Login { phone_number } => {
                let code_reader = self::console::AuthCodeReader::create({
                    let log = log.new(o!("subsystem" => "auth code reader"));
                    let (lines_tx, lines_rx) = futures::channel::mpsc::channel::<String>(5);
                    tokio_spawn.spawn_obj(Box::pin(async move {
                        tokio::io::lines(std::io::BufReader::new(tokio::io::stdin()))
                            .compat()
                            .map_err(|e| e.into())
                            .forward(lines_tx.sink_map_err(|e| -> failure::Error { e.into() }))
                            .await
                            .unwrap_or_else(|e| panic!("XXX unhandled {:?}", e))
                    }).into()).unwrap_or_else(|e| panic!("XXX unhandled {:?}", e));
                    |ctx| self::console::AuthCodeReader::from_context(ctx, log, lines_rx.map(Ok))
                });
                let connect = tg_manager::Connect {
                    phone_number, test_mode,
                    dc_id: None,
                    read_auth_code: Some(code_reader.recipient()),
                };
                Arbiter::spawn_fn(move || {
                    tg_manager.send(connect)
                        .then(mailbox_lift)
                        .and_then(move |(_, client)| {
                            client.send(clacks_rpc::client::CallFunction::encrypted(
                                clacks_mtproto::mtproto::rpc::users::GetFullUser {
                                    id: clacks_mtproto::mtproto::InputUser::Self_,
                                }))
                                .then(mailbox_lift)
                        })
                        .then(move |r| {
                            info!(log, "login complete"; "result" => format!("{:#?}", r));
                            System::current().stop();
                            Ok(())
                        })
                })
            }
            Command::Run {} => {
                let to_spawn: futures::stream::FuturesUnordered<_> = telegram.users.into_iter()
                    .map(move |phone_number| {
                        let spawned = phone_number.clone();
                        let fut = tg_manager.send(tg_manager::SpawnClient { phone_number, test_mode });
                        async move {
                            (spawned, mailbox_lift(fut.compat().await))
                        }
                    })
                    .collect();
                Arbiter::spawn_fn(move || {
                    let log_done = log.clone();
                    let fut = to_spawn
                        .for_each(move |(phone_number, r)| {
                            info!(log, "spawn complete"; "phone_number" => phone_number, "result" => format!("{:#?}", r));
                            future::ready(())
                        })
                        .map(move |()| info!(log_done, "done spawning"))
                        .map(|()| Ok(()));
                    Compat::new(fut)
                })
            }
        }
    })?;

    drop(full_tokio);
    Ok(())
}
