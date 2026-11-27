/// A `--` comment line of a program: the caption the workbench shows above the term.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note(String);

impl Note {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for Note {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
