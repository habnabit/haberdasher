#![feature(await_macro, async_await, futures_api)]

#[macro_use] extern crate tokio;
extern crate chrono;
extern crate futures;
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

async fn make_client_future() -> Result<IrcClient, Error> {
    let config = Config::load("irc.toml")?;
    let client_fut = IrcClient::new_future(config)?;
    let irc::client::PackedIrcClient(client, driver) = await!(client_fut)?;
    tokio::spawn(
        driver.map_err(|e| println!("driver error {:?}", e)));
    Ok(client)
}

fn message_time(msg: &Message) -> Option<chrono::DateTime<chrono::Utc>> {
    let tags = msg.tags.as_ref()?;
    let time_string = tags.iter()
        .filter_map(|t| if t.0 == "time" { t.1.as_ref() } else { None })
        .next()?;
    time_string.parse().ok()
}

fn is_message_from_myself(msg: &Message, my_nick: &str) -> bool {
    match &msg.prefix {
        Some(Prefix::Nickname(nick, ..)) => nick == my_nick,
        _ => false,
    }
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

async fn publish_irc(client: rpc::AgentSubscriberClient) {
    let (mut tx, _rx) = client.publish_venue_updates().expect("initial publish");
    println!("ready to publish");
    let irc_client = await!(make_client_future()).expect("irc connect");
    irc_client.send_cap_req(&ZNC_CAPS).expect("irc cap req");
    irc_client.identify().expect("irc identify");
    println!("irc ready");
    let mut stream = irc_client.stream();
    let mut my_nick = String::new();
    while let Some(msg_res) = await!(stream.next()) {
        let msg = msg_res.expect("message error");
        let from_myself = is_message_from_myself(&msg, &my_nick);
        println!("-> {:?}", msg);
        match &msg.command {
            Command::Response(Response::RPL_WELCOME, args, _) if !args.is_empty() => {
                my_nick = args[0].to_owned();
            }
            Command::NICK(new_nick) if from_myself => {
                my_nick = new_nick.to_owned();
            }
            Command::PRIVMSG(to, text) |
            Command::NOTICE(to, text) => {
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
                    if from_myself {
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
                await!(tx.send_async((venue, Default::default())));
            }
            _ => ()
        }
    }
}

fn main() {
    let env = std::sync::Arc::new(grpcio::EnvBuilder::new().build());
    let channel = grpcio::ChannelBuilder::new(env)
        .connect("127.0.0.1:42253");
    let client = rpc::AgentSubscriberClient::new(channel);
    tokio::run_async(publish_irc(client));
}
