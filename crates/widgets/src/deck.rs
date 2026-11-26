use examples::*;
use layout::*;
use leptos::prelude::*;
use syntax::*;

use crate::*;

/// The deck: one tile per rule and one per example, on a shared column grid so the two rows line
/// up. Every tile has the same parts (a face, a name, then its details), each on its own grid row.
/// An example tile's face is a thumbnail of its reduction graph.
pub struct Tiles;

impl Tiles {
    pub fn html() -> String {
        let rules = Example::RULES
            .iter()
            .map(|r| {
                let k = r.kind().expect("a rule example names its rule");
                Self::tile(
                    r.id(),
                    &format!("k-{}", k.name()),
                    r.sub().unwrap_or_default(),
                    &format!("<span class=\"rdx-face\" aria-hidden=\"true\">{}</span>", k.glyph()),
                    r.label(),
                    &[
                        format!("<span class=\"rdx-sub\">{}</span>", r.sub().unwrap_or_default()),
                        format!("<code>{}</code>", Html::esc(r.schema().unwrap_or_default())),
                    ],
                )
            })
            .collect::<Vec<String>>()
            .concat();
        let more = Example::MORE
            .iter()
            .map(|m| {
                let p = Program::parse(m.src()).expect("examples parse");
                let t = Thumb::new(p.term(), p.env(), m.eta());
                let caption = p.notes().first().cloned().unwrap_or_default();
                Self::tile(
                    m.id(),
                    "",
                    &caption,
                    t.svg(),
                    &Html::esc(m.label()),
                    &[format!("<span class=\"rdx-sub\">{}</span>", t.fate())],
                )
            })
            .collect::<Vec<String>>()
            .concat();
        format!(
            "<div class=\"rdx-deck\">\n      <div class=\"rdx-label\">Rules</div>{rules}\n      <div class=\"rdx-label\">More examples</div>{more}\n    </div>"
        )
    }

    fn tile(id: &str, cls: &str, title: &str, face: &str, name: &str, details: &[String]) -> String {
        format!(
            "<button type=\"button\" class=\"rdx-tile {cls}\" style=\"grid-row: span {}\" data-ex=\"{id}\" aria-pressed=\"false\" title=\"{}\">{face}<b>{name}</b>{}</button>",
            2 + details.len(),
            Html::esc(title),
            details.concat()
        )
    }
}

#[component]
pub fn Deck() -> impl IntoView {
    let session = expect_context::<StoredValue<Session, LocalStorage>>().get_value();
    let wrap = NodeRef::<leptos::html::Div>::new();
    let shown = session.clone();
    session.on(move |e| {
        if e != Event::Load {
            return;
        }
        let ex = shown.state().ex.clone();
        if let Some(wrap) = wrap.get_untracked() {
            Doc::all(&wrap, "[data-ex]").iter().for_each(|el| {
                let on = Doc::attr(el, "data-ex") == ex;
                Doc::toggle(el, "on", on);
                let _ = el.set_attribute("aria-pressed", if on { "true" } else { "false" });
            });
        }
    });
    let click = move |e: web_sys::MouseEvent| {
        let hit = Doc::target(&e).and_then(|t| Doc::closest(&t, "[data-ex]"));
        if let Some(id) = hit.and_then(|h| Doc::attr(&h, "data-ex"))
            && !session.busy()
        {
            session.load_example(&id);
        }
    };
    view! { <div class="rdx-deckwrap" node_ref=wrap on:click=click inner_html=Tiles::html()></div> }
}
