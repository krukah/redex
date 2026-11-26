use rules::*;

/// One step between two vertices, by index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edge {
    from: usize,
    to: usize,
    kind: Kind,
}

impl Edge {
    pub fn new(from: usize, to: usize, kind: Kind) -> Self {
        Self { from, to, kind }
    }

    pub fn from(&self) -> usize {
        self.from
    }

    pub fn to(&self) -> usize {
        self.to
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }
}
