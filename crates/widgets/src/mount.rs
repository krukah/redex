use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::*;

/// Mounts every `[data-redex]` element in a scope with one shared session.
pub struct Mount;

impl Mount {
    /// `data-redex="rules|workbench|graph|race"` picks the widget; `data-example="<id>"` on any of
    /// them picks the first example (default: beta).
    pub fn all(scope: &web_sys::Element) -> Session {
        let session = Session::default();
        let stored = StoredValue::new_local(session.clone());
        let els = scope
            .query_selector_all("[data-redex]")
            .map(|list| {
                (0..list.length())
                    .filter_map(|i| list.item(i))
                    .filter_map(|n| n.dyn_into::<web_sys::HtmlElement>().ok())
                    .collect::<Vec<web_sys::HtmlElement>>()
            })
            .unwrap_or_default();
        els.iter().for_each(|el| {
            let kind = el.dataset().get("redex").unwrap_or_default();
            if !["rules", "workbench", "graph", "race"].contains(&kind.as_str()) {
                return;
            }
            let _ = el.class_list().add_1("redex");
            let handle = leptos::mount::mount_to(el.clone(), move || {
                provide_context(stored);
                match kind.as_str() {
                    "workbench" => view! { <Workbench/> }.into_any(),
                    _ => ().into_any(),
                }
            });
            handle.forget();
        });
        let first = els
            .iter()
            .find_map(|el| el.dataset().get("example").filter(|e| !e.is_empty()))
            .unwrap_or_else(|| "beta".to_string());
        session.load_example(&first);
        session
    }

    pub fn page() -> Option<Session> {
        web_sys::window()?
            .document()?
            .document_element()
            .map(|root| Self::all(&root))
    }
}
