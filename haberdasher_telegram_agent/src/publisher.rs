use actix_async_await::AsyncContextExt;
use futures::compat::*;
use futures::prelude::*;
use haberdasher_rpc::haberdasher as protos;
use haberdasher_rpc::haberdasher_grpc as rpc;
use actix::prelude::*;

use crate::Result;
use crate::backoff::BackedOff;

pub struct PublishDriver {
    channel: grpcio::Channel,
    done: bool,
}

fn remake_channels(channel: grpcio::Channel, access_token: String) -> impl FnMut() -> grpcio::Result<(grpcio::ClientCStreamSender<protos::Venue>, grpcio::ClientCStreamReceiver<protos::Empty>)> {
    let agent_client = rpc::AgentSubscriberClient::new(channel);
    move || {
        let mut builder = grpcio::MetadataBuilder::new();
        builder.add_str("access-token", &access_token)?;
        let call_opts = <grpcio::CallOption as Default>::default()
            .headers(builder.build());
        agent_client.publish_venue_updates_opt(call_opts)
    }
}

impl PublishDriver {
    pub fn from_context(ctx: &mut Context<Self>, access_token: String, venues: impl futures::Stream<Item = protos::Venue> + 'static) -> Result<Self> {
        let env = std::sync::Arc::new(grpcio::EnvBuilder::new().build());
        let channel = grpcio::ChannelBuilder::new(env)
            .connect("127.0.0.1:42253");
        let mut remaker = remake_channels(channel.clone(), access_token);
        let (mut done_tx, done_rx) = futures::channel::mpsc::channel(5);
        let stream_done = async move {
            use futures::select;
            let mut remaker = move || futures::future::ready(remaker());
            let mut backoff = backoff::ExponentialBackoff::default();
            let mut backed_off = BackedOff::new(&mut remaker, &mut backoff);
            let mut venues = Box::pin(venues.map(|v| (v, Default::default())));
            let result: grpcio::Result<_> = loop {
                let (venue_tx, empty_rx) = match backed_off.next().await {
                    Some(Ok(x)) => x,
                    Some(Err(e)) => {
                        info!(slog_scope::logger(), "stream broke on remake"; "error" => ?e);
                        continue;
                    },
                    None => break Ok("retries exhausted"),
                };
                let mut venue_tx = venue_tx.sink_compat();
                let mut sent_all = venue_tx.send_all(&mut venues).fuse();
                let mut empty_done = empty_rx.compat().fuse();
                let result = select! {
                    r = sent_all => r.map(|()| "sent_all"),
                    r = empty_done => r.map(|protos::Empty {}| "empty_done"),
                };
                info!(slog_scope::logger(), "stream broke on venue_incoming"; "result" => ?result);
            };
            info!(slog_scope::logger(), "stream finished on venue_incoming"; "result" => ?result);
            let _ = done_tx.send(PublishingDone).await;
            Ok(())
        };
        let actor = PublishDriver { channel, done: false };
        let _ = ctx.spawn(actix::fut::wrap_future(Compat::new(Box::pin(stream_done))));
        ctx.add_message_stream_03(done_rx);
        Ok(actor)
    }
}

impl Actor for PublishDriver {
    type Context = Context<Self>;
}

struct PublishingDone;

impl Message for PublishingDone {
    type Result = ();
}

impl Handler<PublishingDone> for PublishDriver {
    type Result = ();
    fn handle(&mut self, _: PublishingDone, ctx: &mut Context<Self>) {
        info!(slog_scope::logger(), "oh im done");
        self.done = true;
    }
}
