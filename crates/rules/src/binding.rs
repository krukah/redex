use term::*;

use crate::*;

/// One pattern variable of a rule and where it sits, relative to the redex. A binder binding names
/// the λ or let whose bound name it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    var: Var,
    rel: Path,
    binder: bool,
}

impl Binding {
    pub fn piece(var: &'static str, rel: &str) -> Self {
        Self {
            var: Var::new(var),
            rel: Path::from(rel),
            binder: false,
        }
    }

    pub fn binder(var: &'static str, rel: &str) -> Self {
        Self {
            var: Var::new(var),
            rel: Path::from(rel),
            binder: true,
        }
    }

    pub fn var(&self) -> Var {
        self.var
    }

    pub fn rel(&self) -> &Path {
        &self.rel
    }

    pub fn is_binder(&self) -> bool {
        self.binder
    }
}
