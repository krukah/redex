use leptos::html;
use leptos::prelude::*;

use crate::*;

/// The workbench: one live term drawn as nested blocks.
#[component]
pub fn Workbench() -> impl IntoView {
    let session = expect_context::<StoredValue<Session, LocalStorage>>().get_value();
    let line = NodeRef::<html::Div>::new();
    let shown = session.clone();
    session.on(move |_| {
        if let Some(el) = line.get_untracked() {
            el.set_inner_html(&Blocks::new(&shown.env(), true, false).html(&shown.term()));
        }
    });
    view! {
        <div class="rdx-scroll">
            <div class="rdx-stage">
                <svg class="rdx-arcs" aria-hidden="true"></svg>
                <div class="rdx-line rdx-tm" node_ref=line></div>
            </div>
        </div>
    }
}
