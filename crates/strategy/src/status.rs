/// How a run ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// No redex left for the strategy.
    Done,
    /// The term grew past 1500 nodes.
    Big,
    /// The step cap ran out with a redex still there.
    Cap,
}

impl Status {
    pub fn name(self) -> &'static str {
        match self {
            Self::Done => "done",
            Self::Big => "big",
            Self::Cap => "cap",
        }
    }
}
