/// A highlighter color: the position (from 1) of a pattern variable in its rule's schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Slot(usize);

impl Slot {
    pub const ALL: [Slot; 4] = [Slot(1), Slot(2), Slot(3), Slot(4)];

    pub fn at(index: usize) -> Self {
        Self(index + 1)
    }
}

impl std::fmt::Display for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
