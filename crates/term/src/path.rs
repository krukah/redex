/// A position in a term: one letter per step down (`f` `a` `b` `v` `t`), `""` for the root.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Path(String);

impl Path {
    pub fn root() -> Self {
        Self::default()
    }

    pub fn child(&self, c: char) -> Self {
        Self(format!("{}{c}", self.0))
    }

    pub fn join(&self, rel: &Path) -> Self {
        Self(format!("{}{}", self.0, rel.0))
    }

    pub fn steps(&self) -> impl Iterator<Item = char> + '_ {
        self.0.chars()
    }

    /// Drops the last step.
    pub fn parent(&self) -> Self {
        Self(self.0.chars().take(self.0.chars().count().saturating_sub(1)).collect())
    }
}

impl std::str::FromStr for Path {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl From<&str> for Path {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for Path {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
