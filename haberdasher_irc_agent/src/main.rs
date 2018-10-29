#![feature(await_macro, async_await, futures_api)]

#[macro_use] extern crate failure;
#[macro_use] extern crate futures;
#[macro_use] extern crate tokio;
extern crate chrono;
extern crate grpcio;
extern crate irc;

use failure::Error;
use haberdasher_rpc::haberdasher as protos;
use haberdasher_rpc::haberdasher_grpc as rpc;
use irc::client::prelude::*;
use tokio::prelude::*;

const ZNC_CAPS: &[Capability] = &[
    //Capability::Custom("znc.in/batch"),
    Capability::Custom("znc.in/playback"),
    Capability::Custom("znc.in/server-time-iso"),
];

struct NetworkObserver {
    packed_client: irc::client::PackedIrcClient,
    network: String,
    instance: protos::Instance,
    my_nick: String,
    venue_tx: futures::sync::mpsc::Sender<protos::Venue>,
    irc_rx: irc::client::ClientStream,
    buffered: Option<protos::Venue>,
}

impl NetworkObserver {
    async fn with_config(mut config: Config, network_opt: Option<&str>, venue_tx: futures::sync::mpsc::Sender<protos::Venue>) -> Result<Self, Error> {
        if let Some(network) = network_opt {
            if let Some(password) = config.password {
                config.password = Some(password.replacen(":", &format!("/{}:", network), 1))
            } else {
                bail!("network without server password")
            }
        }
        let client_fut = IrcClient::new_future(config)?;
        let packed_client = await!(client_fut)?;
        let irc_rx = packed_client.0.stream();
        let network = network_opt.unwrap_or("").to_owned();
        let mut instance = protos::Instance::new();
        instance.set_name(network.clone());
        instance.set_id(network.as_bytes().to_owned());
        Ok(NetworkObserver {
            packed_client, network, instance, venue_tx, irc_rx,
            my_nick: "".to_owned(),
            buffered: None,
        })
    }

    fn process_message(&mut self, msg: Message) {
        match &msg.command {
            Command::Response(Response::RPL_WELCOME, args, _) if !args.is_empty() => {
                self.my_nick = args[0].to_owned();
            }
            Command::NICK(new_nick) if self.is_message_from_myself(&msg) => {
                self.my_nick = new_nick.to_owned();
            }
            Command::PRIVMSG(to, text) |
            Command::NOTICE(to, text) => {
                assert!(self.buffered.is_none());
                let venue = self.venue_from_textual_message(to, text, &msg);
                self.buffered = Some(venue);
            }
            _ => {}
        }
    }

    fn venue_from_textual_message(&self, to: &str, text: &str, msg: &Message) -> protos::Venue {
        let from = msg.source_nickname().expect("should have nick");
        let mut from_individual = protos::Individual::new();
        from_individual.set_name(from.to_owned());
        from_individual.set_id(from.as_bytes().to_owned());
        let is_pm = Some(from) == msg.response_target();
        let mut venue = protos::Venue::new();
        if is_pm {
            venue.set_individual(from_individual.clone());
        } else {
            let group = venue.mut_group();
            group.set_name(to.to_owned());
            group.set_id(to.as_bytes().to_owned());
        }
        {
            let last = venue.mut_last_message();
            if self.is_message_from_myself(msg) {
                last.mut_performer().set_myself(true);
            } else {
                last.mut_performer().set_individual(from_individual);
            }
            if let Some(action) = action_text(text) {
                last.set_pose(action.to_owned());
            } else {
                last.set_text(text.to_owned());
            }
            let at_dt = message_time(&msg).unwrap_or_else(|| chrono::Utc::now());
            let at = last.mut_at();
            at.set_seconds(at_dt.timestamp());
            at.set_nanos(at_dt.timestamp_subsec_nanos() as i32);
        }
        venue.set_instance(self.instance.clone());
        venue
    }

    fn is_message_from_myself(&self, msg: &Message) -> bool {
        match &msg.prefix {
            Some(Prefix::Nickname(nick, ..)) => nick == &self.my_nick,
            _ => false,
        }
    }

    fn poll_errorful(&mut self) -> Poll<(), Error> {
        let mut poll_again = false;
        if self.buffered.is_none() {
            match self.irc_rx.poll()? {
                Async::Ready(Some(message)) => {
                    self.process_message(message);
                    poll_again = true;
                }
                Async::Ready(None) => bail!("irc stream closed"),
                Async::NotReady => (),
            }
        }
        if let Some(item) = self.buffered.take() {
            match self.venue_tx.start_send(item)? {
                AsyncSink::NotReady(item) => {
                    self.buffered = Some(item);
                }
                AsyncSink::Ready => {
                    poll_again = true;
                }
            }
        }
        let _: Async<()> = self.venue_tx.poll_complete()?;
        match self.packed_client.1.poll()? {
            Async::Ready(()) => bail!("irc driver finished"),
            Async::NotReady => (),
        }
        Ok(if poll_again { Async::Ready(()) } else { Async::NotReady })
    }

    fn poll_loop(&mut self) -> Poll<(), Error> {
        loop {
            let () = try_ready!(self.poll_errorful());
        }
    }
}

