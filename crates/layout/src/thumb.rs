use explore::*;
use strategy::*;
use term::*;

/// A deck tile's thumbnail of an example's reduction graph, in the graph widget's grammar, and the
/// line that says how the example ends.
#[derive(Debug, Clone)]
pub struct Thumb {
    svg: String,
    fate: String,
}

impl Thumb {
    /// The thumbnail box, rows shown, mark radius.
    const TW: f64 = 120.0;
    const TH: f64 = 46.0;
    const PAD: f64 = 6.0;
    const ROWS: usize = 4;
    const MR: f64 = 3.0;

    pub fn new(term: &Term, env: &Env, eta: bool) -> Self {
        let g = Graph::explore(term, env, eta, 40);
        let at = Self::place(&g);
        let edges = g
            .edges()
            .iter()
            .filter(|e| g.vertices()[e.from()].depth() <= Self::ROWS)
            .map(|e| Self::edge(&g, e, &at))
            .collect::<String>();
        let marks = (0..g.vertices().len())
            .filter(|&i| g.vertices()[i].depth() <= Self::ROWS)
            .map(|i| Self::mark(&g, i, at[i], env))
            .collect::<String>();
        let run = Run::new(term, env, Strategy::Normal, 60);
        let n = run.steps();
        let steps = format!("{n} step{}", if n == 1 { "" } else { "s" });
        let nf = r#"<i class="rdx-nfmark" aria-hidden="true"></i><span class="rdx-sr">normal form</span>"#;
        let fate = match (run.status(), g.cyclic()) {
            (Status::Done, true) => format!("{nf} in {steps}, or loops"),
            (Status::Done, false) => format!("{nf} in {steps}"),
            (_, true) => "loops forever".to_string(),
            (_, false) => "keeps growing".to_string(),
        };
        Self {
            svg: format!(
                r#"<svg class="rdx-thumb" viewBox="0 0 {} {}" aria-hidden="true">{edges}{marks}</svg>"#,
                Number::show(Self::TW),
                Number::show(Self::TH)
            ),
            fate,
        }
    }

    /// Each shown vertex's position; vertices past the shown rows stay at the origin.
    fn place(g: &Graph) -> Vec<(f64, f64)> {
        let shown = |i: usize| g.vertices()[i].depth() <= Self::ROWS;
        let rows = (0..g.vertices().len())
            .filter(|&i| shown(i))
            .fold(Vec::new(), |mut rows: Vec<Vec<usize>>, i| {
                let d = g.vertices()[i].depth();
                rows.resize_with(rows.len().max(d + 1), Vec::new);
                rows[d].push(i);
                rows
            });
        let depth = rows.len() as f64 - 1.0;
        let wide = rows.iter().map(Vec::len).max().unwrap_or(0) as f64;
        let dy = if depth > 0.0 { 16f64.min((Self::TH - 2.0 * Self::PAD) / depth) } else { 0.0 };
        let dx = if wide > 1.0 {
            22f64.min((Self::TW - 2.0 * Self::PAD - 14.0) / (wide - 1.0))
        } else {
            0.0
        };
        let y0 = (Self::TH - depth * dy) / 2.0;
        let mut at = vec![(0.0, 0.0); g.vertices().len()];
        for (d, row) in rows.iter().enumerate() {
            let mut keyed = row
                .iter()
                .map(|&v| (if d > 0 { Self::center(g, v, &at) } else { 0.0 }, v))
                .collect::<Vec<(f64, usize)>>();
            keyed.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            let x0 = Self::TW / 2.0 - 7.0 - ((keyed.len() as f64 - 1.0) * dx) / 2.0;
            keyed
                .iter()
                .enumerate()
                .for_each(|(i, &(_, v))| at[v] = (x0 + i as f64 * dx, y0 + d as f64 * dy));
        }
        at
    }

