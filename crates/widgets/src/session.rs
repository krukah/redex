use std::cell::Ref;
use std::cell::RefCell;
use std::rc::Rc;

use examples::*;
use rules::*;
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
    pub term: Term,
    pub kind: Option<Kind>,
    pub note: Option<String>,
}

/// The shared state of every widget on a page.
#[derive(Debug, Default)]
pub struct State {
    /// δ-unfoldable definitions.
    pub env: Env,
    /// Comment lines from the source.
    pub notes: Vec<String>,
    pub src: String,
    /// The loaded example, if any.
    pub ex: Option<String>,
    /// Whether the example wants η edges in the graph.
    pub eta: bool,
    /// `hist[0]` is the start term.
    pub hist: Vec<Entry>,
    pub idx: usize,
    /// The wrong, capturing result of the last renamed β.
    pub capture: Option<Term>,
    /// A widget is animating or running a strategy.
    pub busy: bool,
}

/// The session every widget on a page shares. Widgets render from it and call its methods; it
/// tells them synchronously, in mount order, what changed.
#[derive(Clone, Default)]
pub struct Session {
    state: Rc<RefCell<State>>,
    listeners: Rc<RefCell<Vec<Rc<dyn Fn(Event)>>>>,
}

impl Session {
    pub fn state(&self) -> Ref<'_, State> {
        self.state.borrow()
    }

    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut State),
    {
        f(&mut self.state.borrow_mut());
    }

    pub fn term(&self) -> Term {
        let s = self.state();
        s.hist[s.idx].term.clone()
    }

    pub fn start(&self) -> Term {
        self.state().hist[0].term.clone()
    }

    pub fn env(&self) -> Env {
        self.state().env.clone()
    }

    pub fn busy(&self) -> bool {
        self.state().busy
    }

    pub fn on<F>(&self, f: F)
    where
        F: Fn(Event) + 'static,
    {
        self.listeners.borrow_mut().push(Rc::new(f));
    }

    pub fn emit(&self, e: Event) {
        let listeners = self.listeners.borrow().clone();
        listeners.iter().for_each(|f| f(e));
    }

    /// Fails with a readable message when the source does not parse.
    pub fn load(&self, src: &str, id: Option<&str>, eta: bool) -> Result<(), String> {
        let prog = Program::parse(src)?;
        prog.env().iter().for_each(|(name, _)| Consts::add(name));
        self.update(|s| {
            *s = State {
                env: prog.env().clone(),
                notes: prog.notes().to_vec(),
                src: src.to_string(),
                ex: id.map(String::from),
                eta,
                hist: vec![Entry {
                    term: prog.term().clone(),
                    kind: None,
                    note: None,
                }],
                idx: 0,
                capture: None,
                busy: s.busy,
            };
        });
        self.emit(Event::Load);
        Ok(())
    }

    pub fn load_example(&self, id: &str) {
        if let Some(ex) = Example::find(id) {
            let _ = self.load(ex.src(), Some(id), ex.eta());
        }
    }

    pub fn push(&self, entry: Entry) {
        self.update(|s| {
            s.hist.truncate(s.idx + 1);
            s.hist.push(entry);
            s.idx = s.hist.len() - 1;
        });
        self.emit(Event::Step);
    }

    pub fn seek(&self, i: usize) {
        self.update(|s| s.idx = i);
        self.emit(Event::Step);
    }

    pub fn reset(&self) {
        self.update(|s| {
            s.hist.truncate(1);
            s.idx = 0;
            s.capture = None;
        });
        self.emit(Event::Step);
    }

    pub fn reveal(&self) {
        self.emit(Event::Reveal);
    }

    pub fn replace_history(&self, hist: Vec<Entry>) {
        self.update(|s| {
            s.idx = hist.len() - 1;
            s.hist = hist;
            s.capture = None;
        });
        self.emit(Event::Step);
    }
}
