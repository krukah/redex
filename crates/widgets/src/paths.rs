use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

use explore::*;
use layout::*;
use leptos::html;
use leptos::prelude::*;
use markup::Selector;
use markup::*;
use motion::*;
use rules::*;
use strategy::*;
use syntax::*;
use term::Env;
use term::*;
use units::*;
use wasm_bindgen::JsCast;
use web_sys::Element;

use crate::*;

/// The reduction graph, explored and laid out, and the markup the graph widget draws from it. Rows
/// are steps from the start; a mark's shape is its form; edges carry rule ink; a route is a band.
pub struct Chart {
    graph: Graph,
    plot: Plot,
    env: Env,
    plate: Plate,
}

impl Chart {
    fn env(&self) -> &Env {
        &self.env
    }

    /// Stop exploring after this many terms.
    pub const CAP: Cap = Cap::new(70);
    /// A route follows a strategy this far.
    const ROUTE: Cap = Cap::new(40);
    const KINDS: [Kind; 7] = [
        Kind::Alpha,
        Kind::Beta,
        Kind::Eta,
        Kind::Delta,
        Kind::Zeta,
        Kind::Iota,
        Kind::Proj,
    ];
    pub const ROUTES: [Option<Strategy>; 4] = [
        None,
        Some(Strategy::Normal),
        Some(Strategy::Applicative),
        Some(Strategy::Whnf),
    ];

    pub fn new(start: &Term, env: &Env, eta: bool, plate: Plate) -> Self {
        let graph = Graph::explore(start, env, eta, Self::CAP);
        let plot = Plot::new(&graph, env);
        Self {
            graph,
            plot,
            env: env.clone(),
            plate,
        }
    }

    fn plate(&self) -> Plate {
        self.plate
    }

    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    pub fn plot(&self) -> &Plot {
        &self.plot
    }

    pub fn svg(&self, sel: Option<Id>) -> Node {
        let js = Num::Js;
        let px = Px::new;
        let grid = self.plot().rows().map(|d| {
            let y = Plot::row(d);
            Node::list([
                Node::el(Tag::Line)
                    .class([Class::Gridline])
                    .attr(Attr::X1(js(Plot::ML - px(18.0))))
                    .attr(Attr::X2(js(self.plot().width() - px(12.0))))
                    .attr(Attr::Y1(js(y)))
                    .attr(Attr::Y2(js(y))),
                Node::el(Tag::Text)
                    .class([Class::Ax])
                    .attr(Attr::X(js(Plot::ML - px(26.0))))
                    .attr(Attr::Y(js(y)))
                    .kid(Node::text(d)),
            ])
        });
        let head = Node::el(Tag::Text)
            .class([Class::Ax, Class::Head])
            .attr(Attr::X(js(Plot::ML - px(26.0))))
            .attr(Attr::Y(js(Plot::MT - px(18.0))))
            .kid(Node::text("steps"));
        let edges = self.graph().edges().iter().zip(self.plot().routes()).map(|(e, d)| {
            let title = if e.kind().ink() == Kind::Proj {
                Node::text(Kind::Proj.glyph())
            } else {
                Node::text(e.kind())
            };
            Node::el(Tag::Path)
                .class([Class::Ge, Class::Ink(e.kind())])
                .attr(Attr::Source(e.from()))
                .attr(Attr::Target(e.to()))
                .attr(Attr::Style(Style::Row(self.graph().vertex(e.from()).depth())))
                .attr(Attr::Unit)
                .attr(Attr::D(d.clone()))
                .attr(Attr::MarkerEnd(self.plate(), e.kind().ink()))
                .kid(Node::el(Tag::Title).kid(title))
        });
        let nodes = self.graph().vertices().map(|(i, v)| self.vertex(i, v, sel));
        let markers = Self::KINDS.iter().map(|k| {
            Node::el(Tag::Marker)
                .attr(Attr::Marker(self.plate(), *k))
                .attr(Attr::ViewBox(js(Px::ZERO), js(Px::ZERO), js(px(10.0)), js(px(10.0))))
                .attr(Attr::RefX(js(px(8.0))))
                .attr(Attr::RefY(js(px(5.0))))
                .attr(Attr::MarkerWidth(js(px(5.5))))
                .attr(Attr::MarkerHeight(js(px(5.5))))
                .attr(Attr::Orient)
                .kid(
                    Node::el(Tag::Path)
                        .attr(Attr::D(Self::arrowhead()))
                        .attr(Attr::Style(Style::Fill(*k))),
                )
        });
        Node::list([
            Node::el(Tag::Defs).kids(markers),
            Node::el(Tag::G).class([Class::Grid]).kids(grid).kid(head),
            Node::el(Tag::G).class([Class::Routes]),
            Node::el(Tag::G).class([Class::Edges]).kids(edges),
            Node::el(Tag::G).class([Class::Nodes]).kids(nodes),
        ])
    }

