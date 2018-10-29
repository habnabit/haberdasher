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

const METHOD_AGENT_SUBSCRIBER_ESTABLISH_CLIENT: ::grpcio::Method<super::haberdasher::EstablishClientRequest, super::haberdasher::Empty> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/haberdasher.AgentSubscriber/EstablishClient",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_AGENT_SUBSCRIBER_HANDLE_AGENT_REQUESTS: ::grpcio::Method<super::haberdasher::AgentResponse, super::haberdasher::AgentRequest> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Duplex,
    name: "/haberdasher.AgentSubscriber/HandleAgentRequests",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_AGENT_SUBSCRIBER_PUBLISH_VENUE_UPDATES: ::grpcio::Method<super::haberdasher::Venue, super::haberdasher::Empty> = ::grpcio::Method {
    ty: ::grpcio::MethodType::ClientStreaming,
    name: "/haberdasher.AgentSubscriber/PublishVenueUpdates",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

pub struct AgentSubscriberClient {
    client: ::grpcio::Client,
}

impl AgentSubscriberClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        AgentSubscriberClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn establish_client_opt(&self, req: &super::haberdasher::EstablishClientRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::haberdasher::Empty> {
        self.client.unary_call(&METHOD_AGENT_SUBSCRIBER_ESTABLISH_CLIENT, req, opt)
    }

    pub fn establish_client(&self, req: &super::haberdasher::EstablishClientRequest) -> ::grpcio::Result<super::haberdasher::Empty> {
        self.establish_client_opt(req, ::grpcio::CallOption::default())
    }

    pub fn establish_client_async_opt(&self, req: &super::haberdasher::EstablishClientRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::haberdasher::Empty>> {
        self.client.unary_call_async(&METHOD_AGENT_SUBSCRIBER_ESTABLISH_CLIENT, req, opt)
    }

    pub fn establish_client_async(&self, req: &super::haberdasher::EstablishClientRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::haberdasher::Empty>> {
        self.establish_client_async_opt(req, ::grpcio::CallOption::default())
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
    fn establish_client(&mut self, ctx: ::grpcio::RpcContext, req: super::haberdasher::EstablishClientRequest, sink: ::grpcio::UnarySink<super::haberdasher::Empty>);
    fn handle_agent_requests(&mut self, ctx: ::grpcio::RpcContext, stream: ::grpcio::RequestStream<super::haberdasher::AgentResponse>, sink: ::grpcio::DuplexSink<super::haberdasher::AgentRequest>);
    fn publish_venue_updates(&mut self, ctx: ::grpcio::RpcContext, stream: ::grpcio::RequestStream<super::haberdasher::Venue>, sink: ::grpcio::ClientStreamingSink<super::haberdasher::Empty>);
}

pub fn create_agent_subscriber<S: AgentSubscriber + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_AGENT_SUBSCRIBER_ESTABLISH_CLIENT, move |ctx, req, resp| {
        instance.establish_client(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_duplex_streaming_handler(&METHOD_AGENT_SUBSCRIBER_HANDLE_AGENT_REQUESTS, move |ctx, req, resp| {
        instance.handle_agent_requests(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_client_streaming_handler(&METHOD_AGENT_SUBSCRIBER_PUBLISH_VENUE_UPDATES, move |ctx, req, resp| {
        instance.publish_venue_updates(ctx, req, resp)
    });
    builder.build()
}
