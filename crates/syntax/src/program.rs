use std::collections::HashSet;

use term::*;

use crate::*;

/// A parsed source: definitions, `--` notes, and the term after them.
#[derive(Debug, Clone)]
pub struct Program {
    env: Env,
    notes: Vec<Note>,
    term: Term,
}

impl Program {
    pub fn parse(src: &Source) -> Result<Self, Failure> {
        let mut env = Env::default();
        let mut notes = Vec::new();
        let mut body = Vec::new();
        for (i, line) in src
            .as_str()
            .split('\n')
            .map(|l| l.strip_suffix('\r').unwrap_or(l))
            .enumerate()
        {
            match (Line::note(line), Line::def(line)) {
                (Some(note), _) => notes.push(Note::from(note)),
                (None, Some((name, def))) => Self::define(&mut env, i + 1, &Name::from(name), def)?,
                (None, None) => body.push(line),
            }
        }
        match Line::trim(&body.join("\n")) {
            "" => Err(Failure::from("Write a term after the definitions.")),
            text => Ok(Self {
                term: Self::resolve(&Parser::parse(text)?, &env, &HashSet::new()),
                env,
                notes,
            }),
        }
    }

    fn define(env: &mut Env, line: usize, name: &Name, src: &str) -> Result<(), Failure> {
        if name.prim().is_some() || Prim::alias(name.as_str()).is_some() {
            return Err(Failure::new(format!("{name} is built in and cannot be redefined.")).at(line));
        }
        let t = Parser::parse(src).map_err(|e| e.at(line))?;
        let t = Self::resolve(&t, env, &HashSet::new());
        env.set(name, t);
        Ok(())
    }

    /// Free names that are primitives or definitions become constants.
    fn resolve(t: &Term, env: &Env, bound: &HashSet<Name>) -> Term {
        let inside = |x: &Name| bound.iter().cloned().chain([x.clone()]).collect::<HashSet<Name>>();
        match t.shape() {
            Shape::Var(n) if bound.contains(n) => t.clone(),
            Shape::Var(n) => {
                let n = Prim::alias(n.as_str()).map_or_else(|| n.clone(), Name::from);
                if n.prim().is_some() || env.has(&n) { Term::cons(n) } else { t.clone() }
            }
            Shape::Const(_) => t.clone(),
            Shape::Lam(x, b) => Term::lam(x.clone(), Self::resolve(b, env, &inside(x))),
            Shape::Let(x, v, b) => {
                Term::local(x.clone(), Self::resolve(v, env, bound), Self::resolve(b, env, &inside(x)))
            }
            Shape::App(f, a) => Term::app(Self::resolve(f, env, bound), Self::resolve(a, env, bound)),
            Shape::Pair(a, b) => Term::pair(Self::resolve(a, env, bound), Self::resolve(b, env, bound)),
            Shape::Proj(i, s) => Term::proj(*i, Self::resolve(s, env, bound)),
        }
    }

    pub fn env(&self) -> &Env {
        &self.env
    }

    pub fn notes(&self) -> &[Note] {
        &self.notes
    }

    pub fn term(&self) -> &Term {
        &self.term
    }
}
