/// A flash the renderer shows once on a node a rule just built.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    /// A variable an α step renamed (`fl-alpha`).
    Renamed,
    /// The binder an α step renamed (`alphaB`).
    Binder,
}

impl Mark {
    pub fn name(self) -> &'static str {
        match self {
            Self::Renamed => "fl-alpha",
            Self::Binder => "alphaB",
        }
    }
}
