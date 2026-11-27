use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

use examples::*;
use leptos::html;
use leptos::prelude::*;
use markup::*;
use motion::*;
use rules::*;
use strategy::*;
use syntax::*;
use term::*;
use units::*;
use web_sys::Element;

use crate::*;

/// One lane of the race: a strategy and what it is like.
#[derive(Debug, Clone, Copy)]
pub struct Lane {
    strategy: Strategy,
    sub: &'static str,
}

/// The strategy race: one start term, three rules for picking the next redex, side by side.
pub struct Track;

impl Lane {
    fn strategy(&self) -> Strategy {
        self.strategy
    }

    fn sub(&self) -> &'static str {
        self.sub
    }
}

impl Track {
    pub const CAP: Cap = Cap::new(24);
    pub const TICK: Ms = Ms::new(420.0);
    const FIRST: Ms = Ms::new(350.0);
    pub const LANES: [Lane; 3] = [
        Lane {
            strategy: Strategy::Normal,
            sub: "Leftmost-outermost. Reaches a normal form whenever one exists.",
        },
        Lane {
            strategy: Strategy::Applicative,
            sub: "Leftmost-innermost. Arguments first, like a strict language.",
        },
        Lane {
            strategy: Strategy::Whnf,
            sub: "What Lean’s whnf does. Stops once the head is a λ, a constructor or stuck.",
        },
    ];

    /// The start term and its definitions for a pick, if there is one yet.
    pub fn source(pick: Race, session: &Session) -> Option<(Term, term::Env)> {
        match pick.src() {
            None if !session.loaded() => None,
            None => Some((session.start(), session.env())),
            Some(src) => Program::parse(&src).ok().map(|p| (p.term().clone(), p.env().clone())),
        }
    }

    pub fn runs((term, env): &(Term, term::Env)) -> Vec<Run> {
        Self::LANES
            .iter()
            .map(|l| Run::new(term, env, l.strategy(), Self::CAP))
            .collect()
    }

    pub fn count(r: &Run) -> Node {
        let n = r.steps();
        match r.status() {
            Status::Done => Node::list([
                Node::text(n),
                Node::el(Tag::Small).kid(Node::text(if n == Step::ONE { "step" } else { "steps" })),
            ]),
            _ => Node::text("∞"),
        }
    }

    fn ending(r: &Run, env: &term::Env) -> (Class, String) {
        match r.status() {
            Status::Cap => (Class::Loops, format!("Still going after {} steps. It loops.", Self::CAP)),
            Status::Big => (Class::Loops, "Stopped: the term keeps growing.".to_string()),
            Status::Done => match r.last().redexes(env).iter().filter(|x| !x.kind().is_eta()).count() {
                0 => (Class::Nf, "Normal form".to_string()),
                left => (
                    Class::Part,
                    format!("Head exposed. {left} redex{} left inside.", if left > 1 { "es" } else { "" }),
                ),
            },
        }
    }

    pub fn pills(pick: Race) -> Node {
        Node::list(Race::ALL.iter().map(|r| {
            let on = *r == pick;
            Node::el(Tag::Button)
                .attr(Attr::Button)
                .class([Class::Pill].into_iter().chain(on.then_some(Class::On)))
                .attr(Attr::Race(*r))
                .attr(Attr::Pressed(on))
                .kid(Node::text(r.label()))
        }))
    }

    pub fn lanes(runs: &[Run], env: &term::Env) -> Node {
        Node::list(runs.iter().zip(Self::LANES).map(|(r, l)| Self::lane(r, l, env)))
    }

    /// A lane: its strategy and count over every term it passed through, then how it ended.
    fn lane(r: &Run, l: Lane, env: &term::Env) -> Node {
        let (cls, msg) = Self::ending(r, env);
        Node::el(Tag::Div)
            .class([Class::Lane])
            .kid(
                Node::el(Tag::Header)
                    .kid(
                        Node::el(Tag::Div)
                            .class([Class::LaneTop])
                            .kid(Node::el(Tag::B).kid(Node::text(l.strategy().label())))
                            .kid(Node::el(Tag::Span).class([Class::Count]).kid(Self::count(r))),
                    )
                    .kid(Node::el(Tag::P).kid(Node::text(l.sub()))),
            )
            .kid(
                Node::el(Tag::Ol)
                    .kids(r.rows().iter().enumerate().map(|(i, row)| Self::row(Step::new(i), row)))
                    .kid(
                        Node::el(Tag::Li)
                            .class([Class::End, cls])
                            .attr(Attr::I(Step::new(r.rows().len())))
                            .kid(Node::text(msg)),
                    ),
            )
    }

    /// One term of a lane, with the rule that produced it.
    fn row(i: Step, row: &Row) -> Node {
        let text = row.term().text();
        Node::el(Tag::Li).attr(Attr::I(i)).kid(Letter::flat(row.kind())).kid(
            Node::el(Tag::Code)
                .attr(Attr::Title(text.clone()))
                .kid(Node::text(text)),
        )
    }
}

/// The lanes as drawn, and the runs they show, for the ticks that reveal them.
struct Heat {
    lanes: Vec<Element>,
    runs: Vec<Run>,
}

impl Heat {
    fn lanes(&self) -> &[Element] {
        &self.lanes
    }

    fn runs(&self) -> &[Run] {
        &self.runs
    }

