use term::*;

use crate::*;

/// A redex: where it sits and which rule it is for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redex {
    path: Path,
    kind: Kind,
}

impl Redex {
    pub fn new(path: Path, kind: Kind) -> Self {
        Self { path, kind }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }
}
