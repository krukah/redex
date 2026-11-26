use crate::*;

#[derive(Debug, Clone)]
pub enum Shape {
    Var(String),
    Const(String),
    Lam(String, Term),
    App(Term, Term),
    Let(String, Term, Term),
    Pair(Term, Term),
    Proj(u8, Term),
}

impl Shape {
    /// The kind as the JS engine names it.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Var(_) => "var",
            Self::Const(_) => "const",
            Self::Lam(..) => "lam",
            Self::App(..) => "app",
            Self::Let(..) => "let",
            Self::Pair(..) => "pair",
            Self::Proj(..) => "proj",
        }
    }

    /// Children with the letter a path uses for each, in reading order.
    pub fn kids(&self) -> Vec<(char, &Term)> {
        match self {
            Self::Var(_) | Self::Const(_) => vec![],
            Self::Lam(_, b) => vec![('b', b)],
            Self::App(f, a) => vec![('f', f), ('a', a)],
            Self::Let(_, v, b) => vec![('v', v), ('b', b)],
            Self::Pair(a, b) => vec![('a', a), ('b', b)],
            Self::Proj(_, t) => vec![('t', t)],
        }
    }

    pub fn kid(&self, c: char) -> Option<&Term> {
        self.kids().into_iter().find(|(k, _)| *k == c).map(|(_, t)| t)
    }

    /// The same shape with child `c` swapped for `t`.
    pub fn with(&self, c: char, t: Term) -> Self {
        match (self, c) {
            (Self::Lam(x, _), 'b') => Self::Lam(x.clone(), t),
            (Self::App(_, a), 'f') => Self::App(t, a.clone()),
            (Self::App(f, _), 'a') => Self::App(f.clone(), t),
            (Self::Let(x, _, b), 'v') => Self::Let(x.clone(), t, b.clone()),
            (Self::Let(x, v, _), 'b') => Self::Let(x.clone(), v.clone(), t),
            (Self::Pair(_, b), 'a') => Self::Pair(t, b.clone()),
            (Self::Pair(a, _), 'b') => Self::Pair(a.clone(), t),
            (Self::Proj(i, _), 't') => Self::Proj(*i, t),
            _ => self.clone(),
        }
    }
}