    /// `M0 0L10 5L0 10z`.
    fn arrowhead() -> Trace {
        let n = |v: f64| Num::Js(Px::new(v));
        Trace::tight(vec![
            Seg::Move(n(0.0), n(0.0)),
            Seg::Line(n(10.0), n(5.0)),
            Seg::Line(n(0.0), n(10.0)),
            Seg::End,
        ])
    }

    fn vertex(&self, i: Id, v: &Vertex, sel: Option<Id>) -> Node {
        let js = Num::Js;
        let (s, r, px) = (self.plot().spot(i), Plot::R, Px::new);
        let (x, y) = (s.x(), s.y());
        let mark = match s.form() {
            Form::Nf => Node::el(Tag::Rect)
                .class([Class::Mark, Class::Nf])
                .attr(Attr::X(js(x - r)))
                .attr(Attr::Y(js(y - r)))
                .attr(Attr::Width(js(r * 2.0)))
                .attr(Attr::Height(js(r * 2.0))),
            Form::Whnf => Node::el(Tag::Path)
                .class([Class::Mark, Class::Whnf])
                .attr(Attr::D(Trace::tight(vec![
                    Seg::Move(js(x), js(y - r - px(1.5))),
                    Seg::Line(js(x + r + px(1.5)), js(y)),
                    Seg::Line(js(x), js(y + r + px(1.5))),
                    Seg::Line(js(x - r - px(1.5)), js(y)),
                    Seg::Close,
                ]))),
            Form::Red => Node::el(Tag::Circle)
                .class([Class::Mark, Class::Red])
                .attr(Attr::Cx(js(x)))
                .attr(Attr::Cy(js(y)))
                .attr(Attr::R(js(r))),
        };
        let root = i == Id::ROOT;
        let classes = [
            Some(Class::Gn),
            root.then_some(Class::Start),
            v.open().then_some(Class::Open),
            (sel == Some(i)).then_some(Class::Sel),
        ]
        .into_iter()
        .flatten();
        let size = v.term().size();
        let ring = root.then(|| {
            Node::el(Tag::Circle)
                .class([Class::Ring])
                .attr(Attr::Cx(js(x)))
                .attr(Attr::Cy(js(y)))
                .attr(Attr::R(js(r + px(4.0))))
        });
        Node::el(Tag::G)
            .class(classes)
            .attr(Attr::Vertex(i))
            .attr(Attr::Style(Style::Row(v.depth())))
            .attr(Attr::Focusable)
            .attr(Attr::Clickable)
            .attr(Attr::Label(format!("{}, size {size}: {}", s.form().label(), v.term().text())))
            .kid(
                Node::el(Tag::Circle)
                    .class([Class::Hit])
                    .attr(Attr::Cx(js(x)))
                    .attr(Attr::Cy(js(y)))
                    .attr(Attr::R(js(px(15.0)))),
            )
            .kid(
                Node::el(Tag::Circle)
                    .class([Class::Halo])
                    .attr(Attr::Cx(js(x)))
                    .attr(Attr::Cy(js(y)))
                    .attr(Attr::R(js(r + px(6.0)))),
            )
            .kids(ring)
            .kid(mark)
            .kid(
                Node::el(Tag::Text)
                    .class([Class::Sz])
                    .attr(Attr::X(js(x + r + px(5.0))))
                    .attr(Attr::Y(js(y + px(9.0))))
                    .kid(Node::text(size)),
            )
    }

