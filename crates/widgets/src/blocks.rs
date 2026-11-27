use markup::*;
use rules::*;
use term::*;

/// How many boxes enclose a node: boxes alternate their background by it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Level(usize);

impl Level {
    pub fn inner(self) -> Self {
        Self(self.0 + 1)
    }

    pub fn parity(self) -> Class {
        if self.0 % 2 == 1 { Class::Odd } else { Class::Even }
    }
}

/// An enclosing binder: its name, its anchor and its hue.
#[derive(Debug, Clone)]
struct Scope {
    x: Name,
    anchor: Anchor,
    hue: Hue,
}

/// One node's drawing in progress: its classes and attributes, written in the JS order.
struct Draft {
    classes: Vec<Class>,
    attrs: Vec<Attr>,
}

/// The nested-block view of a term. Each node is one element carrying its path and uid: λ and let
/// are boxes with a binder tab tinted by the binder's hue; an application spine is one box with
/// the head first; a pair is a box; a variable is a tag in its binder's hue (free ones dashed); a
/// constant is a bold word. With `live`, a redex gets a rule badge on its corner.
pub struct Blocks<'a> {
    env: &'a Env,
    live: bool,
    db: bool,
}

impl<'a> Blocks<'a> {
    pub fn new(env: &'a Env, live: bool, db: bool) -> Self {
        Self { env, live, db }
    }

    pub fn node(&self, t: &Term) -> Node {
        let mut last = Anchor::NONE;
        self.draw(t, &Path::root(), &[], Level(0), false, &mut last)
    }

    /// `spine` marks the function part of an application.
    fn draw(&self, t: &Term, p: &Path, ctx: &[Scope], d: Level, spine: bool, last: &mut Anchor) -> Node {
        let kind = if self.live { t.redex(self.env) } else { None };
        let mut draft = Draft {
            classes: vec![Class::Block],
            attrs: [Attr::Path(p.clone()), Attr::Uid(t.uid())]
                .into_iter()
                .chain(t.from().map(Attr::From))
                .collect(),
        };
        let (inner, boxed) = match (t.nat(), t.shape()) {
            (Some(n), _) => {
                draft.classes.push(Class::Lit);
                (Node::text(n), false)
            }
            (None, Shape::Var(n)) => (self.var(n, ctx, &mut draft), false),
            (None, Shape::Const(n)) => {
                draft
                    .classes
                    .extend([Class::Const].into_iter().chain(self.env.has(n).then_some(Class::Def)));
                (Node::text(n), false)
            }
            (None, Shape::Lam(..) | Shape::Let(..)) => (self.binder(t, p, ctx, d, &mut draft, last), true),
            (None, Shape::App(f, a)) => {
                let k = if spine { d } else { d.inner() };
                draft.classes.push(if spine { Class::Group } else { Class::App });
                (
                    Node::list([
                        self.draw(f, &p.child('f'), ctx, k, true, last),
                        self.draw(a, &p.child('a'), ctx, k, false, last),
                    ]),
                    !spine,
                )
            }
            (None, Shape::Pair(a, b)) => {
                draft.classes.push(Class::Pair);
                let first = self.draw(a, &p.child('a'), ctx, d.inner(), false, last);
                (
                    Node::list([
                        first,
                        Node::el(Tag::Span).class([Class::Comma]).kid(Node::text(",")),
                        self.draw(b, &p.child('b'), ctx, d.inner(), false, last),
                    ]),
                    true,
                )
            }
            (None, Shape::Proj(i, s)) => {
                draft.classes.push(Class::Proj);
                (
                    Node::list([
                        self.draw(s, &p.child('t'), ctx, d, false, last),
                        Node::el(Tag::Span)
                            .class([Class::Field])
                            .kid(Node::text(format!(".{i}"))),
                    ]),
                    false,
                )
            }
        };
        let Draft { mut classes, mut attrs } = draft;
        classes.extend(boxed.then(|| d.parity()));
        classes.extend(kind.map(|k| [Class::Rr, Class::Ink(k)]).into_iter().flatten());
        attrs.extend(kind.map(Attr::Kind));
        classes.extend(
            t.mark()
                .filter(|m| *m != Mark::Binder)
                .map(|m| [Class::Flash, Class::Flashed(m)])
                .into_iter()
                .flatten(),
        );
        classes.extend(t.cap().then_some(Class::Cap));
        let badge = kind.map(|k| {
            Node::el(Tag::Button)
                .attr(Attr::Button)
                .class([Class::Badge, Class::Ink(k)])
                .attr(Attr::Label(format!("Fire {k}")))
                .kid(Node::text(k.glyph()))
        });
        let el = Node::el(Tag::Span).class(classes);
        attrs.into_iter().fold(el, Node::attr).kids(badge).kid(inner)
    }

