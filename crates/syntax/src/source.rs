/// The text of a program, as a reader typed it or an example carries it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Source(String);

impl Source {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for Source {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for Source {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
