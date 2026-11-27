use explore::*;
use rules::*;
use serde_json::Value;
use serde_json::json;
use strategy::*;
use subst::*;
use syntax::*;
use term::*;

/// One frozen fixture from `fixtures/engine`, and what the engine says about its source.
struct Case {
    name: String,
    want: Value,
}

impl Case {
    fn name(&self) -> &str {
        &self.name
    }

    fn want(&self) -> &Value {
        &self.want
    }

    /// Keys the engine does not produce: the blocks view belongs to `widgets`.
    const SKIP: [&'static str; 3] = ["blocks", "debruijn", "id"];

    fn all() -> Vec<Case> {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/engine");
        let mut cases = std::fs::read_dir(&dir)
            .expect("fixtures/engine exists")
            .map(|e| e.expect("a readable entry").path())
            .filter(|p| p.file_name().is_some_and(|n| n != "errors.json"))
            .map(|p| Case {
                name: p.file_stem().expect("a file name").to_string_lossy().into_owned(),
                want: serde_json::from_str(&std::fs::read_to_string(&p).expect("a readable fixture")).expect("json"),
            })
            .collect::<Vec<Case>>();
        cases.sort_by(|a, b| a.name().cmp(b.name()));
        assert!(cases.len() > 90, "found {} fixtures", cases.len());
        cases
    }

    fn src(&self) -> &str {
        self.want()["src"].as_str().expect("a source")
    }

    fn eta(&self) -> bool {
        self.want()["eta"].as_bool().expect("an eta flag")
    }

    /// The fixture's view of the source, computed by the Rust engine.
    fn study(&self) -> Value {
        let base = Term::var("_").uid();
        let prog = Program::parse(&Source::from(self.src())).expect("fixtures parse");
        let (term, env) = (prog.term(), prog.env());
        let mut fv = term.fv().iter().map(ToString::to_string).collect::<Vec<String>>();
        fv.sort();
        json!({
            "src": self.src(),
            "eta": self.eta(),
            "notes": prog.notes().iter().map(ToString::to_string).collect::<Vec<String>>(),
            "env": env.iter().map(|(n, b)| json!({ "name": n.to_string(), "text": b.text(), "tree": b.dump(base) })).collect::<Vec<Value>>(),
            "text": term.text(),
            "key": term.key().to_string(),
            "size": term.size().get(),
            "tree": term.dump(base),
            "fv": fv,
            "redexes": term.redexes(env).iter().map(|r| Self::redex(term, env, r, base)).collect::<Vec<Value>>(),
            "strategies": {
                "normal": Self::runs(term, env, Strategy::Normal),
                "applicative": Self::runs(term, env, Strategy::Applicative),
                "whnf": Self::runs(term, env, Strategy::Whnf),
            },
            "graphs": {
                "plain": Self::graph(term, env, false, 70),
                "eta": Self::graph(term, env, true, 70),
                "thumb": Self::graph(term, env, self.eta(), 40),
            },
        })
    }

    fn redex(term: &Term, env: &Env, r: &Redex, base: Uid) -> Value {
        let node = term.at(r.path()).expect("a redex inside its term");
        let schema = r.kind().schema(node);
        let mut entry = json!({
            "path": r.path().to_string(),
            "kind": r.kind().to_string(),
            "schema": schema.text(),
            "vars": schema.vars().iter().map(ToString::to_string).collect::<Vec<String>>(),
            "bindings": r.kind().bindings(node).iter().map(|b| json!([b.var().to_string(), b.rel().to_string(), b.is_binder()])).collect::<Vec<Value>>(),
        });
        let lam = match (r.kind(), node.shape()) {
            (Kind::Beta, Shape::App(f, a)) => Some((f.clone(), a.clone())),
            (Kind::Zeta, Shape::Let(_, v, _)) => Some((node.clone(), v.clone())),
            _ => None,
        };
        if let Some((lam, arg)) = lam {
            let (x, b) = match lam.shape() {
                Shape::Lam(x, b) | Shape::Let(x, _, b) => (x.clone(), b.clone()),
                _ => unreachable!("β and ζ bind a name"),
            };
            let mut log = Vec::new();
            let body = b.prepare(&x, &arg.fv(), &mut log);
            let naive = b.naive(&x, &arg);
            entry["log"] = json!(
                log.iter()
                    .map(|(a, b)| json!([a.to_string(), b.to_string()]))
                    .collect::<Vec<Value>>()
            );
            entry["prepared"] = json!(body.dump(base));
            entry["naive"] = json!(naive.dump(base));
            entry["naiveText"] = json!(naive.text());
        }
        let result = term.step(r, env);
        entry["result"] = json!(result.dump(base));
        entry["text"] = json!(result.text());
        entry
    }

    fn runs(term: &Term, env: &Env, strategy: Strategy) -> Value {
        let run = |cap| {
            let r = Run::new(term, env, strategy, Cap::new(cap));
            json!({
                "status": r.status().name(),
                "rows": r.rows().iter().map(|row| json!([row.kind().map(|k| k.to_string()), row.term().text()])).collect::<Vec<Value>>(),
            })
        };
        json!({ "24": run(24), "200": run(200) })
    }

    fn graph(term: &Term, env: &Env, eta: bool, cap: usize) -> Value {
        let g = Graph::explore(term, env, eta, Cap::new(cap));
        json!({
            "truncated": g.truncated(),
            "cycle": g.cyclic(),
            "nodes": g.vertices().map(|(_, v)| json!([v.key().to_string(), v.term().text(), v.depth().get(), v.parent().map(Id::get), v.pkind().map(|k| k.to_string()), v.nf(), v.open()])).collect::<Vec<Value>>(),
            "edges": g.edges().iter().map(|e| json!([e.from().get(), e.to().get(), e.kind().to_string()])).collect::<Vec<Value>>(),
        })
    }

    fn check(&self) -> Vec<String> {
        let got = self.study();
        let want = self.want().as_object().expect("a fixture object");
        want.iter()
            .filter(|(k, _)| !Self::SKIP.contains(&k.as_str()))
            .filter_map(|(k, w)| {
                Self::strip(w)
                    .ne(&got[k])
                    .then(|| format!("{}: {k} differs\n want {}\n  got {}", self.name(), Self::strip(w), got[k]))
            })
            .collect()
    }

    /// The fixture value without the blocks view.
    fn strip(v: &Value) -> Value {
        match v {
            Value::Object(m) => Value::Object(
                m.iter()
                    .filter(|(k, _)| !Self::SKIP.contains(&k.as_str()))
                    .map(|(k, v)| (k.clone(), Self::strip(v)))
                    .collect(),
            ),
            Value::Array(xs) => Value::Array(xs.iter().map(Self::strip).collect()),
            _ => v.clone(),
        }
    }

    fn errors() -> Vec<(String, Value)> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/engine/errors.json");
        serde_json::from_str::<Value>(&std::fs::read_to_string(path).expect("errors.json"))
            .expect("json")
            .as_array()
            .expect("a list")
            .iter()
            .map(|e| (e["src"].as_str().unwrap_or_default().to_string(), e.clone()))
            .collect()
    }
}

