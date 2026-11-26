use rules::*;
use term::*;

/// One term of a run and the rule that produced it (`None` for the start).
#[derive(Debug, Clone)]
pub struct Row {
    term: Term,
    kind: Option<Kind>,
}

impl Row {
    pub fn new(term: Term, kind: Option<Kind>) -> Self {
        Self { term, kind }
    }

    pub fn term(&self) -> &Term {
        &self.term
    }

    pub fn kind(&self) -> Option<Kind> {
        self.kind
    }
}
