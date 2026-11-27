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

    /// How many ids after `base` this one was made: negative when it is older.
    pub fn since(self, base: Uid) -> i64 {
        self.0 as i64 - base.0 as i64
    }
}

impl std::str::FromStr for Uid {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self)
    }
}

impl std::fmt::Display for Uid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