    /// The longest lane, in rows: the tick that reveals its end.
    fn last(&self) -> Step {
        Step::new(self.runs().iter().map(|r| r.rows().len()).max().unwrap_or(0))
    }
}

/// The race widget's own state.
struct Stand {
    session: Session,
    anchor: NodeRef<html::Div>,
    pick: Cell<Race>,
    drawn: Cell<bool>,
    timer: RefCell<Option<gloo_timers::callback::Timeout>>,
}

impl Stand {
    fn session(&self) -> &Session {
        &self.session
    }

    fn pick(&self) -> &Cell<Race> {
        &self.pick
    }

    fn drawn(&self) -> &Cell<bool> {
        &self.drawn
    }

    fn timer(&self) -> &RefCell<Option<gloo_timers::callback::Timeout>> {
        &self.timer
    }

    fn anchor(&self) -> NodeRef<html::Div> {
        self.anchor
    }

    fn q(&self, sel: Class) -> Option<Element> {
        Doc::above(self.anchor().get_untracked()).and_then(|r| Doc::q(&r, &sel.into()))
    }

    fn render(self: &Rc<Self>, animate: bool) {
        self.timer().borrow_mut().take();
        self.drawn().set(true);
        let pick = self.pick().get();
        if let Some(pills) = self.q(Class::Pills) {
            Doc::fill(&pills, &Track::pills(pick));
        }
        let Some(lanes) = self.q(Class::Lanes) else { return };
        let Some(src) = Track::source(pick, self.session()) else {
            Doc::fill(&lanes, &Node::none());
            return;
        };
        let runs = Track::runs(&src);
        Doc::fill(&lanes, &Track::lanes(&runs, &src.1));
        if !animate || !Motion::enabled() {
            return;
        }
        let heat = Heat {
            lanes: Doc::all(&lanes, &Class::Lane.into()),
            runs,
        };
        heat.lanes().iter().for_each(|l| {
            Doc::all(l, &Tag::Li.into())
                .iter()
                .filter(|li| Doc::read::<Step>(li, Data::I).is_some_and(|i| i > Step::START))
                .for_each(|li| Doc::add(li, &[Class::Pending]));
            if let Some(c) = Doc::q(l, &Class::Count.into()) {
                Doc::fill(&c, &Node::text(Step::START));
            }
        });
        self.schedule(Rc::new(heat), Step::ONE, Track::FIRST);
    }

    fn schedule(self: &Rc<Self>, heat: Rc<Heat>, i: Step, after: Ms) {
        let me = self.clone();
        *self.timer().borrow_mut() =
            Some(gloo_timers::callback::Timeout::new(after.whole(), move || me.tick(&heat, i)));
    }

    fn tick(self: &Rc<Self>, heat: &Rc<Heat>, i: Step) {
        heat.lanes().iter().zip(heat.runs()).for_each(|(l, run)| {
            if let Some(li) = Doc::q(l, &Query::from(Tag::Li).and(Query::is(Data::I, &i))) {
                Doc::remove(&li, &[Class::Pending]);
                leptos::task::spawn_local(
                    Motion::enter(&li, Tween::default().ms(Ms::new(300.0))).play(Clock::default()),
                );
                if let Some(ol) = Doc::q(l, &Tag::Ol.into()) {
                    ol.set_scroll_top(ol.scroll_height());
                }
            }
            if let Some(c) = Doc::q(l, &Class::Count.into()) {
                Doc::fill(&c, &if i <= run.steps() { Node::text(i) } else { Track::count(run) });
            }
        });
        if i < heat.last() {
            self.schedule(heat.clone(), i.next(), Track::TICK);
        }
    }

    fn click(self: &Rc<Self>, e: &web_sys::Event) {
        let Some(target) = Doc::target(e) else { return };
        if let Some(race) =
            Doc::closest(&target, &Query::Has(Data::Race)).and_then(|p| Doc::read::<Race>(&p, Data::Race))
        {
            self.pick().set(race);
            self.render(true);
        }
        if Doc::closest(&target, &Query::is(Data::Act, &Act::Race)).is_some() {
            self.render(true);
        }
    }
}

#[component]
pub fn Lanes() -> impl IntoView {
    let stand = Rc::new(Stand {
        session: expect_context::<StoredValue<Session, LocalStorage>>().get_value(),
        anchor: NodeRef::new(),
        pick: Cell::new(Race::find("dup").expect("the dup race exists")),
        drawn: Cell::new(false),
        timer: RefCell::new(None),
    });
    let me = stand.clone();
    stand.session().on(move |e| {
        if !me.drawn().get() || (e == Event::Load && me.pick().get().src().is_none()) {
            me.render(false);
        }
    });
    let late = stand.clone();
    stand.anchor().on_load(move |_| {
        if !late.drawn().get() {
            late.render(false);
        }
    });
    let (b1, b2) = (stand.clone(), stand.clone());
    view! {
        <div class=Class::RaceBar.to_string() node_ref=stand.anchor() on:click=move |e| b1.click(&e)>
            <div class=Class::Pills.to_string()></div>
            <button type="button" class=Attr::Class(vec![Class::Btn, Class::Primary]).to_string() data-act=Act::Race.to_string()>
                "Race"
            </button>
        </div>
        <div class=Class::Lanes.to_string() on:click=move |e| b2.click(&e)></div>
    }
}