    pub fn stats(&self) -> String {
        let (n, e, nf) = (self.graph().len(), self.graph().edges().len(), self.graph().normal());
        format!(
            "{n} term{} · {e} step{} · {nf} normal form{}{}",
            if n > 1 { "s" } else { "" },
            if e == 1 { "" } else { "s" },
            if nf == 1 { "" } else { "s" },
            if self.graph().truncated() {
                format!(" · stopped at {} terms", Self::CAP)
            } else {
                String::new()
            }
        )
    }

    pub fn verdict(&self) -> String {
        let nf = self.graph().normal();
        let finished = match (nf, self.graph().truncated()) {
            (1, true) => "Every finished path so far ends at the same normal form.",
            (1, false) => "Every path that finishes ends at the same normal form.",
            _ => "",
        };
        let looping = match (self.graph().cyclic(), nf > 0) {
            (true, true) => " Some paths loop instead of finishing.",
            (true, false) => "No normal form: every path keeps going.",
            (false, _) => "",
        };
        if self.graph().edges().is_empty() {
            "Already in normal form. Nothing to fire.".to_string()
        } else {
            format!("{finished}{looping}").trim().to_string()
        }
    }

    pub fn pills(route: Option<Strategy>) -> Node {
        Node::list(Self::ROUTES.iter().map(|r| {
            let on = *r == route;
            Node::el(Tag::Button)
                .attr(Attr::Button)
                .class([Class::Pill].into_iter().chain(on.then_some(Class::On)))
                .attr(Attr::Route(*r))
                .attr(Attr::Pressed(on))
                .kid(Node::text(r.map_or("None", Strategy::label)))
        }))
    }

    /// The band along the edges a strategy takes, and the note that says how far it got.
    pub fn band(&self, route: Option<Strategy>) -> (Node, String) {
        let Some(strategy) = route else {
            return (Node::none(), String::new());
        };
        let run = Run::new(self.graph().root().term(), self.env(), strategy, Self::ROUTE);
        let find = |t: &Term| self.graph().find(&t.key());
        let rows = run.rows();
        let walked = (1..rows.len())
            .map_while(|i| Some((i, find(rows[i - 1].term())?, find(rows[i].term())?)))
            .collect::<Vec<(usize, Id, Id)>>();
        let left = walked.len() + 1 < rows.len();
        let steps = Step::new(walked.last().map_or(0, |w| w.0));
        let edges = self.graph().edges();
        let paths = walked.iter().filter_map(|&(i, a, b)| {
            edges
                .iter()
                .position(|e| e.joins(a, b) && Some(e.kind()) == rows[i].kind())
                .or_else(|| edges.iter().position(|e| e.joins(a, b)))
                .map(|e| {
                    Node::el(Tag::Path)
                        .class([Class::Band])
                        .attr(Attr::D(self.plot().routes()[e].clone()))
                })
        });
        let end = find(rows[steps.get().min(rows.len() - 1)].term());
        let ring = end.filter(|_| run.status() == Status::Done && !left).map(|e| {
            let s = self.plot().spot(e);
            Node::el(Tag::Circle)
                .class([Class::BandEnd])
                .attr(Attr::Cx(Num::Js(s.x())))
                .attr(Attr::Cy(Num::Js(s.y())))
                .attr(Attr::R(Num::Js(Plot::R + Px::new(9.0))))
        });
        let label = strategy.label();
        let note = match (left, run.status()) {
            (true, _) => format!("{label} leaves the explored graph after {steps} steps."),
            (false, Status::Done) => format!(
                "{label}: {steps} step{} to a {}.",
                if steps == Step::ONE { "" } else { "s" },
                end.map_or("stop", |e| self.plot().spot(e).form().label())
            ),
            (false, _) => format!("{label} never stops: it loops."),
        };
        (Node::list(paths).kids(ring), note)
    }

