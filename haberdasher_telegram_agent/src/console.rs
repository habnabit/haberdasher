use actix::prelude::*;
use actix_async_await::AsyncContextExt;
use clacks_rpc::client;
use futures::channel::oneshot;
use slog::Logger;

use crate::Result;

pub struct AuthCodeReader {
    log: Logger,
    responders: std::collections::VecDeque<oneshot::Sender<String>>,
}

impl AuthCodeReader {
    pub fn from_context(ctx: &mut Context<Self>, log: Logger, lines: impl futures::Stream<Item = Result<String>> + Unpin + 'static) -> Self {
        ctx.add_stream_03(lines);
        AuthCodeReader {
            log,
            responders: Default::default(),
        }
    }
}

impl Actor for AuthCodeReader {
    type Context = Context<Self>;
}

impl StreamHandler<String, failure::Error> for AuthCodeReader {
    fn handle(&mut self, line: String, _: &mut Context<Self>) {
        if let Some(sender) = self.responders.pop_front() {
            let _ = sender.send(line);
        }
    }
}

async_handler!(fn handle()(this: AuthCodeReader, req: client::ReadAuthCode, _ctx) -> client::AuthCodeReply {
    let (tx, rx) = oneshot::channel();
    this.responders.push_back(tx);
    info!(this.log, "auth code request; type the code or 'resend' or 'cancel'"; "req" => ?req);
    Ok(async move {
        let line = rx.await?;
        match line.as_str() {
            "resend" => Ok(client::AuthCodeReply::Resend),
            "cancel" => Ok(client::AuthCodeReply::Cancel),
            _ => Ok(client::AuthCodeReply::Code(line)),
        }
    })
});
