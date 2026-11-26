use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;

use leptos::html;
use leptos::prelude::*;
use motion::*;
use rules::*;
use strategy::*;
use subst::*;
use syntax::*;
use term::*;
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
    name: &'static str,
    slot: usize,
    el: Element,
    uid: Option<String>,
    binder: Option<String>,
    text: Option<String>,
    external: bool,
    rect: Option<DomRect>,
    ghost: Option<Element>,
}

/// A redex matched against its rule: the schema, its variables, and the pieces they bound.
struct Match {
    schema: Schema,
    vars: Vec<&'static str>,
    pieces: Vec<Piece>,
}

/// The redex pinned over the page before a step, and the layout it left.
struct Lifted {
    husk: Option<Element>,
    pieces: Vec<Piece>,
    layout: HashMap<String, DomRect>,
}

/// The workbench's own state, beside the session.
#[derive(Default)]
struct Local {
    arcs: bool,
    db: bool,
    hot: Option<Element>,
    running: Option<Mode>,
    stop: bool,
    note: String,
    raf: i32,
}

/// The workbench. Every firing follows one grammar: match (hovering a badge shows the rule's
/// schema; its pattern variables and the pieces they matched share a highlighter color), bind (the
/// rule card lists the bindings), move (each bound piece flies to every place the result uses it).
struct Bench {
    session: Session,
    anchor: NodeRef<html::Div>,
    local: RefCell<Local>,
}

impl Bench {
    const HINT: &'static str = "Hover a badge to see its rule match. Click to fire it. Click a binder to rename it. <kbd>→</kbd> steps, <kbd>←</kbd> goes back.";
    const NAMES: [&'static str; 20] = [
        "x", "y", "z", "w", "u", "v", "a", "b", "c", "f", "g", "h", "k", "m", "n", "p", "q", "r", "s", "t",
    ];
    /// How long a firing shows its match before anything moves, at speed 1.
    const HOLD: f64 = 560.0;
    /// How long an automatic α step stays on screen before its β fires.
    const RENAME: f64 = 1000.0;

    fn rule_name(kind: Kind) -> &'static str {
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

    /// The text cut to `n` UTF-16 units, as `String.prototype.slice` counts them.
    fn short(s: &str, n: usize) -> String {
        let units = s.encode_utf16().collect::<Vec<u16>>();
        if units.len() > n {
            String::from_utf16_lossy(&units[..n - 1]) + "…"
        } else {
            s.to_string()
        }
    }

    fn root(&self) -> Option<Element> {
        self.anchor.get_untracked().and_then(|a| a.parent_element())
    }

    fn q(&self, sel: &str) -> Option<Element> {
        self.root().and_then(|r| Doc::q(&r, sel))
    }

    fn line(&self) -> Option<Element> {
        self.q(".rdx-line")
    }

    fn stage(&self) -> Option<Element> {
        self.q(".rdx-stage")
    }

    fn node(&self, p: &Path) -> Option<Element> {
        self.line().and_then(|l| Doc::q(&l, &format!("[data-path=\"{p}\"]")))
    }

    fn button(&self, act: &str) -> Option<web_sys::HtmlButtonElement> {
        self.q(&format!("[data-act=\"{act}\"]")).and_then(|b| b.dyn_into().ok())
    }

    fn render(&self) {
        let (t, env, busy) = (self.session.term(), self.session.env(), self.session.busy());
        let db = self.local.borrow().db;
        if let Some(line) = self.line() {
            line.set_inner_html(&Blocks::new(&env, true, db).html(&t));
        }
        t.strip();
        self.arcs();
        self.local.borrow_mut().hot = None;
        if !busy {
            self.show_rule(None);
        }
        self.render_status();
        self.render_trace();
        self.render_capture();
        self.set_controls();
    }

    fn arcs(&self) {
        if let (Some(stage), Some(line)) = (self.stage(), self.line()) {
            Doc::arcs(&stage, &line, self.local.borrow().arcs);
        }
    }

    fn render_source(&self) {
        let s = self.session.state();
        if let Some(notes) = self.q(".rdx-notes") {
            notes.set_inner_html(
                &s.notes
                    .iter()
                    .map(|n| format!("<p><span class=\"rdx-dash\">--</span> {}</p>", Html::esc(n)))
                    .collect::<Vec<String>>()
                    .concat(),
            );
        }
        if let Some(defs) = self.q(".rdx-defs") {
            Doc::hide(&defs, s.env.is_empty());
        }
        if let Some(list) = self.q(".rdx-defs-list") {
            list.set_inner_html(
                &s.env
                    .iter()
                    .map(|(n, b)| {
                        format!(
                            "<div class=\"rdx-def rdx-tm\"><span class=\"b b-const b-def\">{0}</span><span class=\"rk\">:=</span><span data-def=\"{0}\">{1}</span></div>",
                            Html::esc(n),
                            Blocks::new(&s.env, false, false).html(b)
                        )
                    })
                    .collect::<Vec<String>>().concat(),
            );
        }
        if let Some(src) = self
            .q(".rdx-src textarea")
            .and_then(|e| e.dyn_into::<web_sys::HtmlTextAreaElement>().ok())
        {
            src.set_value(&s.src);
        }
        if let Some(err) = self.q(".rdx-err") {
            Doc::hide(&err, true);
        }
    }

