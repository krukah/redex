use std::cell::Cell;

/// A node's identity. One per-thread counter numbers every node ever built, in build order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Uid(u64);

impl Uid {
    pub fn next() -> Self {
        thread_local! {
            static COUNTER: Cell<u64> = const { Cell::new(0) };
        }
        COUNTER.with(|c| {
            c.set(c.get() + 1);
            Self(c.get())
        })
    }

    pub fn get(self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for Uid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
