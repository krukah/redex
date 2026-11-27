use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

use explore::*;
use layout::*;
use leptos::html;
use leptos::prelude::*;
use motion::*;
use rules::*;
use strategy::*;
use syntax::*;
use term::*;
use wasm_bindgen::JsCast;
use web_sys::Element;

use crate::*;

/// A strategy route the graph can draw, or none.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Route {
    id: &'static str,
    label: &'static str,
    strategy: Option<Strategy>,
}

impl Route {
    pub const ALL: [Route; 4] = [
        Self {
            id: "none",
            label: "None",
            strategy: None,
        },
        Self {
            id: "normal",
            label: "Normal order",
            strategy: Some(Strategy::Normal),
        },
        Self {
            id: "applicative",
            label: "Applicative order",
            strategy: Some(Strategy::Applicative),
        },
        Self {
            id: "whnf",
            label: "Head only",
            strategy: Some(Strategy::Whnf),
        },
    ];
}

/// The reduction graph, explored and laid out, and the markup the graph widget draws from it. Rows
/// are steps from the start; a mark's shape is its form; edges carry rule ink; a route is a band.
pub struct Chart {
    graph: Graph,
    plot: Plot,
    env: term::Env,
    uid: String,
    labels: Vec<String>,
}

impl Chart {
    /// Stop exploring after this many terms.
    pub const CAP: usize = 70;
    const KINDS: [Kind; 7] = [
        Kind::Alpha,
        Kind::Beta,
        Kind::Eta,
        Kind::Delta,
        Kind::Zeta,
        Kind::Iota,
        Kind::Proj,
    ];

    pub fn new(start: &Term, env: &term::Env, eta: bool, uid: &str) -> Self {
        let graph = Graph::explore(start, env, eta, Self::CAP);
        let plot = Plot::new(&graph, env);
        let labels = graph.vertices().iter().map(|v| v.term().text()).collect();
        Self {
            graph,
            plot,
            env: env.clone(),
            uid: uid.to_string(),
            labels,
        }
    }

    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    pub fn plot(&self) -> &Plot {
        &self.plot
    }

    fn n(x: f64) -> String {
        Number::show(x)
    }

