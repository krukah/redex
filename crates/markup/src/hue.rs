/// A binder's color, from its name: one of six hues.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hue(u32);

impl Hue {
    const COUNT: u64 = 6;

    /// The JS hash: `h = (h * 31 + codePoint) >>> 0` from 7, modulo six.
    pub fn of(name: &term::Name) -> Self {
        Self((name.as_str().chars().fold(7u64, |h, c| (h * 31 + c as u64) % (1 << 32)) % Self::COUNT) as u32)
    }
}

impl std::fmt::Display for Hue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
