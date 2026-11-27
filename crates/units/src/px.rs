use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Neg;
use std::ops::Sub;

use crate::*;

/// A length or coordinate in CSS pixels. It prints as JavaScript prints a number.
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct Px(f64);

impl Px {
    pub const ZERO: Px = Px(0.0);

    pub const fn new(v: f64) -> Self {
        Self(v)
    }

    pub fn get(self) -> f64 {
        self.0
    }

    pub fn abs(self) -> Self {
        Self(self.0.abs())
    }

    pub fn max(self, other: Px) -> Self {
        Self(self.0.max(other.0))
    }

    pub fn min(self, other: Px) -> Self {
        Self(self.0.min(other.0))
    }

    pub fn midpoint(self, other: Px) -> Self {
        Self(f64::midpoint(self.0, other.0))
    }

    /// `Math.sign(x) || 1`: the direction from zero, rightward when there is none.
    pub fn sign(self) -> f64 {
        if self.0 == 0.0 || self.0.is_nan() { 1.0 } else { self.0.signum() }
    }

    /// `x.toFixed(1)`.
    pub fn tenths(self) -> String {
        Number::fixed(self.0, 1)
    }

    /// Smaller than half a pixel either way: not worth moving.
    pub fn still(self) -> bool {
        self.0.abs() < 0.5
    }
}

impl Add for Px {
    type Output = Px;

    fn add(self, o: Px) -> Px {
        Px(self.0 + o.0)
    }
}

impl Sub for Px {
    type Output = Px;

    fn sub(self, o: Px) -> Px {
        Px(self.0 - o.0)
    }
}

impl Mul<f64> for Px {
    type Output = Px;

    fn mul(self, k: f64) -> Px {
        Px(self.0 * k)
    }
}

impl Div<f64> for Px {
    type Output = Px;

    fn div(self, k: f64) -> Px {
        Px(self.0 / k)
    }
}

impl Div for Px {
    type Output = f64;

    fn div(self, o: Px) -> f64 {
        self.0 / o.0
    }
}

impl Neg for Px {
    type Output = Px;

    fn neg(self) -> Px {
        Px(-self.0)
    }
}

impl std::fmt::Display for Px {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&Number::show(self.0))
    }
}
