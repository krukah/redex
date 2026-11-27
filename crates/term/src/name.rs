use std::collections::HashSet;

use crate::*;

/// The name of a variable, a binder, a constant or a definition.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Name(String);

impl Name {
    const SUBS: [char; 10] = ['₀', '₁', '₂', '₃', '₄', '₅', '₆', '₇', '₈', '₉'];

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// A subscript digit.
    pub fn sub(c: char) -> bool {
        ('₀'..='₉').contains(&c)
    }

    /// The name without its trailing subscript digits.
    pub fn stem(&self) -> Name {
        Self(self.0.trim_end_matches(Self::sub).to_string())
    }

    pub fn prim(&self) -> Option<Prim> {
        Prim::find(&self.0)
    }

    /// The first `stem₁`, `stem₂`, … that is neither in `avoid` nor a constant.
    pub fn fresh(&self, avoid: &HashSet<Name>) -> Name {
        (1..=u32::MAX)
            .map(|i| {
                Self(format!(
                    "{}{}",
                    self.stem().0,
                    i.to_string()
                        .chars()
                        .map(|d| Self::SUBS[d as usize - '0' as usize])
                        .collect::<String>()
                ))
            })
            .find(|n| !avoid.contains(n) && !Consts::has(n))
            .expect("names are unbounded")
    }
}

impl From<&str> for Name {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<Prim> for Name {
    fn from(p: Prim) -> Self {
        Self(p.name().to_string())
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
