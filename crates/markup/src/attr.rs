use examples::*;
use explore::*;
use rules::*;
use strategy::*;
use term::*;

use crate::*;

/// One attribute, its value typed. `Display` writes the value unescaped; `Node` escapes it.
#[derive(Debug, Clone, PartialEq)]
pub enum Attr {
    Class(Vec<Class>),
    /// `type="button"`.
    Button,
    Title(String),
    Label(String),
    /// `aria-hidden="true"`.
    Hidden,
    Pressed(bool),
    /// `tabindex="0"`.
    Focusable,
    /// `role="button"`.
    Clickable,
    Path(Path),
    Uid(Uid),
    From(Uid),
    Binder(Anchor),
    Anchor(Anchor),
    Bid(Anchor),
    Kind(Kind),
    Vertex(Id),
    Source(Id),
    Target(Id),
    Arc(Anchor),
    I(Step),
    Ex(Example),
    Race(Race),
    Route(Option<Strategy>),
    Def(Name),
    Act(Act),
    Style(Style),
    X(Num),
    Y(Num),
    X1(Num),
    X2(Num),
    Y1(Num),
    Y2(Num),
    Cx(Num),
    Cy(Num),
    R(Num),
    Width(Num),
    Height(Num),
    ViewBox(Num, Num, Num, Num),
    D(Trace),
    /// `pathLength="1"`, so `draw` can stroke the path from 0 to 1.
    Unit,
    MarkerEnd(Plate, Kind),
    Marker(Plate, Kind),
    RefX(Num),
    RefY(Num),
    MarkerWidth(Num),
    MarkerHeight(Num),
    /// `orient="auto-start-reverse"`.
    Orient,
}

impl Attr {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Class(_) => "class",
            Self::Button => "type",
            Self::Title(_) => "title",
            Self::Label(_) => "aria-label",
            Self::Hidden => "aria-hidden",
            Self::Pressed(_) => "aria-pressed",
            Self::Focusable => "tabindex",
            Self::Clickable => "role",
            Self::Path(_) => Data::Path.name(),
            Self::Uid(_) => Data::Uid.name(),
            Self::From(_) | Self::Source(_) => Data::From.name(),
            Self::Target(_) => Data::To.name(),
            Self::Binder(_) => Data::Binder.name(),
            Self::Anchor(_) => Data::Anchor.name(),
            Self::Bid(_) => Data::Bid.name(),
            Self::Kind(_) => Data::Kind.name(),
            Self::Vertex(_) | Self::Arc(_) => Data::Id.name(),
            Self::I(_) => Data::I.name(),
            Self::Ex(_) => Data::Ex.name(),
            Self::Race(_) => Data::Race.name(),
            Self::Route(_) => Data::Route.name(),
            Self::Def(_) => Data::Def.name(),
            Self::Act(_) => Data::Act.name(),
            Self::Style(_) => "style",
            Self::X(_) => "x",
            Self::Y(_) => "y",
            Self::X1(_) => "x1",
            Self::X2(_) => "x2",
            Self::Y1(_) => "y1",
            Self::Y2(_) => "y2",
            Self::Cx(_) => "cx",
            Self::Cy(_) => "cy",
            Self::R(_) => "r",
            Self::Width(_) => "width",
            Self::Height(_) => "height",
            Self::ViewBox(..) => "viewBox",
            Self::D(_) => "d",
            Self::Unit => "pathLength",
            Self::MarkerEnd(..) => "marker-end",
            Self::Marker(..) => "id",
            Self::RefX(_) => "refX",
            Self::RefY(_) => "refY",
            Self::MarkerWidth(_) => "markerWidth",
            Self::MarkerHeight(_) => "markerHeight",
            Self::Orient => "orient",
        }
    }
}

impl std::fmt::Display for Attr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Class(cs) => cs
                .iter()
                .enumerate()
                .try_for_each(|(i, c)| if i > 0 { write!(f, " {c}") } else { write!(f, "{c}") }),
            Self::Button => f.write_str("button"),
            Self::Title(s) | Self::Label(s) => f.write_str(s),
            Self::Hidden => f.write_str("true"),
            Self::Pressed(on) => write!(f, "{on}"),
            Self::Focusable => f.write_str("0"),
            Self::Clickable => f.write_str("button"),
            Self::Path(p) => write!(f, "{p}"),
            Self::Uid(u) | Self::From(u) => write!(f, "{u}"),
            Self::Binder(a) | Self::Anchor(a) | Self::Bid(a) | Self::Arc(a) => write!(f, "{a}"),
            Self::Kind(k) => write!(f, "{k}"),
            Self::Vertex(i) | Self::Source(i) | Self::Target(i) => write!(f, "{i}"),
            Self::I(i) => write!(f, "{i}"),
            Self::Ex(e) => f.write_str(e.id()),
            Self::Race(r) => f.write_str(r.id()),
            Self::Route(r) => f.write_str(r.map_or("none", Strategy::id)),
            Self::Def(n) => write!(f, "{n}"),
            Self::Act(a) => write!(f, "{a}"),
            Self::Style(s) => write!(f, "{s}"),
            Self::X(n)
            | Self::Y(n)
            | Self::X1(n)
            | Self::X2(n)
            | Self::Y1(n)
            | Self::Y2(n)
            | Self::Cx(n)
            | Self::Cy(n)
            | Self::R(n) => write!(f, "{n}"),
            Self::Width(n)
            | Self::Height(n)
            | Self::RefX(n)
            | Self::RefY(n)
            | Self::MarkerWidth(n)
            | Self::MarkerHeight(n) => write!(f, "{n}"),
            Self::ViewBox(a, b, c, d) => write!(f, "{a} {b} {c} {d}"),
            Self::D(t) => write!(f, "{t}"),
            Self::Unit => f.write_str("1"),
            Self::MarkerEnd(p, k) => write!(f, "url(#{p}-{k})"),
            Self::Marker(p, k) => write!(f, "{p}-{k}"),
            Self::Orient => f.write_str("auto-start-reverse"),
        }
    }
}
