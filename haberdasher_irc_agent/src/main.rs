#![feature(await_macro, async_await, futures_api)]

#[macro_use] extern crate tokio;
extern crate futures;
extern crate grpcio;
extern crate irc;

use failure::Error;
use haberdasher_rpc::haberdasher as protos;
use haberdasher_rpc::haberdasher_grpc as rpc;
use irc::client::prelude::*;
use tokio::prelude::*;

async fn make_client_future() -> Result<IrcClient, Error> {
    let config = Config::load("irc.toml")?;
    let client_fut = IrcClient::new_future(config)?;
    let irc::client::PackedIrcClient(client, driver) = await!(client_fut)?;
    tokio::spawn(
        driver.map_err(|e| println!("driver error {:?}", e)));
    Ok(client)
}

async fn publish_irc(client: rpc::AgentSubscriberClient) {
    let (tx, rx) = client.publish_venue_updates().expect("initial publish");
    let _empty = await!(rx).expect("first empty");
    println!("hey, whoa");
}

fn main() {
    let env = std::sync::Arc::new(grpcio::EnvBuilder::new().build());
    let channel = grpcio::ChannelBuilder::new(env)
        .connect("127.0.0.1:42253");
    let client = rpc::AgentSubscriberClient::new(channel);
    tokio::run_async(publish_irc(client));
}
