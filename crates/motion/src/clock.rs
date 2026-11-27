use units::*;

/// The speed a motion plays at: durations and delays are written at speed 1 and divided by it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Clock(f64);

impl Clock {
    pub const fn new(speed: f64) -> Self {
        Self(speed)
    }

    /// A speed of zero or NaN plays at 1, as `clock?.speed || 1` does.
    fn speed(self) -> f64 {
        if self.0 == 0.0 || self.0.is_nan() { 1.0 } else { self.0 }
    }

    pub fn scale(self, ms: Ms) -> Ms {
        ms / self.speed()
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self(1.0)
    }
}