    fn var(&self, n: &Name, ctx: &[Scope], draft: &mut Draft) -> Node {
        match ctx.iter().rposition(|s| s.x == *n) {
            None => {
                draft.classes.extend([Class::Bound, Class::Free]);
                Node::text(n)
            }
            Some(j) => {
                draft.classes.extend(
                    [Class::Bound, Class::Hue(ctx[j].hue)]
                        .into_iter()
                        .chain(self.db.then_some(Class::Index)),
                );
                draft.attrs.push(Attr::Bid(ctx[j].anchor));
                if self.db { Node::text(ctx.len() - 1 - j) } else { Node::text(n) }
            }
        }
    }

    fn binder(&self, t: &Term, p: &Path, ctx: &[Scope], d: Level, draft: &mut Draft, last: &mut Anchor) -> Node {
        *last = last.next();
        let (anchor, x) = (*last, t.binder().cloned().unwrap_or_else(|| Name::from("")));
        let hue = Hue::of(&x);
        draft.classes.extend([
            if matches!(t.shape(), Shape::Lam(..)) { Class::Lam } else { Class::Let },
            Class::Hue(hue),
        ]);
        draft.attrs.push(Attr::Binder(anchor));
        let flashed = if t.mark() == Some(Mark::Binder) {
            vec![Class::Flash, Class::Flashed(Mark::Renamed)]
        } else {
            vec![]
        };
        let name = match (self.db, self.live) {
            (true, _) => Node::none(),
            (false, true) => Node::el(Tag::Button)
                .attr(Attr::Button)
                .class([Class::Binder].into_iter().chain(flashed))
                .attr(Attr::Path(p.clone()))
                .attr(Attr::Anchor(anchor))
                .attr(Attr::Title("Rename (α)".to_string()))
                .kid(Node::text(&x)),
            (false, false) => Node::el(Tag::Span)
                .class([Class::Binder])
                .attr(Attr::Anchor(anchor))
                .kid(Node::text(&x)),
        };
        let kw = |s: &str| {
            let k = Node::el(Tag::Span).class([Class::Keyword]);
            (if self.db { k.attr(Attr::Anchor(anchor)) } else { k }).kid(Node::text(s))
        };
        let inside = ctx
            .iter()
            .cloned()
            .chain([Scope { x, anchor, hue }])
            .collect::<Vec<Scope>>();
        let tab = Node::el(Tag::Span).class([Class::Tab]);
        match t.shape() {
            Shape::Let(_, v, b) => {
                let value = self.draw(v, &p.child('v'), ctx, d.inner(), false, last);
                Node::list([
                    tab.kid(kw("let"))
                        .kid(name)
                        .kid(Node::el(Tag::Span).class([Class::Keyword]).kid(Node::text(":=")))
                        .kid(value),
                    self.draw(b, &p.child('b'), &inside, d.inner(), false, last),
                ])
            }
            Shape::Lam(_, b) => Node::list([
                tab.kid(kw("λ")).kid(name),
                self.draw(b, &p.child('b'), &inside, d.inner(), false, last),
            ]),
            _ => Node::none(),
        }
    }
}
