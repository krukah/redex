use examples::*;
use layout::*;
use leptos::html;
use leptos::prelude::*;
use markup::Selector;
use markup::*;
use syntax::*;

use crate::*;

/// The deck: one tile per rule and one per example, on a shared column grid so the two rows line
/// up. Every tile has the same parts (a face, a name, then its details), each on its own grid row.
/// An example tile's face is a thumbnail of its reduction graph.
pub struct Tiles;

impl Tiles {
    pub fn node() -> Node {
        let rules = Example::RULES.iter().map(|r| {
            let k = r.kind().expect("a rule example names its rule");
            Self::tile(
                *r,
                Class::Ink(k),
                r.sub().unwrap_or_default(),
                Node::el(Tag::Span)
                    .class([Class::Face])
                    .attr(Attr::Hidden)
                    .kid(Node::text(k.glyph())),
                r.label(),
                vec![
                    Node::el(Tag::Span)
                        .class([Class::Sub])
                        .kid(Node::text(r.sub().unwrap_or_default())),
                    Node::el(Tag::Code).kid(Node::text(r.schema().unwrap_or_default())),
                ],
            )
        });
        let more = Example::MORE.iter().map(|m| {
            let p = Program::parse(&m.src()).expect("examples parse");
            let t = Thumb::new(p.term(), p.env(), m.eta());
            let caption = p.notes().first().map(ToString::to_string).unwrap_or_default();
            Self::tile(
                *m,
                Class::Blank,
                &caption,
                t.svg().clone(),
                m.label(),
                vec![Node::el(Tag::Span).class([Class::Sub]).kid(t.fate().clone())],
            )
        });
        let label = |s: &str| Node::el(Tag::Div).class([Class::Label]).kid(Node::text(s));
        Node::el(Tag::Div)
            .class([Class::Deck])
            .kid(label("Rules"))
            .kids(rules)
            .kid(label("More examples"))
            .kids(more)
    }

    fn tile(ex: Example, ink: Class, title: &str, face: Node, name: &str, details: Vec<Node>) -> Node {
        Node::el(Tag::Button)
            .attr(Attr::Button)
            .class([Class::Tile, ink])
            .attr(Attr::Style(Style::Span(2 + details.len())))
            .attr(Attr::Ex(ex))
            .attr(Attr::Pressed(false))
            .attr(Attr::Title(title.to_string()))
            .kid(face)
            .kid(Node::el(Tag::B).kid(Node::text(name)))
            .kids(details)
    }

    /// Draws the tiles once, into an empty deck.
    fn fill(wrap: &web_sys::Element) {
        if wrap.child_element_count() == 0 {
            Doc::fill(wrap, &Self::node());
        }
    }
}

#[component]
pub fn Deck() -> impl IntoView {
    let session = expect_context::<StoredValue<Session, LocalStorage>>().get_value();
    let wrap = NodeRef::<html::Div>::new();
    let shown = session.clone();
    session.on(move |e| {
        let Some(wrap) = wrap.get_untracked() else { return };
        Tiles::fill(&wrap);
        if e != Event::Load {
            return;
        }
        let ex = shown.example();
        Doc::all(&wrap, &Selector::Has(Data::Ex)).iter().for_each(|el| {
            let on = Doc::read::<Example>(el, Data::Ex) == ex;
            Doc::toggle(el, Class::On, on);
            Doc::put(el, &Attr::Pressed(on));
        });
    });
    wrap.on_load(|el| Tiles::fill(&el));
    let click = move |e: web_sys::MouseEvent| {
        let hit = Doc::target(&e).and_then(|t| Doc::closest(&t, &Selector::Has(Data::Ex)));
        if let Some(ex) = hit.and_then(|h| Doc::read::<Example>(&h, Data::Ex))
            && !session.busy()
        {
            session.load_example(ex);
        }
    };
    view! { <div class=Class::Deckwrap.to_string() node_ref=wrap on:click=click></div> }
}