    pub fn svg(&self, sel: Option<usize>) -> String {
        let (p, r) = (&self.plot, Plot::R);
        let n = Self::n;
        let grid = (0..p.rows())
            .map(|d| {
                let y = Plot::MT + d as f64 * Plot::ROWH;
                format!(
                    "<line class=\"rdx-gridline\" x1=\"{}\" x2=\"{}\" y1=\"{}\" y2=\"{}\"/><text class=\"rdx-ax\" x=\"{}\" y=\"{}\">{d}</text>",
                    n(Plot::ML - 18.0),
                    n(p.width() - 12.0),
                    n(y),
                    n(y),
                    n(Plot::ML - 26.0),
                    n(y)
                )
            })
            .collect::<Vec<String>>()
            .concat()
            + &format!("<text class=\"rdx-ax head\" x=\"{}\" y=\"{}\">steps</text>", n(Plot::ML - 26.0), n(Plot::MT - 18.0));
        let vs = self.graph.vertices();
        let edges = self
            .graph
            .edges()
            .iter()
            .zip(p.routes())
            .map(|(e, d)| {
                let k = e.kind().class();
                format!(
                    "<path class=\"rdx-ge k-{k}\" data-from=\"{}\" data-to=\"{}\" style=\"--d:{}\" pathLength=\"1\" d=\"{d}\" marker-end=\"url(#{}-{k})\"><title>{}</title></path>",
                    e.from(),
                    e.to(),
                    vs[e.from()].depth(),
                    self.uid,
                    if k == "proj" { "π" } else { e.kind().name() }
                )
            })
            .collect::<Vec<String>>()
            .concat();
        let nodes = vs
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let s = &p.spots()[i];
                let (x, y) = (s.x(), s.y());
                let mark = match s.form() {
                    Form::Nf => format!("<rect class=\"m nf\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"/>", n(x - r), n(y - r), n(2.0 * r), n(2.0 * r)),
                    Form::Whnf => format!(
                        "<path class=\"m whnf\" d=\"M{} {}L{} {}L{} {}L{} {}Z\"/>",
                        n(x),
                        n(y - r - 1.5),
                        n(x + r + 1.5),
                        n(y),
                        n(x),
                        n(y + r + 1.5),
                        n(x - r - 1.5),
                        n(y)
                    ),
                    Form::Red => format!("<circle class=\"m red\" cx=\"{}\" cy=\"{}\" r=\"{}\"/>", n(x), n(y), n(r)),
                };
                let cls = ["rdx-gn", if i == 0 { "start" } else { "" }, if v.open() { "open" } else { "" }, if sel == Some(i) { "sel" } else { "" }]
                    .into_iter()
                    .filter(|c| !c.is_empty())
                    .collect::<Vec<&str>>()
                    .join(" ");
                let size = v.term().size();
                format!(
                    "<g class=\"{cls}\" data-id=\"{i}\" style=\"--d:{}\" tabindex=\"0\" role=\"button\" aria-label=\"{}\"><circle class=\"hit\" cx=\"{}\" cy=\"{}\" r=\"15\"/><circle class=\"halo\" cx=\"{}\" cy=\"{}\" r=\"{}\"/>{}{mark}<text class=\"rdx-sz\" x=\"{}\" y=\"{}\">{size}</text></g>",
                    v.depth(),
                    Html::esc(&format!("{}, size {size}: {}", s.form().label(), self.labels[i])),
                    n(x),
                    n(y),
                    n(x),
                    n(y),
                    n(r + 6.0),
                    if i == 0 { format!("<circle class=\"ring\" cx=\"{}\" cy=\"{}\" r=\"{}\"/>", n(x), n(y), n(r + 4.0)) } else { String::new() },
                    n(x + r + 5.0),
                    n(y + 9.0)
                )
            })
            .collect::<Vec<String>>()
            .concat();
        let markers = Self::KINDS
            .iter()
            .map(|k| {
                format!(
                    "<marker id=\"{uid}-{k}\" viewBox=\"0 0 10 10\" refX=\"8\" refY=\"5\" markerWidth=\"5.5\" markerHeight=\"5.5\" orient=\"auto-start-reverse\"><path d=\"M0 0L10 5L0 10z\" style=\"fill:var(--rdx-{k})\"/></marker>",
                    uid = self.uid,
                    k = k.name(),
                )
            })
            .collect::<Vec<String>>()
            .concat();
        format!(
            "<defs>{markers}</defs><g class=\"grid\">{grid}</g><g class=\"routes\"></g><g class=\"edges\">{edges}</g><g class=\"nodes\">{nodes}</g>"
        )
    }

    pub fn stats(&self) -> String {
        let (n, e, nf) = (self.graph.vertices().len(), self.graph.edges().len(), self.normal());
        format!(
            "{n} term{} · {e} step{} · {nf} normal form{}{}",
            if n > 1 { "s" } else { "" },
            if e == 1 { "" } else { "s" },
            if nf == 1 { "" } else { "s" },
            if self.graph.truncated() {
                format!(" · stopped at {} terms", Self::CAP)
            } else {
                String::new()
            }
        )
    }

    fn normal(&self) -> usize {
        self.graph.vertices().iter().filter(|v| v.nf()).count()
    }

    pub fn verdict(&self) -> String {
        let nf = self.normal();
        let mut v = match (nf, self.graph.truncated()) {
            (1, true) => "Every finished path so far ends at the same normal form.".to_string(),
            (1, false) => "Every path that finishes ends at the same normal form.".to_string(),
            _ => String::new(),
        };
        if self.graph.cyclic() {
            v += if nf > 0 {
                " Some paths loop instead of finishing."
            } else {
                "No normal form: every path keeps going."
            };
        }
        if self.graph.edges().is_empty() {
            v = "Already in normal form. Nothing to fire.".to_string();
        }
        v.trim().to_string()
    }

    pub fn pills(route: &str) -> String {
        Route::ALL
            .iter()
            .map(|r| {
                let on = r.id == route;
                format!(
                    "<button type=\"button\" class=\"rdx-pill{}\" data-route=\"{}\" aria-pressed=\"{on}\">{}</button>",
                    if on { " on" } else { "" },
                    r.id,
                    r.label
                )
            })
            .collect::<Vec<String>>()
            .concat()
    }

    /// The band markup and the note for a route.
    pub fn band(&self, route: &str) -> (String, String) {
        let Some(r) = Route::ALL.iter().find(|r| r.id == route) else {
            return (String::new(), String::new());
        };
        let Some(strategy) = r.strategy else {
            return (String::new(), String::new());
        };
        let run = Run::new(self.graph.root().term(), &self.env, strategy, 40);
        let find = |t: &Term| self.graph.find(&t.key());
        let (mut paths, mut steps, mut left) = (String::new(), 0, false);
        for i in 1..run.rows().len() {
            let (Some(a), Some(b)) = (find(run.rows()[i - 1].term()), find(run.rows()[i].term())) else {
                left = true;
                break;
            };
            let edges = self.graph.edges();
            let hit = edges
                .iter()
                .position(|e| e.from() == a && e.to() == b && Some(e.kind()) == run.rows()[i].kind())
                .or_else(|| edges.iter().position(|e| e.from() == a && e.to() == b));
            if let Some(e) = hit {
                paths += &format!("<path class=\"rdx-band\" d=\"{}\"/>", self.plot.routes()[e]);
            }
            steps = i;
        }
        let end = find(run.rows()[steps.min(run.rows().len() - 1)].term());
        if let Some(end) = end.filter(|_| run.status() == Status::Done && !left) {
            let s = &self.plot.spots()[end];
            paths += &format!(
                "<circle class=\"rdx-band-end\" cx=\"{}\" cy=\"{}\" r=\"{}\"/>",
                Self::n(s.x()),
                Self::n(s.y()),
                Self::n(Plot::R + 9.0)
            );
        }
        let note = match (left, run.status()) {
            (true, _) => format!("{} leaves the explored graph after {steps} steps.", r.label),
            (false, Status::Done) => format!(
                "{}: {steps} step{} to a {}.",
                r.label,
                if steps == 1 { "" } else { "s" },
                end.map_or("stop", |e| self.plot.spots()[e].form().label())
            ),
            (false, _) => format!("{} never stops: it loops.", r.label),
        };
        (paths, note)
    }

    pub fn tip(&self, i: usize) -> String {
        let v = &self.graph.vertices()[i];
        format!(
            "<div class=\"rdx-tip-meta\"><b>{}</b> · size {} · {} step{} from the start</div><div class=\"rdx-tm\">{}</div>",
            self.plot.spots()[i].form().label(),
            v.term().size(),
            v.depth(),
            if v.depth() == 1 { "" } else { "s" },
            Html::esc(&self.labels[i])
        )
    }

    pub fn selected(&self, sel: Option<usize>) -> String {
        let Some(i) = sel else {
            return "<p>Hover a mark to read its term. Click it to open it in the workbench.</p>".to_string();
        };
        let v = &self.graph.vertices()[i];
        let where_ = match v.depth() {
            0 => "Start term".to_string(),
            d => format!("{d} step{} from the start on the shortest path", if d > 1 { "s" } else { "" }),
        };
        format!(
            "<div class=\"rdx-scroll\"><div class=\"rdx-stage small flat rdx-tm\">{}</div></div><div class=\"rdx-row\"><span>{where_} · {} · size {}</span><span class=\"rdx-grow\"></span><button class=\"rdx-btn primary\" type=\"button\" data-act=\"open\">Open in workbench</button></div>",
            Blocks::new(&self.env, false, false).html(v.term()),
            self.plot.spots()[i].form().label(),
            v.term().size()
        )
    }

    /// The start-to-here chain of a vertex, as history entries.
    pub fn chain(&self, i: usize) -> Vec<Entry> {
        let vs = self.graph.vertices();
        let mut chain = std::iter::successors(Some(i), |&k| vs[k].parent()).collect::<Vec<usize>>();
        chain.reverse();
        chain
            .iter()
            .enumerate()
            .map(|(j, &k)| Entry {
                term: vs[k].term().clone(),
                kind: if j > 0 { vs[k].pkind() } else { None },
                note: None,
            })
            .collect()
    }
}

