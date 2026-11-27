use term::*;

use crate::*;

/// A rule. `EtaS` is η for structures; it shares η's ink and letter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Alpha,
    Beta,
    Eta,
    EtaS,
    Delta,
    Zeta,
    Iota,
    Proj,
}

impl Kind {
    pub const ALL: [Kind; 8] = [
        Self::Alpha,
        Self::Beta,
        Self::Eta,
        Self::EtaS,
        Self::Delta,
        Self::Zeta,
        Self::Iota,
        Self::Proj,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::Alpha => "alpha",
            Self::Beta => "beta",
            Self::Eta => "eta",
            Self::EtaS => "etaS",
            Self::Delta => "delta",
            Self::Zeta => "zeta",
            Self::Iota => "iota",
            Self::Proj => "proj",
        }
    }

    /// The rule whose ink and letter this one wears: η for structures wears η's.
    pub fn ink(self) -> Kind {
        match self {
            Self::EtaS => Self::Eta,
            _ => self,
        }
    }

    pub fn glyph(self) -> Glyph {
        Glyph::new(match self {
            Self::Alpha => 'α',
            Self::Beta => 'β',
            Self::Eta | Self::EtaS => 'η',
            Self::Delta => 'δ',
            Self::Zeta => 'ζ',
            Self::Iota => 'ι',
            Self::Proj => 'π',
        })
    }

    pub fn is_eta(self) -> bool {
        matches!(self, Self::Eta | Self::EtaS)
    }

    /// The rule's schema over pattern variables, for the redex `t`.
    pub fn schema(self, t: &Term) -> Schema {
        Schema::from(match self {
            Self::Alpha => "λ{x}. {t}  ≡  λ{y}. {t}[{x} := {y}]",
            Self::Beta => "(λ{x}. {t}) {s}  ⟶  {t}[{x} := {s}]",
            Self::Zeta => "let {x} := {v}; {t}  ⟶  {t}[{x} := {v}]",
            Self::Delta => "{c}  ⟶  the definition of {c}",
            Self::Iota if Self::zero(t) => "Nat.rec {z} {s} 0  ⟶  {z}",
            Self::Iota => "Nat.rec {z} {s} (succ {n})  ⟶  {s} {n} (Nat.rec {z} {s} {n})",
            Self::Proj if matches!(t.shape(), Shape::Proj(Field::First, _)) => "({a}, {b}).1  ⟶  {a}",
            Self::Proj => "({a}, {b}).2  ⟶  {b}",
            Self::Eta => "λ{x}. {f} {x}  ⟶  {f}",
            Self::EtaS => "({p}.1, {p}.2)  ⟶  {p}",
        })
    }

    /// Where each pattern variable sits inside the redex `t`.
    pub fn bindings(self, t: &Term) -> Vec<Binding> {
        match self {
            Self::Beta => vec![
                Binding::binder("x", "f"),
                Binding::piece("t", "fb"),
                Binding::piece("s", "a"),
            ],
            Self::Zeta => vec![
                Binding::binder("x", ""),
                Binding::piece("v", "v"),
                Binding::piece("t", "b"),
            ],
            Self::Delta => vec![Binding::piece("c", "")],
            Self::Iota if Self::zero(t) => vec![Binding::piece("z", "ffa"), Binding::piece("s", "fa")],
            Self::Iota => vec![
                Binding::piece("z", "ffa"),
                Binding::piece("s", "fa"),
                Binding::piece("n", "aa"),
            ],
            Self::Proj => vec![Binding::piece("a", "ta"), Binding::piece("b", "tb")],
            Self::Eta => vec![Binding::binder("x", ""), Binding::piece("f", "bf")],
            Self::EtaS => vec![Binding::piece("p", "at")],
            Self::Alpha => vec![],
        }
    }

    /// Whether the recursor's target is a constant (`zero`), not `succ n`.
    fn zero(t: &Term) -> bool {
        t.rec_args()
            .is_some_and(|(_, _, n)| matches!(n.shape(), Shape::Const(_)))
    }
}

impl std::fmt::Display for Kind {
    /// The rule's name in markup and fixtures (`etaS` included).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl std::str::FromStr for Kind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL.into_iter().find(|k| k.name() == s).ok_or(())
    }
}
