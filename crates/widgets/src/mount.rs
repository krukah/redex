use examples::*;
use leptos::prelude::*;
use markup::Selector;
use markup::*;
use wasm_bindgen::JsCast;

use crate::*;

/// The widget a host element asks for with `data-redex`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Widget {
    Rules,
    Workbench,
    Graph,
    Race,
}

impl std::str::FromStr for Widget {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rules" => Ok(Self::Rules),
            "workbench" => Ok(Self::Workbench),
            "graph" => Ok(Self::Graph),
            "race" => Ok(Self::Race),
            _ => Err(()),
        }
    }
}

/// Mounts every `[data-redex]` element in a scope with one shared session.
pub struct Mount;

impl Mount {
    /// `data-redex="rules|workbench|graph|race"` picks the widget; `data-example="<id>"` on any of
    /// them picks the first example (default: beta).
    pub fn all(scope: &web_sys::Element) -> Session {
        let session = Session::default();
        let stored = StoredValue::new_local(session.clone());
        let els = Doc::all(scope, &Selector::Has(Data::Redex))
            .into_iter()
            .filter_map(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
            .collect::<Vec<web_sys::HtmlElement>>();
        els.iter().for_each(|el| {
            let Some(widget) = Doc::read::<Widget>(el, Data::Redex) else {
                return;
            };
            Doc::add(el, &[Class::Redex]);
            leptos::mount::mount_to(el.clone(), move || {
                provide_context(stored);
                match widget {
                    Widget::Rules => view! { <Deck/> }.into_any(),
                    Widget::Workbench => view! { <Workbench/> }.into_any(),
                    Widget::Graph => view! { <Paths/> }.into_any(),
                    Widget::Race => view! { <Lanes/> }.into_any(),
                }
            })
            .forget();
        });
        let first = els
            .iter()
            .find_map(|el| el.get_attribute(Data::Example.name()).filter(|e| !e.is_empty()))
            .unwrap_or_else(|| "beta".to_string());
        if let Some(ex) = Example::find(&first) {
            session.load_example(ex);
        }
        session
    }

    pub fn page() -> Option<Session> {
        web_sys::window()?
            .document()?
            .document_element()
            .map(|root| Self::all(&root))
    }
}
