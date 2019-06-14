use backoff::backoff::Backoff;
use futures::stream::{Stream, unfold};
use futures_timer::{Delay, TimerHandle};
use std::future::Future;
use std::time::Instant;

pub struct BackedOff<'a, F> {
    initial: bool,
    create: &'a mut dyn FnMut() -> F,
    backoff: &'a mut dyn Backoff,
}

impl<'a, F: Future> BackedOff<'a, F> {
    pub fn new(create: &'a mut dyn FnMut() -> F, backoff: &'a mut dyn Backoff) -> Self {
        BackedOff { initial: true, create, backoff }
    }

    pub async fn next(&mut self) -> Option<F::Output> {
        if self.initial {
            self.initial = false;
        } else {
            let delay = self.backoff.next_backoff()?;
            Delay::new(delay).await.ok()?;
        }
        Some((self.create)().await)
    }
}
