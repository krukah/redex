use crate::*;

/// Where a vertex sits in the plot, and its mark.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spot {
    x: f64,
    y: f64,
    form: Form,
}

impl Spot {
    pub fn new(x: f64, y: f64, form: Form) -> Self {
        Self { x, y, form }
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn form(&self) -> Form {
        self.form
    }
}
