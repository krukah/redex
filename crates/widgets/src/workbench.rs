use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;

use leptos::html;
use leptos::prelude::*;
use markup::Selector;
use markup::*;
use motion::*;
use rules::*;
use strategy::*;
use subst::*;
use syntax::*;
use term::*;
use units::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::DomRect;
use web_sys::Element;

use crate::*;

/// What a run does: one step, the head to WHNF, or everything to a normal form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Step,
    Whnf,
    Nf,
}

/// One pattern variable's piece of the term while a redex is matched.
#[derive(Clone)]
struct Piece {
    var: Var,
    slot: Option<Slot>,
    el: Element,
    uid: Option<Uid>,
    binder: Option<Anchor>,
    text: Option<String>,
    external: bool,
    rect: Option<DomRect>,
    ghost: Option<Element>,
}

impl Piece {
    fn new(var: Var, slot: Option<Slot>, el: Element) -> Self {
        Self {
            var,
            slot,
            el,
            uid: None,
            binder: None,
            text: None,
            external: false,
            rect: None,
            ghost: None,
        }
    }

    fn var(&self) -> Var {
        self.var
    }

    fn slot(&self) -> Option<Slot> {
        self.slot
    }

    fn el(&self) -> &Element {
        &self.el
    }

    fn uid(&self) -> Option<Uid> {
        self.uid
    }

    fn binder(&self) -> Option<Anchor> {
        self.binder
    }

    fn text(&self) -> Option<&String> {
        self.text.as_ref()
    }

    fn external(&self) -> bool {
        self.external
    }

    fn rect(&self) -> Option<&DomRect> {
        self.rect.as_ref()
    }

    fn ghost(&self) -> Option<&Element> {
        self.ghost.as_ref()
    }

    /// A piece of the term moves; a binder's tab and a definition's body stay where they are.
    fn travels(&self) -> bool {
        self.uid().is_some() && !self.external()
    }

    fn measured(self) -> Self {
        let rect = Some(self.el().get_bounding_client_rect());
        Self { rect, ..self }
    }

    fn haunted(self, ghost: Option<Element>) -> Self {
        Self { ghost, ..self }
    }
}

/// A redex matched against its rule: the schema and the pieces its variables bound.
struct Match {
    schema: Schema,
    pieces: Vec<Piece>,
}

impl Match {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn pieces(&self) -> &[Piece] {
        &self.pieces
    }
}

/// The redex pinned over the page before a step, and the layout it left.
struct Lifted {
    husk: Option<Element>,
    pieces: Vec<Piece>,
    stage: Option<DomRect>,
    layout: HashMap<Uid, DomRect>,
}

impl Lifted {
    fn husk(&self) -> Option<&Element> {
        self.husk.as_ref()
    }

    fn pieces(&self) -> &[Piece] {
        &self.pieces
    }

    fn stage(&self) -> Option<&DomRect> {
        self.stage.as_ref()
    }

    fn layout(&self) -> &HashMap<Uid, DomRect> {
        &self.layout
    }
}

/// One arc from each bound occurrence up to its binder. Both ends rise to a shared apex; longer
/// links rise higher, never past the ceiling.
struct Links;

impl Links {
    fn draw(stage: &Element, line: &Element, on: bool) {
        let Some(svg) = Doc::q(stage, &Class::Arcs.into()) else {
            return;
        };
        if !on {
            Doc::fill(&svg, &Node::none());
            return;
        }
        let base = stage.get_bounding_client_rect();
        let ceiling = Px::new(line.get_bounding_client_rect().top() - base.top() - 30.0);
        let arcs = Doc::all(line, &Selector::Has(Data::Bid)).into_iter().filter_map(|v| {
            let id = Doc::read::<Anchor>(&v, Data::Bid)?;
            let anchor = Doc::q(line, &Selector::is(Data::Anchor, &id))?;
            let (a, b) = (anchor.get_bounding_client_rect(), v.get_bounding_client_rect());
            let (x1, x2) =
                (Px::new(a.left() + a.width() / 2.0 - base.left()), Px::new(b.left() + b.width() / 2.0 - base.left()));
            let (y1, y2) = (Px::new(a.top() - base.top() + 2.0), Px::new(b.top() - base.top() + 2.0));
            let apex = ceiling.max(y1.min(y2) - Px::new(10.0) - (x2 - x1).abs() * 0.12);
            let cap = Doc::has(&v, Class::Cap).then_some(Class::Cap);
            let f = Num::Tenths;
            Some(Node::list([
                Node::el(Tag::Path)
                    .class([Class::Arc].into_iter().chain(cap))
                    .attr(Attr::Arc(id))
                    .attr(Attr::D(Trace::spaced(vec![
                        Seg::Move(f(x1), f(y1)),
                        Seg::Cubic(f(x1), f(apex), f(x2), f(apex), f(x2), f(y2)),
                    ]))),
                Node::el(Tag::Circle)
                    .class([Class::ArcEnd].into_iter().chain(cap))
                    .attr(Attr::Arc(id))
                    .attr(Attr::Cx(f(x2)))
                    .attr(Attr::Cy(f(y2)))
                    .attr(Attr::R(Num::Js(Px::new(2.2)))),
            ]))
        });
        Doc::fill(&svg, &Node::list(arcs));
    }
}

/// The workbench. Every firing follows one grammar: match (hovering a badge shows the rule's
/// schema; its pattern variables and the pieces they matched share a highlighter color), bind (the
/// rule card lists the bindings), move (each bound piece flies to every place the result uses it).
struct Bench {
    session: Session,
    anchor: NodeRef<html::Div>,
    arcs: Cell<bool>,
    db: Cell<bool>,
    hot: RefCell<Option<Element>>,
    running: Cell<Option<Mode>>,
    stop: Cell<bool>,
    note: RefCell<Option<Note>>,
    raf: Cell<i32>,
}

