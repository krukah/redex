use crate::*;

/// One command of SVG path data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Seg {
    Move(Num, Num),
    Line(Num, Num),
    Quad(Num, Num, Num, Num),
    Cubic(Num, Num, Num, Num, Num, Num),
    /// `Z`.
    Close,
    /// `z`, as the arrowheads write it.
    End,
}

impl std::fmt::Display for Seg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Move(x, y) => write!(f, "M{x} {y}"),
            Self::Line(x, y) => write!(f, "L{x} {y}"),
            Self::Quad(a, b, x, y) => write!(f, "Q{a} {b} {x} {y}"),
            Self::Cubic(a, b, c, d, x, y) => write!(f, "C{a} {b} {c} {d} {x} {y}"),
            Self::Close => f.write_str("Z"),
            Self::End => f.write_str("z"),
        }
    }
}
