/// A binder in a drawn term, numbered in drawing order from 1: what its variables' arcs point to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Anchor(usize);

impl Anchor {
    pub const NONE: Anchor = Anchor(0);

    pub fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

impl std::str::FromStr for Anchor {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self)
    }
}

impl std::fmt::Display for Anchor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
