use units::*;

use crate::*;

/// Where a vertex sits in the plot, and its mark.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spot {
    x: Px,
    y: Px,
    form: Form,
}

impl Spot {
    pub fn new(x: Px, y: Px, form: Form) -> Self {
        Self { x, y, form }
    }

    pub fn x(&self) -> Px {
        self.x
    }

    pub fn y(&self) -> Px {
        self.y
    }

    pub fn form(&self) -> Form {
        self.form
    }
}
