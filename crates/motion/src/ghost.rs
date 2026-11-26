use wasm_bindgen::JsCast;
use web_sys::DomRect;
use web_sys::Element;

/// A copy of an element pinned over the page at its own size, ready to move, so the original can
/// be replaced or removed while the copy plays. It inherits the original's text settings and keeps
/// its exact box: the same picture, not a re-layout of it in another context.
pub struct Ghost;

impl Ghost {
    const INHERITED: [&'static str; 9] = [
        "font-family",
        "font-size",
        "font-weight",
        "font-style",
        "font-variant-ligatures",
        "line-height",
        "letter-spacing",
        "white-space",
        "color",
    ];

    /// Pins a copy of `el` centered on `at` (default: where `el` is). `hidden` starts it
    /// transparent, for a copy that will fade in; `mark` goes on the copy's class list.
    pub fn pin(el: &Element, at: Option<&DomRect>, hidden: bool, mark: &[&str]) -> Option<Element> {
        let doc = web_sys::window()?.document()?;
        let r = el.get_bounding_client_rect();
        let (cx, cy) = Self::center(at.unwrap_or(&r));
        let cs = web_sys::window()?.get_computed_style(el).ok().flatten()?;
        let w = doc.create_element("div").ok()?;
        w.set_class_name("redex rdx-ghost");
        let inherited = Self::INHERITED
            .iter()
            .map(|p| format!("{p}:{}", cs.get_property_value(p).unwrap_or_default()))
            .collect::<Vec<String>>()
            .join(";");
        let css = format!(
            "left:{}px;top:{}px;width:{}px;height:{}px;{inherited}{}",
            cx - r.width() / 2.0,
            cy - r.height() / 2.0,
            r.width(),
            r.height(),
            if hidden { ";opacity:0" } else { "" }
        );
        if let Ok(style) = js_sys::Reflect::get(&w, &"style".into()) {
            let _ = js_sys::Reflect::set(&style, &"cssText".into(), &css.into());
        }
        let copy = el.clone_node_with_deep(true).ok()?.dyn_into::<Element>().ok()?;
        let _ = copy.class_list().remove_3("hot", "soft", "arriving");
        mark.iter().for_each(|m| {
            let _ = copy.class_list().add_1(m);
        });
        if let Ok(style) =
            js_sys::Reflect::get(&copy, &"style".into()).and_then(JsCast::dyn_into::<web_sys::CssStyleDeclaration>)
        {
            let _ = style.set_property("width", &format!("{}px", r.width()));
            let _ = style.set_property("height", &format!("{}px", r.height()));
            let _ = style.set_property("flex", "none");
        }
        w.append_child(&copy).ok()?;
        doc.body()?.append_child(&w).ok()?;
        Some(w)
    }

    pub fn center(r: &DomRect) -> (f64, f64) {
        (r.left() + r.width() / 2.0, r.top() + r.height() / 2.0)
    }

    /// The offset from rect `a`'s center to rect `b`'s.
    pub fn delta(a: &DomRect, b: &DomRect) -> (f64, f64) {
        let ((ax, ay), (bx, by)) = (Self::center(a), Self::center(b));
        (bx - ax, by - ay)
    }
}
