use crate::*;

/// δ-unfoldable definitions, in definition order. Redefining a name keeps its first position.
#[derive(Debug, Clone, Default)]
pub struct Env(Vec<(Name, Term)>);

impl Env {
    pub fn set(&mut self, name: &Name, term: Term) {
        match self.0.iter_mut().find(|(n, _)| n == name) {
            Some(slot) => slot.1 = term,
            None => self.0.push((name.clone(), term)),
        }
    }

    pub fn get(&self, name: &Name) -> Option<&Term> {
        self.0.iter().find(|(n, _)| n == name).map(|(_, t)| t)
    }

    pub fn has(&self, name: &Name) -> bool {
        self.get(name).is_some()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Name, &Term)> {
        self.0.iter().map(|(n, t)| (n, t))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
