use std::future::Future;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

/// A future that resolves when the first of its two futures does, like `Promise.race`.
pub struct First(Pin<Box<dyn Future<Output = ()>>>, Pin<Box<dyn Future<Output = ()>>>);

impl First {
    pub fn new(a: Pin<Box<dyn Future<Output = ()>>>, b: Pin<Box<dyn Future<Output = ()>>>) -> Self {
        Self(a, b)
    }
}

impl Future for First {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let Self(a, b) = &mut *self;
        if a.as_mut().poll(cx).is_ready() || b.as_mut().poll(cx).is_ready() {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}
