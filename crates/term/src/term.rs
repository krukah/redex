use std::collections::HashSet;
use std::rc::Rc;

use crate::*;

/// A lambda term: a shared, immutable tree of nodes.
#[derive(Debug, Clone)]
pub struct Term(Rc<Node>);

impl Term {
    fn make(shape: Shape) -> Self {
        Self(Rc::new(Node::new(Uid::next(), shape)))
    }

    pub fn var(n: &str) -> Self {
        Self::make(Shape::Var(n.to_string()))
    }

    pub fn cons(n: &str) -> Self {
        Self::make(Shape::Const(n.to_string()))
    }

    pub fn lam(x: &str, b: Term) -> Self {
        Self::make(Shape::Lam(x.to_string(), b))
    }

    pub fn app(f: Term, a: Term) -> Self {
        Self::make(Shape::App(f, a))
    }

    pub fn local(x: &str, v: Term, b: Term) -> Self {
        Self::make(Shape::Let(x.to_string(), v, b))
    }

    pub fn pair(a: Term, b: Term) -> Self {
        Self::make(Shape::Pair(a, b))
    }

    pub fn proj(i: u8, t: Term) -> Self {
        Self::make(Shape::Proj(i, t))
    }

    /// Records that this node was built from `old`.
    pub fn derived(self, old: &Term) -> Self {
        self.0.set_from(Some(old.uid()));
        self
    }

    pub fn marked(self, mark: Mark) -> Self {
        self.0.set_mark(Some(mark));
        self
    }

    pub fn flagged(self, renamed: bool) -> Self {
        if renamed { self.marked(Mark::Binder) } else { self }
    }

    pub fn node(&self) -> &Node {
        &self.0
    }

    pub fn uid(&self) -> Uid {
        self.0.uid()
    }

    pub fn shape(&self) -> &Shape {
        self.0.shape()
    }

    pub fn from(&self) -> Option<Uid> {
        self.0.from()
    }

    pub fn mark(&self) -> Option<Mark> {
        self.0.mark()
    }

    pub fn cap(&self) -> bool {
        self.0.cap()
    }

