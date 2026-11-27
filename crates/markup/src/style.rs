use explore::*;
use rules::*;

/// An inline style the widgets write.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    /// A deck tile spans this many grid rows.
    Span(usize),
    /// A graph mark or edge sits on this row, for the row-by-row entrance.
    Row(Depth),
    /// An arrowhead takes its rule's ink.
    Fill(Kind),
}

impl std::fmt::Display for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Span(n) => write!(f, "grid-row: span {n}"),
            Self::Row(d) => write!(f, "--d:{d}"),
            Self::Fill(k) => write!(f, "fill:var(--rdx-{k})"),
        }
    }
}
