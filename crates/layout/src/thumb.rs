use explore::*;
use markup::*;
use strategy::*;
use term::*;
use units::*;

/// A deck tile's thumbnail of an example's reduction graph, in the graph widget's grammar, and the
/// line that says how the example ends. Coordinates print to tenths, as the fixtures record them.
#[derive(Debug, Clone)]
pub struct Thumb {
    svg: Node,
    fate: Node,
}

impl Thumb {
    /// The thumbnail box, rows shown, mark radius.
    const TW: Px = Px::new(120.0);
    const TH: Px = Px::new(46.0);
    const PAD: Px = Px::new(6.0);
    const ROWS: Depth = Depth::new(4);
    const MR: Px = Px::new(3.0);
    const CAP: Cap = Cap::new(40);
    const RUN: Cap = Cap::new(60);

    pub fn new(term: &Term, env: &Env, eta: bool) -> Self {
        let g = Graph::explore(term, env, eta, Self::CAP);
        let at = Self::place(&g);
        Self {
            svg: Node::el(Tag::Svg)
                .class([Class::Thumb])
                .attr(Attr::ViewBox(Num::Js(Px::ZERO), Num::Js(Px::ZERO), Num::Js(Self::TW), Num::Js(Self::TH)))
                .attr(Attr::Hidden)
                .kids(
                    g.edges()
                        .iter()
                        .filter(|e| g.vertex(e.from()).depth() <= Self::ROWS)
                        .map(|e| Self::edge(&g, e, &at)),
                )
                .kids(
                    g.vertices()
                        .filter(|(_, v)| v.depth() <= Self::ROWS)
                        .map(|(i, v)| Self::mark(i, v, at[i.get()], env)),
                ),
            fate: Self::ending(term, env, g.cyclic()),
        }
    }

    fn ending(term: &Term, env: &Env, cyclic: bool) -> Node {
        let run = Run::new(term, env, Strategy::Normal, Self::RUN);
        let n = run.steps();
        let steps = format!("{n} step{}", if n == Step::ONE { "" } else { "s" });
        let nf = [
            Node::el(Tag::I).class([Class::Nfmark]).attr(Attr::Hidden),
            Node::el(Tag::Span).class([Class::Sr]).kid(Node::text("normal form")),
        ];
        match (run.status(), cyclic) {
            (Status::Done, true) => Node::list(nf).kid(Node::text(format!(" in {steps}, or loops"))),
            (Status::Done, false) => Node::list(nf).kid(Node::text(format!(" in {steps}"))),
            (_, true) => Node::text("loops forever"),
            (_, false) => Node::text("keeps growing"),
        }
    }

    /// Each shown vertex's position; vertices past the shown rows stay at the origin.
    fn place(g: &Graph) -> Vec<(Px, Px)> {
        let rows =
            g.vertices()
                .filter(|(_, v)| v.depth() <= Self::ROWS)
                .fold(Vec::new(), |mut rows: Vec<Vec<Id>>, (i, v)| {
                    rows.resize_with(rows.len().max(v.depth().get() + 1), Vec::new);
                    rows[v.depth().get()].push(i);
                    rows
                });
        let depth = rows.len() as f64 - 1.0;
        let wide = rows.iter().map(Vec::len).max().unwrap_or(0) as f64;
        let dy = if depth > 0.0 {
            Px::new(16.0).min((Self::TH - Self::PAD * 2.0) / depth)
        } else {
            Px::ZERO
        };
        let dx = if wide > 1.0 {
            Px::new(22.0).min((Self::TW - Self::PAD * 2.0 - Px::new(14.0)) / (wide - 1.0))
        } else {
            Px::ZERO
        };
        let mut at = vec![(Px::ZERO, Px::ZERO); g.len()];
        for (d, row) in rows.iter().enumerate() {
            let mut keyed = row
                .iter()
                .map(|&v| (if d > 0 { Self::center(g, v, &at) } else { Px::ZERO }, v))
                .collect::<Vec<(Px, Id)>>();
            keyed.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            keyed.iter().enumerate().for_each(|(i, &(_, v))| {
                at[v.get()] = (
                    (Self::TW / 2.0 - Px::new(7.0) - (dx * (keyed.len() as f64 - 1.0)) / 2.0) + dx * i as f64,
                    ((Self::TH - dy * depth) / 2.0) + dy * d as f64,
                );
            });
        }
        at
    }