    pub fn tip(&self, i: Id) -> Node {
        let v = self.graph().vertex(i);
        let d = v.depth();
        let meta = format!(
            " · size {} · {d} step{} from the start",
            v.term().size(),
            if d == Depth::new(1) { "" } else { "s" }
        );
        Node::list([
            Node::el(Tag::Div)
                .class([Class::TipMeta])
                .kid(Node::el(Tag::B).kid(Node::text(self.plot().spot(i).form().label())))
                .kid(Node::text(meta)),
            Node::el(Tag::Div).class([Class::Tm]).kid(Node::text(v.term().text())),
        ])
    }

    pub fn selected(&self, sel: Option<Id>) -> Node {
        let Some(i) = sel else {
            return Node::el(Tag::P)
                .kid(Node::text("Hover a mark to read its term. Click it to open it in the workbench."));
        };
        let v = self.graph().vertex(i);
        let place = match v.depth() {
            Depth::START => "Start term".to_string(),
            d => format!("{d} step{} from the start on the shortest path", if d > Depth::new(1) { "s" } else { "" }),
        };
        Node::list([
            Node::el(Tag::Div).class([Class::Scroll]).kid(
                Node::el(Tag::Div)
                    .class([Class::Stage, Class::Small, Class::Flat, Class::Tm])
                    .kid(Blocks::new(self.env(), false, false).node(v.term())),
            ),
            Node::el(Tag::Div)
                .class([Class::Row])
                .kid(Node::el(Tag::Span).kid(Node::text(format!(
                    "{place} · {} · size {}",
                    self.plot().spot(i).form().label(),
                    v.term().size()
                ))))
                .kid(Node::el(Tag::Span).class([Class::Grow]))
                .kid(
                    Node::el(Tag::Button)
                        .class([Class::Btn, Class::Primary])
                        .attr(Attr::Button)
                        .attr(Attr::Act(Act::Open))
                        .kid(Node::text("Open in workbench")),
                ),
        ])
    }

    /// The start-to-here chain of a vertex, as history entries.
    pub fn chain(&self, i: Id) -> Vec<Entry> {
        let mut chain = std::iter::successors(Some(i), |&k| self.graph().vertex(k).parent()).collect::<Vec<Id>>();
        chain.reverse();
        chain
            .iter()
            .enumerate()
            .map(|(j, &k)| {
                Entry::new(
                    self.graph().vertex(k).term().clone(),
                    if j > 0 { self.graph().vertex(k).pkind() } else { None },
                    None,
                )
            })
            .collect()
    }
}

/// The graph widget's own state.
struct Board {
    session: Session,
    anchor: NodeRef<html::Div>,
    chart: RefCell<Option<Chart>>,
    sel: Cell<Option<Id>>,
    route: Cell<Option<Strategy>>,
    plate: Plate,
}

impl Board {
    /// The entrance: marks fade in row by row, then the edges leaving them draw.
    const ROW: Ms = Ms::new(70.0);

    fn session(&self) -> &Session {
        &self.session
    }

    fn chart(&self) -> &RefCell<Option<Chart>> {
        &self.chart
    }

    fn sel(&self) -> &Cell<Option<Id>> {
        &self.sel
    }

    fn route(&self) -> &Cell<Option<Strategy>> {
        &self.route
    }

    fn q(&self, sel: &Selector) -> Option<Element> {
        Doc::above(self.anchor().get_untracked()).and_then(|r| Doc::q(&r, sel))
    }

    fn anchor(&self) -> NodeRef<html::Div> {
        self.anchor
    }

    fn svg(&self) -> Option<Element> {
        self.q(&Class::Gsvg.into())
    }

    fn eta(&self) -> Option<web_sys::HtmlInputElement> {
        self.q(&Selector::is(Data::Opt, &Opt::Eta))
            .and_then(|e| e.dyn_into().ok())
    }

