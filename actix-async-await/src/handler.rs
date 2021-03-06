use std::future::Future;

use ::actix::dev::{MessageResponse, ResponseChannel};
use ::actix::prelude::*;
use futures_core::future::LocalFutureObj;

use crate::prelude::*;

/// A specialized future object for async message handler
pub struct ResponseStdFuture<T> {
    inner: LocalFutureObj<'static, T>,
}

impl<F, T> From<F> for ResponseStdFuture<T>
where
    F: Future<Output = T> + 'static,
{
    fn from(fut: F) -> Self {
        ResponseStdFuture {
            inner: LocalFutureObj::new(Box::pin(fut)),
        }
    }
}

impl<A, M, T> MessageResponse<A, M> for ResponseStdFuture<T>
where
    A: Actor,
    A::Context: AsyncContext<A>,
    M: Message<Result = T>,
    M::Result: Send,
    T: 'static,
{
    fn handle<R: ResponseChannel<M>>(self, _: &mut A::Context, tx: Option<R>) {
        let fut = async move {
            let res = self.inner.await;
            if let Some(tx) = tx {
                tx.send(res);
            }
        };
        Arbiter::spawn_async(Box::pin(fut))
    }
}

impl<T> std::ops::Deref for ResponseStdFuture<T> {
    type Target = LocalFutureObj<'static, T>;
    fn deref(&self) -> &Self::Target { &self.inner }
}

impl<T> std::ops::DerefMut for ResponseStdFuture<T> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.inner }
}

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use futures_util::compat::*;

    use super::*;

    struct Truck;

    impl Actor for Truck {
        type Context = Context<Truck>;
    }

    struct Parcel(u32);

    impl Message for Parcel {
        type Result = String;
    }

    impl Handler<Parcel> for Truck {
        type Result = ResponseStdFuture<String>;

        fn handle(&mut self, msg: Parcel, _: &mut Self::Context) -> Self::Result {
            let result: Rc<str> = format!("[{}]", msg.0 * 2).into();
            ResponseStdFuture::from(async move { String::from(&*result) })
        }
    }

    actix_test_cases! {
        async fn basic() -> Result<(), MailboxError> {
            let truck = Truck.start();
            let res = await!(truck.send(Parcel(42)).compat())?;
            assert_eq!(res, "[84]");
            Ok(())
        }
    }
}