/// The graph widget's own state.
struct Board {
    session: Session,
    anchor: NodeRef<html::Div>,
    chart: RefCell<Option<Chart>>,
    sel: Cell<Option<usize>>,
    route: Cell<&'static str>,
    uid: String,
}

impl Board {
    fn q(&self, sel: &str) -> Option<Element> {
        self.anchor
            .get_untracked()
            .and_then(|a| a.parent_element())
            .and_then(|r| Doc::q(&r, sel))
    }

    fn eta(&self) -> Option<web_sys::HtmlInputElement> {
        self.q("[data-opt=\"eta\"]").and_then(|e| e.dyn_into().ok())
    }

    fn build(&self, start: &Term) {
        let eta = self.eta().is_some_and(|e| e.checked());
        *self.chart.borrow_mut() = Some(Chart::new(start, &self.session.env(), eta, &self.uid));
        self.sel.set(None);
        self.draw();
    }

    fn draw(&self) {
        let Some(svg) = self.q(".rdx-gsvg") else { return };
        {
            let chart = self.chart.borrow();
            let Some(chart) = chart.as_ref() else { return };
            let (w, h) = (Number::show(chart.plot().width()), Number::show(chart.plot().height()));
            let _ = svg.set_attribute("width", &w);
            let _ = svg.set_attribute("height", &h);
            let _ = svg.set_attribute("viewBox", &format!("0 0 {w} {h}"));
            svg.set_inner_html(&chart.svg(self.sel.get()));
            let row = |e: &Element| {
                e.dyn_ref::<web_sys::SvgElement>()
                    .and_then(|s| s.style().get_property_value("--d").ok())
                    .and_then(|d| d.trim().parse::<f64>().ok())
                    .unwrap_or(0.0)
                    * 70.0
            };
            let grow = Doc::all(&svg, ".rdx-gn")
                .iter()
                .map(|g| Motion::after(row(g), Motion::enter(g, Tween::default().ms(400.0))))
                .chain(
                    Doc::all(&svg, ".rdx-ge")
                        .iter()
                        .map(|p| Motion::after(row(p) + 90.0, Motion::draw(p, Tween::default().ms(500.0)))),
                )
                .collect();
            leptos::task::spawn_local(Motion::all(grow).play(Clock::default()));
            if let Some(stats) = self.q(".rdx-gstats") {
                stats.set_text_content(Some(&chart.stats()));
            }
            if let Some(verdict) = self.q(".rdx-verdict") {
                verdict.set_text_content(Some(&chart.verdict()));
            }
        }
        self.render_routes();
        self.render_sel();
        self.mark();
    }

