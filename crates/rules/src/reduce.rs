use subst::*;
use term::*;

use crate::*;

/// The rules on terms.
pub trait Reduce {
    /// The rule this node is a redex for, if any.
    fn redex(&self, env: &Env) -> Option<Kind>;
    /// Every redex in the term, in pre-order.
    fn redexes(&self, env: &Env) -> Vec<Redex>;
    /// The contractum of this redex under `kind`.
    fn contract(&self, kind: Kind, env: &Env) -> Term;
    /// The term with the redex `r` contracted.
    fn step(&self, r: &Redex, env: &Env) -> Term;
    /// `Nat.rec z s 0 ⟶ z`, `Nat.rec z s (succ n) ⟶ s n (Nat.rec z s n)`.
    fn iota(&self) -> Term;
    /// Pushes the redexes under `path` onto `out`, in pre-order.
    fn collect(&self, env: &Env, path: &Path, out: &mut Vec<Redex>);
}

impl Reduce for Term {
    fn redex(&self, env: &Env) -> Option<Kind> {
        match self.shape() {
            Shape::App(f, _) if matches!(f.shape(), Shape::Lam(..)) => Some(Kind::Beta),
            Shape::App(..) => self.rec_args().filter(|(_, _, n)| n.is_ctor()).map(|_| Kind::Iota),
            Shape::Const(n) => env.has(n).then_some(Kind::Delta),
            Shape::Let(..) => Some(Kind::Zeta),
            Shape::Proj(_, t) => matches!(t.shape(), Shape::Pair(..)).then_some(Kind::Proj),
            Shape::Lam(x, b) => match b.shape() {
                Shape::App(f, a)
                    if a.name() == Some(x) && matches!(a.shape(), Shape::Var(_)) && !f.fv().contains(x) =>
                {
                    Some(Kind::Eta)
                }
                _ => None,
            },
            Shape::Pair(a, b) => match (a.shape(), b.shape()) {
                (Shape::Proj(Field::First, p), Shape::Proj(Field::Second, q)) if p.key() == q.key() => Some(Kind::EtaS),
                _ => None,
            },
            Shape::Var(_) => None,
        }
    }

    fn redexes(&self, env: &Env) -> Vec<Redex> {
        let mut out = Vec::new();
        self.collect(env, &Path::root(), &mut out);
        out
    }

    fn contract(&self, kind: Kind, env: &Env) -> Term {
        let kid = |path: &str| {
            self.at(&Path::from(path))
                .expect("a redex has the shape its rule matches")
        };
        match kind {
            Kind::Beta => match kid("f").shape() {
                Shape::Lam(x, b) => b.substitute(x, kid("a")),
                _ => self.clone(),
            },
            Kind::Zeta => match self.shape() {
                Shape::Let(x, v, b) => b.substitute(x, v),
                _ => self.clone(),
            },
            Kind::Delta => self
                .name()
                .and_then(|n| env.get(n))
                .map_or_else(|| self.clone(), Term::copy),
            Kind::Iota => self.iota(),
            Kind::Proj if matches!(self.shape(), Shape::Proj(Field::First, _)) => kid("ta").copy(),
            Kind::Proj => kid("tb").copy(),
            Kind::Eta => kid("bf").copy(),
            Kind::EtaS => kid("at").copy(),
            Kind::Alpha => self.clone(),
        }
    }

    fn step(&self, r: &Redex, env: &Env) -> Term {
        self.replace(
            r.path(),
            self.at(r.path())
                .expect("a redex's path is inside its term")
                .contract(r.kind(), env),
        )
    }

    fn iota(&self) -> Term {
        match self.rec_args() {
            Some((z, _, n)) if matches!(n.shape(), Shape::Const(_)) => z.copy(),
            Some((z, s, n)) => {
                let m = n.kid('a').expect("succ has an argument");
                Term::app(
                    Term::app(s.copy(), m.copy()),
                    Term::app(Term::app(Term::app(Term::cons(Prim::Rec), z.copy()), s.copy()), m.copy()),
                )
            }
            None => self.clone(),
        }
    }

    fn collect(&self, env: &Env, path: &Path, out: &mut Vec<Redex>) {
        if let Some(k) = self.redex(env) {
            out.push(Redex::new(path.clone(), k));
        }
        self.kids()
            .iter()
            .for_each(|(c, k)| k.collect(env, &path.child(*c), out));
    }
}
