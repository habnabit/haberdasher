use actix_async_await::AsyncContextExt;
use futures::prelude::*;
use haberdasher_rpc::haberdasher as protos;
use haberdasher_rpc::haberdasher_grpc as rpc;
use serde::Deserialize;
use tokio::prelude::*;
use actix::prelude::*;
use clacks_rpc::client;
use slog::Logger;

use crate::Result;

struct PublishStarter {
    log: Logger,
    channel: grpcio::Channel,
    access_token: String,
}

impl PublishStarter {
    pub fn from_context(ctx: &mut Context<Self>, log: Logger, access_token: String, venues: impl futures::Stream<Item = protos::Venue> + Unpin + 'static) -> Self {
        let env = std::sync::Arc::new(grpcio::EnvBuilder::new().build());
        let channel = grpcio::ChannelBuilder::new(env)
            .connect("127.0.0.1:42253");
        //ctx.add_stream_03(venues.map(Ok));
        Self { log, channel, access_token }
    }

    // fn poll_errorful(&mut self) -> Poll<(), Error> {
    //     let mut ret = Async::NotReady;
    //     if self.buffered.is_none() {
    //         match unready!(ret, self.venue_rx.poll()) {
    //             Ok(Async::Ready(Some(x))) => {
    //                 self.buffered = Some(x);
    //             }
    //             Ok(Async::NotReady) => {}
    //             Ok(Async::Ready(None)) |
    //             Err(()) => unreachable!(),
    //         }
    //     }
    //     {
    //         let PublishDriver { channel, access_token, buffered, .. } = self;
    //         let inst = || {
    //             let mut builder = grpcio::MetadataBuilder::new();
    //             builder.add_str("access-token", &access_token.clone())?;
    //             let agent_client = rpc::AgentSubscriberClient::new(channel.clone());
    //             let call_opts = <grpcio::CallOption as Default>::default()
    //                 .headers(builder.build());
    //             let (venue_tx, empty_rx) = agent_client.publish_venue_updates_opt(call_opts)?;
    //             Ok(VenuePublisher { venue_tx, empty_rx })
    //         };
    //         let _: Async<()> = unready!(ret, self.venue_tx.poll_loop(&inst, buffered))?;
    //     }
    //     Ok(ret)
    // }
}

impl Actor for PublishStarter {
    type Context = Context<Self>;
}

impl StreamHandler<protos::Venue, ()> for PublishStarter {
    fn handle(&mut self, venue: protos::Venue, _: &mut Context<Self>) {
    }
}

struct PublishDriver {
    venue_tx: grpcio::StreamingCallSink<protos::Venue>,
    empty_rx: grpcio::ClientCStreamReceiver<protos::Empty>,
}

impl PublishDriver {
    fn new(access_token: String, venues: impl futures::Stream<Item = protos::Venue> + 'static) -> Result<()> {
        // let env = std::sync::Arc::new(grpcio::EnvBuilder::new().build());
        // let channel = grpcio::ChannelBuilder::new(env)
        //     .connect("127.0.0.1:42253");
        // let mut builder = grpcio::MetadataBuilder::new();
        // builder.add_str("access-token", &access_token)?;
        // let agent_client = rpc::AgentSubscriberClient::new(channel.clone());
        // let call_opts = <grpcio::CallOption as Default>::default()
        //     .headers(builder.build());
        // let (venue_tx, empty_rx) = agent_client.publish_venue_updates_opt(call_opts)?;
        // let stream_done = venues
        //     .then(|r| match r {
        //         Ok(v) => Ok::<_, failure::Error>((v, Default::default())),
        //         Err(_) => unreachable!(),
        //     })
        //     .forward(venue_tx.sink_from_err::<failure::Error>());
        Ok(())
    }
}
