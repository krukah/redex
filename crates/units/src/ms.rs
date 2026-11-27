use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;

/// A duration in milliseconds.
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct Ms(f64);

impl Ms {
    pub const ZERO: Ms = Ms(0.0);

    pub const fn new(v: f64) -> Self {
        Self(v)
    }

    pub fn get(self) -> f64 {
        self.0
    }

    /// Whole milliseconds, as `setTimeout` truncates them; negative waits are none.
    pub fn whole(self) -> u32 {
        if self.0 > 0.0 { self.0 as u32 } else { 0 }
    }
}

impl Add for Ms {
    type Output = Ms;

    fn add(self, o: Ms) -> Ms {
        Ms(self.0 + o.0)
    }
}

impl Mul<f64> for Ms {
    type Output = Ms;

    fn mul(self, k: f64) -> Ms {
        Ms(self.0 * k)
    }
}

impl Div<f64> for Ms {
    type Output = Ms;

    fn div(self, k: f64) -> Ms {
        Ms(self.0 / k)
    }
}

impl Sub for Ms {
    type Output = Ms;

    fn sub(self, o: Ms) -> Ms {
        Ms(self.0 - o.0)
    }
}
