/// A rule's pattern variable, as its schema writes it between braces: `x` in `{x}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Var(&'static str);

impl Var {
    pub const fn new(s: &'static str) -> Self {
        Self(s)
    }
}

impl std::fmt::Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}
