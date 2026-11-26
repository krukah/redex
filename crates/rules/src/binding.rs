use term::*;

/// One pattern variable of a rule and where it sits, relative to the redex. A binder binding names
/// the λ or let whose bound name it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    name: &'static str,
    rel: Path,
    binder: bool,
}

impl Binding {
    pub fn piece(name: &'static str, rel: &str) -> Self {
        Self {
            name,
            rel: Path::from(rel),
            binder: false,
        }
    }

    pub fn binder(name: &'static str, rel: &str) -> Self {
        Self {
            name,
            rel: Path::from(rel),
            binder: true,
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn rel(&self) -> &Path {
        &self.rel
    }

    pub fn is_binder(&self) -> bool {
        self.binder
    }
}