    fn render_routes(&self) {
        if let Some(pills) = self.q("[data-routes]") {
            pills.set_inner_html(&Chart::pills(self.route.get()));
        }
        let (band, note) = self
            .chart
            .borrow()
            .as_ref()
            .map(|c| c.band(self.route.get()))
            .unwrap_or_default();
        if let Some(g) = self.q(".rdx-gsvg .routes") {
            g.set_inner_html(&band);
        }
        if let Some(n) = self.q(".rdx-route-note") {
            n.set_text_content(Some(&note));
        }
    }

    fn render_sel(&self) {
        if let (Some(b), Some(c)) = (self.q(".rdx-gsel"), self.chart.borrow().as_ref()) {
            b.set_inner_html(&c.selected(self.sel.get()));
        }
    }

    fn select(&self, i: usize) {
        self.sel.set(Some(i));
        if let Some(svg) = self.q(".rdx-gsvg") {
            Doc::all(&svg, ".rdx-gn")
                .iter()
                .for_each(|g| Doc::toggle(g, "sel", Doc::attr(g, "data-id") == Some(i.to_string())));
        }
        self.render_sel();
    }

    /// The workbench's position: its term gets a halo, the steps it walked are marked.
    fn mark(&self) {
        let chart = self.chart.borrow();
        let (Some(chart), Some(svg)) = (chart.as_ref(), self.q(".rdx-gsvg")) else {
            return;
        };
        let id = |t: &Term| chart.graph().find(&t.key()).map_or(-1, |i| i as i64);
        let (here, walked) = {
            let s = self.session.state();
            let here = id(&s.hist[s.idx].term);
            let walked = (1..=s.idx)
                .map(|i| (id(&s.hist[i - 1].term), id(&s.hist[i].term)))
                .filter(|(a, b)| *a >= 0 && *b >= 0 && a != b)
                .map(|(a, b)| format!("{a}>{b}"))
                .collect::<Vec<String>>();
            (here, walked)
        };
        Doc::all(&svg, ".rdx-gn")
            .iter()
            .for_each(|g| Doc::toggle(g, "here", Doc::attr(g, "data-id") == Some(here.to_string())));
        Doc::all(&svg, ".rdx-ge").iter().for_each(|g| {
            let key = format!(
                "{}>{}",
                Doc::attr(g, "data-from").unwrap_or_default(),
                Doc::attr(g, "data-to").unwrap_or_default()
            );
            Doc::toggle(g, "walked", walked.contains(&key));
        });
    }

