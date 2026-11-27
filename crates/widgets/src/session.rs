use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

use examples::*;
use rules::*;
use strategy::*;
use syntax::*;
use term::*;

/// What changed: a new program, a move through the history, or a request that the workbench bring
/// itself into view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Load,
    Step,
    Reveal,
}

/// One history entry: a term, the rule that produced it, and a note for the trace.
#[derive(Debug, Clone)]
pub struct Entry {
    term: Term,
    kind: Option<Kind>,
    note: Option<Note>,
}

impl Entry {
    pub fn new(term: Term, kind: Option<Kind>, note: Option<Note>) -> Self {
        Self { term, kind, note }
    }

    pub fn term(&self) -> &Term {
        &self.term
    }

    pub fn kind(&self) -> Option<Kind> {
        self.kind
    }

    pub fn note(&self) -> Option<&Note> {
        self.note.as_ref()
    }
}

/// A loaded program and how far through it the reader is.
#[derive(Debug, Default)]
struct Program {
    env: Env,
    notes: Vec<Note>,
    src: Source,
    example: Option<Example>,
    eta: bool,
    hist: Vec<Entry>,
    idx: Step,
    capture: Option<Term>,
}

impl Program {
    fn env(&self) -> &Env {
        &self.env
    }

    fn notes(&self) -> &Vec<Note> {
        &self.notes
    }

    fn src(&self) -> &Source {
        &self.src
    }

    fn example(&self) -> Option<Example> {
        self.example
    }

    fn eta(&self) -> bool {
        self.eta
    }

    fn hist(&self) -> &Vec<Entry> {
        &self.hist
    }

    fn hist_mut(&mut self) -> &mut Vec<Entry> {
        &mut self.hist
    }

    fn idx(&self) -> Step {
        self.idx
    }

    fn idx_mut(&mut self) -> &mut Step {
        &mut self.idx
    }

    fn capture(&self) -> Option<&Term> {
        self.capture.as_ref()
    }

    fn capture_mut(&mut self) -> &mut Option<Term> {
        &mut self.capture
    }

    fn here(&self) -> &Term {
        self.hist()[self.idx().get()].term()
    }
}

/// The session every widget on a page shares. Widgets render from it and call its methods; it
/// tells them synchronously, in mount order, what changed.
#[derive(Clone, Default)]
pub struct Session {
    program: Rc<RefCell<Program>>,
    busy: Rc<Cell<bool>>,
    listeners: Rc<RefCell<Vec<Rc<dyn Fn(Event)>>>>,
}

impl Session {
    fn program(&self) -> &Rc<RefCell<Program>> {
        &self.program
    }

    fn flag(&self) -> &Rc<Cell<bool>> {
        &self.busy
    }

    fn listeners(&self) -> &Rc<RefCell<Vec<Rc<dyn Fn(Event)>>>> {
        &self.listeners
    }

    pub fn term(&self) -> Term {
        self.program().borrow().here().clone()
    }

    pub fn start(&self) -> Term {
        self.program().borrow().hist()[0].term().clone()
    }

    pub fn env(&self) -> Env {
        self.program().borrow().env().clone()
    }

    pub fn notes(&self) -> Vec<Note> {
        self.program().borrow().notes().clone()
    }

    /// The source the program was loaded from.
    pub fn src(&self) -> Source {
        self.program().borrow().src().clone()
    }

    pub fn example(&self) -> Option<Example> {
        self.program().borrow().example()
    }

    /// Whether the program wants η edges in the graph.
    pub fn eta(&self) -> bool {
        self.program().borrow().eta()
    }

    /// `history()[0]` is the start term.
    pub fn history(&self) -> Vec<Entry> {
        self.program().borrow().hist().clone()
    }

    pub fn loaded(&self) -> bool {
        !self.program().borrow().hist().is_empty()
    }

    /// The position in the history.
    pub fn idx(&self) -> Step {
        self.program().borrow().idx()
    }

    /// The last position in the history.
    pub fn last(&self) -> Step {
        Step::new(self.program().borrow().hist().len().saturating_sub(1))
    }

    /// The wrong, capturing result of the last renamed β.
    pub fn capture(&self) -> Option<Term> {
        self.program().borrow().capture().cloned()
    }

    pub fn set_capture(&self, t: Term) {
        *self.program().borrow_mut().capture_mut() = Some(t);
    }

    /// A widget is animating or running a strategy.
    pub fn busy(&self) -> bool {
        self.flag().get()
    }

    pub fn set_busy(&self, busy: bool) {
        self.flag().set(busy);
    }

    pub fn on<F>(&self, f: F)
    where
        F: Fn(Event) + 'static,
    {
        self.listeners().borrow_mut().push(Rc::new(f));
    }

    fn emit(&self, e: Event) {
        let listeners = self.listeners().borrow().clone();
        listeners.iter().for_each(|f| f(e));
    }

    /// Fails with a readable message when the source does not parse.
    pub fn load(&self, src: &Source, example: Option<Example>, eta: bool) -> Result<(), Failure> {
        let prog = syntax::Program::parse(src)?;
        prog.env().iter().for_each(|(name, _)| Consts::add(name));
        *self.program().borrow_mut() = Program {
            env: prog.env().clone(),
            notes: prog.notes().to_vec(),
            src: src.clone(),
            example,
            eta,
            hist: vec![Entry::new(prog.term().clone(), None, None)],
            idx: Step::START,
            capture: None,
        };
        self.emit(Event::Load);
        Ok(())
    }

    pub fn load_example(&self, ex: Example) {
        let _ = self.load(&ex.src(), Some(ex), ex.eta());
    }

    pub fn push(&self, entry: Entry) {
        {
            let mut p = self.program().borrow_mut();
            let keep = p.idx().get() + 1;
            p.hist_mut().truncate(keep);
            p.hist_mut().push(entry);
            *p.idx_mut() = Step::new(p.hist().len() - 1);
        }
        self.emit(Event::Step);
    }

    pub fn seek(&self, i: Step) {
        *self.program().borrow_mut().idx_mut() = i;
        self.emit(Event::Step);
    }

    pub fn reset(&self) {
        {
            let mut p = self.program().borrow_mut();
            p.hist_mut().truncate(1);
            *p.idx_mut() = Step::START;
            *p.capture_mut() = None;
        }
        self.emit(Event::Step);
    }

    pub fn reveal(&self) {
        self.emit(Event::Reveal);
    }

    pub fn replace_history(&self, hist: Vec<Entry>) {
        {
            let mut p = self.program().borrow_mut();
            *p.idx_mut() = Step::new(hist.len() - 1);
            *p.hist_mut() = hist;
            *p.capture_mut() = None;
        }
        self.emit(Event::Step);
    }
}
