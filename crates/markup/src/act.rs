/// What a button does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Act {
    Step,
    Whnf,
    Nf,
    Back,
    Reset,
    Load,
    Rebuild,
    Open,
    Race,
}

impl Act {
    const ALL: [Act; 9] = [
        Self::Step,
        Self::Whnf,
        Self::Nf,
        Self::Back,
        Self::Reset,
        Self::Load,
        Self::Rebuild,
        Self::Open,
        Self::Race,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::Step => "step",
            Self::Whnf => "whnf",
            Self::Nf => "nf",
            Self::Back => "back",
            Self::Reset => "reset",
            Self::Load => "load",
            Self::Rebuild => "rebuild",
            Self::Open => "open",
            Self::Race => "race",
        }
    }
}

impl std::str::FromStr for Act {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL.into_iter().find(|a| a.name() == s).ok_or(())
    }
}

impl std::fmt::Display for Act {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}
