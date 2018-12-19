use actix::prelude::*;
use clacks_rpc::client;
use futures::{sync, unsync};
use slog::Logger;

pub struct AuthCodeReader {
    log: Logger,
    responders: std::collections::VecDeque<unsync::oneshot::Sender<String>>,
}

impl AuthCodeReader {
    pub fn from_context(ctx: &mut Context<Self>, log: Logger, lines: impl futures::Stream<Item = String, Error = std::io::Error> + 'static) -> Self {
        ctx.add_stream(lines);
        AuthCodeReader {
            log,
            responders: Default::default(),
        }
    }
}

impl Actor for AuthCodeReader {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!(self.log, "started");
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        info!(self.log, "stopped");
    }
}

impl StreamHandler<String, std::io::Error> for AuthCodeReader {
    fn handle(&mut self, line: String, ctx: &mut Context<Self>) {
        info!(self.log, "got a line"; "line" => ?line);
        if let Some(sender) = self.responders.pop_front() {
            let _ = sender.send(line);
        }
    }

    fn error(&mut self, err: std::io::Error, ctx: &mut Self::Context) -> Running {
        error!(self.log, "stream error"; "err" => ?err);
        Running::Stop
    }

    fn finished(&mut self, ctx: &mut Self::Context) {
        info!(self.log, "finished");
    }
}

async_handler!(fn handle()(this: AuthCodeReader, req: client::ReadAuthCode, ctx) -> client::AuthCodeReply {
    let (tx, rx) = unsync::oneshot::channel();
    this.responders.push_back(tx);
    info!(this.log, "auth code request; type the code or 'resend' or 'cancel'"; "req" => ?req);
    Ok(async move {
        let line = await!(rx)?;
        match line.as_str() {
            "resend" => Ok(client::AuthCodeReply::Resend),
            "cancel" => Ok(client::AuthCodeReply::Cancel),
            _ => Ok(client::AuthCodeReply::Code(line)),
        }
    })
});
