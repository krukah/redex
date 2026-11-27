/// The value of a numeral: a chain of `succ` ending in `zero`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Nat(usize);

impl Nat {
    pub const ZERO: Nat = Nat(0);
    /// The largest numeral the parser takes: each one is a chain of `succ`.
    pub const MAX: Nat = Nat(40);

    pub const fn new(n: usize) -> Self {
        Self(n)
    }

    pub fn get(self) -> usize {
        self.0
    }

    pub fn succ(self) -> Self {
        Self(self.0 + 1)
    }
}

impl std::fmt::Display for Nat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