    /// The mean x of a vertex's parents one row up.
    fn center(g: &Graph, v: usize, at: &[(f64, f64)]) -> f64 {
        let ps = g
            .edges()
            .iter()
            .filter(|e| e.to() == v && g.vertices()[v].depth() == g.vertices()[e.from()].depth() + 1)
            .map(|e| at[e.from()].0)
            .collect::<Vec<f64>>();
        if ps.is_empty() {
            0.0
        } else {
            ps.iter().fold(0.0, |s, x| s + x) / ps.len() as f64
        }
    }

    fn edge(g: &Graph, e: &Edge, at: &[(f64, f64)]) -> String {
        let f = |v: f64| Number::fixed(v, 1);
        let ((ax, ay), (bx, by), mr) = (at[e.from()], at[e.to()], Self::MR);
        let (da, db) = (g.vertices()[e.from()].depth(), g.vertices()[e.to()].depth());
        let (d, cut) = match () {
            () if db > Self::ROWS => (format!("M{} {} L{} {}", f(ax), f(ay + mr), f(ax), f(ay + mr + 6.0)), " cut"),
            () if e.from() == e.to() => (
                format!(
                    "M{} {} C{} {} {} {} {} {}",
                    f(ax + 2.0),
                    f(ay - 2.0),
                    f(ax + 9.0),
                    f(ay - 13.0),
                    f(ax + 16.0),
                    f(ay - 2.0),
                    f(ax + mr + 1.0),
                    f(ay + 1.0)
                ),
                "",
            ),
            () if db == da + 1 => (format!("M{} {} L{} {}", f(ax), f(ay + mr), f(bx), f(by - mr)), ""),
            () if db == da => {
                let lift = 5.0 + (bx - ax).abs() * 0.25;
                (
                    format!(
                        "M{} {} Q{} {} {} {}",
                        f(ax),
                        f(ay - mr),
                        f(f64::midpoint(ax, bx)),
                        f(ay - lift),
                        f(bx),
                        f(by - mr)
                    ),
                    "",
                )
            }
            () => {
                let sx = ax.max(bx) + 10.0 + (ay - by).abs() * 0.3;
                (
                    format!(
                        "M{} {} C{} {} {} {} {} {}",
                        f(ax + mr),
                        f(ay),
                        f(sx),
                        f(ay),
                        f(sx),
                        f(by),
                        f(bx + mr),
                        f(by)
                    ),
                    "",
                )
            }
        };
        format!(r#"<path class="te k-{}{cut}" d="{d}"/>"#, e.kind().class())
    }

    fn mark(g: &Graph, i: usize, (x, y): (f64, f64), env: &Env) -> String {
        let f = |v: f64| Number::fixed(v, 1);
        let v = &g.vertices()[i];
        let open = if v.open() { " open" } else { "" };
        let mr = Self::MR;
        let ring = if i == 0 {
            format!(r#"<circle class="ring" cx="{}" cy="{}" r="{}"/>"#, f(x), f(y), Number::show(mr + 2.4))
        } else {
            String::new()
        };
        let shape = match (v.nf(), Strategy::Whnf.next(v.term(), env)) {
            (true, _) => format!(
                r#"<rect class="m nf" x="{}" y="{}" width="{}" height="{}"/>"#,
                f(x - mr),
                f(y - mr),
                Number::show(2.0 * mr),
                Number::show(2.0 * mr)
            ),
            (false, None) => {
                let r = mr + 0.8;
                format!(
                    r#"<path class="m whnf{open}" d="M{} {}L{} {}L{} {}L{} {}Z"/>"#,
                    f(x),
                    f(y - r),
                    f(x + r),
                    f(y),
                    f(x),
                    f(y + r),
                    f(x - r),
                    f(y)
                )
            }
            (false, Some(_)) => {
                format!(r#"<circle class="m red{open}" cx="{}" cy="{}" r="{}"/>"#, f(x), f(y), Number::show(mr))
            }
        };
        format!("{ring}{shape}")
    }

    pub fn svg(&self) -> &str {
        &self.svg
    }

    pub fn fate(&self) -> &str {
        &self.fate
    }
}
