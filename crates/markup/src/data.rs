/// The `data-*` attributes the widgets write and read back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Data {
    /// A node's path in its term.
    Path,
    /// A node's uid.
    Uid,
    /// The uid a node was built from; on a graph edge, the vertex it leaves.
    From,
    /// The vertex a graph edge reaches.
    To,
    /// A λ or let box's anchor.
    Binder,
    /// The anchor a binder tab answers to.
    Anchor,
    /// The anchor a bound variable belongs to.
    Bid,
    /// A redex's rule.
    Kind,
    /// A graph vertex, or the anchor an arc belongs to.
    Id,
    /// A position in a trace or a race lane.
    I,
    /// An example a tile loads.
    Ex,
    /// A race a pill picks.
    Race,
    /// A route a pill picks.
    Route,
    /// The pill row of the routes.
    Routes,
    /// A button's action.
    Act,
    /// A checkbox's option.
    Opt,
    /// A definition's body in the definitions list.
    Def,
    /// A matched piece, while the husk of a firing is pinned.
    Piece,
    /// The widget a host element asks for.
    Redex,
    /// The example a host element asks to start from.
    Example,
}

impl Data {
    pub fn name(self) -> &'static str {
        match self {
            Self::Path => "data-path",
            Self::Uid => "data-uid",
            Self::From => "data-from",
            Self::To => "data-to",
            Self::Binder => "data-binder",
            Self::Anchor => "data-anchor",
            Self::Bid => "data-bid",
            Self::Kind => "data-kind",
            Self::Id => "data-id",
            Self::I => "data-i",
            Self::Ex => "data-ex",
            Self::Race => "data-race",
            Self::Route => "data-route",
            Self::Routes => "data-routes",
            Self::Act => "data-act",
            Self::Opt => "data-opt",
            Self::Def => "data-def",
            Self::Piece => "data-piece",
            Self::Redex => "data-redex",
            Self::Example => "data-example",
        }
    }
}