    fn render_status(&self) {
        let (t, env) = (self.session.term(), self.session.env());
        let rs = t.redexes(&env);
        let main = rs.iter().filter(|r| !r.kind().is_eta()).count();
        let head = match (main, Strategy::Whnf.next(&t, &env)) {
            (0, _) => format!(
                "<b class=\"rdx-nf\">Normal form</b>{}",
                if rs.is_empty() { "" } else { " <span>except for η</span>" }
            ),
            (_, None) => "<b class=\"rdx-whnf\">Weak head normal form</b>".to_string(),
            _ => format!("<b>{} redex{}</b>", rs.len(), if rs.len() > 1 { "es" } else { "" }),
        };
        let tally = rs.iter().fold(Vec::<(Kind, usize)>::new(), |mut tally, r| {
            match tally.iter_mut().find(|(k, _)| *k == r.kind()) {
                Some(slot) => slot.1 += 1,
                None => tally.push((r.kind(), 1)),
            }
            tally
        });
        let marks = tally
            .iter()
            .map(|(k, n)| {
                format!(
                    "<span class=\"rdx-tally\">{}{}</span>",
                    Doc::flat(Some(*k)),
                    if *n > 1 { format!("×{n}") } else { String::new() }
                )
            })
            .collect::<Vec<String>>()
            .concat();
        let (idx, last) = {
            let s = self.session.state();
            (s.idx, s.hist.len() - 1)
        };
        let note = self.local.borrow().note.clone();
        let html = format!(
            "{head}{}<span class=\"rdx-stepno\">step {idx}{} · size {}</span>{}",
            if marks.is_empty() {
                String::new()
            } else {
                format!("<span class=\"rdx-tallies\">{marks}</span>")
            },
            if last > idx { format!(" of {last}") } else { String::new() },
            t.size(),
            if note.is_empty() {
                String::new()
            } else {
                format!("<span class=\"rdx-note\">{}</span>", Html::esc(&note))
            }
        );
        if let Some(status) = self.q(".rdx-status") {
            status.set_inner_html(&html);
        }
    }

    fn render_trace(&self) {
        let Some(ol) = self
            .q(".rdx-trace ol")
            .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
        else {
            return;
        };
        let idx = self.session.state().idx;
        ol.set_inner_html(
            &self
                .session
                .state()
                .hist
                .iter()
                .enumerate()
                .map(|(i, h)| {
                    format!(
                        "<li class=\"{}{}\"><button type=\"button\" data-i=\"{i}\"><span class=\"rdx-ti\">{i}</span>{}<code>{}</code>{}</button></li>",
                        if i == idx { "cur" } else { "" },
                        if i > idx { " fut" } else { "" },
                        Doc::flat(h.kind),
                        Html::esc(&h.term.text()),
                        h.note.as_ref().map(|n| format!("<small>{}</small>", Html::esc(n))).unwrap_or_default()
                    )
                })
                .collect::<Vec<String>>().concat(),
        );
        let Some(li) = ol
            .children()
            .item(idx as u32)
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
        let capture = self.session.state().capture.clone();
        let Some(box_) = self.q(".rdx-capture") else { return };
        Doc::hide(&box_, capture.is_none());
        let Some(capture) = capture else { return };
        if let (Some(stage), Some(line)) = (Doc::q(&box_, ".rdx-stage"), Doc::q(&box_, ".rdx-line")) {
            line.set_inner_html(&Blocks::new(&self.session.env(), false, false).html(&capture));
            Doc::arcs(&stage, &line, true);
        }
    }

    fn set_controls(&self) {
        let (t, env, busy) = (self.session.term(), self.session.env(), self.session.busy());
        let (idx, len) = {
            let s = self.session.state();
            (s.idx, s.hist.len())
        };
        let running = self.local.borrow().running;
        let (n, w) = (Strategy::Normal.next(&t, &env).is_some(), Strategy::Whnf.next(&t, &env).is_some());
        let set = |act: &str, disabled: bool, label: Option<&str>| {
            if let Some(b) = self.button(act) {
                b.set_disabled(disabled);
                if let Some(label) = label {
                    b.set_text_content(Some(label));
                }
            }
        };
        set("step", busy || !n, None);
        set(
            "whnf",
            if busy { running != Some(Mode::Whnf) } else { !w },
            Some(if running == Some(Mode::Whnf) { "Stop" } else { "To WHNF" }),
        );
        set(
            "nf",
            if busy { running != Some(Mode::Nf) } else { !n },
            Some(if running == Some(Mode::Nf) { "Stop" } else { "Normalize" }),
        );
        set("back", busy || idx == 0, None);
        set("reset", busy || (idx == 0 && len == 1), None);
    }

