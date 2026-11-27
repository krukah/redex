/// A rule's letter: α β η δ ζ ι π.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Glyph(char);

impl Glyph {
    pub const fn new(c: char) -> Self {
        Self(c)
    }
}

impl std::fmt::Display for Glyph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
