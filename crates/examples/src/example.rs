use rules::*;

/// An example program. The rule examples carry their rule, a one-line description and the schema
/// as the deck shows it; the rest carry a label. `eta` asks the graph to include η.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Example {
    id: &'static str,
    label: &'static str,
    kind: Option<Kind>,
    sub: Option<&'static str>,
    schema: Option<&'static str>,
    eta: bool,
    src: &'static str,
}

impl Example {
    pub const RULES: [Example; 7] = [
        Self {
            id: "alpha",
            label: "alpha",
            kind: Some(Kind::Alpha),
            sub: Some("rename a binder"),
            schema: Some("λx. t ≡ λy. t[x:=y]"),
            eta: false,
            src: r"-- The argument y is free. Dropping it under λy would capture it.
-- Fire β: the engine renames the inner binder first.
-- Click any binder to rename it yourself. The links never move.
(λx. λy. x) y",
        },
        Self {
            id: "beta",
            label: "beta",
            kind: Some(Kind::Beta),
            sub: Some("apply a function"),
            schema: Some("(λx. t) s ⟶ t[x:=s]"),
            eta: false,
            src: r"-- f appears twice, so the argument is copied twice.
(λf. f (f a)) (λy. (y, y))",
        },
        Self {
            id: "eta",
            label: "eta",
            kind: Some(Kind::Eta),
            sub: Some("drop a wrapper"),
            schema: Some("λx. f x ⟶ f"),
            eta: true,
            src: r"-- Two η-redexes around a β-redex. Every route ends at g.
-- Step skips η, as Lean does: η is for comparing terms, not computing.
-- Click the η marks yourself.
λz. (λy. g y) z",
        },
        Self {
            id: "delta",
            label: "delta",
            kind: Some(Kind::Delta),
            sub: Some("unfold a definition"),
            schema: Some("c ⟶ body of c"),
            eta: false,
            src: r"-- Constants unfold to their definitions, one at a time.
def id := λx. x
def twice := λf. λx. f (f x)
twice id a",
        },
        Self {
            id: "zeta",
            label: "zeta",
            kind: Some(Kind::Zeta),
            sub: Some("inline a let"),
            schema: Some("let x := v; t ⟶ t[x:=v]"),
            eta: false,
            src: r"-- let x := v; t acts like (λx. t) v, but it is a separate rule.
let p := (a, b); (p.2, p.1)",
        },
        Self {
            id: "iota",
            label: "iota",
            kind: Some(Kind::Iota),
            sub: Some("eliminate a constructor"),
            schema: Some("Nat.rec z s (succ n) ⟶ s n (Nat.rec z s n)"),
            eta: false,
            src: r"-- The recursor fires only when its target is a constructor.
def double := λn. Nat.rec 0 (λk r. succ (succ r)) n
double 2",
        },
        Self {
            id: "proj",
            label: "proj",
            kind: Some(Kind::Proj),
            sub: Some("project from a pair"),
            schema: Some("(a, b).1 ⟶ a"),
            eta: false,
            src: r"-- Projections fire when they meet a pair.
(λp. (p.2, p.1)) (a, b)",
        },
    ];

    pub const MORE: [Example; 7] = [
        Self {
            id: "etas",
            label: "Structure η",
            kind: None,
            sub: None,
            schema: None,
            eta: true,
            src: r"-- (p.1, p.2) is p. Here η and two π steps reach the same pair.
((a, b).1, (a, b).2)",
        },
        Self {
            id: "diamond",
            label: "Diamond",
            kind: None,
            sub: None,
            schema: None,
            eta: false,
            src: r"-- Two redexes, two orders, one answer.
(λx. x x) ((λy. y) a)",
        },
        Self {
            id: "omega",
            label: "Ω loops",
            kind: None,
            sub: None,
            schema: None,
            eta: false,
            src: r"-- Ω reduces to itself, so it has no normal form.
(λx. x x) (λx. x x)",
        },
        Self {
            id: "lazy",
            label: "Lazy escape",
            kind: None,
            sub: None,
            schema: None,
            eta: false,
            src: r"-- The inner redex loops forever. The outer one throws it away.
(λx. z) ((λw. w w) (λw. w w))",
        },
        Self {
            id: "capture",
            label: "Deep capture",
            kind: None,
            sub: None,
            schema: None,
            eta: false,
            src: r"-- Two binders in the way. Both get renamed before β.
(λx. λy. λz. x y z) (y z)",
        },
        Self {
            id: "church",
            label: "Church 2 + 2",
            kind: None,
            sub: None,
            schema: None,
            eta: false,
            src: r"-- Numbers as iterators: n f x applies f n times.
def two := λf. λx. f (f x)
def plus := λm. λn. λf. λx. m f (n f x)
plus two two",
        },
        Self {
            id: "mix",
            label: "All rules",
            kind: None,
            sub: None,
            schema: None,
            eta: false,
            src: r"-- δ, β, ζ, π and ι in one term.
def double := λn. Nat.rec 0 (λk r. succ (succ r)) n
let p := (double 1, b); p.1",
        },
    ];

    pub fn all() -> impl Iterator<Item = Example> {
        Self::RULES.into_iter().chain(Self::MORE)
    }

    pub fn find(id: &str) -> Option<Example> {
        Self::all().find(|e| e.id == id)
    }

    pub fn id(&self) -> &'static str {
        self.id
    }

    pub fn label(&self) -> &'static str {
        self.label
    }

    pub fn kind(&self) -> Option<Kind> {
        self.kind
    }

    pub fn sub(&self) -> Option<&'static str> {
        self.sub
    }

    pub fn schema(&self) -> Option<&'static str> {
        self.schema
    }

    pub fn eta(&self) -> bool {
        self.eta
    }

    pub fn src(&self) -> &'static str {
        self.src
    }
}

impl std::str::FromStr for Example {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::find(s).ok_or(())
    }
}
