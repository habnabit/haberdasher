extern crate futures;
extern crate grpcio;
extern crate tokio;
extern crate haberdasher_rpc;

use haberdasher_rpc::haberdasher as protos;
use haberdasher_rpc::haberdasher_grpc as rpc;
use tokio::prelude::*;

#[derive(Debug, Clone)]
struct AgentSubscriberImpl;

impl haberdasher_rpc::haberdasher_grpc::AgentSubscriber for AgentSubscriberImpl {
    fn establish(
        &mut self, ctx: grpcio::RpcContext,
        stream: grpcio::RequestStream<protos::AgentResponse>,
        sink: grpcio::DuplexSink<protos::AgentRequest>)
    {

    }

     fn publish_venue_updates(
        &mut self, ctx: grpcio::RpcContext,
        stream: grpcio::RequestStream<protos::Venue>,
        sink: grpcio::ClientStreamingSink<protos::Empty>)
    {
        println!("incoming venue updates");
        ctx.spawn({
            stream
                .for_each(|v| {
                    println!("update: {:?}", v);
                    Ok(())
                })
                .map_err(|e| println!("error {:?}", e))
                .then(move |r| {
                    sink.success(protos::Empty::new());
                    r
                })
        });
    }
}

fn main() {
    let service = rpc::create_agent_subscriber(AgentSubscriberImpl);
    let env = std::sync::Arc::new(grpcio::Environment::new(1));
    let mut server = grpcio::ServerBuilder::new(env)
        .register_service(service)
        .bind("127.0.0.1", 42253)
        .build()
        .unwrap();
    server.start();
    println!("bound to {:?}", server.bind_addrs());
    tokio::run(futures::empty());
}
