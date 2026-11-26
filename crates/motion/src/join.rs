use std::future::Future;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

/// A future that resolves when every one of its futures has, like `Promise.all`.
pub struct Join(Vec<Option<Pin<Box<dyn Future<Output = ()>>>>>);

impl Join {
    pub fn new(futures: Vec<Pin<Box<dyn Future<Output = ()>>>>) -> Self {
        Self(futures.into_iter().map(Some).collect())
    }
}

impl Future for Join {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        self.0.iter_mut().for_each(|slot| {
            if slot.as_mut().is_some_and(|f| f.as_mut().poll(cx).is_ready()) {
                *slot = None;
            }
        });
        if self.0.iter().all(Option::is_none) { Poll::Ready(()) } else { Poll::Pending }
    }
}