impl Bench {
    const NAMES: [&'static str; 20] = [
        "x", "y", "z", "w", "u", "v", "a", "b", "c", "f", "g", "h", "k", "m", "n", "p", "q", "r", "s", "t",
    ];
    /// How long a firing shows its match before anything moves, at speed 1.
    const HOLD: Ms = Ms::new(560.0);
    /// How long an automatic α step stays on screen before its β fires.
    const RENAME: Ms = Ms::new(1000.0);
    /// Between the pieces of a firing, and between the copies of one piece.
    const PIECES: Ms = Ms::new(50.0);
    const COPIES: Ms = Ms::new(40.0);
    /// How much of a bound piece the rule card shows, in UTF-16 units as the JS counts them.
    const SHORT: usize = 34;
    /// A run stops after this many steps, or once the term passes this size.
    const STEPS: Step = Step::new(200);
    const LARGE: Size = Size::new(2000);

    fn session(&self) -> &Session {
        &self.session
    }

    fn anchor(&self) -> NodeRef<html::Div> {
        self.anchor
    }

    fn arcs(&self) -> &Cell<bool> {
        &self.arcs
    }

    fn db(&self) -> &Cell<bool> {
        &self.db
    }

    fn hot(&self) -> &RefCell<Option<Element>> {
        &self.hot
    }

    fn running(&self) -> &Cell<Option<Mode>> {
        &self.running
    }

    fn stop(&self) -> &Cell<bool> {
        &self.stop
    }

    fn note(&self) -> &RefCell<Option<Note>> {
        &self.note
    }

    fn raf(&self) -> &Cell<i32> {
        &self.raf
    }

