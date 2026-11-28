use crate::*;

/// A part a step plays after its hold, the phase it belongs to, and its fixed window there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// The rest of the redex, and the old binding links, fade out where they stand.
    Fade,
    /// A bound piece travels in a straight line to a place the result uses it.
    Fly,
    /// The piece's two copies (as it was, as it lands) crossfade while it travels.
    Swap,
    /// The result fades in where the redex was.
    Arrive,
    /// The context slides and resizes from the old layout to the new one.
    Glide,
    /// The binding links, drawn for the new layout, fade in once nothing moves.
    Relink,
}

impl Role {
    pub const ALL: [Role; 6] = [
        Self::Fade,
        Self::Fly,
        Self::Swap,
        Self::Arrive,
        Self::Glide,
        Self::Relink,
    ];

    pub fn phase(self) -> Phase {
        match self {
            Self::Relink => Phase::Settle,
            _ => Phase::Move,
        }
    }

    /// Where the part starts and how long it runs, as fractions of its phase.
    pub fn window(self) -> (f64, f64) {
        match self {
            Self::Fade => (0.0, 0.55),
            Self::Swap => (0.0, 0.45),
            Self::Fly | Self::Arrive | Self::Glide | Self::Relink => (0.0, 1.0),
        }
    }
}
