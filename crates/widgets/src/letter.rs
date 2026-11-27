use markup::*;
use rules::*;

/// A rule's letter in running text (trace, status, race lanes), or a dot for the start.
pub struct Letter;

impl Letter {
    pub fn flat(kind: Option<Kind>) -> Node {
        match kind {
            Some(k) => Node::el(Tag::Span)
                .class([Class::Badge, Class::Flat, Class::Ink(k)])
                .kid(Node::text(k.glyph())),
            None => Node::el(Tag::Span)
                .class([Class::Badge, Class::Flat, Class::Inkless])
                .kid(Node::text("·")),
        }
    }
}
