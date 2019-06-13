use std::collections::HashMap;
use haberdasher_rpc::haberdasher as protos;
use haberdasher_rpc::haberdasher_grpc as rpc;
use serde::Deserialize;
use tokio::prelude::*;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
struct Token(String);

#[derive(Debug, Clone, Deserialize)]
struct AgentConfig {
    name: Option<String>,
    token: Token,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct AgentSubscriberConfig {
    agents: HashMap<String, AgentConfig>,
}

#[derive(Debug)]
struct AgentSubscriberSharedState {
    agents_by_token: HashMap<Token, AgentConfig>,
}

#[derive(Debug, Clone)]
struct AgentSubscriberImpl {
    shared: std::sync::Arc<std::sync::RwLock<AgentSubscriberSharedState>>,
}

impl AgentSubscriberImpl {
    fn from_config(config: AgentSubscriberConfig) -> Self {
        let agents_by_token = config.agents.into_iter()
            .map(|(name, mut agent)| {
                if agent.name.is_none() {
                    agent.name = Some(name);
                }
                (agent.token.clone(), agent)
            })
            .collect::<HashMap<_, _>>();
        let shared = AgentSubscriberSharedState { agents_by_token };
        let shared = std::sync::Arc::new(std::sync::RwLock::new(shared));
        AgentSubscriberImpl { shared }
    }

    fn agent_of_context(&self, ctx: &grpcio::RpcContext) -> Option<owning_ref::RwLockReadGuardRef<AgentSubscriberSharedState, AgentConfig>> {
        let (_, value) = ctx.request_headers()
            .iter()
            .find(|(h, _)| *h == "access-token")?;
        let decoded = Token(std::str::from_utf8(value).ok()?.to_owned());
        let shared = self.shared.read().unwrap();
        let shared = owning_ref::RwLockReadGuardRef::new(shared);
        shared.try_map(|shared| {
            shared.agents_by_token.get(&decoded).ok_or(())
        }).ok()
    }
}

impl haberdasher_rpc::haberdasher_grpc::AgentSubscriber for AgentSubscriberImpl {
    fn handle_agent_requests(
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
        let peer = ctx.peer();
        let res: Box<dyn Future<Item = _, Error = failure::Error> + Send> = if let Some(agent) = self.agent_of_context(&ctx) {
            println!("allowed incoming venue updates from {:?} ({:?})", peer, agent);
            let segment = {
                let agent_name = agent.name.as_ref().map(String::as_str).unwrap_or("<bogus>").to_owned();
                let mut segment = <protos::origin::Segment as Default>::default();
                segment.agent = agent_name;
                segment
            };
            Box::new({
                stream
                    .map(move |mut v| {
                        let mut origin = v.origin.unwrap_or_default();
                        origin.path.insert(0, segment.clone());
                        v.origin = Some(origin);
                        v
                    })
                    .for_each(|v| {
                        println!("update: {:#?}", v);
                        Ok(())
                    })
                    .then(move |r| {
                        sink.success(protos::Empty {})
                            .and_then(move |()| r)
                    })
                    .map_err(|e| e.into())
            })
        } else {
            println!("denied incoming venue updates from {:?}", peer);
            Box::new({
                sink.fail(grpcio::RpcStatus::new(grpcio::RpcStatusCode::PermissionDenied, None))
                    .map_err(|e| e.into())
            })
        };
        ctx.spawn({
            res
                .map_err(|e| println!("error publishing: {:?}", e))
        });
    }
}

fn main() -> Result<(), failure::Error> {
    let argv: Vec<_> = std::env::args().collect();
    let config = {
        let mut infile = std::fs::File::open(&argv[1])?;
        let mut content = String::new();
        infile.read_to_string(&mut content)?;
        toml::from_str(&content)?
    };
    let service = rpc::create_agent_subscriber(AgentSubscriberImpl::from_config(config));
    let env = std::sync::Arc::new(grpcio::Environment::new(1));
    let mut server = grpcio::ServerBuilder::new(env)
        .register_service(service)
        .bind("127.0.0.1", 42253)
        .build()?;
    server.start();
    println!("bound to {:?}", server.bind_addrs());
    tokio::run(futures::empty());
    Ok(())
}
