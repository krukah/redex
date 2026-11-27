use std::ops::Add;

/// A count of reduction steps, or a position in a sequence of them: step 0 is the start term.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Step(usize);

impl Step {
    pub const START: Step = Step(0);
    pub const ONE: Step = Step(1);

    pub const fn new(n: usize) -> Self {
        Self(n)
    }

    pub fn get(self) -> usize {
        self.0
    }

    pub fn next(self) -> Self {
        Self(self.0 + 1)
    }

    /// The step before, if this is not the start.
    pub fn back(self) -> Option<Self> {
        self.0.checked_sub(1).map(Self)
    }
}

impl Add<usize> for Step {
    type Output = Step;

    fn add(self, n: usize) -> Step {
        Step(self.0 + n)
    }
}

impl std::str::FromStr for Step {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self)
    }
}

impl std::fmt::Display for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