    fn show_tip(&self, g: Option<Element>) {
        let Some(tip) = self
            .q(".rdx-tip")
            .and_then(|t| t.dyn_into::<web_sys::HtmlElement>().ok())
        else {
            return;
        };
        let Some(g) = g else {
            tip.set_hidden(true);
            return;
        };
        let Some(i) = Doc::attr(&g, "data-id").and_then(|i| i.parse::<usize>().ok()) else {
            return;
        };
        if let Some(c) = self.chart.borrow().as_ref() {
            tip.set_inner_html(&c.tip(i));
        }
        tip.set_hidden(false);
        let (Some(wrap), Some(hit)) = (self.q(".rdx-gwrap"), Doc::q(&g, ".hit")) else {
            return;
        };
        let (wrap, r) = (wrap.get_bounding_client_rect(), hit.get_bounding_client_rect());
        let (tw, th) = (f64::from(tip.offset_width()), f64::from(tip.offset_height()));
        let left = (r.left() - wrap.left() + r.width() / 2.0 - tw / 2.0)
            .max(8.0)
            .min(wrap.width() - tw - 8.0);
        let below = r.top() - wrap.top() < th + 12.0;
        let top = if below {
            r.bottom() - wrap.top() + 6.0
        } else {
            r.top() - wrap.top() - th - 6.0
        };
        let _ = tip.style().set_property("left", &format!("{left}px"));
        let _ = tip.style().set_property("top", &format!("{top}px"));
    }

    fn open(&self, i: usize) {
        if self.session.busy() {
            return;
        }
        let Some(chain) = self.chart.borrow().as_ref().map(|c| c.chain(i)) else {
            return;
        };
        self.session.replace_history(chain);
        self.session.reveal();
    }

    fn click(&self, e: &web_sys::Event) {
        let Some(target) = Doc::target(e) else { return };
        match Doc::closest(&target, "[data-act]")
            .and_then(|a| Doc::attr(&a, "data-act"))
            .as_deref()
        {
            Some("rebuild") => self.build(&self.session.term()),
            Some("open") => {
                if let Some(i) = self.sel.get() {
                    self.open(i);
                }
            }
            _ => {}
        }
        if let Some(id) = Doc::closest(&target, "[data-route]").and_then(|r| Doc::attr(&r, "data-route")) {
            self.route
                .set(Route::ALL.iter().find(|r| r.id == id).map_or("none", |r| r.id));
            self.render_routes();
        }
    }

    fn mark_of(e: &web_sys::Event) -> Option<Element> {
        Doc::target(e).and_then(|t| Doc::closest(&t, ".rdx-gn"))
    }