impl Future for NetworkObserver {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<(), ()> {
        self.poll_loop().map_err(|e| {
            println!("error observing network {:?}/{:?}: {:?}", self.network, self.my_nick, e);
        })
    }
}

fn message_time(msg: &Message) -> Option<chrono::DateTime<chrono::Utc>> {
    let tags = msg.tags.as_ref()?;
    let time_string = tags.iter()
        .filter_map(|t| if t.0 == "time" { t.1.as_ref() } else { None })
        .next()?;
    time_string.parse().ok()
}


fn action_text(line: &str) -> Option<&str> {
    let trimmed_start = line.trim_start_matches("\u{1}ACTION ");
    if line == trimmed_start {
        return None;
    }
    let trimmed_full = trimmed_start.trim_end_matches('\u{1}');
    if trimmed_start == trimmed_full {
        return None;
    }
    Some(trimmed_full)
}

async fn drive_one_network(config: Config, network_opt: Option<String>, venue_tx: futures::sync::mpsc::Sender<protos::Venue>) -> Result<(), Error> {
    loop {
        let observer = await!(NetworkObserver::with_config(
            config.clone(), network_opt.as_ref().map(String::as_str), venue_tx.clone()))?;
        {
            let irc_client = &observer.packed_client.0;
            irc_client.send_cap_req(&ZNC_CAPS)?;
            irc_client.identify()?;
        }
        let _: Result<(), ()> = await!(observer);
    }
}

fn drive_all_networks(venue_tx: futures::sync::mpsc::Sender<protos::Venue>) {
    let mut config = Config::load("irc.toml").expect("couldn't load config");
    let mut futures = vec![];
    if let Some(networks) = config.alt_nicks.take() {
        for network in networks {
            futures.push(drive_one_network(config.clone(), Some(network), venue_tx.clone()));
        }
    } else {
        futures.push(drive_one_network(config, None, venue_tx));
    }
    for fut in futures {
        tokio::spawn_async(async {
            if let Err(e) = await!(fut) {
                println!("error driving network: {:?}", e)
            }
        })
    }
}

struct PublishDriver {
    agent_client: rpc::AgentSubscriberClient,
    buffered: Option<protos::Venue>,
    venue_tx: grpcio::StreamingCallSink<protos::Venue>,
    venue_rx: futures::sync::mpsc::Receiver<protos::Venue>,
    empty_rx: grpcio::ClientCStreamReceiver<protos::Empty>,
}

impl PublishDriver {
    fn new() -> Result<(futures::sync::mpsc::Sender<protos::Venue>, Self), Error> {
        let env = std::sync::Arc::new(grpcio::EnvBuilder::new().build());
        let channel = grpcio::ChannelBuilder::new(env)
            .connect("127.0.0.1:42253");
        let agent_client = rpc::AgentSubscriberClient::new(channel);
        let (venue_remote_tx, venue_rx) = futures::sync::mpsc::channel(125);
        let (venue_tx, empty_rx) = agent_client.publish_venue_updates()?;
        Ok((venue_remote_tx, PublishDriver {
            agent_client, venue_tx, venue_rx, empty_rx,
            buffered: None,
        }))
    }

    fn reestablish_publication(&mut self) -> Result<(), Error> {
        let (venue_tx, empty_rx) = self.agent_client.publish_venue_updates()?;
        self.venue_tx = venue_tx;
        self.empty_rx = empty_rx;
        Ok(())
    }

    fn poll_errorful(&mut self) -> Poll<(), Error> {
        let mut poll_again = false;
        match self.empty_rx.poll() {
            Ok(Async::Ready(_empty)) => {
                println!("venue publish ended naturally");
                self.reestablish_publication()?;
                poll_again = true;
            }
            Err(e) => {
                println!("venue publish ended with error: {:?}", e);
                self.reestablish_publication()?
            }
            Ok(Async::NotReady) => (),
        }
        if self.buffered.is_none() {
            match self.venue_rx.poll() {
                Ok(Async::Ready(Some(x))) => {
                    self.buffered = Some(x);
                    poll_again = true;
                }
                Ok(Async::NotReady) => (),
                Ok(Async::Ready(None)) |
                Err(()) => unreachable!(),
            }
        }
        if let Some(item) = self.buffered.take() {
            match self.venue_tx.start_send((item, Default::default()))? {
                AsyncSink::NotReady((item, _)) => {
                    self.buffered = Some(item);
                }
                AsyncSink::Ready => {
                    poll_again = true;
                }
            }
        }
        let _: Async<()> = self.venue_tx.poll_complete()?;
        Ok(if poll_again { Async::Ready(()) } else { Async::NotReady })
    }

    fn poll_loop(&mut self) -> Poll<(), Error> {
        loop {
            let () = try_ready!(self.poll_errorful());
        }
    }
}

impl Future for PublishDriver {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<(), ()> {
        self.poll_loop().map_err(|e| {
            println!("error driving publication: {:?}", e);
        })
    }
}

fn main() {
    let (venue_tx, driver) = PublishDriver::new().expect("couldn't even start");
    tokio::run(driver.join(future::lazy(move || {
        drive_all_networks(venue_tx);
        Ok(())
    })).map(|((), ())| ()));
}
