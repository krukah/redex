use std::str::FromStr;

use markup::*;
use units::*;
use wasm_bindgen::JsCast;
use web_sys::Element;

/// The DOM, typed: queries by `Selector`, data attributes read back into their types, classes by
/// `Class`, and markup built into real elements. Every query runs inside a widget's own root.
pub struct Doc;

impl Doc {
    pub fn q(root: &Element, sel: &Selector) -> Option<Element> {
        root.query_selector(&sel.to_string()).ok().flatten()
    }

    pub fn all(root: &Element, sel: &Selector) -> Vec<Element> {
        root.query_selector_all(&sel.to_string())
            .map(|list| {
                (0..list.length())
                    .filter_map(|i| list.item(i))
                    .filter_map(|n| n.dyn_into::<Element>().ok())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The parent element of a node a view holds a reference to.
    pub fn above<E>(el: Option<E>) -> Option<Element>
    where
        E: Into<web_sys::Node>,
    {
        let node = el?.into();
        node.parent_element()
    }

    pub fn closest(el: &Element, sel: &Selector) -> Option<Element> {
        el.closest(&sel.to_string()).ok().flatten()
    }

    /// The element an event happened on.
    pub fn target(e: &web_sys::Event) -> Option<Element> {
        e.target().and_then(|t| t.dyn_into::<Element>().ok())
    }

    pub fn read<T>(el: &Element, data: Data) -> Option<T>
    where
        T: FromStr,
    {
        el.get_attribute(data.name()).and_then(|v| v.parse().ok())
    }

    pub fn write<T>(el: &Element, data: Data, value: &T)
    where
        T: std::fmt::Display,
    {
        let _ = el.set_attribute(data.name(), &value.to_string());
    }

    /// Sets one attribute of the markup vocabulary on a live element.
    pub fn put(el: &Element, attr: &Attr) {
        let _ = el.set_attribute(attr.name(), &attr.to_string());
    }

    pub fn erase(el: &Element, data: Data) {
        let _ = el.remove_attribute(data.name());
    }

    pub fn add(el: &Element, classes: &[Class]) {
        classes.iter().for_each(|c| {
            let _ = el.class_list().add_1(&c.to_string());
        });
    }

    pub fn remove(el: &Element, classes: &[Class]) {
        classes.iter().for_each(|c| {
            let _ = el.class_list().remove_1(&c.to_string());
        });
    }

    pub fn toggle(el: &Element, class: Class, on: bool) {
        let _ = el.class_list().toggle_with_force(&class.to_string(), on);
    }

    pub fn has(el: &Element, class: Class) -> bool {
        el.class_list().contains(&class.to_string())
    }

    pub fn hide(el: &Element, hidden: bool) {
        if let Some(h) = el.dyn_ref::<web_sys::HtmlElement>() {
            h.set_hidden(hidden);
        }
    }

    pub fn children(el: &Element) -> Vec<Element> {
        let list = el.children();
        (0..list.length()).filter_map(|i| list.item(i)).collect()
    }

    /// The element's box, in pixels from the viewport's corner.
    pub fn top(el: &Element) -> Px {
        Px::new(el.get_bounding_client_rect().top())
    }

    /// Replaces an element's children with the markup.
    pub fn fill(el: &Element, node: &Node) {
        el.set_text_content(None);
        if let Some(doc) = el.owner_document() {
            Self::build(&doc, node).iter().for_each(|n| {
                let _ = el.append_child(n);
            });
        }
    }

    /// The DOM nodes for a piece of markup: elements in their namespace, attributes in writing
    /// order, text as text nodes.
    fn build(doc: &web_sys::Document, node: &Node) -> Vec<web_sys::Node> {
        match node {
            Node::Text(s) => vec![doc.create_text_node(s).into()],
            Node::List(nodes) => nodes.iter().flat_map(|n| Self::build(doc, n)).collect(),
            Node::El(tag, attrs, kids) => {
                let made = if tag.svg() {
                    doc.create_element_ns(Some(Tag::SVG), tag.name())
                } else {
                    doc.create_element(tag.name())
                };
                let Ok(el) = made else { return vec![] };
                attrs.iter().for_each(|a| {
                    let _ = el.set_attribute(a.name(), &a.to_string());
                });
                kids.iter().flat_map(|k| Self::build(doc, k)).for_each(|k| {
                    let _ = el.append_child(&k);
                });
                vec![el.into()]
            }
        }
    }
}
