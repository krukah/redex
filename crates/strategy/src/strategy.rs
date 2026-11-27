use rules::*;
use term::*;

/// A rule for choosing the next redex. None of them performs η, the way Lean never η-reduces
/// while computing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    /// Leftmost-outermost.
    Normal,
    /// Leftmost-innermost.
    Applicative,
    /// The head only, to weak head normal form.
    Whnf,
}

impl Strategy {
    pub const ALL: [Strategy; 3] = [Self::Normal, Self::Applicative, Self::Whnf];

    /// The strategy's name in the race lanes and on the graph's route pills.
    pub fn label(self) -> &'static str {
        match self {
            Self::Normal => "Normal order",
            Self::Applicative => "Applicative order",
            Self::Whnf => "Head only",
        }
    }

    /// The strategy's id in markup (`data-route`).
    pub fn id(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Applicative => "applicative",
            Self::Whnf => "whnf",
        }
    }

    pub fn next(self, t: &Term, env: &Env) -> Option<Redex> {
        match self {
            Self::Normal => t.redexes(env).into_iter().find(|r| !r.kind().is_eta()),
            Self::Applicative => Self::innermost(t, env, &Path::root()),
            Self::Whnf => Self::head(t, env, &Path::root()),
        }
    }

    fn innermost(t: &Term, env: &Env, p: &Path) -> Option<Redex> {
        t.kids()
            .iter()
            .find_map(|(c, k)| Self::innermost(k, env, &p.child(*c)))
            .or_else(|| t.redex(env).filter(|k| !k.is_eta()).map(|k| Redex::new(p.clone(), k)))
    }

    fn head(t: &Term, env: &Env, p: &Path) -> Option<Redex> {
        match t.shape() {
            Shape::App(..) => Self::spine(t, env, p),
            Shape::Let(..) => Some(Redex::new(p.clone(), Kind::Zeta)),
            Shape::Const(n) => env.has(n).then(|| Redex::new(p.clone(), Kind::Delta)),
            Shape::Proj(_, s) if matches!(s.shape(), Shape::Pair(..)) => Some(Redex::new(p.clone(), Kind::Proj)),
            Shape::Proj(_, s) => Self::head(s, env, &p.child('t')),
            _ => None,
        }
    }

    /// The head redex of an application spine.
    fn spine(t: &Term, env: &Env, p: &Path) -> Option<Redex> {
        let (h, n) = std::iter::successors(Some(t), |h| match h.shape() {
            Shape::App(f, _) => Some(f),
            _ => None,
        })
        .enumerate()
        .last()
        .map(|(n, h)| (h, n))
        .expect("a spine has a head");
        let hp = Path::from(format!("{p}{}", "f".repeat(n)));
        match h.shape() {
            Shape::Lam(..) => Some(Redex::new(hp.parent(), Kind::Beta)),
            Shape::Let(..) => Some(Redex::new(hp, Kind::Zeta)),
            Shape::Const(c) if env.has(c) => Some(Redex::new(hp, Kind::Delta)),
            Shape::Proj(..) => Self::head(h, env, &hp),
            Shape::Const(c) if c.prim() == Some(Prim::Rec) && n >= 3 => {
                let rel = Path::from("f".repeat(n - 3));
                let arg = t
                    .at(&rel)
                    .and_then(|node| node.kid('a'))
                    .expect("Nat.rec has three arguments");
                if arg.is_ctor() {
                    Some(Redex::new(p.join(&rel), Kind::Iota))
                } else {
                    Self::head(arg, env, &p.join(&rel).child('a'))
                }
            }
            _ => None,
        }
    }
}
