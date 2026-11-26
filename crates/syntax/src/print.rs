use term::*;

/// Lean-like plain text with as few parentheses as the grammar allows.
pub trait Print {
    fn text(&self) -> String;
    fn printed(&self, prec: u8) -> String;
}

impl Print for Term {
    fn text(&self) -> String {
        self.printed(0)
    }

    fn printed(&self, prec: u8) -> String {
        let wrap = |s: String, on: bool| if on { format!("({s})") } else { s };
        match (self.nat(), self.shape()) {
            (Some(n), _) => n.to_string(),
            (None, Shape::Var(n) | Shape::Const(n)) => n.clone(),
            (None, Shape::Lam(x, b)) => wrap(format!("λ{x}. {}", b.printed(0)), prec >= 1),
            (None, Shape::Let(x, v, b)) => wrap(format!("let {x} := {}; {}", v.printed(0), b.printed(0)), prec >= 1),
            (None, Shape::App(f, a)) => wrap(format!("{} {}", f.printed(1), a.printed(2)), prec >= 2),
            (None, Shape::Pair(a, b)) => format!("({}, {})", a.printed(0), b.printed(0)),
            (None, Shape::Proj(i, t)) => format!("{}.{i}", t.printed(3)),
        }
    }
}