    pub fn same(&self, other: &Term) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }

    pub fn kids(&self) -> Vec<(char, &Term)> {
        self.shape().kids()
    }

    pub fn kid(&self, c: char) -> Option<&Term> {
        self.shape().kid(c)
    }

    /// The bound name of a λ or let.
    pub fn binder(&self) -> Option<&str> {
        match self.shape() {
            Shape::Lam(x, _) | Shape::Let(x, ..) => Some(x),
            _ => None,
        }
    }

    /// The name of a variable or constant.
    pub fn name(&self) -> Option<&str> {
        match self.shape() {
            Shape::Var(n) | Shape::Const(n) => Some(n),
            _ => None,
        }
    }

    pub fn is_const(&self, n: &str) -> bool {
        matches!(self.shape(), Shape::Const(c) if c == n)
    }

    /// A deep copy whose every node records the node it copies.
    pub fn copy(&self) -> Self {
        match self.shape() {
            Shape::Var(n) => Self::var(n),
            Shape::Const(n) => Self::cons(n),
            Shape::Lam(x, b) => Self::lam(x, b.copy()),
            Shape::App(f, a) => Self::app(f.copy(), a.copy()),
            Shape::Let(x, v, b) => Self::local(x, v.copy(), b.copy()),
            Shape::Pair(a, b) => Self::pair(a.copy(), b.copy()),
            Shape::Proj(i, t) => Self::proj(*i, t.copy()),
        }
        .derived(self)
    }

    pub fn size(&self) -> usize {
        1 + self.kids().iter().map(|(_, k)| k.size()).sum::<usize>()
    }

    pub fn fv(&self) -> HashSet<String> {
        match self.shape() {
            Shape::Var(n) => HashSet::from([n.clone()]),
            Shape::Const(_) => HashSet::new(),
            Shape::Lam(x, b) => b.fv().into_iter().filter(|y| y != x).collect(),
            Shape::Let(x, v, b) => b.fv().into_iter().filter(|y| y != x).chain(v.fv()).collect(),
            _ => self.kids().iter().flat_map(|(_, k)| k.fv()).collect(),
        }
    }

    /// Every variable and binder name in the term.
    pub fn names(&self) -> HashSet<String> {
        self.name()
            .filter(|_| matches!(self.shape(), Shape::Var(_)))
            .or(self.binder())
            .map(ToString::to_string)
            .into_iter()
            .chain(self.kids().iter().flat_map(|(_, k)| k.names()))
            .collect()
    }

    /// The α-equivalence key: the term with de Bruijn indices.
    pub fn key(&self) -> String {
        self.keyed(&mut Vec::new())
    }

    fn keyed(&self, ctx: &mut Vec<String>) -> String {
        match self.shape() {
            Shape::Var(n) => match ctx.iter().rposition(|y| y == n) {
                Some(i) => format!("#{}", ctx.len() - 1 - i),
                None => format!("${n}"),
            },
            Shape::Const(n) => n.clone(),
            Shape::Lam(x, b) => format!("λ({})", Self::under(ctx, x, b)),
            Shape::App(f, a) => format!("@({},{})", f.keyed(ctx), a.keyed(ctx)),
            Shape::Let(x, v, b) => format!("L({},{})", v.keyed(ctx), Self::under(ctx, x, b)),
            Shape::Pair(a, b) => format!("P({},{})", a.keyed(ctx), b.keyed(ctx)),
            Shape::Proj(i, t) => format!("π{i}({})", t.keyed(ctx)),
        }
    }

    fn under(ctx: &mut Vec<String>, x: &str, b: &Term) -> String {
        ctx.push(x.to_string());
        let s = b.keyed(ctx);
        ctx.pop();
        s
    }

    /// The numeral this term spells with `succ` and `zero`, if any.
    pub fn nat(&self) -> Option<usize> {
        match self.shape() {
            Shape::Const(n) if n == "zero" => Some(0),
            Shape::App(f, a) if f.is_const("succ") => a.nat().map(|n| n + 1),
            _ => None,
        }
    }

    /// `zero` or `succ _`.
    pub fn is_ctor(&self) -> bool {
        match self.shape() {
            Shape::Const(n) => n == "zero",
            Shape::App(f, _) => f.is_const("succ"),
            _ => false,
        }
    }

    /// `Nat.rec z s n` as `(z, s, n)`.
    pub fn rec_args(&self) -> Option<(&Term, &Term, &Term)> {
        let Shape::App(fff, n) = self.shape() else { return None };
        let Shape::App(ff, s) = fff.shape() else { return None };
        let Shape::App(f, z) = ff.shape() else { return None };
        f.is_const("Nat.rec").then_some((z, s, n))
    }

    pub fn at(&self, path: &Path) -> Option<&Term> {
        path.steps().try_fold(self, |t, c| t.kid(c))
    }

    /// The term with the node at `path` swapped for `s`. Nodes along the path keep their uid and
    /// provenance and lose their mark.
    pub fn replace(&self, path: &Path, s: Term) -> Term {
        self.replaced(&path.steps().collect::<Vec<char>>(), s)
    }

    fn replaced(&self, steps: &[char], s: Term) -> Term {
        match steps.split_first() {
            None => s,
            Some((c, rest)) => match self.kid(*c) {
                None => self.clone(),
                Some(k) => {
                    let node = Node::new(self.uid(), self.shape().with(*c, k.replaced(rest, s)));
                    node.set_from(self.from());
                    node.set_cap(self.cap());
                    Self(Rc::new(node))
                }
            },
        }
    }

    /// Forgets every mark and provenance in the term.
    pub fn strip(&self) {
        self.0.set_mark(None);
        self.0.set_from(None);
        self.kids().iter().for_each(|(_, k)| k.strip());
    }

    pub fn set_cap(&self, cap: bool) {
        self.0.set_cap(cap);
    }

    /// One line per term, ids relative to `base`: the format of `fixtures/engine`.
    pub fn dump(&self, base: u64) -> String {
        let rel = |u: Uid| u.get() as i64 - base as i64;
        let label = match self.shape() {
            Shape::Var(n) | Shape::Const(n) | Shape::Lam(n, _) | Shape::Let(n, ..) => n.clone(),
            Shape::Proj(i, _) => i.to_string(),
            Shape::App(..) | Shape::Pair(..) => String::new(),
        };
        let kids = self.kids().iter().map(|(_, k)| k.dump(base)).collect::<Vec<String>>();
        format!(
            "{}:{label}#{}{}{}{}{}",
            self.shape().kind(),
            rel(self.uid()),
            self.from().map(|f| format!("<{}", rel(f))).unwrap_or_default(),
            self.mark().map(|m| format!("!{}", m.name())).unwrap_or_default(),
            if self.cap() { "^" } else { "" },
            if kids.is_empty() { String::new() } else { format!("({})", kids.join(" ")) },
        )
    }
}
