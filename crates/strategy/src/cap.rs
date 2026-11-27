/// The most steps a run takes, or the most terms an exploration keeps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cap(usize);

impl Cap {
    pub const fn new(n: usize) -> Self {
        Self(n)
    }

    pub fn get(self) -> usize {
        self.0
    }
}

impl std::fmt::Display for Cap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
