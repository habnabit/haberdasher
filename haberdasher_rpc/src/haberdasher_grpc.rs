// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]

const METHOD_AGENT_SUBSCRIBER_HANDLE_AGENT_REQUESTS: ::grpcio::Method<super::haberdasher::AgentResponse, super::haberdasher::AgentRequest> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Duplex,
    name: "/haberdasher.AgentSubscriber/HandleAgentRequests",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pr_ser, de: ::grpcio::pr_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pr_ser, de: ::grpcio::pr_de },
};

const METHOD_AGENT_SUBSCRIBER_PUBLISH_VENUE_UPDATES: ::grpcio::Method<super::haberdasher::Venue, super::haberdasher::Empty> = ::grpcio::Method {
    ty: ::grpcio::MethodType::ClientStreaming,
    name: "/haberdasher.AgentSubscriber/PublishVenueUpdates",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pr_ser, de: ::grpcio::pr_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pr_ser, de: ::grpcio::pr_de },
};

#[derive(Clone)]
pub struct AgentSubscriberClient {
    client: ::grpcio::Client,
}

impl AgentSubscriberClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        AgentSubscriberClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn handle_agent_requests_opt(&self, opt: ::grpcio::CallOption) -> ::grpcio::Result<(::grpcio::ClientDuplexSender<super::haberdasher::AgentResponse>, ::grpcio::ClientDuplexReceiver<super::haberdasher::AgentRequest>)> {
        self.client.duplex_streaming(&METHOD_AGENT_SUBSCRIBER_HANDLE_AGENT_REQUESTS, opt)
    }

    pub fn handle_agent_requests(&self) -> ::grpcio::Result<(::grpcio::ClientDuplexSender<super::haberdasher::AgentResponse>, ::grpcio::ClientDuplexReceiver<super::haberdasher::AgentRequest>)> {
        self.handle_agent_requests_opt(::grpcio::CallOption::default())
    }

    pub fn publish_venue_updates_opt(&self, opt: ::grpcio::CallOption) -> ::grpcio::Result<(::grpcio::ClientCStreamSender<super::haberdasher::Venue>, ::grpcio::ClientCStreamReceiver<super::haberdasher::Empty>)> {
        self.client.client_streaming(&METHOD_AGENT_SUBSCRIBER_PUBLISH_VENUE_UPDATES, opt)
    }

    pub fn publish_venue_updates(&self) -> ::grpcio::Result<(::grpcio::ClientCStreamSender<super::haberdasher::Venue>, ::grpcio::ClientCStreamReceiver<super::haberdasher::Empty>)> {
        self.publish_venue_updates_opt(::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::futures::Future<Item = (), Error = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait AgentSubscriber {
    fn handle_agent_requests(&mut self, ctx: ::grpcio::RpcContext, stream: ::grpcio::RequestStream<super::haberdasher::AgentResponse>, sink: ::grpcio::DuplexSink<super::haberdasher::AgentRequest>);
    fn publish_venue_updates(&mut self, ctx: ::grpcio::RpcContext, stream: ::grpcio::RequestStream<super::haberdasher::Venue>, sink: ::grpcio::ClientStreamingSink<super::haberdasher::Empty>);
}

pub fn create_agent_subscriber<S: AgentSubscriber + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let mut instance = s.clone();
    builder = builder.add_duplex_streaming_handler(&METHOD_AGENT_SUBSCRIBER_HANDLE_AGENT_REQUESTS, move |ctx, req, resp| {
        instance.handle_agent_requests(ctx, req, resp)
    });
    let mut instance = s;
    builder = builder.add_client_streaming_handler(&METHOD_AGENT_SUBSCRIBER_PUBLISH_VENUE_UPDATES, move |ctx, req, resp| {
        instance.publish_venue_updates(ctx, req, resp)
    });
    builder.build()
}

const METHOD_AGENT_AGGREGATOR_SUBSCRIBE_TO_VENUE_UPDATES: ::grpcio::Method<super::haberdasher::Empty, super::haberdasher::Venue> = ::grpcio::Method {
    ty: ::grpcio::MethodType::ServerStreaming,
    name: "/haberdasher.AgentAggregator/SubscribeToVenueUpdates",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pr_ser, de: ::grpcio::pr_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pr_ser, de: ::grpcio::pr_de },
};

#[derive(Clone)]
pub struct AgentAggregatorClient {
    client: ::grpcio::Client,
}

impl AgentAggregatorClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        AgentAggregatorClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn subscribe_to_venue_updates_opt(&self, req: &super::haberdasher::Empty, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientSStreamReceiver<super::haberdasher::Venue>> {
        self.client.server_streaming(&METHOD_AGENT_AGGREGATOR_SUBSCRIBE_TO_VENUE_UPDATES, req, opt)
    }

    pub fn subscribe_to_venue_updates(&self, req: &super::haberdasher::Empty) -> ::grpcio::Result<::grpcio::ClientSStreamReceiver<super::haberdasher::Venue>> {
        self.subscribe_to_venue_updates_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::futures::Future<Item = (), Error = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait AgentAggregator {
    fn subscribe_to_venue_updates(&mut self, ctx: ::grpcio::RpcContext, req: super::haberdasher::Empty, sink: ::grpcio::ServerStreamingSink<super::haberdasher::Venue>);
}

pub fn create_agent_aggregator<S: AgentAggregator + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let mut instance = s;
    builder = builder.add_server_streaming_handler(&METHOD_AGENT_AGGREGATOR_SUBSCRIBE_TO_VENUE_UPDATES, move |ctx, req, resp| {
        instance.subscribe_to_venue_updates(ctx, req, resp)
    });
    builder.build()
}
