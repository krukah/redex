use std::collections::HashSet;

use term::*;

use crate::*;

/// A parsed source: definitions, `--` notes, and the term after them.
#[derive(Debug, Clone)]
pub struct Program {
    env: Env,
    notes: Vec<String>,
    term: Term,
}

impl Program {
    const ALIAS: [(&'static str, &'static str); 2] = [("Nat.zero", "zero"), ("Nat.succ", "succ")];
    pub fn parse(src: &str) -> Result<Self, String> {
        let mut env = Env::default();
        let mut notes = Vec::new();
        let mut body = Vec::new();
        for (i, line) in src.split('\n').map(|l| l.strip_suffix('\r').unwrap_or(l)).enumerate() {
            match (Line::note(line), Line::def(line)) {
                (Some(note), _) => notes.push(note.to_string()),
                (None, Some((name, def))) => Self::define(&mut env, i + 1, name, def)?,
                (None, None) => body.push(line),
            }
        }
        match Line::trim(&body.join("\n")) {
            "" => Err("Write a term after the definitions.".to_string()),
            text => Ok(Self {
                term: Self::resolve(&Parser::parse(text)?, &env, &HashSet::new()),
                env,
                notes,
            }),
        }
    }

    fn define(env: &mut Env, line: usize, name: &str, src: &str) -> Result<(), String> {
        if Consts::prim(name) || Self::ALIAS.iter().any(|(a, _)| *a == name) {
            return Err(format!("Line {line}: {name} is built in and cannot be redefined."));
        }
        let t = Parser::parse(src).map_err(|e| format!("Line {line}: {e}"))?;
        let t = Self::resolve(&t, env, &HashSet::new());
        env.set(name, t);
        Ok(())
    }

    /// Free names that are primitives or definitions become constants.
    fn resolve(t: &Term, env: &Env, bound: &HashSet<String>) -> Term {
        let inside = |x: &str| {
            bound
                .iter()
                .cloned()
                .chain([x.to_string()])
                .collect::<HashSet<String>>()
        };
        match t.shape() {
            Shape::Var(n) if bound.contains(n) => t.clone(),
            Shape::Var(n) => {
                let n = Self::ALIAS.iter().find(|(a, _)| a == n).map_or(n.as_str(), |(_, p)| p);
                if Consts::prim(n) || env.has(n) { Term::cons(n) } else { t.clone() }
            }
            Shape::Const(_) => t.clone(),
            Shape::Lam(x, b) => Term::lam(x, Self::resolve(b, env, &inside(x))),
            Shape::Let(x, v, b) => Term::local(x, Self::resolve(v, env, bound), Self::resolve(b, env, &inside(x))),
            Shape::App(f, a) => Term::app(Self::resolve(f, env, bound), Self::resolve(a, env, bound)),
            Shape::Pair(a, b) => Term::pair(Self::resolve(a, env, bound), Self::resolve(b, env, bound)),
            Shape::Proj(i, s) => Term::proj(*i, Self::resolve(s, env, bound)),
        }
    }

    pub fn env(&self) -> &Env {
        &self.env
    }

    pub fn notes(&self) -> &[String] {
        &self.notes
    }

    pub fn term(&self) -> &Term {
        &self.term
    }
}