#[test]
fn every_fixture_matches_the_js_engine() {
    let failures = Case::all().iter().flat_map(Case::check).collect::<Vec<String>>();
    assert!(
        failures.is_empty(),
        "{} mismatches:\n{}",
        failures.len(),
        failures.iter().take(6).cloned().collect::<Vec<String>>().join("\n\n")
    );
}

#[test]
fn parse_errors_read_as_in_js() {
    Case::errors().into_iter().for_each(|(src, want)| {
        let got = match Program::parse(&Source::from(src.as_str())) {
            Ok(p) => json!({
                "src": src,
                "error": null,
                "text": p.term().text(),
                "notes": p.notes().iter().map(ToString::to_string).collect::<Vec<String>>(),
                "defs": p.env().iter().map(|(n, b)| json!([n.to_string(), b.text()])).collect::<Vec<Value>>(),
            }),
            Err(e) => json!({ "src": src, "error": e.to_string(), "text": null, "notes": null, "defs": null }),
        };
        assert_eq!(got, want);
    });
}

#[test]
fn examples_and_races_are_the_fixture_sources() {
    let cases = Case::all();
    let src = |name: String| {
        cases
            .iter()
            .find(|c| c.name() == name)
            .map(|c| (c.src().to_string(), c.eta()))
    };
    examples::Example::all()
        .for_each(|e| assert_eq!(src(format!("example-{}", e.id())), Some((e.src().to_string(), e.eta()))));
    examples::Race::ALL
        .iter()
        .filter_map(|r| r.src().map(|s| (r.id(), s)))
        .for_each(|(id, s)| {
            assert_eq!(src(format!("race-{id}")), Some((s.to_string(), false)));
        });
}
