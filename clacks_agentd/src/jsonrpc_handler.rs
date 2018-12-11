use actix::prelude::*;
use clacks_mtproto::mtproto;
use failure::Error;
use futures::prelude::*;
use futures::future;
use jsonrpc_core::{self, MetaIoHandler, NoopMiddleware};
use slog::Logger;
use std::sync::Mutex;


pub struct JsonRpcHandlerActor {
    handler: MetaIoHandler<RpcMetadata, NoopMiddleware>,
}

impl JsonRpcHandlerActor {
    pub fn new(log: Logger, tg_manager: Addr<::tg_manager::TelegramManagerActor>) -> Self {
		use self::rpc_trait::Rpc;
		let mut handler = MetaIoHandler::default();
		handler.extend_with(RpcImpl::new(tg_manager).to_delegate());
		JsonRpcHandlerActor {
			handler,
		}
    }
}

impl Actor for JsonRpcHandlerActor {
    type Context = Context<Self>;
}

pub struct HandleRequest {
    pub request: jsonrpc_core::Request,
}

impl Message for HandleRequest {
    type Result = Result<Option<jsonrpc_core::Response>, ()>;
}

impl Handler<HandleRequest> for JsonRpcHandlerActor {
    type Result = ResponseFuture<Option<jsonrpc_core::Response>, ()>;

    fn handle(&mut self, request: HandleRequest, ctx: &mut Self::Context) -> Self::Result {
		Box::new({
			self.handler.handle_rpc_request(request.request, RpcMetadata(ctx.address()))
		})
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendRawFailure;
type JsonRpcResponseFuture<T> = Box<Future<Item = T, Error = jsonrpc_core::Error> + Send>;

mod rpc_trait {
	use clacks_mtproto::mtproto::TLObject;
	use jsonrpc_core::Result;
	use jsonrpc_macros::pubsub::Subscriber;
	use jsonrpc_pubsub::SubscriptionId;
	use super::{JsonRpcResponseFuture, SendRawFailure};

	build_rpc_trait! {
		pub trait Rpc {
			type Metadata;

			#[rpc(name = "connect")]
			fn connect(&self, String) -> JsonRpcResponseFuture<()>;

			#[rpc(name = "hazmat.send_raw")]
			fn send_raw(&self, TLObject) -> JsonRpcResponseFuture<::std::result::Result<TLObject, SendRawFailure>>;

			#[pubsub(name = "unhandled")] {
				#[rpc(name = "unhandled_subscribe")]
				fn unhandled_subscribe(&self, Self::Metadata, Subscriber<String>);

				#[rpc(name = "unhandled_unsubscribe")]
				fn unhandled_unsubscribe(&self, SubscriptionId) -> JsonRpcResponseFuture<bool>;
			}
		}
	}
}

#[derive(Clone)]
struct RpcMetadata(Addr<JsonRpcHandlerActor>);

impl ::jsonrpc_core::Metadata for RpcMetadata {}
impl ::jsonrpc_pubsub::PubSubMetadata for RpcMetadata {
	fn session(&self) -> Option<::std::sync::Arc<::jsonrpc_pubsub::Session>> { None }
}

struct RpcImpl {
	tg_manager: Addr<::tg_manager::TelegramManagerActor>,
}

impl RpcImpl {
	fn new(tg_manager: Addr<::tg_manager::TelegramManagerActor>) -> Self {
		RpcImpl { tg_manager }
	}
}

impl rpc_trait::Rpc for RpcImpl {
	type Metadata = RpcMetadata;

	fn connect(&self, phone_number: String) -> JsonRpcResponseFuture<()> {
		Box::new({
			self.tg_manager.send(::tg_manager::Connect { phone_number })
				.map_err(|e| {
					let e: Error = e.into();
					println!("failed? {:?}", e);
					jsonrpc_core::Error::internal_error()
				})
		})
	}

	fn send_raw(&self, req: mtproto::TLObject) -> JsonRpcResponseFuture<::std::result::Result<mtproto::TLObject, SendRawFailure>>
	{
		Box::new({
			self.tg_manager.send(::clacks_rpc::client::SendMessage::encrypted(req))
				.map(|r| r.map_err(|e| {
					println!("failed? {:?}", e);
					SendRawFailure
				}))
				.map_err(|e| {
					println!("failed? {:?}", e);
					jsonrpc_core::Error::internal_error()
				})
		})
	}

	fn unhandled_subscribe(&self, meta: Self::Metadata, subscriber: ::jsonrpc_macros::pubsub::Subscriber<String>) {

	}

	fn unhandled_unsubscribe(&self, subscription: ::jsonrpc_pubsub::SubscriptionId) -> JsonRpcResponseFuture<bool> {
		Box::new(future::ok(false))
	}
}
