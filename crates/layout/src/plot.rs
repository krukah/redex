use explore::*;
use term::*;

use crate::*;

/// The graph widget's geometry. Rows are BFS depth; within a row, vertices sort by the mean x of
/// their parents one row up. Edges between rows point down, edges within a row arc above it, and
/// edges back to an earlier row swing out to the right. Numbers print as JavaScript prints them.
#[derive(Debug, Clone)]
pub struct Plot {
    width: f64,
    height: f64,
    rows: usize,
    spots: Vec<Spot>,
    routes: Vec<String>,
}

impl Plot {
    /// Column pitch, row pitch, mark radius.
    pub const GX: f64 = 46.0;
    pub const ROWH: f64 = 58.0;
    pub const R: f64 = 6.0;
    /// Margins: the axis on the left, loop-backs on the right.
    pub const ML: f64 = 54.0;
    pub const MT: f64 = 30.0;
    pub const MR: f64 = 96.0;

    pub fn new(g: &Graph, env: &Env) -> Self {
        let rows = Self::layers(g);
        let span = rows
            .iter()
            .map(|r| r.len() as f64 - 1.0)
            .fold(f64::NEG_INFINITY, f64::max)
            * Self::GX;
        let mut xs = vec![0.0; g.vertices().len()];
        let mut spots = vec![Spot::new(0.0, 0.0, Form::Red); g.vertices().len()];
        for (d, row) in rows.iter().enumerate() {
            let row = Self::order(g, row, d, &xs);
            let x0 = Self::ML + (span - (row.len() as f64 - 1.0) * Self::GX) / 2.0;
            for (i, &v) in row.iter().enumerate() {
                xs[v] = x0 + i as f64 * Self::GX;
                spots[v] = Spot::new(xs[v], Self::MT + d as f64 * Self::ROWH, Form::of(&g.vertices()[v], env));
            }
        }
        let routes = Self::paths(g, &spots);
        Self {
            width: Self::ML + span + Self::MR,
            height: Self::MT * 2.0 + (rows.len() as f64 - 1.0) * Self::ROWH,
            rows: rows.len(),
            spots,
            routes,
        }
    }

    /// Vertex indices by depth, in insertion order.
    fn layers(g: &Graph) -> Vec<Vec<usize>> {
        g.vertices()
            .iter()
            .enumerate()
            .fold(Vec::new(), |mut rows: Vec<Vec<usize>>, (i, v)| {
                rows.resize_with(rows.len().max(v.depth() + 1), Vec::new);
                rows[v.depth()].push(i);
                rows
            })
    }

    /// A row sorted by the mean x of each vertex's parents one row up (a stable sort).
    fn order(g: &Graph, row: &[usize], d: usize, xs: &[f64]) -> Vec<usize> {
        if d == 0 {
            return row.to_vec();
        }
        let bc = |v: usize| {
            let ps = g
                .edges()
                .iter()
                .filter(|e| e.to() == v && g.vertices()[v].depth() == g.vertices()[e.from()].depth() + 1)
                .map(|e| xs[e.from()])
                .collect::<Vec<f64>>();
            if ps.is_empty() {
                0.0
            } else {
                ps.iter().fold(0.0, |s, x| s + x) / ps.len() as f64
            }
        };
        let mut keyed = row.iter().map(|&v| (bc(v), v)).collect::<Vec<(f64, usize)>>();
        keyed.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        keyed.into_iter().map(|(_, v)| v).collect()
    }

    fn paths(g: &Graph, spots: &[Spot]) -> Vec<String> {
        let count = |a: usize, b: usize| g.edges().iter().filter(|e| e.from() == a && e.to() == b).count();
        g.edges()
            .iter()
            .enumerate()
            .map(|(i, e)| {
                let pi = g.edges()[..i]
                    .iter()
                    .filter(|o| o.from() == e.from() && o.to() == e.to())
                    .count();
                let off = (pi as f64 - (count(e.from(), e.to()) as f64 - 1.0) / 2.0) * 12.0;
                let (da, db) = (g.vertices()[e.from()].depth(), g.vertices()[e.to()].depth());
                Self::route(&spots[e.from()], &spots[e.to()], e.from() == e.to(), da, db, off)
            })
            .collect()
    }

    fn route(a: &Spot, b: &Spot, same: bool, da: usize, db: usize, off: f64) -> String {
        let n = term::Number::show;
        let (ax, ay, bx, by, r) = (a.x(), a.y(), b.x(), b.y(), Self::R);
        match () {
            () if same => format!(
                "M{} {} C{} {} {} {} {} {}",
                n(ax + 4.0),
                n(ay - 5.0),
                n(ax + 20.0),
                n(ay - 34.0),
                n(ax + 40.0),
                n(ay - 10.0),
                n(ax + r + 3.0),
                n(ay + 1.0)
            ),
            () if db == da + 1 => {
                let (y1, y2) = (ay + r + 1.0, by - r - 3.0);
                let c = (y2 - y1) * 0.5;
                format!(
                    "M{} {} C{} {} {} {} {} {}",
                    n(ax),
                    n(y1),
                    n(ax + off),
                    n(y1 + c),
                    n(bx + off),
                    n(y2 - c),
                    n(bx),
                    n(y2)
                )
            }
            () if db == da => {
                let lift = 16.0 + (bx - ax).abs() * 0.18 + off;
                let dir = if bx - ax == 0.0 || (bx - ax).is_nan() { 1.0 } else { (bx - ax).signum() };
                format!(
                    "M{} {} C{} {} {} {} {} {}",
                    n(ax + dir * 3.0),
                    n(ay - r),
                    n(ax + dir * 6.0),
                    n(ay - lift),
                    n(bx - dir * 6.0),
                    n(by - lift),
                    n(bx - dir * 2.0),
                    n(by - r - 3.0)
                )
            }
            () => {
                let sx = ax.max(bx) + 34.0 + (ay - by).abs() * 0.12 + off;
                format!(
                    "M{} {} C{} {} {} {} {} {}",
                    n(ax + r + 1.0),
                    n(ay),
                    n(sx),
                    n(ay),
                    n(sx),
                    n(by),
                    n(bx + r + 3.0),
                    n(by)
                )
            }
        }
    }

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn height(&self) -> f64 {
        self.height
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn spots(&self) -> &[Spot] {
        &self.spots
    }

    /// The SVG path of each edge, in edge order.
    pub fn routes(&self) -> &[String] {
        &self.routes
    }
}
