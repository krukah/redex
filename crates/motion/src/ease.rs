/// Easing curves, by role.
pub struct Ease;

impl Ease {
    /// Ease-in-out, a little quicker to start than to stop; for travel and layout.
    pub const MOVE: &'static str = "cubic-bezier(.45, 0, .2, 1)";
    /// Decelerate; things arriving.
    pub const ENTER: &'static str = "cubic-bezier(0, 0, .2, 1)";
    /// Ease-in-out over opacity; a fade that neither pops nor lingers.
    pub const EXIT: &'static str = "cubic-bezier(.4, 0, .6, 1)";
}
