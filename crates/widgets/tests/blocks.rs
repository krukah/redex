use rules::*;
use serde_json::Value;
use syntax::*;
use term::*;
use widgets::*;

/// One engine fixture and the blocks markup the JS renderer wrote for it, ids relative to a probe.
struct Fixture(String, Value);

impl Fixture {
    fn all() -> Vec<Fixture> {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/engine");
        let mut all = std::fs::read_dir(dir)
            .expect("fixtures/engine exists")
            .map(|e| e.expect("an entry").path())
            .filter(|p| p.file_name().is_some_and(|n| n != "errors.json"))
            .map(|p| {
                Fixture(
                    p.file_stem().expect("a name").to_string_lossy().into_owned(),
                    serde_json::from_str(&std::fs::read_to_string(&p).expect("readable")).expect("json"),
                )
            })
            .collect::<Vec<Fixture>>();
        all.sort_by(|a, b| a.0.cmp(&b.0));
        all
    }

    /// Rewrites every `data-uid` and `data-from` relative to `base`.
    fn rebase(html: &str, base: u64) -> String {
        ["data-uid=\"", "data-from=\""]
            .iter()
            .fold(html.to_string(), |html, attr| {
                html.split(attr)
                    .enumerate()
                    .map(|(i, part)| match (i, part.split_once('"')) {
                        (0, _) | (_, None) => part.to_string(),
                        (_, Some((n, rest))) => {
                            format!("{}{}\"{rest}", attr, n.parse::<i64>().expect("an id") - base as i64)
                        }
                    })
                    .collect()
            })
    }

    fn check(&self) -> Vec<String> {
        let base = Term::var("_").uid().get();
        let prog = Program::parse(self.1["src"].as_str().expect("a source")).expect("parses");
        let (term, env) = (prog.term(), prog.env());
        let want = |v: &Value| v.as_str().expect("markup").to_string();
        let mut out = vec![
            (
                "blocks".to_string(),
                want(&self.1["blocks"]),
                Self::rebase(&Blocks::new(env, true, false).html(term), base),
            ),
            (
                "debruijn".to_string(),
                want(&self.1["debruijn"]),
                Self::rebase(&Blocks::new(env, true, true).html(term), base),
            ),
        ];
        out.extend(
            env.iter()
                .zip(self.1["env"].as_array().expect("defs"))
                .map(|((n, b), w)| {
                    (
                        format!("def {n}"),
                        want(&w["blocks"]),
                        Self::rebase(&Blocks::new(env, false, false).html(b), base),
                    )
                }),
        );
        out.extend(
            term.redexes(env)
                .iter()
                .zip(self.1["redexes"].as_array().expect("redexes"))
                .map(|(r, w)| {
                    if matches!(r.kind(), Kind::Beta | Kind::Zeta) {
                        let node = term.at(r.path()).expect("inside");
                        let (x, b, arg) = match node.shape() {
                            Shape::App(f, a) => match f.shape() {
                                Shape::Lam(x, b) => (x.clone(), b.clone(), a.clone()),
                                _ => unreachable!("β has a λ"),
                            },
                            Shape::Let(x, v, b) => (x.clone(), b.clone(), v.clone()),
                            _ => unreachable!("β and ζ bind"),
                        };
                        subst::Substitute::prepare(&b, &x, &arg.fv(), &mut Vec::new());
                        subst::Substitute::naive(&b, &x, &arg);
                    }
                    (
                        format!("redex {} {}", r.path(), r.kind().name()),
                        want(&w["blocks"]),
                        Self::rebase(&Blocks::new(env, true, false).html(&term.step(r, env)), base),
                    )
                }),
        );
        out.into_iter()
            .filter(|(_, w, g)| w != g)
            .map(|(what, w, g)| format!("{}: {what}\n want {w}\n  got {g}", self.0))
            .collect()
    }
}

#[test]
fn blocks_markup_matches_the_js_renderer() {
    let failures = Fixture::all().iter().flat_map(Fixture::check).collect::<Vec<String>>();
    assert!(
        failures.is_empty(),
        "{} mismatches:\n{}",
        failures.len(),
        failures.iter().take(4).cloned().collect::<Vec<String>>().join("\n\n")
    );
}
