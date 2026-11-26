use examples::*;
use layout::*;
use syntax::*;

/// The deck as the JS page drew it, from `fixtures/dom/deck.json`.
struct Deck(String);

impl Deck {
    fn load() -> Self {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/dom/deck.json");
        let v = serde_json::from_str::<serde_json::Value>(&std::fs::read_to_string(path).expect("deck.json"))
            .expect("json");
        Self(
            v["html"]
                .as_str()
                .expect("html")
                .replace("\"></path>", "\"/>")
                .replace("\"></circle>", "\"/>")
                .replace("\"></rect>", "\"/>"),
        )
    }
}

#[test]
fn every_thumbnail_and_fate_matches_the_js_deck() {
    let deck = Deck::load();
    Example::MORE.iter().for_each(|e| {
        let p = Program::parse(e.src()).expect("parses");
        let t = Thumb::new(p.term(), p.env(), e.eta());
        assert!(deck.0.contains(t.svg()), "{}: {}", e.id(), t.svg());
        assert!(deck.0.contains(&format!("<span class=\"rdx-sub\">{}</span>", t.fate())), "{}: {}", e.id(), t.fate());
    });
}
