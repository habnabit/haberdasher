extern crate capnp;
extern crate futures;
extern crate tokio;
extern crate haberdasher_rpc;

use capnp::capability::Promise;
use tokio::prelude::*;

struct AgentServerImpl;

impl haberdasher_rpc::agent_server::Server for AgentServerImpl {
    fn publish(
        &mut self,
        params: haberdasher_rpc::agent_server::PublishParams,
        _results: haberdasher_rpc::agent_server::PublishResults)
    -> Promise<(), capnp::Error> {
        println!("published: {:?}", params.get()?);
        Promise::ok(())
    }
}

pub fn main() {
    use std::net::ToSocketAddrs;
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        println!("usage: {} server ADDRESS[:PORT]", args[0]);
        return;
    }

    let addr = args[2].to_socket_addrs().unwrap().next().expect("could not parse address");
    let socket = tokio::net::TcpListener::bind(&addr).unwrap();

    tokio::runtime::current_thread::block_on_all(futures::lazy(move || {
        let root = haberdasher_rpc::agent_server::ToClient::new(AgentServerImpl)
            .from_server::<capnp_rpc::Server>();

        socket.incoming()
            .for_each(move |socket| {
                socket.set_nodelay(true)?;
                let (reader, writer) = socket.split();

                let network = capnp_rpc::twoparty::VatNetwork::new(
                    reader, writer, capnp_rpc::rpc_twoparty_capnp::Side::Server,
                    Default::default());

                let rpc_system = capnp_rpc::RpcSystem::new(Box::new(network), Some(root.clone().client));
                tokio::runtime::current_thread::spawn(rpc_system.map_err(|e| println!("error: {:?}", e)));
                Ok(())
            })
            .map_err(|e| println!("error: {:?}", e))
    })).unwrap();
}
