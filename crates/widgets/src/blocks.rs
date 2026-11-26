use rules::*;
use term::*;

use crate::*;

/// The nested-block view of a term, as markup. Each node is one element carrying its path and uid:
/// λ and let are boxes with a binder tab tinted by the binder's hue; an application spine is one box
/// with the head first; a pair is a box; a variable is a tag in its binder's hue (free ones dashed);
/// a constant is a bold word. With `live`, a redex gets a rule badge on its corner.
pub struct Blocks<'a> {
    env: &'a Env,
    live: bool,
    db: bool,
}

/// An enclosing binder: its name, its anchor id and its hue.
struct Scope {
    x: String,
    id: usize,
    hue: u32,
}

impl<'a> Blocks<'a> {
    const HUES: u32 = 6;

    pub fn new(env: &'a Env, live: bool, db: bool) -> Self {
        Self { env, live, db }
    }

    pub fn html(&self, t: &Term) -> String {
        self.node(t, &Path::root(), &[], 0, false, &mut 0)
    }

    /// A binder's hue from its name.
    pub fn hue(name: &str) -> u32 {
        (name.chars().fold(7u64, |h, c| (h * 31 + c as u64) % (1 << 32)) % u64::from(Self::HUES)) as u32
    }

    /// `d` is the depth of the nearest enclosing box, for alternating backgrounds; `spine` marks
    /// the function part of an application.
    fn node(&self, t: &Term, p: &Path, ctx: &[Scope], d: usize, spine: bool, nid: &mut usize) -> String {
        let kind = if self.live { t.redex(self.env) } else { None };
        let mut cls = vec!["b".to_string()];
        let mut attrs = format!(
            " data-path=\"{p}\" data-uid=\"{}\"{}",
            t.uid(),
            t.from().map(|f| format!(" data-from=\"{f}\"")).unwrap_or_default()
        );
        let (inner, boxed) = match (t.nat(), t.shape()) {
            (Some(n), _) => {
                cls.push("b-lit".into());
                (n.to_string(), false)
            }
            (None, Shape::Var(n)) => (self.var(n, ctx, &mut cls, &mut attrs), false),
            (None, Shape::Const(n)) => {
                cls.push("b-const".into());
                if self.env.has(n) {
                    cls.push("b-def".into());
                }
                (Html::esc(n), false)
            }
            (None, Shape::Lam(..) | Shape::Let(..)) => (self.binder(t, p, ctx, d, &mut cls, &mut attrs, nid), true),
            (None, Shape::App(f, a)) => {
                let k = if spine { d } else { d + 1 };
                cls.push(if spine { "b-grp" } else { "b-app" }.into());
                (
                    self.node(f, &p.child('f'), ctx, k, true, nid) + &self.node(a, &p.child('a'), ctx, k, false, nid),
                    !spine,
                )
            }
            (None, Shape::Pair(a, b)) => {
                cls.push("b-pair".into());
                (
                    self.node(a, &p.child('a'), ctx, d + 1, false, nid)
                        + "<span class=\"rp\">,</span>"
                        + &self.node(b, &p.child('b'), ctx, d + 1, false, nid),
                    true,
                )
            }
            (None, Shape::Proj(i, s)) => {
                cls.push("b-proj".into());
                (self.node(s, &p.child('t'), ctx, d, false, nid) + &format!("<span class=\"bj\">.{i}</span>"), false)
            }
        };
        if boxed {
            cls.push(if d % 2 == 1 { "odd" } else { "even" }.into());
        }
        if let Some(k) = kind {
            cls.push("rr".into());
            cls.push(format!("k-{}", k.class()));
            attrs += &format!(" data-kind=\"{}\"", k.name());
        }
        if let Some(m) = t.mark().filter(|m| *m != Mark::Binder) {
            cls.push("fl".into());
            cls.push(m.name().into());
        }
        if t.cap() {
            cls.push("cap".into());
        }
        let badge = kind
            .map(|k| {
                format!(
                    "<button type=\"button\" class=\"rm k-{}\" aria-label=\"Fire {}\">{}</button>",
                    k.class(),
                    k.name(),
                    k.glyph()
                )
            })
            .unwrap_or_default();
        format!("<span class=\"{}\"{attrs}>{badge}{inner}</span>", cls.join(" "))
    }

    fn var(&self, n: &str, ctx: &[Scope], cls: &mut Vec<String>, attrs: &mut String) -> String {
        match ctx.iter().rposition(|s| s.x == n) {
            None => {
                cls.extend(["b-var".into(), "b-free".into()]);
                Html::esc(n)
            }
            Some(j) => {
                cls.extend(["b-var".into(), format!("h{}", ctx[j].hue)]);
                if self.db {
                    cls.push("b-ix".into());
                }
                *attrs += &format!(" data-bid=\"{}\"", ctx[j].id);
                if self.db { (ctx.len() - 1 - j).to_string() } else { Html::esc(n) }
            }
        }
    }

    fn binder(
        &self,
        t: &Term,
        p: &Path,
        ctx: &[Scope],
        d: usize,
        cls: &mut Vec<String>,
        attrs: &mut String,
        nid: &mut usize,
    ) -> String {
        *nid += 1;
        let (id, x) = (*nid, t.binder().unwrap_or_default());
        let hue = Self::hue(x);
        cls.extend([
            if matches!(t.shape(), Shape::Lam(..)) { "b-lam" } else { "b-let" }.to_string(),
            format!("h{hue}"),
        ]);
        *attrs += &format!(" data-binder=\"{id}\"");
        let fl = if t.mark() == Some(Mark::Binder) { " fl fl-alpha" } else { "" };
        let name = match (self.db, self.live) {
            (true, _) => String::new(),
            (false, true) => format!(
                "<button type=\"button\" class=\"rb{fl}\" data-path=\"{p}\" data-anchor=\"{id}\" title=\"Rename (α)\">{}</button>",
                Html::esc(x)
            ),
            (false, false) => format!("<span class=\"rb\" data-anchor=\"{id}\">{}</span>", Html::esc(x)),
        };
        let kw = |s: &str| {
            format!(
                "<span class=\"rk\"{}>{s}</span>",
                if self.db { format!(" data-anchor=\"{id}\"") } else { String::new() }
            )
        };
        let inside = ctx
            .iter()
            .map(|s| Scope {
                x: s.x.clone(),
                id: s.id,
                hue: s.hue,
            })
            .chain([Scope {
                x: x.to_string(),
                id,
                hue,
            }])
            .collect::<Vec<Scope>>();
        match t.shape() {
            Shape::Let(_, v, b) => {
                let value = self.node(v, &p.child('v'), ctx, d + 1, false, nid);
                format!(
                    "<span class=\"bt\">{}{name}<span class=\"rk\">:=</span>{value}</span>{}",
                    kw("let"),
                    self.node(b, &p.child('b'), &inside, d + 1, false, nid)
                )
            }
            Shape::Lam(_, b) => format!(
                "<span class=\"bt\">{}{name}</span>{}",
                kw("λ"),
                self.node(b, &p.child('b'), &inside, d + 1, false, nid)
            ),
            _ => String::new(),
        }
    }
}
