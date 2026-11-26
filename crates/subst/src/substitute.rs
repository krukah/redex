use std::collections::HashSet;

use term::*;

/// Substitution on terms. Every node built from an old one records it in `from`.
pub trait Substitute {
    /// `t[x := s]`, renaming a binder only when it would capture a free variable of `s` (`fvs`).
    /// Copies of `s` carry `mark`.
    fn subst(&self, x: &str, s: &Term, fvs: &HashSet<String>, mark: Option<Mark>) -> Term;
    /// `t[x := s]` with `fvs` computed from `s`.
    fn substitute(&self, x: &str, s: &Term) -> Term;
    /// Renames only the binders that would capture a free variable of the argument, logging
    /// each `(old, new)`.
    fn prepare(&self, x: &str, fvs: &HashSet<String>, log: &mut Vec<(String, String)>) -> Term;
    /// Substitution without renaming, to show what capture looks like. Captured variables get `cap`.
    fn naive(&self, x: &str, s: &Term) -> Term;
    /// Flags every free occurrence of a name in `free`.
    fn tag(&self, free: &HashSet<String>, bound: &HashSet<String>) -> Term;
    /// The binder `y` over this body, and the body, after `[x := s]`.
    fn under(&self, y: &str, x: &str, s: &Term, fvs: &HashSet<String>, mark: Option<Mark>) -> (String, Term);
    /// The binder `y` over this body, and the body, after `prepare`; true when `y` was renamed.
    fn rename(&self, y: &str, x: &str, fvs: &HashSet<String>, log: &mut Vec<(String, String)>) -> (String, Term, bool);
    /// A fresh name for `y` that this body, `x` and `fvs` do not use.
    fn avoid(&self, y: &str, x: &str, fvs: &HashSet<String>) -> String;
}

impl Substitute for Term {
    fn subst(&self, x: &str, s: &Term, fvs: &HashSet<String>, mark: Option<Mark>) -> Term {
        match self.shape() {
            Shape::Var(n) if n == x => match mark {
                Some(m) => s.copy().marked(m),
                None => s.copy(),
            },
            Shape::Var(_) | Shape::Const(_) => self.clone(),
            Shape::App(f, a) => Term::app(f.subst(x, s, fvs, mark), a.subst(x, s, fvs, mark)).derived(self),
            Shape::Pair(a, b) => Term::pair(a.subst(x, s, fvs, mark), b.subst(x, s, fvs, mark)).derived(self),
            Shape::Proj(i, t) => Term::proj(*i, t.subst(x, s, fvs, mark)).derived(self),
            Shape::Lam(y, b) => {
                let (y, b) = b.under(y, x, s, fvs, mark);
                Term::lam(&y, b).derived(self)
            }
            Shape::Let(y, v, b) => {
                let v = v.subst(x, s, fvs, mark);
                let (y, b) = b.under(y, x, s, fvs, mark);
                Term::local(&y, v, b).derived(self)
            }
        }
    }

    fn substitute(&self, x: &str, s: &Term) -> Term {
        self.subst(x, s, &s.fv(), None)
    }

    fn prepare(&self, x: &str, fvs: &HashSet<String>, log: &mut Vec<(String, String)>) -> Term {
        match self.shape() {
            Shape::Var(_) | Shape::Const(_) => self.clone(),
            Shape::App(f, a) => Term::app(f.prepare(x, fvs, log), a.prepare(x, fvs, log)).derived(self),
            Shape::Pair(a, b) => Term::pair(a.prepare(x, fvs, log), b.prepare(x, fvs, log)).derived(self),
            Shape::Proj(i, t) => Term::proj(*i, t.prepare(x, fvs, log)).derived(self),
            Shape::Lam(y, _) if y == x => self.clone(),
            Shape::Let(y, v, b) if y == x => Term::local(y, v.prepare(x, fvs, log), b.clone()).derived(self),
            Shape::Lam(y, b) => {
                let (y, b, renamed) = b.rename(y, x, fvs, log);
                Term::lam(&y, b).derived(self).flagged(renamed)
            }
            Shape::Let(y, v, b) => {
                let v = v.prepare(x, fvs, log);
                let (y, b, renamed) = b.rename(y, x, fvs, log);
                Term::local(&y, v, b).derived(self).flagged(renamed)
            }
        }
    }

    fn naive(&self, x: &str, s: &Term) -> Term {
        match self.shape() {
            Shape::Var(n) if n == x => s.copy().tag(&s.fv(), &HashSet::new()),
            Shape::Var(_) | Shape::Const(_) => self.copy(),
            Shape::App(f, a) => Term::app(f.naive(x, s), a.naive(x, s)),
            Shape::Pair(a, b) => Term::pair(a.naive(x, s), b.naive(x, s)),
            Shape::Proj(i, t) => Term::proj(*i, t.naive(x, s)),
            Shape::Lam(y, _) if y == x => self.copy(),
            Shape::Lam(y, b) => Term::lam(y, b.naive(x, s)),
            Shape::Let(y, v, b) => {
                let v = v.naive(x, s);
                Term::local(y, v, if y == x { b.copy() } else { b.naive(x, s) })
            }
        }
    }

    fn tag(&self, free: &HashSet<String>, bound: &HashSet<String>) -> Term {
        match self.shape() {
            Shape::Var(n) if !bound.contains(n) && free.contains(n) => self.set_cap(true),
            Shape::Lam(y, b) => {
                b.tag(free, &bound.iter().cloned().chain([y.clone()]).collect());
            }
            Shape::Let(y, v, b) => {
                v.tag(free, bound);
                b.tag(free, &bound.iter().cloned().chain([y.clone()]).collect());
            }
            _ => self.kids().iter().for_each(|(_, k)| {
                k.tag(free, bound);
            }),
        }
        self.clone()
    }

    fn under(&self, y: &str, x: &str, s: &Term, fvs: &HashSet<String>, mark: Option<Mark>) -> (String, Term) {
        match y == x {
            true => (y.to_string(), self.clone()),
            false if fvs.contains(y) && self.fv().contains(x) => {
                let z = self.avoid(y, x, fvs);
                let b = self.subst(y, &Term::var(&z), &HashSet::from([z.clone()]), None);
                (z, b.subst(x, s, fvs, mark))
            }
            false => (y.to_string(), self.subst(x, s, fvs, mark)),
        }
    }

    fn rename(&self, y: &str, x: &str, fvs: &HashSet<String>, log: &mut Vec<(String, String)>) -> (String, Term, bool) {
        if fvs.contains(y) && self.fv().contains(x) {
            let z = self.avoid(y, x, fvs);
            log.push((y.to_string(), z.clone()));
            let b = self.subst(y, &Term::var(&z), &HashSet::from([z.clone()]), Some(Mark::Renamed));
            (z, b.prepare(x, fvs, log), true)
        } else {
            (y.to_string(), self.prepare(x, fvs, log), false)
        }
    }

    fn avoid(&self, y: &str, x: &str, fvs: &HashSet<String>) -> String {
        Name::fresh(y, &fvs.iter().cloned().chain(self.names()).chain([x.to_string()]).collect())
    }
}