    fn rule(kind: Kind) -> &'static str {
        match kind {
            Kind::Alpha => "α-conversion",
            Kind::Beta => "β-reduction",
            Kind::Eta => "η-reduction",
            Kind::EtaS => "η for structures",
            Kind::Delta => "δ-unfolding",
            Kind::Zeta => "ζ-reduction",
            Kind::Iota => "ι-reduction",
            Kind::Proj => "projection",
        }
    }

    /// The text cut to `SHORT` UTF-16 units, as `String.prototype.slice` counts them.
    fn short(s: &str) -> String {
        let units = s.encode_utf16().collect::<Vec<u16>>();
        if units.len() > Self::SHORT {
            String::from_utf16_lossy(&units[..Self::SHORT - 1]) + "…"
        } else {
            s.to_string()
        }
    }

    fn hint() -> Node {
        Node::el(Tag::P).class([Class::Hint]).kids([
            Node::text("Hover a badge to see its rule match. Click to fire it. Click a binder to rename it. "),
            Node::el(Tag::Kbd).kid(Node::text("→")),
            Node::text(" steps, "),
            Node::el(Tag::Kbd).kid(Node::text("←")),
            Node::text(" goes back."),
        ])
    }

    fn root(&self) -> Option<Element> {
        Doc::above(self.anchor().get_untracked())
    }

    fn q(&self, sel: &Selector) -> Option<Element> {
        self.root().and_then(|r| Doc::q(&r, sel))
    }

    fn line(&self) -> Option<Element> {
        self.q(&Class::Line.into())
    }

    fn stage(&self) -> Option<Element> {
        self.q(&Class::Stage.into())
    }

    fn node(&self, p: &Path) -> Option<Element> {
        self.line().and_then(|l| Doc::q(&l, &Selector::is(Data::Path, p)))
    }

    fn button(&self, act: Act) -> Option<web_sys::HtmlButtonElement> {
        self.q(&Selector::is(Data::Act, &act)).and_then(|b| b.dyn_into().ok())
    }

    fn source(&self) -> Option<web_sys::HtmlTextAreaElement> {
        self.q(&Selector::from(Class::Src).within(Tag::Textarea.into()))
            .and_then(|e| e.dyn_into().ok())
    }

    fn render(&self) {
        let t = self.session().term();
        if let Some(line) = self.line() {
            Doc::fill(&line, &Blocks::new(&self.session().env(), true, self.db().get()).node(&t));
        }
        t.strip();
        self.links();
        *self.hot().borrow_mut() = None;
        if !self.session().busy() {
            self.show_rule(None);
        }
        self.render_status();
        self.render_trace();
        self.render_capture();
        self.set_controls();
    }

    fn links(&self) {
        if let (Some(stage), Some(line)) = (self.stage(), self.line()) {
            Links::draw(&stage, &line, self.arcs().get());
        }
    }

    fn render_source(&self) {
        let env = self.session().env();
        if let Some(notes) = self.q(&Class::Notes.into()) {
            Doc::fill(
                &notes,
                &Node::list(self.session().notes().iter().map(|n| {
                    Node::el(Tag::P)
                        .kid(Node::el(Tag::Span).class([Class::Dash]).kid(Node::text("--")))
                        .kid(Node::text(format!(" {n}")))
                })),
            );
        }
        if let Some(defs) = self.q(&Class::Defs.into()) {
            Doc::hide(&defs, env.is_empty());
        }
        if let Some(list) = self.q(&Class::DefsList.into()) {
            let defs = env.iter().map(|(n, b)| {
                Node::el(Tag::Div)
                    .class([Class::DefRow, Class::Tm])
                    .kid(
                        Node::el(Tag::Span)
                            .class([Class::Block, Class::Const, Class::Def])
                            .kid(Node::text(n)),
                    )
                    .kid(Node::el(Tag::Span).class([Class::Keyword]).kid(Node::text(":=")))
                    .kid(
                        Node::el(Tag::Span)
                            .attr(Attr::Def(n.clone()))
                            .kid(Blocks::new(&env, false, false).node(b)),
                    )
            });
            Doc::fill(&list, &Node::list(defs));
        }
        if let Some(src) = self.source() {
            src.set_value(&self.session().src());
        }
        if let Some(err) = self.q(&Class::Err.into()) {
            Doc::hide(&err, true);
        }
    }

    fn render_status(&self) {
        let (t, env) = (self.session().term(), self.session().env());
        let rs = t.redexes(&env);
        let main = rs.iter().filter(|r| !r.kind().is_eta()).count();
        let head = match (main, Strategy::Whnf.next(&t, &env)) {
            (0, _) => Node::el(Tag::B)
                .class([Class::NfLabel])
                .kid(Node::text("Normal form"))
                .kids(
                    (!rs.is_empty())
                        .then(|| Node::list([Node::text(" "), Node::el(Tag::Span).kid(Node::text("except for η"))])),
                ),
            (_, None) => Node::el(Tag::B)
                .class([Class::WhnfLabel])
                .kid(Node::text("Weak head normal form")),
            _ => {
                Node::el(Tag::B).kid(Node::text(format!("{} redex{}", rs.len(), if rs.len() > 1 { "es" } else { "" })))
            }
        };
        let tally = rs.iter().fold(Vec::<(Kind, usize)>::new(), |mut tally, r| {
            match tally.iter_mut().find(|(k, _)| *k == r.kind()) {
                Some(slot) => slot.1 += 1,
                None => tally.push((r.kind(), 1)),
            }
            tally
        });
        let marks = (!tally.is_empty()).then(|| {
            Node::el(Tag::Span)
                .class([Class::Tallies])
                .kids(tally.iter().map(|(k, n)| {
                    Node::el(Tag::Span)
                        .class([Class::Tally])
                        .kid(Letter::flat(Some(*k)))
                        .kids((*n > 1).then(|| Node::text(format!("×{n}"))))
                }))
        });
        let (idx, last) = (self.session().idx(), self.session().last());
        let stepno =
            format!("step {idx}{} · size {}", if last > idx { format!(" of {last}") } else { String::new() }, t.size());
        let note = self
            .note()
            .borrow()
            .clone()
            .map(|n| Node::el(Tag::Span).class([Class::Note]).kid(Node::text(n)));
        if let Some(status) = self.q(&Class::Status.into()) {
            Doc::fill(
                &status,
                &Node::list([head])
                    .kids(marks)
                    .kid(Node::el(Tag::Span).class([Class::Stepno]).kid(Node::text(stepno)))
                    .kids(note),
            );
        }
    }

    fn render_trace(&self) {
        let Some(ol) = self
            .q(&Selector::from(Class::Trace).within(Tag::Ol.into()))
            .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
        else {
            return;
        };
        let idx = self.session().idx();
        let rows = self.session().history().into_iter().enumerate().map(|(i, h)| {
            let i = Step::new(i);
            let place = match i.cmp(&idx) {
                std::cmp::Ordering::Equal => vec![Class::Cur],
                std::cmp::Ordering::Greater => vec![Class::Blank, Class::Fut],
                std::cmp::Ordering::Less => vec![Class::Blank],
            };
            let button = Node::el(Tag::Button)
                .attr(Attr::Button)
                .attr(Attr::I(i))
                .kid(Node::el(Tag::Span).class([Class::Ti]).kid(Node::text(i)))
                .kid(Letter::flat(h.kind()))
                .kid(Node::el(Tag::Code).kid(Node::text(h.term().text())))
                .kids(h.note().map(|n| Node::el(Tag::Small).kid(Node::text(n))));
            Node::el(Tag::Li).class(place).kid(button)
        });
        Doc::fill(&ol, &Node::list(rows));
        let Some(li) = ol
            .children()
            .item(idx.get() as u32)
            .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
        else {
            return;
        };
        let (top, bot) = (li.offset_top(), li.offset_top() + li.offset_height());
        if top < ol.scroll_top() {
            ol.set_scroll_top(top - 4);
        } else if bot > ol.scroll_top() + ol.client_height() {
            ol.set_scroll_top(bot - ol.client_height() + 4);
        }
    }

    fn render_capture(&self) {
        let capture = self.session().capture();
        let Some(panel) = self.q(&Class::Capture.into()) else {
            return;
        };
        Doc::hide(&panel, capture.is_none());
        let Some(capture) = capture else { return };
        if let (Some(stage), Some(line)) = (Doc::q(&panel, &Class::Stage.into()), Doc::q(&panel, &Class::Line.into())) {
            Doc::fill(&line, &Blocks::new(&self.session().env(), false, false).node(&capture));
            Links::draw(&stage, &line, true);
        }
    }

    fn set_controls(&self) {
        let (t, env, busy) = (self.session().term(), self.session().env(), self.session().busy());
        let (idx, last) = (self.session().idx(), self.session().last());
        let running = self.running().get();
        let (n, w) = (Strategy::Normal.next(&t, &env).is_some(), Strategy::Whnf.next(&t, &env).is_some());
        let set = |act: Act, disabled: bool, label: Option<&str>| {
            if let Some(b) = self.button(act) {
                b.set_disabled(disabled);
                if let Some(label) = label {
                    b.set_text_content(Some(label));
                }
            }
        };
        set(Act::Step, busy || !n, None);
        set(
            Act::Whnf,
            if busy { running != Some(Mode::Whnf) } else { !w },
            Some(if running == Some(Mode::Whnf) { "Stop" } else { "To WHNF" }),
        );
        set(
            Act::Nf,
            if busy { running != Some(Mode::Nf) } else { !n },
            Some(if running == Some(Mode::Nf) { "Stop" } else { "Normalize" }),
        );
        set(Act::Back, busy || idx == Step::START, None);
        set(Act::Reset, busy || (idx == Step::START && last == Step::START), None);
    }

    fn matched(&self, path: &Path, kind: Kind) -> Match {
        let t = self.session().term();
        let redex = t.at(path).cloned().expect("a redex inside the term");
        let schema = kind.schema(&redex);
        let mut pieces = kind
            .bindings(&redex)
            .iter()
            .filter_map(|b| {
                let owner = self.node(&path.join(b.rel()))?;
                let piece = Piece::new(b.var(), schema.slot(b.var()), owner.clone());
                if b.is_binder() {
                    let tab = Doc::q(&owner, &Selector::Scope.child(Class::Tab.into()).child(Class::Binder.into()))
                        .or_else(|| {
                            Doc::q(&owner, &Selector::Scope.child(Class::Tab.into()).child(Class::Keyword.into()))
                        })?;
                    Some(Piece {
                        el: tab,
                        binder: Doc::read(&owner, Data::Binder),
                        ..piece
                    })
                } else {
                    Some(Piece {
                        uid: Doc::read(&owner, Data::Uid),
                        text: redex.at(b.rel()).map(Print::text),
                        ..piece
                    })
                }
            })
            .collect::<Vec<Piece>>();
        if kind == Kind::Delta {
            let body = redex.name().and_then(|n| {
                self.q(&Selector::from(Class::DefsList)
                    .within(Selector::is(Data::Def, n))
                    .child(Class::Block.into()))
            });
            if let Some(body) = body {
                pieces.push(Piece {
                    uid: Doc::read(&body, Data::Uid),
                    external: true,
                    ..Piece::new(Var::new("c"), Slot::ALL.first().copied(), body)
                });
            }
        }
        Match { schema, pieces }
    }

    fn paint(&self, m: &Match, strength: Class) {
        self.unpaint();
        let line = self.line();
        m.pieces().iter().for_each(|pc| {
            let Some(slot) = pc.slot() else { return };
            let classes = [Class::Moving, Class::Slot(slot), strength];
            Doc::add(pc.el(), &classes);
            if let (Some(b), Some(line)) = (pc.binder(), &line) {
                Doc::all(line, &Selector::is(Data::Bid, &b))
                    .iter()
                    .for_each(|v| Doc::add(v, &classes));
            }
        });
    }

    fn unpaint(&self) {
        if let Some(root) = self.root() {
            let classes = [Class::Moving, Class::Soft, Class::Hard]
                .into_iter()
                .chain(Slot::ALL.map(Class::Slot))
                .collect::<Vec<Class>>();
            Doc::all(&root, &Class::Moving.into())
                .iter()
                .for_each(|e| Doc::remove(e, &classes));
        }
    }

    fn show_rule(&self, hit: Option<(&Element, &Match)>) {
        let Some(card) = self.q(&Class::Rule.into()) else {
            return;
        };
        let Some((rx, m)) = hit else {
            Doc::fill(&card, &Self::hint());
            return;
        };
        let Some(kind) = Doc::read::<Kind>(rx, Data::Kind) else {
            return;
        };
        let mv =
            |slot: Option<Slot>| Node::el(Tag::Span).class([Class::Moving].into_iter().chain(slot.map(Class::Slot)));
        let schema = m.schema().parts().into_iter().map(|p| match p {
            Part::Var(v) => mv(m.schema().slot(v)).kid(Node::text(v)),
            Part::Text(s) => Node::text(s),
        });
        let binds = m
            .pieces()
            .iter()
            .filter(|pc| pc.slot().is_some() && !pc.external())
            .map(|pc| {
                let shown = if pc.binder().is_some() {
                    pc.el().text_content().unwrap_or_default()
                } else {
                    pc.text().cloned().unwrap_or_default()
                };
                Node::el(Tag::Span)
                    .class([Class::Bind])
                    .kid(mv(pc.slot()).kid(Node::text(pc.var())))
                    .kid(Node::text(" = "))
                    .kid(Node::el(Tag::Code).kid(Node::text(Self::short(&shown))))
            });
        Doc::fill(
            &card,
            &Node::list([
                Node::el(Tag::P)
                    .class([Class::Schema])
                    .kid(Letter::flat(Some(kind)))
                    .kid(Node::el(Tag::B).kid(Node::text(Self::rule(kind))))
                    .kid(Node::el(Tag::Code).kids(schema)),
                Node::el(Tag::P).class([Class::Binds]).kids(binds),
            ]),
        );
    }

    fn set_hot(&self, rx: Option<Element>) {
        if *self.hot().borrow() == rx {
            return;
        }
        if let Some(old) = self.hot().borrow().as_ref() {
            Doc::remove(old, &[Class::Hot]);
        }
        self.hot().borrow_mut().clone_from(&rx);
        let Some(rx) = rx else {
            self.unpaint();
            self.show_rule(None);
            return;
        };
        Doc::add(&rx, &[Class::Hot]);
        let (Some(path), Some(kind)) = (Doc::read::<Path>(&rx, Data::Path), Doc::read::<Kind>(&rx, Data::Kind)) else {
            return;
        };
        let m = self.matched(&path, kind);
        self.paint(&m, Class::Soft);
        self.show_rule(Some((&rx, &m)));
    }

    fn set_lit(&self, id: Option<Anchor>) {
        let Some(stage) = self.stage() else { return };
        Doc::all(&stage, &Class::Lighted.into())
            .iter()
            .for_each(|e| Doc::remove(e, &[Class::Lighted]));
        if let Some(id) = id {
            let sel = Selector::is(Data::Id, &id)
                .or(Selector::is(Data::Bid, &id))
                .or(Selector::from(Class::Binder).and(Selector::is(Data::Anchor, &id)));
            Doc::all(&stage, &sel)
                .iter()
                .for_each(|e| Doc::add(e, &[Class::Lighted]));
        }
    }

    /// Before the step: pins the redex and its pieces over the page, and measures the layout.
    fn lift(&self, m: &Match, path: &Path) -> Lifted {
        let rx = self.node(path);
        let pieces = m.pieces().iter().cloned().map(Piece::measured).collect::<Vec<Piece>>();
        pieces
            .iter()
            .enumerate()
            .filter(|(_, pc)| pc.travels())
            .for_each(|(i, pc)| Doc::write(pc.el(), Data::Piece, &i));
        let husk = rx.and_then(|rx| Ghost::pin(&rx, None, false, &[]));
        if let Some(husk) = &husk {
            Doc::all(husk, &Selector::Has(Data::Piece))
                .iter()
                .filter_map(|e| e.dyn_ref::<web_sys::HtmlElement>().cloned())
                .for_each(|e| {
                    let _ = e.style().set_property("visibility", "hidden");
                });
        }
        let pieces = pieces
            .into_iter()
            .map(|pc| {
                Doc::erase(pc.el(), Data::Piece);
                let ghost = pc.uid().and_then(|_| Ghost::pin(pc.el(), None, false, &[]));
                pc.haunted(ghost)
            })
            .collect();
        let layout = self
            .line()
            .map(|line| {
                Doc::all(&line, &Selector::Has(Data::Uid))
                    .iter()
                    .filter_map(|e| Some((Doc::read::<Uid>(e, Data::Uid)?, e.get_bounding_client_rect())))
                    .collect()
            })
            .unwrap_or_default();
        Lifted {
            husk,
            pieces,
            stage: self.stage().as_ref().map(Element::get_bounding_client_rect),
            layout,
        }
    }

    /// After the step: the composition that carries the old layout to the new one. The husk fades
    /// out, each piece flies to every node of the result built from it (or fades), the result fades
    /// in, and the context glides.
    fn move_to(self: &Rc<Self>, lifted: &Lifted, path: &Path) -> Motion {
        let Some(result) = self.node(path) else {
            return Motion::all(
                lifted
                    .husk()
                    .into_iter()
                    .chain(lifted.pieces().iter().filter_map(Piece::ghost))
                    .map(|g| Motion::exit(g, Tween::default()))
                    .collect(),
            );
        };
        let husk = lifted
            .husk()
            .map_or_else(Motion::none, |h| Motion::exit(h, Tween::default()));
        let enter = Motion::enter(&result, Tween::default());
        let travels = lifted
            .pieces()
            .iter()
            .map(|pc| Self::travel(pc, &result))
            .collect::<Vec<Option<Motion>>>();
        let glide = self.glide(lifted, &result);
        Motion::all(vec![husk, enter, Motion::stagger(Self::PIECES, travels), glide])
    }

    fn travel(pc: &Piece, result: &Element) -> Option<Motion> {
        let ghost = pc.ghost()?.clone();
        let uid = pc.uid();
        let hits = std::iter::once(result.clone())
            .chain(Doc::all(result, &Selector::Has(Data::Uid).or(Selector::Has(Data::From))))
            .filter(|e| Doc::read::<Uid>(e, Data::Uid) == uid || Doc::read::<Uid>(e, Data::From) == uid)
            .collect::<Vec<Element>>();
        if hits.is_empty() {
            return Some(if pc.external() {
                Motion::act(move || ghost.remove())
            } else {
                Motion::exit(&ghost, Tween::default())
            });
        }
        let rect = pc.rect()?.clone();
        let slot = pc.slot();
        let copies = hits.iter().enumerate().map(|(j, tg)| {
            let src = if j > 0 { Self::twin(&ghost) } else { Some(ghost.clone()) }?;
            let marks = slot
                .map(|s| vec![Class::Moving, Class::Slot(s), Class::Hard])
                .unwrap_or_default();
            let dst = Ghost::pin(tg, Some(&rect), true, &marks)?;
            let (dx, dy) = Ghost::delta(&rect, &tg.get_bounding_client_rect());
            Doc::add(tg, &[Class::Arriving]);
            let (tg, a, b) = (tg.clone(), src.clone(), dst.clone());
            Some(Motion::seq(vec![
                Motion::all(vec![
                    Motion::fly(&src, dx, dy, Tween::default()),
                    Motion::fly(&dst, dx, dy, Tween::default()),
                    Motion::crossfade(&src, &dst, Tween::default()),
                ]),
                Motion::act(move || {
                    Doc::remove(&tg, &[Class::Arriving]);
                    if let Some(s) = slot {
                        Doc::add(&tg, &[Class::Landed, Class::Slot(s)]);
                    }
                }),
                Motion::act(move || {
                    a.remove();
                    b.remove();
                }),
            ]))
        });
        Some(Motion::stagger(Self::COPIES, copies.collect()))
    }

    fn twin(g: &Element) -> Option<Element> {
        let copy = g.clone_node_with_deep(true).ok()?;
        web_sys::window()?
            .document()?
            .body()?
            .append_child(&copy)
            .ok()?
            .dyn_into::<Element>()
            .ok()
    }

    /// Boxes that enclose the result change size, nodes beside it slide, new context fades in,
    /// and the stage resizes. Binding links step aside until the layout settles.
    fn glide(self: &Rc<Self>, lifted: &Lifted, result: &Element) -> Motion {
        let (Some(stage), Some(line)) = (self.stage(), self.line()) else {
            return Motion::none();
        };
        let mut moves = lifted
            .stage()
            .map(|old| {
                vec![Motion::resize(
                    &stage,
                    old,
                    &stage.get_bounding_client_rect(),
                    Tween::default(),
                )]
            })
            .unwrap_or_default();
        Self::visit(&line, lifted.layout(), result, &mut moves);
        let (on, off, me) = (stage.clone(), stage, self.clone());
        Motion::seq(vec![
            Motion::act(move || Doc::add(&on, &[Class::Gliding])),
            Motion::all(moves),
            Motion::act(move || {
                Doc::remove(&off, &[Class::Gliding]);
                me.links();
            }),
        ])
    }

    fn visit(parent: &Element, before: &HashMap<Uid, DomRect>, result: &Element, moves: &mut Vec<Motion>) {
        for c in Doc::children(parent) {
            if &c == result {
                continue;
            }
            let Some(uid) = Doc::read::<Uid>(&c, Data::Uid) else {
                Self::visit(&c, before, result, moves);
                continue;
            };
            let (old, now) = (before.get(&uid), c.get_bounding_client_rect());
            if c.contains(Some(result)) {
                moves.extend(old.map(|old| Motion::resize(&c, old, &now, Tween::default())));
                Self::visit(&c, before, result, moves);
            } else {
                moves.push(old.map_or_else(
                    || Motion::enter(&c, Tween::default()),
                    |old| {
                        Motion::slide(
                            &c,
                            Px::new(old.left() - now.left()),
                            Px::new(old.top() - now.top()),
                            Tween::default(),
                        )
                    },
                ));
            }
        }
    }

    /// The α step that must come first when a β or ζ would capture: its term and its trace note.
    fn rename_first(&self, t: &Term, path: &Path, kind: Kind) -> Option<(Term, Note)> {
        let redex = t.at(path)?;
        let (lam, arg) = match (kind, redex.shape()) {
            (Kind::Beta, Shape::App(f, a)) => (f.clone(), a.clone()),
            (Kind::Zeta, Shape::Let(_, v, _)) => (redex.clone(), v.clone()),
            _ => return None,
        };
        let (Shape::Lam(x, b) | Shape::Let(x, _, b)) = lam.shape() else {
            return None;
        };
        let mut log = Vec::new();
        let body = b.prepare(x, &arg.fv(), &mut log);
        if log.is_empty() {
            return None;
        }
        self.session().set_capture(b.naive(x, &arg));
        let renamed = match redex.shape() {
            Shape::Let(y, v, _) => Term::local(y.clone(), v.clone(), body),
            _ => Term::app(Term::lam(x.clone(), body), arg),
        };
        let olds = log
            .iter()
            .map(|(a, _)| a.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        let moves = log
            .iter()
            .map(|(a, b)| format!("{a} → {b}"))
            .collect::<Vec<String>>()
            .join(", ");
        Some((
            t.replace(path, renamed),
            Note::from(format!("{moves}, so the argument’s free {olds} stays free").as_str()),
        ))
    }

    async fn fire_at(self: Rc<Self>, path: Path, kind: Kind, clock: Clock) {
        if let Some((renamed, note)) = self.rename_first(&self.session().term(), &path, kind) {
            self.session().push(Entry::new(renamed, Some(Kind::Alpha), Some(note)));
            if Motion::enabled() {
                Motion::wait(clock.scale(Self::RENAME)).await;
            }
        }
        let mut lifted = None;
        if Motion::enabled() {
            let (rx, m) = (self.node(&path), self.matched(&path, kind));
            *self.hot().borrow_mut() = None;
            if let Some(rx) = &rx {
                Doc::add(rx, &[Class::Firing]);
            }
            self.show_rule(rx.as_ref().map(|rx| (rx, &m)));
            self.paint(&m, Class::Hard);
            Motion::wait(clock.scale(Self::HOLD)).await;
            lifted = Some(self.lift(&m, &path));
        }
        let (now, env) = (self.session().term(), self.session().env());
        let next = now.replace(&path, now.at(&path).expect("a redex inside the term").contract(kind, &env));
        self.session().push(Entry::new(next, Some(kind), None));
        if let Some(lifted) = lifted {
            self.move_to(&lifted, &path).play(clock).await;
        }
    }

    async fn fire_once(self: Rc<Self>, path: Path, kind: Kind) {
        if self.session().busy() {
            return;
        }
        self.session().set_busy(true);
        *self.note().borrow_mut() = None;
        self.set_controls();
        self.clone().fire_at(path, kind, Clock::default()).await;
        self.session().set_busy(false);
        self.unpaint();
        self.show_rule(None);
        self.set_controls();
    }

    /// The speed of a run's `n`th step: the first few at reading pace, then faster.
    fn pace(mode: Mode, n: Step) -> Clock {
        match (mode, n.get()) {
            (Mode::Step, _) => Clock::default(),
            (_, 0..3) => Clock::new(1.3),
            (_, 3..12) => Clock::new(2.2),
            _ => Clock::new(4.5),
        }
    }

    async fn run(self: Rc<Self>, mode: Mode) {
        if self.session().busy() {
            if self.running().get() == Some(mode) {
                self.stop().set(true);
            }
            return;
        }
        self.session().set_busy(true);
        self.running().set(if mode == Mode::Step { None } else { Some(mode) });
        self.stop().set(false);
        *self.note().borrow_mut() = None;
        self.set_controls();
        let mut seen = self
            .session()
            .history()
            .iter()
            .take(self.session().idx().get() + 1)
            .map(|h| h.term().key())
            .collect::<HashSet<Key>>();
        let (mut n, mut msg) = (Step::START, None);
        while !self.stop().get() {
            let (t, env) = (self.session().term(), self.session().env());
            let Some(r) = (if mode == Mode::Whnf { Strategy::Whnf } else { Strategy::Normal }).next(&t, &env) else {
                break;
            };
            self.clone()
                .fire_at(r.path().clone(), r.kind(), Self::pace(mode, n))
                .await;
            n = n.next();
            if mode == Mode::Step {
                break;
            }
            let term = self.session().term();
            msg = match () {
                () if !seen.insert(term.key()) => {
                    Some("Stopped: this term came back to one it already passed through. It loops.")
                }
                () if n >= Self::STEPS => Some("Stopped after 200 steps."),
                () if term.size() > Self::LARGE => Some("Stopped: the term grew past 2000 nodes."),
                () => None,
            };
            if msg.is_some() {
                break;
            }
        }
        self.session().set_busy(false);
        self.running().set(None);
        *self.note().borrow_mut() = msg.map(Note::from);
        self.unpaint();
        self.show_rule(None);
        self.render_status();
        self.set_controls();
    }

    fn rename(&self, path: &Path) {
        if self.session().busy() || self.db().get() {
            return;
        }
        let t = self.session().term();
        let Some(n) = t.at(path).cloned() else { return };
        let (Some(x), Some(b)) = (n.binder().cloned(), n.kid('b').cloned()) else {
            return;
        };
        let avoid = b.names().into_iter().chain([x.clone()]).collect::<HashSet<Name>>();
        let names = Self::NAMES.map(Name::from);
        let start = names.iter().position(|c| *c == x.stem()).map_or(0, |i| i + 1);
        let y = (0..names.len())
            .map(|j| &names[(start + j) % names.len()])
            .find(|c| !avoid.contains(*c) && !Consts::has(c))
            .cloned()
            .unwrap_or_else(|| x.fresh(&avoid));
        let body = b.subst(&x, &Term::var(y.clone()), &HashSet::from([y.clone()]), Some(Mark::Renamed));
        let m = match n.shape() {
            Shape::Let(_, v, _) => Term::local(y.clone(), v.clone(), body),
            _ => Term::lam(y.clone(), body),
        }
        .marked(Mark::Binder);
        *self.note().borrow_mut() = None;
        self.session().push(Entry::new(
            t.replace(path, m),
            Some(Kind::Alpha),
            Some(Note::from(format!("{x} → {y}").as_str())),
        ));
    }

    fn back(&self) {
        let idx = self.session().idx();
        if let Some(prev) = idx.back().filter(|_| !self.session().busy()) {
            *self.note().borrow_mut() = None;
            self.session().seek(prev);
        }
    }

    fn act(self: &Rc<Self>, act: Act) {
        match act {
            Act::Step => leptos::task::spawn_local(self.clone().run(Mode::Step)),
            Act::Whnf => leptos::task::spawn_local(self.clone().run(Mode::Whnf)),
            Act::Nf => leptos::task::spawn_local(self.clone().run(Mode::Nf)),
            Act::Back => self.back(),
            Act::Reset if !self.session().busy() => {
                *self.note().borrow_mut() = None;
                self.session().reset();
            }
            Act::Load if !self.session().busy() => self.load(),
            _ => {}
        }
    }

    fn load(&self) {
        let src = self
            .source()
            .as_ref()
            .map(web_sys::HtmlTextAreaElement::value)
            .unwrap_or_default();
        if let Err(e) = self.session().load(&src, None, self.session().eta()) {
            if let Some(err) = self.q(&Class::Err.into()) {
                err.set_text_content(Some(&e.to_string()));
                Doc::hide(&err, false);
            }
            if let Some(details) = self
                .q(&Class::Src.into())
                .and_then(|e| e.dyn_into::<web_sys::HtmlDetailsElement>().ok())
            {
                details.set_open(true);
            }
        }
    }

    fn over(&self, e: &web_sys::Event) {
        if self.session().busy() {
            return;
        }
        let Some(target) = Doc::target(e) else { return };
        self.set_hot(Doc::closest(&target, &Class::Rr.into()));
        let b = Doc::closest(&target, &Selector::Has(Data::Bid).or(Class::Binder.into()));
        self.set_lit(
            b.and_then(|b| Doc::read::<Anchor>(&b, Data::Bid).or_else(|| Doc::read::<Anchor>(&b, Data::Anchor))),
        );
    }

    fn leave(&self) {
        if !self.session().busy() {
            self.set_hot(None);
            self.set_lit(None);
        }
    }

    fn focus(&self, e: &web_sys::Event) {
        let rx = Doc::target(e).and_then(|t| Doc::closest(&t, &Class::Rr.into()));
        if rx.is_some() && !self.session().busy() {
            self.set_hot(rx);
        }
    }

    fn click(self: &Rc<Self>, e: &web_sys::Event) {
        if self.session().busy() {
            return;
        }
        let Some(target) = Doc::target(e) else { return };
        if let Some(bn) = Doc::closest(&target, &Class::Binder.into()).filter(|b| b.tag_name() == "BUTTON") {
            if let Some(p) = Doc::read::<Path>(&bn, Data::Path) {
                self.rename(&p);
            }
            return;
        }
        if let Some(rx) = Doc::closest(&target, &Class::Rr.into())
            && let (Some(path), Some(kind)) = (Doc::read::<Path>(&rx, Data::Path), Doc::read::<Kind>(&rx, Data::Kind))
        {
            leptos::task::spawn_local(self.clone().fire_once(path, kind));
        }
    }

    fn pick(&self, e: &web_sys::Event) {
        let row =
            Doc::target(e).and_then(|t| Doc::closest(&t, &Selector::from(Class::Trace).within(Selector::Has(Data::I))));
        if let Some(i) = row.and_then(|r| Doc::read::<Step>(&r, Data::I))
            && !self.session().busy()
        {
            *self.note().borrow_mut() = None;
            self.session().seek(i);
        }
    }

    fn toggle(&self, e: &web_sys::Event, opt: Opt) {
        let on = Doc::target(e)
            .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
            .as_ref()
            .is_some_and(web_sys::HtmlInputElement::checked);
        match opt {
            Opt::Arcs => self.arcs().set(on),
            Opt::Db => self.db().set(on),
            Opt::Eta => {}
        }
        self.render();
    }

    fn key(self: &Rc<Self>, e: &web_sys::KeyboardEvent) {
        let typing = Doc::target(e).is_some_and(|t| {
            Doc::closest(
                &t,
                &Selector::from(Tag::Textarea)
                    .or(Tag::Input.into())
                    .or(Tag::Select.into()),
            )
            .is_some()
        });
        if e.meta_key() || e.ctrl_key() || e.alt_key() || typing {
            return;
        }
        let (busy, t, env, idx) =
            (self.session().busy(), self.session().term(), self.session().env(), self.session().idx());
        if e.key() == "ArrowRight" && !busy && Strategy::Normal.next(&t, &env).is_some() {
            e.prevent_default();
            leptos::task::spawn_local(self.clone().run(Mode::Step));
        }
        if e.key() == "ArrowLeft" && !busy && idx > Step::START {
            e.prevent_default();
            self.back();
        }
    }

    /// Scrolls the workbench to the top of the window.
    fn reveal(&self) {
        let opts = web_sys::ScrollIntoViewOptions::new();
        opts.set_behavior(if Motion::enabled() {
            web_sys::ScrollBehavior::Smooth
        } else {
            web_sys::ScrollBehavior::Auto
        });
        opts.set_block(web_sys::ScrollLogicalPosition::Start);
        if let Some(root) = self.root() {
            root.scroll_into_view_with_scroll_into_view_options(&opts);
        }
    }

    /// Redraws the binding links on the next frame, after a resize or once the fonts load.
    fn redraw(self: &Rc<Self>) {
        let Some(window) = web_sys::window() else { return };
        let _ = window.cancel_animation_frame(self.raf().get());
        let me = self.clone();
        let frame = Closure::once_into_js(move || {
            me.links();
            if me.session().capture().is_some()
                && let Some(panel) = me.q(&Class::Capture.into())
                && let (Some(stage), Some(line)) =
                    (Doc::q(&panel, &Class::Stage.into()), Doc::q(&panel, &Class::Line.into()))
            {
                Links::draw(&stage, &line, true);
            }
        });
        self.raf()
            .set(window.request_animation_frame(frame.unchecked_ref()).unwrap_or(0));
    }

    /// Listeners that outlive the view: the session, the keyboard, and the fonts.
    fn listen(self: &Rc<Self>) {
        let me = self.clone();
        self.session().on(move |e| match e {
            Event::Reveal => me.reveal(),
            Event::Load => {
                *me.note().borrow_mut() = None;
                me.render_source();
                me.render();
            }
            Event::Step => me.render(),
        });
        let me = self.clone();
        let keys = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(move |e: web_sys::KeyboardEvent| me.key(&e));
        if let Some(doc) = web_sys::window().as_ref().and_then(web_sys::Window::document) {
            let _ = doc.add_event_listener_with_callback("keydown", keys.as_ref().unchecked_ref());
            let me = self.clone();
            let ready = Closure::<dyn FnMut(JsValue)>::new(move |_| me.redraw());
            if let Ok(fonts) = doc.fonts().ready() {
                let _ = fonts.then(&ready);
            }
            ready.forget();
        }
        keys.forget();
    }

    /// Watches the stage's box once the view is in the page.
    fn observe(self: &Rc<Self>) {
        let me = self.clone();
        let resized = Closure::<dyn Fn(JsValue)>::new(move |_| me.redraw());
        if let (Ok(observer), Some(parent)) =
            (web_sys::ResizeObserver::new(resized.as_ref().unchecked_ref()), Doc::above(self.stage()))
        {
            observer.observe(&parent);
        }
        resized.forget();
    }
}

