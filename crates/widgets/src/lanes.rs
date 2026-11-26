use std::cell::RefCell;
use std::rc::Rc;

use leptos::html;
use leptos::prelude::*;
use motion::*;
use rules::*;
use strategy::*;
use syntax::*;
use term::*;
use web_sys::Element;

use crate::*;

/// One lane of the race: a strategy, its name and what it is like.
#[derive(Debug, Clone, Copy)]
pub struct Lane {
    name: &'static str,
    sub: &'static str,
    strategy: Strategy,
}

/// The strategy race: one start term, three rules for picking the next redex, side by side.
pub struct Track;

impl Track {
    pub const CAP: usize = 24;
    pub const TICK: u32 = 420;
    pub const LANES: [Lane; 3] = [
        Lane {
            name: "Normal order",
            sub: "Leftmost-outermost. Reaches a normal form whenever one exists.",
            strategy: Strategy::Normal,
        },
        Lane {
            name: "Applicative order",
            sub: "Leftmost-innermost. Arguments first, like a strict language.",
            strategy: Strategy::Applicative,
        },
        Lane {
            name: "Head only",
            sub: "What Lean’s whnf does. Stops once the head is a λ, a constructor or stuck.",
            strategy: Strategy::Whnf,
        },
    ];

    /// The start term and its definitions for a pick, if there is one yet.
    pub fn source(pick: &str, session: &Session) -> Option<(Term, term::Env)> {
        match examples::Race::find(pick).and_then(|r| r.src()) {
            None if session.state().hist.is_empty() => None,
            None => Some((session.start(), session.env())),
            Some(src) => Program::parse(src).ok().map(|p| (p.term().clone(), p.env().clone())),
        }
    }

    pub fn runs(src: &(Term, term::Env)) -> Vec<Run> {
        Self::LANES
            .iter()
            .map(|l| Run::new(&src.0, &src.1, l.strategy, Self::CAP))
            .collect()
    }

    pub fn count(r: &Run) -> String {
        let n = r.steps();
        match r.status() {
            Status::Done => format!("{n}<small>step{}</small>", if n == 1 { "" } else { "s" }),
            _ => "∞".to_string(),
        }
    }

    fn ending(r: &Run, env: &term::Env) -> (&'static str, String) {
        match r.status() {
            Status::Cap => ("loops", format!("Still going after {} steps. It loops.", Self::CAP)),
            Status::Big => ("loops", "Stopped: the term keeps growing.".to_string()),
            Status::Done => match r.last().redexes(env).iter().filter(|x| !x.kind().is_eta()).count() {
                0 => ("nf", "Normal form".to_string()),
                left => {
                    ("part", format!("Head exposed. {left} redex{} left inside.", if left > 1 { "es" } else { "" }))
                }
            },
        }
    }

    pub fn pills(pick: &str) -> String {
        examples::Race::ALL
            .iter()
            .map(|r| {
                let on = r.id() == pick;
                format!(
                    "<button type=\"button\" class=\"rdx-pill{}\" data-race=\"{}\" aria-pressed=\"{on}\">{}</button>",
                    if on { " on" } else { "" },
                    r.id(),
                    Html::esc(r.label())
                )
            })
            .collect::<Vec<String>>()
            .concat()
    }

    pub fn lanes(runs: &[Run], env: &term::Env) -> String {
        runs.iter()
            .zip(Self::LANES)
            .map(|(r, l)| {
                let (cls, msg) = Self::ending(r, env);
                let rows = r
                    .rows()
                    .iter()
                    .enumerate()
                    .map(|(i, row)| {
                        let text = Html::esc(&row.term().text());
                        format!("<li data-i=\"{i}\">{}<code title=\"{text}\">{text}</code></li>", Doc::flat(row.kind()))
                    })
                    .collect::<Vec<String>>()
                    .concat();
                format!(
                    "<div class=\"rdx-lane\"><header><div class=\"rdx-lane-top\"><b>{}</b><span class=\"rdx-count\">{}</span></div><p>{}</p></header><ol>{rows}<li class=\"rdx-end {cls}\" data-i=\"{}\">{msg}</li></ol></div>",
                    l.name,
                    Self::count(r),
                    l.sub,
                    r.rows().len()
                )
            })
            .collect::<Vec<String>>()
            .concat()
    }
}

/// The race widget's own state.
struct Stand {
    session: Session,
    anchor: NodeRef<html::Div>,
    pick: RefCell<String>,
    timer: RefCell<Option<gloo_timers::callback::Timeout>>,
}