    fn build(&self, start: &Term) {
        let eta = self.eta().as_ref().is_some_and(web_sys::HtmlInputElement::checked);
        *self.chart().borrow_mut() = Some(Chart::new(start, &self.session().env(), eta, self.plate()));
        self.sel().set(None);
        self.draw();
    }

    fn plate(&self) -> Plate {
        self.plate
    }

    fn draw(&self) {
        let Some(svg) = self.svg() else { return };
        if let Some(chart) = self.chart().borrow().as_ref() {
            let (w, h) = (Num::Js(chart.plot().width()), Num::Js(chart.plot().height()));
            Doc::put(&svg, &Attr::Width(w));
            Doc::put(&svg, &Attr::Height(h));
            Doc::put(&svg, &Attr::ViewBox(Num::Js(Px::ZERO), Num::Js(Px::ZERO), w, h));
            Doc::fill(&svg, &chart.svg(self.sel().get()));
            let row = |e: &Element| {
                Self::ROW
                    * e.dyn_ref::<web_sys::SvgElement>()
                        .and_then(|s| s.style().get_property_value("--d").ok())
                        .and_then(|d| d.trim().parse::<f64>().ok())
                        .unwrap_or(0.0)
            };
            let grow = Doc::all(&svg, &Class::Gn.into())
                .iter()
                .map(|g| Motion::after(row(g), Motion::enter(g, Tween::default().ms(Ms::new(400.0)))))
                .chain(Doc::all(&svg, &Class::Ge.into()).iter().map(|p| {
                    Motion::after(row(p) + Ms::new(90.0), Motion::draw(p, Tween::default().ms(Ms::new(500.0))))
                }))
                .collect();
            leptos::task::spawn_local(Motion::all(grow).play(Clock::default()));
            if let Some(stats) = self.q(&Class::Gstats.into()) {
                stats.set_text_content(Some(&chart.stats()));
            }
            if let Some(verdict) = self.q(&Class::Verdict.into()) {
                verdict.set_text_content(Some(&chart.verdict()));
            }
        }
        self.render_routes();
        self.render_sel();
        self.mark();
    }

    fn render_routes(&self) {
        if let Some(pills) = self.q(&Selector::Has(Data::Routes)) {
            Doc::fill(&pills, &Chart::pills(self.route().get()));
        }
        let (band, note) = self
            .chart()
            .borrow()
            .as_ref()
            .map_or_else(|| (Node::none(), String::new()), |c| c.band(self.route().get()));
        if let Some(g) = self.q(&Selector::from(Class::Gsvg).within(Class::Routes.into())) {
            Doc::fill(&g, &band);
        }
        if let Some(n) = self.q(&Class::RouteNote.into()) {
            n.set_text_content(Some(&note));
        }
    }

    fn render_sel(&self) {
        if let (Some(b), Some(c)) = (self.q(&Class::Gsel.into()), self.chart().borrow().as_ref()) {
            Doc::fill(&b, &c.selected(self.sel().get()));
        }
    }

    fn select(&self, i: Id) {
        self.sel().set(Some(i));
        if let Some(svg) = self.svg() {
            Doc::all(&svg, &Class::Gn.into())
                .iter()
                .for_each(|g| Doc::toggle(g, Class::Sel, Doc::read::<Id>(g, Data::Id) == Some(i)));
        }
        self.render_sel();
    }

    /// The workbench's position: its term gets a halo, the steps it walked are marked.
    fn mark(&self) {
        let chart = self.chart().borrow();
        let (Some(chart), Some(svg)) = (chart.as_ref(), self.svg()) else {
            return;
        };
        let id = |t: &Term| chart.graph().find(&t.key());
        let hist = self.session().history();
        let here = id(hist[self.session().idx().get()].term());
        let walked = (1..=self.session().idx().get())
            .filter_map(|i| Some((id(hist[i - 1].term())?, id(hist[i].term())?)))
            .filter(|(a, b)| a != b)
            .collect::<Vec<(Id, Id)>>();
        Doc::all(&svg, &Class::Gn.into())
            .iter()
            .for_each(|g| Doc::toggle(g, Class::Here, here.is_some() && Doc::read::<Id>(g, Data::Id) == here));
        Doc::all(&svg, &Class::Ge.into()).iter().for_each(|g| {
            let edge = Doc::read::<Id>(g, Data::From).zip(Doc::read::<Id>(g, Data::To));
            Doc::toggle(g, Class::Walked, edge.is_some_and(|e| walked.contains(&e)));
        });
    }

