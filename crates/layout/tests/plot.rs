use explore::*;
use layout::*;
use serde_json::Value;
use strategy::*;
use syntax::*;

/// The graph widget's drawing of one term, as `scripts/golden-dom.mjs` read it from the JS page.
struct Drawn(Value);

impl Drawn {
    fn all() -> Vec<(String, Drawn)> {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/dom");
        let mut all = std::fs::read_dir(dir)
            .expect("fixtures/dom exists")
            .map(|e| e.expect("an entry").path())
            .filter(|p| p.file_name().is_some_and(|n| n.to_string_lossy().starts_with("graph-")))
            .map(|p| {
                (
                    p.file_stem().expect("a name").to_string_lossy().into_owned(),
                    Drawn(serde_json::from_str(&std::fs::read_to_string(&p).expect("readable")).expect("json")),
                )
            })
            .collect::<Vec<(String, Drawn)>>();
        all.sort_by(|a, b| a.0.cmp(&b.0));
        assert!(all.len() > 90, "found {} drawings", all.len());
        all
    }

    /// Each tag that starts with `open`, up to its `>`.
    fn tags(svg: &str, open: &str) -> Vec<String> {
        svg.match_indices(open)
            .map(|(i, _)| svg[i..].split_once('>').map_or("", |(t, _)| t).to_string())
            .collect()
    }

    /// The value of `name` inside a tag.
    fn attr(tag: &str, name: &str) -> String {
        tag.split_once(&format!(" {name}=\""))
            .and_then(|(_, rest)| rest.split_once('"'))
            .map_or("", |(v, _)| v)
            .to_string()
    }

    fn check(&self, name: &str) -> Vec<String> {
        let prog = Program::parse(&Source::from(self.0["src"].as_str().expect("a source"))).expect("parses");
        self.0["graphs"]
            .as_array()
            .expect("two drawings")
            .iter()
            .flat_map(|want| {
                let g = Graph::explore(prog.term(), prog.env(), want["eta"].as_bool().expect("eta"), Cap::new(70));
                let plot = Plot::new(&g, prog.env());
                let svg = want["svg"].as_str().expect("svg");
                let hits = Self::tags(svg, "<circle class=\"hit\"")
                    .iter()
                    .map(|t| (Self::attr(t, "cx"), Self::attr(t, "cy")))
                    .collect::<Vec<(String, String)>>();
                let paths = Self::tags(svg, "<path class=\"rdx-ge")
                    .iter()
                    .map(|t| Self::attr(t, "d"))
                    .collect::<Vec<String>>();
                let got = g
                    .vertices()
                    .map(|(i, _)| (plot.spot(i).x().to_string(), plot.spot(i).y().to_string()))
                    .collect::<Vec<(String, String)>>();
                [
                    (want["width"].as_str() != Some(&plot.width().to_string()), "width"),
                    (want["height"].as_str() != Some(&plot.height().to_string()), "height"),
                    (hits != got, "spots"),
                    (paths != plot.routes().iter().map(ToString::to_string).collect::<Vec<String>>(), "routes"),
                ]
                .into_iter()
                .filter(|(bad, _)| *bad)
                .map(|(_, what)| format!("{name} (eta {}): {what} differ", want["eta"]))
                .collect::<Vec<String>>()
            })
            .collect()
    }
}

#[test]
fn every_graph_lays_out_as_the_js_widget_drew_it() {
    let failures = Drawn::all()
        .iter()
        .flat_map(|(n, d)| d.check(n))
        .collect::<Vec<String>>();
    assert!(failures.is_empty(), "{} mismatches:\n{}", failures.len(), failures.join("\n"));
}
