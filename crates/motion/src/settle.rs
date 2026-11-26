use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::Context;
use std::task::Poll;
use std::task::Waker;

use wasm_bindgen::prelude::*;

/// A future that resolves when a promise settles, either way.
pub struct Settle {
    state: Rc<RefCell<(bool, Option<Waker>)>>,
    _done: Closure<dyn FnMut(JsValue)>,
}

impl Settle {
    pub fn new(promise: &js_sys::Promise) -> Self {
        let state = Rc::new(RefCell::new((false, None::<Waker>)));
        let shared = state.clone();
        let done = Closure::<dyn FnMut(JsValue)>::new(move |_| {
            let mut s = shared.borrow_mut();
            s.0 = true;
            if let Some(w) = s.1.take() {
                w.wake();
            }
        });
        let _ = promise.then2(&done, &done);
        Self { state, _done: done }
    }
}

impl Future for Settle {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let mut s = self.state.borrow_mut();
        if s.0 {
            Poll::Ready(())
        } else {
            s.1 = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
