use explore::*;
use strategy::*;
use term::*;

/// A graph mark's shape: ○ reducible, ◇ weak head normal form, ■ normal form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Form {
    Red,
    Whnf,
    Nf,
}

impl Form {
    pub fn of(v: &Vertex, env: &Env) -> Self {
        match (v.nf(), Strategy::Whnf.next(v.term(), env)) {
            (true, _) => Self::Nf,
            (false, Some(_)) => Self::Red,
            (false, None) => Self::Whnf,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Red => "reducible",
            Self::Whnf => "weak head normal form",
            Self::Nf => "normal form",
        }
    }
}