    fn id_of(g: &Element) -> Option<usize> {
        Doc::attr(g, "data-id").and_then(|i| i.parse().ok())
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
        route: Cell::new("none"),
        uid: format!("rdx{}", Doc::next_id()),
    });
    let me = board.clone();
    board.session.on(move |e| {
        if e == Event::Load {
            if let Some(eta) = me.eta() {
                eta.set_checked(me.session.state().eta);
            }
            me.build(&me.session.start());
        } else if e == Event::Step {
            me.mark();
        }
    });
    let (b1, b2, b3, b4, b5, b6, b7, b8, b9, b10) = (
        board.clone(),
        board.clone(),
        board.clone(),
        board.clone(),
        board.clone(),
        board.clone(),
        board.clone(),
        board.clone(),
        board.clone(),
        board.clone(),
    );
    let anchor = board.anchor;
    view! {
        <div class="rdx-gbar" node_ref=anchor on:click=move |e| b1.click(&e)>
            <span class="rdx-gstats"></span>
            <span class="rdx-grow"></span>
            <label class="rdx-toggle">
                <input
                    type="checkbox"
                    data-opt="eta"
                    on:change=move |_| {
                        let start = b2.chart.borrow().as_ref().map(|c| c.graph().root().term().clone());
                        if let Some(start) = start {
                            b2.build(&start);
                        }
                    }
                />
                " Include η"
            </label>
            <button type="button" class="rdx-btn" data-act="rebuild">
                "Rebuild from current term"
            </button>
        </div>
        <div class="rdx-gbar" on:click=move |e| b3.click(&e)>
            <span class="rdx-label">"Route"</span>
            <div class="rdx-pills" data-routes></div>
            <span class="rdx-route-note"></span>
        </div>
        <p class="rdx-verdict"></p>
        <div class="rdx-gwrap">
            <div class="rdx-gscroll">
                <svg
                    class="rdx-gsvg"
                    role="img"
                    aria-label="Reduction graph"
                    on:mouseover=move |e| b4.show_tip(Board::mark_of(&e))
                    on:mouseleave=move |_| b5.show_tip(None)
                    on:focusin=move |e| b6.show_tip(Board::mark_of(&e))
                    on:focusout=move |_| b9.show_tip(None)
                    on:click=move |e| {
                        if let Some(i) = Board::mark_of(&e).as_ref().and_then(Board::id_of) {
                            b10.select(i);
                        }
                    }
                    on:keydown=move |e: web_sys::KeyboardEvent| {
                        if let Some(i) = Board::mark_of(&e).as_ref().and_then(Board::id_of)
                            && (e.key() == "Enter" || e.key() == " ")
                        {
                            e.prevent_default();
                            b7.select(i);
                        }
                    }
                ></svg>
            </div>
            <div class="rdx-tip" hidden></div>
        </div>
        <div class="rdx-glegend">
            <span>
                <svg width="14" height="14">
                    <circle cx="7" cy="7" r="5" class="m red"></circle>
                </svg>
                "reducible"
            </span>
            <span>
                <svg width="14" height="14">
                    <path d="M7 1.5L12.5 7L7 12.5L1.5 7Z" class="m whnf"></path>
                </svg>
                "weak head normal form"
            </span>
            <span>
                <svg width="14" height="14">
                    <rect x="2" y="2" width="10" height="10" class="m nf"></rect>
                </svg>
                "normal form"
            </span>
            <span>
                <svg width="18" height="18">
                    <circle cx="9" cy="9" r="4.5" class="m red"></circle>
                    <circle cx="9" cy="9" r="8" class="ring"></circle>
                </svg>
                "start"
            </span>
            <span>
                <svg width="18" height="18">
                    <circle cx="9" cy="9" r="4.5" class="m red"></circle>
                    <circle cx="9" cy="9" r="8" class="halo"></circle>
                </svg>
                "workbench is here"
            </span>
            <span>
                <b class="rdx-sz">"12"</b>
                " term size"
            </span>
        </div>
        <div class="rdx-gsel" on:click=move |e| b8.click(&e)></div>
    }
}
