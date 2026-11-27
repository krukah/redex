/// How many steps a vertex lies from the start on a shortest path: its row in the graph.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Depth(usize);

impl Depth {
    pub const START: Depth = Depth(0);

    pub const fn new(n: usize) -> Self {
        Self(n)
    }

    pub fn get(self) -> usize {
        self.0
    }

    pub fn next(self) -> Self {
        Self(self.0 + 1)
    }

    /// One row further down than `other`.
    pub fn below(self, other: Depth) -> bool {
        self.0 == other.0 + 1
    }
}

impl std::fmt::Display for Depth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
