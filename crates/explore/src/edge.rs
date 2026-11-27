use rules::*;

use crate::*;

/// One step between two vertices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edge {
    from: Id,
    to: Id,
    kind: Kind,
}

impl Edge {
    pub fn new(from: Id, to: Id, kind: Kind) -> Self {
        Self { from, to, kind }
    }

    pub fn from(&self) -> Id {
        self.from
    }

    pub fn to(&self) -> Id {
        self.to
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }

    /// Whether the edge joins these two vertices, in this direction.
    pub fn joins(&self, from: Id, to: Id) -> bool {
        self.from() == from && self.to() == to
    }
}
