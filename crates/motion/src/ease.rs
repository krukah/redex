/// Easing curves, by role.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ease {
    /// Ease-in-out, a little quicker to start than to stop; for travel and layout.
    Move,
    /// Decelerate; things arriving.
    Enter,
    /// Ease-in-out over opacity; a fade that neither pops nor lingers.
    Exit,
}

impl std::fmt::Display for Ease {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Move => "cubic-bezier(.45, 0, .2, 1)",
            Self::Enter => "cubic-bezier(0, 0, .2, 1)",
            Self::Exit => "cubic-bezier(.4, 0, .6, 1)",
        })
    }
}
