/// Why a source did not parse, in words a reader can act on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure(String);

impl Failure {
    pub fn new(msg: String) -> Self {
        Self(msg)
    }

    /// The same failure, placed on a line of the source.
    pub fn at(self, line: usize) -> Self {
        Self(format!("Line {line}: {}", self.0))
    }
}

impl From<&str> for Failure {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