    fn matched(&self, path: &Path, kind: Kind) -> Match {
        let t = self.session.term();
        let redex = t.at(path).cloned().expect("a redex inside the term");
        let schema = kind.schema(&redex);
        let vars = schema.vars();
        let mut pieces = kind
            .bindings(&redex)
            .iter()
            .filter_map(|b| {
                let slot = vars.iter().position(|v| *v == b.name()).map_or(0, |i| i + 1);
                let owner = self.node(&path.join(b.rel()))?;
                let piece = Piece {
                    name: b.name(),
                    slot,
                    el: owner.clone(),
                    uid: None,
                    binder: None,
                    text: None,
                    external: false,
                    rect: None,
                    ghost: None,
                };
                if b.is_binder() {
                    let tab = Doc::q(&owner, ":scope > .bt > .rb").or_else(|| Doc::q(&owner, ":scope > .bt > .rk"))?;
                    Some(Piece {
                        el: tab,
                        binder: Doc::attr(&owner, "data-binder"),
                        ..piece
                    })
                } else {
                    Some(Piece {
                        uid: Doc::attr(&owner, "data-uid"),
                        text: redex.at(b.rel()).map(Print::text),
                        ..piece
                    })
                }
            })
            .collect::<Vec<Piece>>();
        if kind == Kind::Delta {
            let body = redex
                .name()
                .and_then(|n| self.q(&format!(".rdx-defs-list [data-def=\"{}\"] > .b", web_sys::css::escape(n))));
            if let Some(body) = body {
                pieces.push(Piece {
                    name: "c",
                    slot: 1,
                    uid: Doc::attr(&body, "data-uid"),
                    el: body,
                    binder: None,
                    text: None,
                    external: true,
                    rect: None,
                    ghost: None,
                });
            }
        }
        Match { schema, vars, pieces }
    }

    fn paint(&self, m: &Match, strength: &str) {
        self.unpaint();
        let line = self.line();
        m.pieces.iter().filter(|pc| pc.slot > 0).for_each(|pc| {
            let slot = format!("s{}", pc.slot);
            Doc::add(&pc.el, &["mv", &slot, strength]);
            if let (Some(b), Some(line)) = (&pc.binder, &line) {
                Doc::all(line, &format!("[data-bid=\"{b}\"]"))
                    .iter()
                    .for_each(|v| Doc::add(v, &["mv", &slot, strength]));
            }
        });
    }

    fn unpaint(&self) {
        if let Some(root) = self.root() {
            Doc::all(&root, ".mv")
                .iter()
                .for_each(|e| Doc::remove(e, &["mv", "s1", "s2", "s3", "s4", "soft", "hard"]));
        }
    }

    fn show_rule(&self, hit: Option<(&Element, &Match)>) {
        let Some(rule) = self.q(".rdx-rule") else { return };
        let Some((rx, m)) = hit else {
            rule.set_inner_html(&format!("<p class=\"rdx-hint\">{}</p>", Self::HINT));
            return;
        };
        let Some(kind) = Doc::attr(rx, "data-kind").and_then(|k| Kind::parse(&k)) else {
            return;
        };
        let schema = m
            .schema
            .parts()
            .iter()
            .map(|(var, s)| match var {
                true => format!(
                    "<span class=\"mv s{}\">{s}</span>",
                    m.vars.iter().position(|v| v == s).map_or(0, |i| i + 1)
                ),
                false => (*s).to_string(),
            })
            .collect::<Vec<String>>()
            .concat();
        let binds = m
            .pieces
            .iter()
            .filter(|pc| pc.slot > 0 && !pc.external)
            .map(|pc| {
                let shown = if pc.binder.is_some() {
                    pc.el.text_content().unwrap_or_default()
                } else {
                    pc.text.clone().unwrap_or_default()
                };
                format!(
                    "<span class=\"rdx-bind\"><span class=\"mv s{}\">{}</span> = <code>{}</code></span>",
                    pc.slot,
                    pc.name,
                    Html::esc(&Self::short(&shown, 34))
                )
            })
            .collect::<Vec<String>>()
            .concat();
        rule.set_inner_html(&format!(
            "<p class=\"rdx-schema\"><span class=\"rm flat k-{}\">{}</span><b>{}</b><code>{schema}</code></p><p class=\"rdx-binds\">{binds}</p>",
            kind.class(),
            kind.glyph(),
            Self::rule_name(kind)
        ));
    }

