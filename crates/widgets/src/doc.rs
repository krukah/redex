use rules::*;
use term::*;
use wasm_bindgen::JsCast;
use web_sys::Element;

/// DOM queries shared by the widgets. Every query runs inside a widget's own root.
pub struct Doc;

impl Doc {
    /// A page-wide counter for ids that must be unique in the document (SVG marker ids).
    pub fn next_id() -> u64 {
        thread_local! {
            static NEXT: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
        }
        NEXT.with(|n| {
            n.set(n.get() + 1);
            n.get()
        })
    }

    pub fn q(root: &Element, sel: &str) -> Option<Element> {
        root.query_selector(sel).ok().flatten()
    }

    pub fn all(root: &Element, sel: &str) -> Vec<Element> {
        root.query_selector_all(sel)
            .map(|list| {
                (0..list.length())
                    .filter_map(|i| list.item(i))
                    .filter_map(|n| n.dyn_into::<Element>().ok())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn closest(el: &Element, sel: &str) -> Option<Element> {
        el.closest(sel).ok().flatten()
    }

    /// The element an event happened on.
    pub fn target(e: &web_sys::Event) -> Option<Element> {
        e.target().and_then(|t| t.dyn_into::<Element>().ok())
    }

    pub fn attr(el: &Element, name: &str) -> Option<String> {
        el.get_attribute(name)
    }

    pub fn add(el: &Element, classes: &[&str]) {
        classes.iter().for_each(|c| {
            let _ = el.class_list().add_1(c);
        });
    }

    pub fn remove(el: &Element, classes: &[&str]) {
        classes.iter().for_each(|c| {
            let _ = el.class_list().remove_1(c);
        });
    }

    pub fn toggle(el: &Element, class: &str, on: bool) {
        let _ = el.class_list().toggle_with_force(class, on);
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

    /// A rule letter in running UI text (trace, status, race lanes).
    pub fn flat(kind: Option<Kind>) -> String {
        match kind {
            Some(k) => format!("<span class=\"rm flat k-{}\">{}</span>", k.class(), k.glyph()),
            None => "<span class=\"rm flat k-none\">·</span>".to_string(),
        }
    }

    /// One arc from each bound occurrence up to its binder. Both ends rise to a shared apex; longer
    /// links rise higher, never past the ceiling.
    pub fn arcs(stage: &Element, line: &Element, on: bool) {
        let Some(svg) = Self::q(stage, ".rdx-arcs") else { return };
        if !on {
            svg.set_inner_html("");
            return;
        }
        let base = stage.get_bounding_client_rect();
        let ceiling = line.get_bounding_client_rect().top() - base.top() - 30.0;
        let f = |n: f64| Number::fixed(n, 1);
        let out = Self::all(line, "[data-bid]")
            .iter()
            .filter_map(|v| {
                let id = Self::attr(v, "data-bid")?;
                let anchor = Self::q(line, &format!("[data-anchor=\"{id}\"]"))?;
                let (a, b) = (anchor.get_bounding_client_rect(), v.get_bounding_client_rect());
                let (x1, x2) = (a.left() + a.width() / 2.0 - base.left(), b.left() + b.width() / 2.0 - base.left());
                let (y1, y2) = (a.top() - base.top() + 2.0, b.top() - base.top() + 2.0);
                let apex = ceiling.max(y1.min(y2) - 10.0 - (x2 - x1).abs() * 0.12);
                let cap = if v.class_list().contains("cap") { " cap" } else { "" };
                Some(format!(
                    "<path class=\"rarc{cap}\" data-id=\"{id}\" d=\"M{} {} C{} {} {} {} {} {}\"/><circle class=\"rarc-end{cap}\" data-id=\"{id}\" cx=\"{}\" cy=\"{}\" r=\"2.2\"/>",
                    f(x1),
                    f(y1),
                    f(x1),
                    f(apex),
                    f(x2),
                    f(apex),
                    f(x2),
                    f(y2),
                    f(x2),
                    f(y2)
                ))
            })
            .collect::<String>();
        svg.set_inner_html(&out);
    }
}
