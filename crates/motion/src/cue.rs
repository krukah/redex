/// A moment in a firing, marked on the page's performance timeline (`performance.mark`) so a
/// browser check can read how long each part took.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cue {
    /// A step begins.
    Fire,
    /// The automatic α step that a capturing β or ζ needs first begins.
    Alpha,
    /// The match is painted and the rule card shows the bindings; the hold begins.
    Hold,
    /// The hold is over; the redex is pinned and the step pushed.
    Lift,
    /// The move begins.
    Move,
    /// The move has resolved.
    Moved,
}

impl Cue {
    fn name(self) -> &'static str {
        match self {
            Self::Fire => "redex:fire",
            Self::Alpha => "redex:alpha",
            Self::Hold => "redex:hold",
            Self::Lift => "redex:lift",
            Self::Move => "redex:move",
            Self::Moved => "redex:moved",
        }
    }

    pub fn mark(self) {
        if let Some(p) = web_sys::window().as_ref().and_then(web_sys::Window::performance) {
            let _ = p.mark(self.name());
        }
    }
}