    fn set_hot(&self, rx: Option<Element>) {
        if self.local.borrow().hot == rx {
            return;
        }
        if let Some(old) = self.local.borrow().hot.as_ref() {
            Doc::remove(old, &["hot"]);
        }
        self.local.borrow_mut().hot.clone_from(&rx);
        let Some(rx) = rx else {
            self.unpaint();
            self.show_rule(None);
            return;
        };
        Doc::add(&rx, &["hot"]);
        let (Some(path), Some(kind)) =
            (Doc::attr(&rx, "data-path"), Doc::attr(&rx, "data-kind").and_then(|k| Kind::parse(&k)))
        else {
            return;
        };
        let m = self.matched(&Path::from(path), kind);
        self.paint(&m, "soft");
        self.show_rule(Some((&rx, &m)));
    }

    fn set_lit(&self, id: Option<String>) {
        let Some(stage) = self.stage() else { return };
        Doc::all(&stage, ".lit").iter().for_each(|e| Doc::remove(e, &["lit"]));
        if let Some(id) = id {
            Doc::all(&stage, &format!("[data-id=\"{id}\"], [data-bid=\"{id}\"], .rb[data-anchor=\"{id}\"]"))
                .iter()
                .for_each(|e| Doc::add(e, &["lit"]));
        }
    }

    /// Before the step: pins the redex and its pieces over the page, and measures the layout.
    fn lift(&self, m: &Match, path: &Path) -> Lifted {
        let rx = self.node(path);
        let mut pieces = m
            .pieces
            .iter()
            .map(|pc| Piece {
                rect: Some(pc.el.get_bounding_client_rect()),
                ..pc.clone()
            })
            .collect::<Vec<Piece>>();
        pieces
            .iter()
            .enumerate()
            .filter(|(_, pc)| pc.uid.is_some() && !pc.external)
            .for_each(|(i, pc)| {
                let _ = pc.el.set_attribute("data-piece", &i.to_string());
            });
        let husk = rx.and_then(|rx| Ghost::pin(&rx, None, false, &[]));
        if let Some(husk) = &husk {
            Doc::all(husk, "[data-piece]")
                .iter()
                .filter_map(|e| e.dyn_ref::<web_sys::HtmlElement>().cloned())
                .for_each(|e| {
                    let _ = e.style().set_property("visibility", "hidden");
                });
        }
        pieces.iter_mut().for_each(|pc| {
            let _ = pc.el.remove_attribute("data-piece");
            if pc.uid.is_some() {
                pc.ghost = Ghost::pin(&pc.el, None, false, &[]);
            }
        });
        Lifted {
            husk,
            pieces,
            layout: self.layout(),
        }
    }

    fn layout(&self) -> HashMap<String, DomRect> {
        let mut rects = HashMap::new();
        if let Some(stage) = self.stage() {
            rects.insert("stage".to_string(), stage.get_bounding_client_rect());
        }
        if let Some(line) = self.line() {
            Doc::all(&line, "[data-uid]").iter().for_each(|e| {
                rects.insert(Doc::attr(e, "data-uid").unwrap_or_default(), e.get_bounding_client_rect());
            });
        }
        rects
    }

    /// After the step: the composition that carries the old layout to the new one. The husk fades
    /// out, each piece flies to every node of the result built from it (or fades), the result fades
    /// in, and the context glides.
    fn move_to(self: &Rc<Self>, lifted: Lifted, path: &Path) -> Motion {
        let Some(result) = self.node(path) else {
            return Motion::all(
                lifted
                    .husk
                    .iter()
                    .chain(lifted.pieces.iter().filter_map(|pc| pc.ghost.as_ref()))
                    .map(|g| Motion::exit(g, Tween::default()))
                    .collect(),
            );
        };
        let husk = lifted
            .husk
            .as_ref()
            .map_or_else(Motion::none, |h| Motion::exit(h, Tween::default()));
        let enter = Motion::enter(&result, Tween::default());
        let travels = lifted
            .pieces
            .iter()
            .map(|pc| Self::travel(pc, &result))
            .collect::<Vec<Option<Motion>>>();
        let glide = self.glide(&lifted.layout, &result);
        Motion::all(vec![husk, enter, Motion::stagger(50.0, travels), glide])
    }

