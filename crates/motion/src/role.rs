/// A part the move phase plays, and its fixed window inside the phase, as fractions of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// The rest of the redex fades out where it stands.
    Fade,
    /// A bound piece travels in a straight line to a place the result uses it.
    Fly,
    /// The piece's two copies (as it was, as it lands) crossfade while it travels.
    Swap,
    /// The result fades in where the redex was.
    Arrive,
    /// The context slides and resizes from the old layout to the new one.
    Glide,
}

impl Role {
    pub const ALL: [Role; 5] = [Self::Fade, Self::Fly, Self::Swap, Self::Arrive, Self::Glide];

    /// Where the part starts and how long it runs, as fractions of the move phase.
    pub fn window(self) -> (f64, f64) {
        match self {
            Self::Fade => (0.0, 0.55),
            Self::Swap => (0.0, 0.45),
            Self::Fly | Self::Arrive | Self::Glide => (0.0, 1.0),
        }
    }
}