impl Stand {
    fn q(&self, sel: &str) -> Option<Element> {
        self.anchor
            .get_untracked()
            .and_then(|a| a.parent_element())
            .and_then(|r| Doc::q(&r, sel))
    }

    fn render(self: &Rc<Self>, animate: bool) {
        self.timer.borrow_mut().take();
        let pick = self.pick.borrow().clone();
        if let Some(pills) = self.q(".rdx-pills") {
            pills.set_inner_html(&Track::pills(&pick));
        }
        let Some(lanes) = self.q(".rdx-lanes") else { return };
        let Some(src) = Track::source(&pick, &self.session) else {
            lanes.set_inner_html("");
            return;
        };
        let runs = Track::runs(&src);
        lanes.set_inner_html(&Track::lanes(&runs, &src.1));
        if !animate || !Motion::enabled() {
            return;
        }
        let els = Doc::all(&lanes, ".rdx-lane");
        let max = runs.iter().map(|r| r.rows().len()).max().unwrap_or(0);
        els.iter().for_each(|l| {
            Doc::all(l, "li")
                .iter()
                .filter(|li| {
                    Doc::attr(li, "data-i")
                        .and_then(|i| i.parse::<usize>().ok())
                        .unwrap_or(0)
                        > 0
                })
                .for_each(|li| Doc::add(li, &["pending"]));
            if let Some(c) = Doc::q(l, ".rdx-count") {
                c.set_text_content(Some("0"));
            }
        });
        self.schedule(Rc::new((els, runs)), 1, max, 350);
    }

    fn schedule(self: &Rc<Self>, race: Rc<(Vec<Element>, Vec<Run>)>, i: usize, max: usize, ms: u32) {
        let me = self.clone();
        let timeout = gloo_timers::callback::Timeout::new(ms, move || me.tick(&race, i, max));
        *self.timer.borrow_mut() = Some(timeout);
    }

    fn tick(self: &Rc<Self>, race: &Rc<(Vec<Element>, Vec<Run>)>, i: usize, max: usize) {
        race.0.iter().zip(&race.1).for_each(|(l, run)| {
            if let Some(li) = Doc::q(l, &format!("li[data-i=\"{i}\"]")) {
                Doc::remove(&li, &["pending"]);
                leptos::task::spawn_local(Motion::enter(&li, Tween::default().ms(300.0)).play(Clock::default()));
                if let Some(ol) = Doc::q(l, "ol") {
                    ol.set_scroll_top(ol.scroll_height());
                }
            }
            if let Some(c) = Doc::q(l, ".rdx-count") {
                c.set_inner_html(&if i <= run.steps() { i.to_string() } else { Track::count(run) });
            }
        });
        if i < max {
            self.schedule(race.clone(), i + 1, max, Track::TICK);
        }
    }

    fn click(self: &Rc<Self>, e: &web_sys::Event) {
        let Some(target) = Doc::target(e) else { return };
        if let Some(id) = Doc::closest(&target, "[data-race]").and_then(|p| Doc::attr(&p, "data-race")) {
            *self.pick.borrow_mut() = id;
            self.render(true);
        }
        if Doc::closest(&target, "[data-act=\"race\"]").is_some() {
            self.render(true);
        }
    }
}

#[component]
pub fn Lanes() -> impl IntoView {
    let session = expect_context::<StoredValue<Session, LocalStorage>>().get_value();
    let stand = Rc::new(Stand {
        session,
        anchor: NodeRef::new(),
        pick: RefCell::new("dup".to_string()),
        timer: RefCell::new(None),
    });
    let me = stand.clone();
    stand.session.on(move |e| {
        if e == Event::Load && *me.pick.borrow() == "bench" {
            me.render(false);
        }
    });
    let initial = Track::source("dup", &stand.session)
        .map(|src| Track::lanes(&Track::runs(&src), &src.1))
        .unwrap_or_default();
    let (b1, b2) = (stand.clone(), stand.clone());
    view! {
        <div class="rdx-race-bar" node_ref=stand.anchor on:click=move |e| b1.click(&e)>
            <div class="rdx-pills" inner_html=Track::pills("dup")></div>
            <button type="button" class="rdx-btn primary" data-act="race">
                "Race"
            </button>
        </div>
        <div class="rdx-lanes" inner_html=initial on:click=move |e| b2.click(&e)></div>
    }
}
