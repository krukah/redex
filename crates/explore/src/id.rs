/// A vertex of a reduction graph, by its place in exploration order. The start is `Id::ROOT`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id(usize);

impl Id {
    pub const ROOT: Id = Id(0);

    pub fn get(self) -> usize {
        self.0
    }
}

impl From<usize> for Id {
    fn from(i: usize) -> Self {
        Self(i)
    }
}

impl std::str::FromStr for Id {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self)
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
