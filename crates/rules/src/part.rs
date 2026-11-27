use crate::*;

/// A piece of a schema: literal text, or a pattern variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    Text(&'static str),
    Var(Var),
}
