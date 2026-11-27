use explore::*;
use markup::*;
use term::*;
use units::*;

use crate::*;

/// The graph widget's geometry. Rows are BFS depth; within a row, vertices sort by the mean x of
/// their parents one row up. Edges between rows point down, edges within a row arc above it, and
/// edges back to an earlier row swing out to the right. Every expression keeps the JS order of
/// operations, since the numbers must print the same to the last digit.
#[derive(Debug, Clone)]
pub struct Plot {
    width: Px,
    height: Px,
    rows: usize,
    spots: Vec<Spot>,
    routes: Vec<Trace>,
}

impl Plot {
    /// Column pitch, row pitch, mark radius.
    pub const GX: Px = Px::new(46.0);
    pub const ROWH: Px = Px::new(58.0);
    pub const R: Px = Px::new(6.0);
    /// Margins: the axis on the left, loop-backs on the right.
    pub const ML: Px = Px::new(54.0);
    pub const MT: Px = Px::new(30.0);
    pub const MR: Px = Px::new(96.0);

    pub fn new(g: &Graph, env: &Env) -> Self {
        let rows = Self::layers(g);
        let span = Self::GX
            * rows
                .iter()
                .map(|r| r.len() as f64 - 1.0)
                .fold(f64::NEG_INFINITY, f64::max);
        let mut spots = vec![Spot::new(Px::ZERO, Px::ZERO, Form::Red); g.len()];
        for (d, row) in rows.iter().enumerate() {
            let row = Self::order(g, row, Depth::new(d), &spots);
            row.iter().enumerate().for_each(|(i, &v)| {
                spots[v.get()] = Spot::new(
                    (Self::ML + (span - Self::GX * (row.len() as f64 - 1.0)) / 2.0) + Self::GX * i as f64,
                    Self::row(Depth::new(d)),
                    Form::of(g.vertex(v), env),
                );
            });
        }
        Self {
            width: Self::ML + span + Self::MR,
            height: Self::MT * 2.0 + Self::ROWH * (rows.len() as f64 - 1.0),
            rows: rows.len(),
            routes: Self::paths(g, &spots),
            spots,
        }
    }

    /// The y of a row.
    pub fn row(d: Depth) -> Px {
        Self::MT + Self::ROWH * d.get() as f64
    }

    /// Vertex ids by depth, in exploration order.
    fn layers(g: &Graph) -> Vec<Vec<Id>> {
        g.vertices().fold(Vec::new(), |mut rows: Vec<Vec<Id>>, (i, v)| {
            rows.resize_with(rows.len().max(v.depth().get() + 1), Vec::new);
            rows[v.depth().get()].push(i);
            rows
        })
    }

    /// A row sorted by the mean x of each vertex's parents one row up (a stable sort).
    fn order(g: &Graph, row: &[Id], d: Depth, spots: &[Spot]) -> Vec<Id> {
        if d == Depth::START {
            return row.to_vec();
        }
        let mut keyed = row
            .iter()
            .map(|&v| (Self::center(g, v, spots), v))
            .collect::<Vec<(Px, Id)>>();
        keyed.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        keyed.into_iter().map(|(_, v)| v).collect()
    }

    /// The mean x of a vertex's parents one row up, in edge order; zero when it has none.
    fn center(g: &Graph, v: Id, spots: &[Spot]) -> Px {
        let ps = g
            .edges()
            .iter()
            .filter(|e| e.to() == v && g.vertex(v).depth().below(g.vertex(e.from()).depth()))
            .map(|e| spots[e.from().get()].x())
            .collect::<Vec<Px>>();
        if ps.is_empty() {
            Px::ZERO
        } else {
            ps.iter().fold(Px::ZERO, |s, x| s + *x) / ps.len() as f64
        }
    }

    fn paths(g: &Graph, spots: &[Spot]) -> Vec<Trace> {
        g.edges()
            .iter()
            .enumerate()
            .map(|(i, e)| {
                Self::route(
                    &spots[e.from().get()],
                    &spots[e.to().get()],
                    e.from() == e.to(),
                    (g.vertex(e.from()).depth(), g.vertex(e.to()).depth()),
                    Px::new(
                        (g.edges()[..i].iter().filter(|o| o.joins(e.from(), e.to())).count() as f64
                            - (g.edges().iter().filter(|o| o.joins(e.from(), e.to())).count() as f64 - 1.0) / 2.0)
                            * 12.0,
                    ),
                )
            })
            .collect()
    }

    fn route(a: &Spot, b: &Spot, same: bool, (da, db): (Depth, Depth), off: Px) -> Trace {
        let n = Num::Js;
        let (ax, ay, bx, by, r) = (a.x(), a.y(), b.x(), b.y(), Self::R);
        let px = Px::new;
        Trace::spaced(match () {
            () if same => vec![
                Seg::Move(n(ax + px(4.0)), n(ay - px(5.0))),
                Seg::Cubic(
                    n(ax + px(20.0)),
                    n(ay - px(34.0)),
                    n(ax + px(40.0)),
                    n(ay - px(10.0)),
                    n(ax + r + px(3.0)),
                    n(ay + px(1.0)),
                ),
            ],
            () if db.below(da) => {
                let (y1, y2) = (ay + r + px(1.0), by - r - px(3.0));
                let c = (y2 - y1) * 0.5;
                vec![
                    Seg::Move(n(ax), n(y1)),
                    Seg::Cubic(n(ax + off), n(y1 + c), n(bx + off), n(y2 - c), n(bx), n(y2)),
                ]
            }
            () if db == da => {
                let lift = px(16.0) + (bx - ax).abs() * 0.18 + off;
                let dir = (bx - ax).sign();
                vec![
                    Seg::Move(n(ax + px(dir * 3.0)), n(ay - r)),
                    Seg::Cubic(
                        n(ax + px(dir * 6.0)),
                        n(ay - lift),
                        n(bx - px(dir * 6.0)),
                        n(by - lift),
                        n(bx - px(dir * 2.0)),
                        n(by - r - px(3.0)),
                    ),
                ]
            }
            () => {
                let sx = ax.max(bx) + px(34.0) + (ay - by).abs() * 0.12 + off;
                vec![
                    Seg::Move(n(ax + r + px(1.0)), n(ay)),
                    Seg::Cubic(n(sx), n(ay), n(sx), n(by), n(bx + r + px(3.0)), n(by)),
                ]
            }
        })
    }

    fn count(&self) -> usize {
        self.rows
    }

    fn spots(&self) -> &[Spot] {
        &self.spots
    }

    pub fn width(&self) -> Px {
        self.width
    }

    pub fn height(&self) -> Px {
        self.height
    }

    /// Every row, top to bottom.
    pub fn rows(&self) -> impl Iterator<Item = Depth> {
        (0..self.count()).map(Depth::new)
    }

    pub fn spot(&self, v: Id) -> &Spot {
        &self.spots()[v.get()]
    }

    /// The path of each edge, in edge order.
    pub fn routes(&self) -> &[Trace] {
        &self.routes
    }
}
