use std::iter::Sum;
use std::ops::Add;

/// A term's size: its number of nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Size(usize);

impl Size {
    pub const ONE: Size = Size(1);

    pub const fn new(n: usize) -> Self {
        Self(n)
    }

    pub fn get(self) -> usize {
        self.0
    }
}

impl Add for Size {
    type Output = Size;

    fn add(self, o: Size) -> Size {
        Size(self.0 + o.0)
    }
}

impl Sum for Size {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Size>,
    {
        iter.fold(Size(0), Add::add)
    }
}

impl std::fmt::Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
