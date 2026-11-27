use units::*;

/// A number in an SVG attribute or path, with the JavaScript formatting the fixtures record.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Num {
    /// `String(x)`.
    Js(Px),
    /// `x.toFixed(1)`.
    Tenths(Px),
}

impl std::fmt::Display for Num {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Js(p) => write!(f, "{p}"),
            Self::Tenths(p) => f.write_str(&p.tenths()),
        }
    }
}