#[component]
pub fn Workbench() -> impl IntoView {
    let session = expect_context::<StoredValue<Session, LocalStorage>>().get_value();
    let anchor = NodeRef::<html::Div>::new();
    let bench = Rc::new(Bench {
        session,
        anchor,
        arcs: Cell::new(true),
        db: Cell::new(false),
        hot: RefCell::new(None),
        running: Cell::new(None),
        stop: Cell::new(false),
        note: RefCell::new(None),
        raf: Cell::new(0),
    });
    bench.listen();
    let [b0, b1, b2, b3, b4, b5, b6, b7, b8] = std::array::from_fn(|_| bench.clone());
    anchor.on_load(move |_| b0.observe());
    let act = move |a: Act| {
        let b = b7.clone();
        move |_: web_sys::MouseEvent| b.act(a)
    };
    let opt = move |o: Opt| {
        let b = b8.clone();
        move |e: web_sys::Event| b.toggle(&e, o)
    };
    let cls = |cs: &[Class]| Attr::Class(cs.to_vec()).to_string();
    view! {
        <div class=Class::Notes.to_string() node_ref=anchor></div>
        <div class=Class::Rule.to_string() aria-live="polite"></div>
        <div class=Class::Scroll.to_string()>
            <div class=Class::Stage.to_string()>
                <svg class=Class::Arcs.to_string() aria-hidden="true"></svg>
                <div
                    class=cls(&[Class::Line, Class::Tm])
                    on:mouseover=move |e| b1.over(&e)
                    on:mouseleave=move |_| b2.leave()
                    on:focusin=move |e| b3.focus(&e)
                    on:click=move |e| b4.click(&e)
                ></div>
            </div>
        </div>
        <div class=Class::Status.to_string()></div>
        <div class=Class::Controls.to_string()>
            <button type="button" class=cls(&[Class::Btn, Class::Primary]) data-act=Act::Step.to_string() on:click=act(Act::Step)>
                "Step"
            </button>
            <button type="button" class=Class::Btn.to_string() data-act=Act::Whnf.to_string() on:click=act(Act::Whnf)>
                "To WHNF"
            </button>
            <button type="button" class=Class::Btn.to_string() data-act=Act::Nf.to_string() on:click=act(Act::Nf)>
                "Normalize"
            </button>
            <button type="button" class=Class::Btn.to_string() data-act=Act::Back.to_string() on:click=act(Act::Back)>
                "Back"
            </button>
            <button type="button" class=Class::Btn.to_string() data-act=Act::Reset.to_string() on:click=act(Act::Reset)>
                "Reset"
            </button>
            <span class=Class::Grow.to_string()></span>
            <label class=Class::Toggle.to_string()>
                <input type="checkbox" data-opt=Opt::Arcs.to_string() checked on:change=opt(Opt::Arcs)/>
                " Binding links"
            </label>
            <label class=Class::Toggle.to_string()>
                <input type="checkbox" data-opt=Opt::Db.to_string() on:change=opt(Opt::Db)/>
                " de Bruijn"
            </label>
        </div>
        <div class=Class::Capture.to_string() hidden>
            <p>
                <b>"Why α fired first."</b>
                " Without renaming, substitution gives the term below. The dashed link is the argument’s free variable, caught by an inner binder, which changes what the term means."
            </p>
            <div class=Class::Scroll.to_string()>
                <div class=cls(&[Class::Stage, Class::Small])>
                    <svg class=Class::Arcs.to_string() aria-hidden="true"></svg>
                    <div class=cls(&[Class::Line, Class::Tm])></div>
                </div>
            </div>
        </div>
        <div class=Class::Defs.to_string() hidden>
            <div class=Class::Label.to_string()>"Definitions " <span>"· δ unfolds these"</span></div>
            <div class=Class::DefsList.to_string()></div>
        </div>
        <details class=Class::Trace.to_string() open>
            <summary>"Trace"</summary>
            <ol on:click=move |e| b5.pick(&e)></ol>
        </details>
        <details class=Class::Src.to_string()>
            <summary>"Edit source"</summary>
            <textarea spellcheck="false" rows="5" aria-label="Program source"></textarea>
            <div class=Class::SrcRow.to_string()>
                <button type="button" class=cls(&[Class::Btn, Class::Primary]) data-act=Act::Load.to_string() on:click=move |_| b6.act(Act::Load)>
                    "Load"
                </button>
                <p class=Class::Syntax.to_string()>
                    <code>"λx. t"</code> " or " <code>"fun x => t"</code> " · " <code>"let x := v; t"</code> " · "
                    <code>"(a, b)"</code> " · " <code>"p.1"</code> " · " <code>"Nat.rec z s n"</code> " · numerals · "
                    <code>"def name := t"</code> " on its own line · " <code>"--"</code> " comments"
                </p>
            </div>
            <p class=Class::Err.to_string() hidden></p>
        </details>
    }
}
