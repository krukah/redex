use units::*;

/// One phase of a step. Every step, of every rule and every size, is the same three phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// The redex is matched: its pieces painted, the rule card showing what each variable bound.
    Hold,
    /// Every piece flies to where the result uses it, the rest of the redex fades, the result
    /// fades in and the context glides; all of it starts together and ends with the phase.
    Move,
    /// Everything is at rest before the next step.
    Settle,
}

impl Phase {
    pub const ALL: [Phase; 3] = [Self::Hold, Self::Move, Self::Settle];

    /// The phase's length at speed 1.
    pub fn ms(self) -> Ms {
        match self {
            Self::Hold => Ms::new(560.0),
            Self::Move => Ms::new(620.0),
            Self::Settle => Ms::new(240.0),
        }
    }
}