    /// The mean x of a vertex's parents one row up.
    fn center(g: &Graph, v: Id, at: &[(Px, Px)]) -> Px {
        let ps = g
            .edges()
            .iter()
            .filter(|e| e.to() == v && g.vertex(v).depth().below(g.vertex(e.from()).depth()))
            .map(|e| at[e.from().get()].0)
            .collect::<Vec<Px>>();
        if ps.is_empty() {
            Px::ZERO
        } else {
            ps.iter().fold(Px::ZERO, |s, x| s + *x) / ps.len() as f64
        }
    }

    fn edge(g: &Graph, e: &Edge, at: &[(Px, Px)]) -> Node {
        let f = Num::Tenths;
        let px = Px::new;
        let ((ax, ay), (bx, by), mr) = (at[e.from().get()], at[e.to().get()], Self::MR);
        let (da, db) = (g.vertex(e.from()).depth(), g.vertex(e.to()).depth());
        let (segs, cut) = match () {
            () if db > Self::ROWS => (vec![Seg::Move(f(ax), f(ay + mr)), Seg::Line(f(ax), f(ay + mr + px(6.0)))], true),
            () if e.from() == e.to() => (
                vec![
                    Seg::Move(f(ax + px(2.0)), f(ay - px(2.0))),
                    Seg::Cubic(
                        f(ax + px(9.0)),
                        f(ay - px(13.0)),
                        f(ax + px(16.0)),
                        f(ay - px(2.0)),
                        f(ax + mr + px(1.0)),
                        f(ay + px(1.0)),
                    ),
                ],
                false,
            ),
            () if db.below(da) => (vec![Seg::Move(f(ax), f(ay + mr)), Seg::Line(f(bx), f(by - mr))], false),
            () if db == da => (
                vec![
                    Seg::Move(f(ax), f(ay - mr)),
                    Seg::Quad(f(ax.midpoint(bx)), f(ay - (px(5.0) + (bx - ax).abs() * 0.25)), f(bx), f(by - mr)),
                ],
                false,
            ),
            () => {
                let sx = ax.max(bx) + px(10.0) + (ay - by).abs() * 0.3;
                (
                    vec![
                        Seg::Move(f(ax + mr), f(ay)),
                        Seg::Cubic(f(sx), f(ay), f(sx), f(by), f(bx + mr), f(by)),
                    ],
                    false,
                )
            }
        };
        Node::el(Tag::Path)
            .class(
                [Class::Edge, Class::Ink(e.kind())]
                    .into_iter()
                    .chain(cut.then_some(Class::Cut)),
            )
            .attr(Attr::D(Trace::spaced(segs)))
    }

    fn mark(i: Id, v: &Vertex, (x, y): (Px, Px), env: &Env) -> Node {
        let f = Num::Tenths;
        let mr = Self::MR;
        let open = v.open().then_some(Class::Open);
        Node::list(
            (i == Id::ROOT)
                .then(|| {
                    Node::el(Tag::Circle)
                        .class([Class::Ring])
                        .attr(Attr::Cx(f(x)))
                        .attr(Attr::Cy(f(y)))
                        .attr(Attr::R(Num::Js(mr + Px::new(2.4))))
                })
                .into_iter()
                .chain([match (v.nf(), Strategy::Whnf.next(v.term(), env)) {
                    (true, _) => Node::el(Tag::Rect)
                        .class([Class::Mark, Class::Nf])
                        .attr(Attr::X(f(x - mr)))
                        .attr(Attr::Y(f(y - mr)))
                        .attr(Attr::Width(Num::Js(mr * 2.0)))
                        .attr(Attr::Height(Num::Js(mr * 2.0))),
                    (false, None) => {
                        let r = mr + Px::new(0.8);
                        Node::el(Tag::Path)
                            .class([Class::Mark, Class::Whnf].into_iter().chain(open))
                            .attr(Attr::D(Trace::tight(vec![
                                Seg::Move(f(x), f(y - r)),
                                Seg::Line(f(x + r), f(y)),
                                Seg::Line(f(x), f(y + r)),
                                Seg::Line(f(x - r), f(y)),
                                Seg::Close,
                            ])))
                    }
                    (false, Some(_)) => Node::el(Tag::Circle)
                        .class([Class::Mark, Class::Red].into_iter().chain(open))
                        .attr(Attr::Cx(f(x)))
                        .attr(Attr::Cy(f(y)))
                        .attr(Attr::R(Num::Js(mr))),
                }]),
        )
    }

    pub fn svg(&self) -> &Node {
        &self.svg
    }

    pub fn fate(&self) -> &Node {
        &self.fate
    }
}