    fn show_tip(&self, g: Option<Element>) {
        let Some(tip) = self
            .q(&Class::Tip.into())
            .and_then(|t| t.dyn_into::<web_sys::HtmlElement>().ok())
        else {
            return;
        };
        let Some(g) = g else {
            tip.set_hidden(true);
            return;
        };
        let Some(i) = Doc::read::<Id>(&g, Data::Id) else { return };
        if let Some(c) = self.chart().borrow().as_ref() {
            Doc::fill(&tip, &c.tip(i));
        }
        tip.set_hidden(false);
        let (Some(wrap), Some(hit)) = (self.q(&Class::Gwrap.into()), Doc::q(&g, &Class::Hit.into())) else {
            return;
        };
        let (wrap, r) = (wrap.get_bounding_client_rect(), hit.get_bounding_client_rect());
        let (tw, th) = (Px::new(f64::from(tip.offset_width())), Px::new(f64::from(tip.offset_height())));
        let left = (Px::new(r.left() - wrap.left() + r.width() / 2.0) - tw / 2.0)
            .max(Px::new(8.0))
            .min(Px::new(wrap.width()) - tw - Px::new(8.0));
        let below = Px::new(r.top() - wrap.top()) < th + Px::new(12.0);
        let top = if below {
            Px::new(r.bottom() - wrap.top() + 6.0)
        } else {
            Px::new(r.top() - wrap.top()) - th - Px::new(6.0)
        };
        let _ = tip.style().set_property("left", &format!("{left}px"));
        let _ = tip.style().set_property("top", &format!("{top}px"));
    }

    fn open(&self, i: Id) {
        if self.session().busy() {
            return;
        }
        let Some(chain) = self.chart().borrow().as_ref().map(|c| c.chain(i)) else {
            return;
        };
        self.session().replace_history(chain);
        self.session().reveal();
    }

    fn click(&self, e: &web_sys::Event) {
        let Some(target) = Doc::target(e) else { return };
        match Doc::closest(&target, &Selector::Has(Data::Act)).and_then(|a| Doc::read::<Act>(&a, Data::Act)) {
            Some(Act::Rebuild) => self.build(&self.session().term()),
            Some(Act::Open) => {
                if let Some(i) = self.sel().get() {
                    self.open(i);
                }
            }
            _ => {}
        }
        if let Some(route) =
            Doc::closest(&target, &Selector::Has(Data::Route)).and_then(|r| r.get_attribute(Data::Route.name()))
        {
            self.route().set(
                Chart::ROUTES
                    .into_iter()
                    .find(|r| r.map_or("none", Strategy::id) == route)
                    .flatten(),
            );
            self.render_routes();
        }
    }

    fn mark_of(e: &web_sys::Event) -> Option<Element> {
        Doc::target(e).and_then(|t| Doc::closest(&t, &Class::Gn.into()))
    }

    fn rebuild(&self) {
        let start = self.chart().borrow().as_ref().map(|c| c.graph().root().term().clone());
        if let Some(start) = start {
            self.build(&start);
        }
    }

    fn key(&self, e: &web_sys::KeyboardEvent) {
        if let Some(i) = Self::mark_of(e).and_then(|g| Doc::read::<Id>(&g, Data::Id))
            && (e.key() == "Enter" || e.key() == " ")
        {
            e.prevent_default();
            self.select(i);
        }
    }

    fn pick(&self, e: &web_sys::Event) {
        if let Some(i) = Self::mark_of(e).and_then(|g| Doc::read::<Id>(&g, Data::Id)) {
            self.select(i);
        }
    }
}

