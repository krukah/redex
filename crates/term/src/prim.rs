/// A built-in constant: the constructors of `Nat` and its recursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Prim {
    Zero,
    Succ,
    Rec,
}

impl Prim {
    pub const ALL: [Prim; 3] = [Self::Zero, Self::Succ, Self::Rec];

    pub fn name(self) -> &'static str {
        match self {
            Self::Zero => "zero",
            Self::Succ => "succ",
            Self::Rec => "Nat.rec",
        }
    }

    pub fn find(name: &str) -> Option<Prim> {
        Self::ALL.into_iter().find(|p| p.name() == name)
    }

    /// Lean's full names for the constructors: `Nat.zero`, `Nat.succ`.
    pub fn alias(name: &str) -> Option<Prim> {
        match name {
            "Nat.zero" => Some(Self::Zero),
            "Nat.succ" => Some(Self::Succ),
            _ => None,
        }
    }
}
