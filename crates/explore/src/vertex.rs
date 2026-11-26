use rules::*;
use term::*;

/// One term of the graph, keyed by α-equivalence.
#[derive(Debug, Clone)]
pub struct Vertex {
    key: String,
    term: Term,
    depth: usize,
    parent: Option<usize>,
    pkind: Option<Kind>,
    nf: bool,
    open: bool,
}

impl Vertex {
    pub fn new(key: String, term: Term, depth: usize, parent: Option<usize>, pkind: Option<Kind>) -> Self {
        Self {
            key,
            term,
            depth,
            parent,
            pkind,
            nf: false,
            open: false,
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn term(&self) -> &Term {
        &self.term
    }

    /// Steps from the start on a shortest path.
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// The vertex this one was first reached from.
    pub fn parent(&self) -> Option<usize> {
        self.parent
    }

    /// The rule of the step from the parent.
    pub fn pkind(&self) -> Option<Kind> {
        self.pkind
    }

    /// No redex left (η counts only when the graph includes it).
    pub fn nf(&self) -> bool {
        self.nf
    }

    /// Has redexes the exploration did not follow.
    pub fn open(&self) -> bool {
        self.open
    }

    pub fn set_nf(&mut self, nf: bool) {
        self.nf = nf;
    }

    pub fn set_open(&mut self) {
        self.open = true;
    }
}