#[component]
pub fn Paths() -> impl IntoView {
    let session = expect_context::<StoredValue<Session, LocalStorage>>().get_value();
    let board = Rc::new(Board {
        session,
        anchor: NodeRef::new(),
        chart: RefCell::new(None),
        sel: Cell::new(None),
        route: Cell::new(None),
        plate: Plate::next(),
    });
    let me = board.clone();
    board.session().on(move |e| match e {
        Event::Load => {
            if let Some(eta) = me.eta() {
                eta.set_checked(me.session().eta());
            }
            me.build(&me.session().start());
        }
        Event::Step => me.mark(),
        Event::Reveal => {}
    });
    let anchor = board.anchor();
    let [b1, b2, b3, b4, b5, b6, b7, b8, b9, b10] = std::array::from_fn(|_| board.clone());
    let legend = |class: &[Class]| Attr::Class(class.to_vec()).to_string();
    view! {
        <div class=Class::Gbar.to_string() node_ref=anchor on:click=move |e| b1.click(&e)>
            <span class=Class::Gstats.to_string()></span>
            <span class=Class::Grow.to_string()></span>
            <label class=Class::Toggle.to_string()>
                <input type="checkbox" data-opt=Opt::Eta.to_string() on:change=move |_| b2.rebuild()/>
                " Include η"
            </label>
            <button type="button" class=Class::Btn.to_string() data-act=Act::Rebuild.to_string()>
                "Rebuild from current term"
            </button>
        </div>
        <div class=Class::Gbar.to_string() on:click=move |e| b3.click(&e)>
            <span class=Class::Label.to_string()>"Route"</span>
            <div class=Class::Pills.to_string() data-routes></div>
            <span class=Class::RouteNote.to_string()></span>
        </div>
        <p class=Class::Verdict.to_string()></p>
        <div class=Class::Gwrap.to_string()>
            <div class=Class::Gscroll.to_string()>
                <svg
                    class=Class::Gsvg.to_string()
                    role="img"
                    aria-label="Reduction graph"
                    on:mouseover=move |e| b4.show_tip(Board::mark_of(&e))
                    on:mouseleave=move |_| b5.show_tip(None)
                    on:focusin=move |e| b6.show_tip(Board::mark_of(&e))
                    on:focusout=move |_| b7.show_tip(None)
                    on:click=move |e| b8.pick(&e)
                    on:keydown=move |e: web_sys::KeyboardEvent| b9.key(&e)
                ></svg>
            </div>
            <div class=Class::Tip.to_string() hidden></div>
        </div>
        <div class=Class::Glegend.to_string()>
            <span>
                <svg width="14" height="14">
                    <circle cx="7" cy="7" r="5" class=legend(&[Class::Mark, Class::Red])></circle>
                </svg>
                "reducible"
            </span>
            <span>
                <svg width="14" height="14">
                    <path d="M7 1.5L12.5 7L7 12.5L1.5 7Z" class=legend(&[Class::Mark, Class::Whnf])></path>
                </svg>
                "weak head normal form"
            </span>
            <span>
                <svg width="14" height="14">
                    <rect x="2" y="2" width="10" height="10" class=legend(&[Class::Mark, Class::Nf])></rect>
                </svg>
                "normal form"
            </span>
            <span>
                <svg width="18" height="18">
                    <circle cx="9" cy="9" r="4.5" class=legend(&[Class::Mark, Class::Red])></circle>
                    <circle cx="9" cy="9" r="8" class=Class::Ring.to_string()></circle>
                </svg>
                "start"
            </span>
            <span>
                <svg width="18" height="18">
                    <circle cx="9" cy="9" r="4.5" class=legend(&[Class::Mark, Class::Red])></circle>
                    <circle cx="9" cy="9" r="8" class=Class::Halo.to_string()></circle>
                </svg>
                "workbench is here"
            </span>
            <span>
                <b class=Class::Sz.to_string()>"12"</b>
                " term size"
            </span>
        </div>
        <div class=Class::Gsel.to_string() on:click=move |e| b10.click(&e)></div>
    }
}
