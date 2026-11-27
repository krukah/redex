use crate::*;

/// SVG path data: commands in order. `spaced` puts a space between commands (`M0 0 L1 1`), as the
/// edges are written; marks run them together (`M0 0L1 1Z`).
#[derive(Debug, Clone, PartialEq)]
pub struct Trace {
    segs: Vec<Seg>,
    spaced: bool,
}

impl Trace {
    pub fn spaced(segs: Vec<Seg>) -> Self {
        Self { segs, spaced: true }
    }

    pub fn tight(segs: Vec<Seg>) -> Self {
        Self { segs, spaced: false }
    }
}

impl std::fmt::Display for Trace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.segs
            .iter()
            .enumerate()
            .try_for_each(|(i, s)| if i > 0 && self.spaced { write!(f, " {s}") } else { write!(f, "{s}") })
    }
}