    fn travel(pc: &Piece, result: &Element) -> Option<Motion> {
        let ghost = pc.ghost.clone()?;
        let uid = pc.uid.clone().unwrap_or_default();
        let hits = std::iter::once(result.clone())
            .chain(Doc::all(result, "[data-uid], [data-from]"))
            .filter(|e| {
                Doc::attr(e, "data-uid").as_ref() == Some(&uid) || Doc::attr(e, "data-from").as_ref() == Some(&uid)
            })
            .collect::<Vec<Element>>();
        if hits.is_empty() {
            return Some(if pc.external {
                Motion::act(move || ghost.remove())
            } else {
                Motion::exit(&ghost, Tween::default())
            });
        }
        let rect = pc.rect.clone()?;
        let slot = pc.slot;
        Some(Motion::stagger(
            40.0,
            hits.iter()
                .enumerate()
                .map(|(j, tg)| {
                    let src = if j > 0 { Self::twin(&ghost) } else { Some(ghost.clone()) }?;
                    let mark = format!("s{slot}");
                    let marks = if slot > 0 { vec!["mv", mark.as_str(), "hard"] } else { vec![] };
                    let dst = Ghost::pin(tg, Some(&rect), true, &marks)?;
                    let (dx, dy) = Ghost::delta(&rect, &tg.get_bounding_client_rect());
                    Doc::add(tg, &["arriving"]);
                    let (tg, a, b) = (tg.clone(), src.clone(), dst.clone());
                    Some(Motion::seq(vec![
                        Motion::all(vec![
                            Motion::fly(&src, dx, dy, Tween::default()),
                            Motion::fly(&dst, dx, dy, Tween::default()),
                            Motion::crossfade(&src, &dst, Tween::default()),
                        ]),
                        Motion::act(move || {
                            Doc::remove(&tg, &["arriving"]);
                            if slot > 0 {
                                Doc::add(&tg, &["landed", &format!("s{slot}")]);
                            }
                        }),
                        Motion::act(move || {
                            a.remove();
                            b.remove();
                        }),
                    ]))
                })
                .collect(),
        ))
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
    fn glide(self: &Rc<Self>, before: &HashMap<String, DomRect>, result: &Element) -> Motion {
        let (Some(stage), Some(line)) = (self.stage(), self.line()) else {
            return Motion::none();
        };
        let mut moves = before
            .get("stage")
            .map(|old| {
                vec![Motion::resize(
                    &stage,
                    old,
                    &stage.get_bounding_client_rect(),
                    Tween::default(),
                )]
            })
            .unwrap_or_default();
        Self::visit(&line, before, result, &mut moves);
        let (on, off, me) = (stage.clone(), stage, self.clone());
        Motion::seq(vec![
            Motion::act(move || Doc::add(&on, &["moving"])),
            Motion::all(moves),
            Motion::act(move || {
                Doc::remove(&off, &["moving"]);
                me.arcs();
            }),
        ])
    }

    fn visit(parent: &Element, before: &HashMap<String, DomRect>, result: &Element, moves: &mut Vec<Motion>) {
        for c in Doc::children(parent) {
            if &c == result {
                continue;
            }
            let Some(uid) = Doc::attr(&c, "data-uid") else {
                Self::visit(&c, before, result, moves);
                continue;
            };
            let (old, now) = (before.get(&uid), c.get_bounding_client_rect());
            if c.contains(Some(result)) {
                if let Some(old) = old {
                    moves.push(Motion::resize(&c, old, &now, Tween::default()));
                }
                Self::visit(&c, before, result, moves);
            } else {
                match old {
                    None => moves.push(Motion::enter(&c, Tween::default())),
                    Some(old) => {
                        moves.push(Motion::slide(&c, old.left() - now.left(), old.top() - now.top(), Tween::default()));
                    }
                }
            }
        }
    }

    async fn fire_at(self: Rc<Self>, path: Path, kind: Kind, speed: f64) {
        let t = self.session.term();
        let redex = t.at(&path).cloned().expect("a redex inside the term");
        let bound = match (kind, redex.shape()) {
            (Kind::Beta, Shape::App(f, a)) => Some((f.clone(), a.clone())),
            (Kind::Zeta, Shape::Let(_, v, _)) => Some((redex.clone(), v.clone())),
            _ => None,
        };
        if let Some((lam, arg)) = bound {
            let (x, b) = match lam.shape() {
                Shape::Lam(x, b) | Shape::Let(x, _, b) => (x.clone(), b.clone()),
                _ => unreachable!("β and ζ bind a name"),
            };
            let mut log = Vec::new();
            let body = b.prepare(&x, &arg.fv(), &mut log);
            if !log.is_empty() {
                let naive = b.naive(&x, &arg);
                self.session.update(|s| s.capture = Some(naive));
                let renamed = match redex.shape() {
                    Shape::Let(y, v, _) => Term::local(y, v.clone(), body),
                    _ => Term::app(Term::lam(&x, body), arg),
                };
                let olds = log.iter().map(|(a, _)| a.clone()).collect::<Vec<String>>().join(", ");
                let note = format!(
                    "{}, so the argument’s free {olds} stays free",
                    log.iter()
                        .map(|(a, b)| format!("{a} → {b}"))
                        .collect::<Vec<String>>()
                        .join(", ")
                );
                self.session.push(Entry {
                    term: t.replace(&path, renamed),
                    kind: Some(Kind::Alpha),
                    note: Some(note),
                });
                if Motion::enabled() {
                    Motion::wait(Self::RENAME / speed).await;
                }
            }
        }
        let mut lifted = None;
        if Motion::enabled() {
            let (rx, m) = (self.node(&path), self.matched(&path, kind));
            self.local.borrow_mut().hot = None;
            if let Some(rx) = &rx {
                Doc::add(rx, &["firing"]);
            }
            self.show_rule(rx.as_ref().map(|rx| (rx, &m)));
            self.paint(&m, "hard");
            Motion::wait(Self::HOLD / speed).await;
            lifted = Some(self.lift(&m, &path));
        }
        let now = self.session.term();
        let env = self.session.env();
        let next = now.replace(&path, now.at(&path).expect("a redex inside the term").contract(kind, &env));
        self.session.push(Entry {
            term: next,
            kind: Some(kind),
            note: None,
        });
        if let Some(lifted) = lifted {
            self.move_to(lifted, &path).play(Clock::new(speed)).await;
        }
    }

    async fn fire_once(self: Rc<Self>, path: Path, kind: Kind) {
        if self.session.busy() {
            return;
        }
        self.session.update(|s| s.busy = true);
        self.local.borrow_mut().note.clear();
        self.set_controls();
        self.clone().fire_at(path, kind, 1.0).await;
        self.session.update(|s| s.busy = false);
        self.unpaint();
        self.show_rule(None);
        self.set_controls();
    }

    async fn run(self: Rc<Self>, mode: Mode) {
        if self.session.busy() {
            if self.local.borrow().running == Some(mode) {
                self.local.borrow_mut().stop = true;
            }
            return;
        }
        self.session.update(|s| s.busy = true);
        {
            let mut w = self.local.borrow_mut();
            w.running = if mode == Mode::Step { None } else { Some(mode) };
            w.stop = false;
            w.note.clear();
        }
        self.set_controls();
        let mut seen = {
            let s = self.session.state();
            s.hist[..=s.idx]
                .iter()
                .map(|h| h.term.key())
                .collect::<HashSet<String>>()
        };
        let (mut n, mut msg) = (0, "");
        while !self.local.borrow().stop {
            let (t, env) = (self.session.term(), self.session.env());
            let Some(r) = (if mode == Mode::Whnf { Strategy::Whnf } else { Strategy::Normal }).next(&t, &env) else {
                break;
            };
            let speed = match (mode, n) {
                (Mode::Step, _) => 1.0,
                (_, 0..3) => 1.3,
                (_, 3..12) => 2.2,
                _ => 4.5,
            };
            self.clone().fire_at(r.path().clone(), r.kind(), speed).await;
            n += 1;
            if mode == Mode::Step {
                break;
            }
            let term = self.session.term();
            if !seen.insert(term.key()) {
                msg = "Stopped: this term came back to one it already passed through. It loops.";
                break;
            }
            if n >= 200 {
                msg = "Stopped after 200 steps.";
                break;
            }
            if term.size() > 2000 {
                msg = "Stopped: the term grew past 2000 nodes.";
                break;
            }
        }
        self.session.update(|s| s.busy = false);
        {
            let mut w = self.local.borrow_mut();
            w.running = None;
            w.note = msg.to_string();
        }
        self.unpaint();
        self.show_rule(None);
        self.render_status();
        self.set_controls();
    }

    fn rename(&self, path: &Path) {
        if self.session.busy() || self.local.borrow().db {
            return;
        }
        let t = self.session.term();
        let Some(n) = t.at(path).cloned() else { return };
        let (Some(x), Some(b)) = (n.binder().map(String::from), n.kid('b').cloned()) else {
            return;
        };
        let avoid = b.names().into_iter().chain([x.clone()]).collect::<HashSet<String>>();
        let len = Self::NAMES.len() as isize;
        let i = Self::NAMES
            .iter()
            .position(|c| *c == Name::strip(&x))
            .map_or(-1, |i| i as isize);
        let y = (1..=len)
            .map(|j| Self::NAMES[((i + j) % len) as usize])
            .find(|c| !avoid.contains(*c) && !Consts::has(c))
            .map_or_else(|| Name::fresh(&x, &avoid), String::from);
        let body = b.subst(&x, &Term::var(&y), &HashSet::from([y.clone()]), Some(Mark::Renamed));
        let m = match n.shape() {
            Shape::Let(_, v, _) => Term::local(&y, v.clone(), body),
            _ => Term::lam(&y, body),
        }
        .marked(Mark::Binder);
        self.local.borrow_mut().note.clear();
        self.session.push(Entry {
            term: t.replace(path, m),
            kind: Some(Kind::Alpha),
            note: Some(format!("{x} → {y}")),
        });
    }

    fn back(&self) {
        let idx = self.session.state().idx;
        if !self.session.busy() && idx > 0 {
            self.local.borrow_mut().note.clear();
            self.session.seek(idx - 1);
        }
    }

    fn act(self: &Rc<Self>, act: &str) {
        match act {
            "step" => leptos::task::spawn_local(self.clone().run(Mode::Step)),
            "whnf" => leptos::task::spawn_local(self.clone().run(Mode::Whnf)),
            "nf" => leptos::task::spawn_local(self.clone().run(Mode::Nf)),
            "back" => self.back(),
            "reset" if !self.session.busy() => {
                self.local.borrow_mut().note.clear();
                self.session.reset();
            }
            "load" if !self.session.busy() => self.load(),
            _ => {}
        }
    }

    fn load(&self) {
        let src = self
            .q(".rdx-src textarea")
            .and_then(|e| e.dyn_into::<web_sys::HtmlTextAreaElement>().ok())
            .map(|t| t.value())
            .unwrap_or_default();
        let eta = self.session.state().eta;
        if let Err(e) = self.session.load(&src, None, eta) {
            if let Some(err) = self.q(".rdx-err") {
                err.set_text_content(Some(&e));
                Doc::hide(&err, false);
            }
            if let Some(details) = self
                .q(".rdx-src")
                .and_then(|e| e.dyn_into::<web_sys::HtmlDetailsElement>().ok())
            {
                details.set_open(true);
            }
        }
    }

    fn over(&self, e: &web_sys::Event) {
        if self.session.busy() {
            return;
        }
        let Some(target) = Doc::target(e) else { return };
        self.set_hot(Doc::closest(&target, ".rr"));
        let b = Doc::closest(&target, "[data-bid], .rb");
        self.set_lit(b.and_then(|b| {
            Doc::attr(&b, "data-bid")
                .filter(|s| !s.is_empty())
                .or_else(|| Doc::attr(&b, "data-anchor"))
        }));
    }

    fn click(self: &Rc<Self>, e: &web_sys::Event) {
        if self.session.busy() {
            return;
        }
        let Some(target) = Doc::target(e) else { return };
        if let Some(bn) = Doc::closest(&target, ".rb").filter(|b| b.tag_name() == "BUTTON") {
            self.rename(&Path::from(Doc::attr(&bn, "data-path").unwrap_or_default()));
            return;
        }
        if let Some(rx) = Doc::closest(&target, ".rr") {
            let (path, kind) = (
                Doc::attr(&rx, "data-path").unwrap_or_default(),
                Doc::attr(&rx, "data-kind").and_then(|k| Kind::parse(&k)),
            );
            if let Some(kind) = kind {
                leptos::task::spawn_local(self.clone().fire_once(Path::from(path), kind));
            }
        }
    }

    fn key(self: &Rc<Self>, e: &web_sys::KeyboardEvent) {
        let typing = Doc::target(e).is_some_and(|t| Doc::closest(&t, "textarea, input, select").is_some());
        if e.meta_key() || e.ctrl_key() || e.alt_key() || typing {
            return;
        }
        let (busy, t, env, idx) =
            (self.session.busy(), self.session.term(), self.session.env(), self.session.state().idx);
        if e.key() == "ArrowRight" && !busy && Strategy::Normal.next(&t, &env).is_some() {
            e.prevent_default();
            leptos::task::spawn_local(self.clone().run(Mode::Step));
        }
        if e.key() == "ArrowLeft" && !busy && idx > 0 {
            e.prevent_default();
            self.back();
        }
    }

    /// Redraws the binding links on the next frame, after a resize or once the fonts load.
    fn redraw(self: &Rc<Self>) {
        let Some(window) = web_sys::window() else { return };
        let _ = window.cancel_animation_frame(self.local.borrow().raf);
        let me = self.clone();
        let frame = Closure::once_into_js(move || {
            me.arcs();
            if me.session.state().capture.is_some()
                && let Some(box_) = me.q(".rdx-capture")
                && let (Some(stage), Some(line)) = (Doc::q(&box_, ".rdx-stage"), Doc::q(&box_, ".rdx-line"))
            {
                Doc::arcs(&stage, &line, true);
            }
        });
        self.local.borrow_mut().raf = window.request_animation_frame(frame.unchecked_ref()).unwrap_or(0);
    }

    /// Listeners that outlive the view: the session, the keyboard, resizes and fonts.
    fn listen(self: &Rc<Self>) {
        let me = self.clone();
        self.session.on(move |e| {
            if e == Event::Load {
                me.local.borrow_mut().note.clear();
                me.render_source();
            }
            me.render();
        });
        let me = self.clone();
        let keys = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(move |e: web_sys::KeyboardEvent| me.key(&e));
        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
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
        if let (Ok(observer), Some(parent)) = (
            web_sys::ResizeObserver::new(resized.as_ref().unchecked_ref()),
            self.stage().and_then(|s| s.parent_element()),
        ) {
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
        local: RefCell::new(Local {
            arcs: true,
            ..Local::default()
        }),
    });
    bench.listen();
    let mounted = bench.clone();
    anchor.on_load(move |_| mounted.observe());
    let (b1, b2, b3, b4, b5, b6, b7, b8) = (
        bench.clone(),
        bench.clone(),
        bench.clone(),
        bench.clone(),
        bench.clone(),
        bench.clone(),
        bench.clone(),
        bench,
    );
    let act = move |name: &'static str| {
        let b = b7.clone();
        move |_: web_sys::MouseEvent| b.act(name)
    };
    let opt = move |arcs: bool| {
        let b = b8.clone();
        move |e: web_sys::Event| {
            let on = Doc::target(&e)
                .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                .is_some_and(|i| i.checked());
            if arcs {
                b.local.borrow_mut().arcs = on;
            } else {
                b.local.borrow_mut().db = on;
            }
            b.render();
        }
    };
    view! {
        <div class="rdx-notes" node_ref=anchor></div>
        <div class="rdx-rule" aria-live="polite"></div>
        <div class="rdx-scroll">
            <div class="rdx-stage">
                <svg class="rdx-arcs" aria-hidden="true"></svg>
                <div
                    class="rdx-line rdx-tm"
                    on:mouseover=move |e| b1.over(&e)
                    on:mouseleave=move |_| {
                        if !b2.session.busy() {
                            b2.set_hot(None);
                            b2.set_lit(None);
                        }
                    }
                    on:focusin=move |e| {
                        let rx = Doc::target(&e).and_then(|t| Doc::closest(&t, ".rr"));
                        if rx.is_some() && !b3.session.busy() {
                            b3.set_hot(rx);
                        }
                    }
                    on:click=move |e| b4.click(&e)
                ></div>
            </div>
        </div>
        <div class="rdx-status"></div>
        <div class="rdx-controls">
            <button type="button" class="rdx-btn primary" data-act="step" on:click=act("step")>
                "Step"
            </button>
            <button type="button" class="rdx-btn" data-act="whnf" on:click=act("whnf")>
                "To WHNF"
            </button>
            <button type="button" class="rdx-btn" data-act="nf" on:click=act("nf")>
                "Normalize"
            </button>
            <button type="button" class="rdx-btn" data-act="back" on:click=act("back")>
                "Back"
            </button>
            <button type="button" class="rdx-btn" data-act="reset" on:click=act("reset")>
                "Reset"
            </button>
            <span class="rdx-grow"></span>
            <label class="rdx-toggle">
                <input type="checkbox" data-opt="arcs" checked on:change=opt(true)/>
                " Binding links"
            </label>
            <label class="rdx-toggle">
                <input type="checkbox" data-opt="db" on:change=opt(false)/>
                " de Bruijn"
            </label>
        </div>
        <div class="rdx-capture" hidden>
            <p>
                <b>"Why α fired first."</b>
                " Without renaming, substitution gives the term below. The dashed link is the argument’s free variable, caught by an inner binder, which changes what the term means."
            </p>
            <div class="rdx-scroll">
                <div class="rdx-stage small">
                    <svg class="rdx-arcs" aria-hidden="true"></svg>
                    <div class="rdx-line rdx-tm"></div>
                </div>
            </div>
        </div>
        <div class="rdx-defs" hidden>
            <div class="rdx-label">"Definitions " <span>"· δ unfolds these"</span></div>
            <div class="rdx-defs-list"></div>
        </div>
        <details class="rdx-trace" open>
            <summary>"Trace"</summary>
            <ol on:click=move |e| {
                let row = Doc::target(&e).and_then(|t| Doc::closest(&t, ".rdx-trace [data-i]"));
                if let Some(i) = row.and_then(|r| Doc::attr(&r, "data-i")).and_then(|i| i.parse::<usize>().ok()) && !b5.session.busy() {
                    b5.local.borrow_mut().note.clear();
                    b5.session.seek(i);
                }
            }></ol>
        </details>
        <details class="rdx-src">
            <summary>"Edit source"</summary>
            <textarea spellcheck="false" rows="5" aria-label="Program source"></textarea>
            <div class="rdx-src-row">
                <button type="button" class="rdx-btn primary" data-act="load" on:click=move |_| b6.act("load")>
                    "Load"
                </button>
                <p class="rdx-syntax">
                    <code>"λx. t"</code> " or " <code>"fun x => t"</code> " · " <code>"let x := v; t"</code> " · "
                    <code>"(a, b)"</code> " · " <code>"p.1"</code> " · " <code>"Nat.rec z s n"</code> " · numerals · "
                    <code>"def name := t"</code> " on its own line · " <code>"--"</code> " comments"
                </p>
            </div>
            <p class="rdx-err" hidden></p>
        </details>
    }
}
