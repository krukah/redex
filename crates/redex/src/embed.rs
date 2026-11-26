use wasm_bindgen::prelude::*;

/// Mounts every `[data-redex]` element under `scope` (default: the whole document) with one shared
/// session. Host pages call it once, after `init()`.
// lint:allow(free-function) wasm-bindgen exports a module-level JS function only from a bare fn
#[wasm_bindgen(js_name = mountAll)]
pub fn mount_all(scope: Option<web_sys::Element>) {
    console_error_panic_hook::set_once();
    match scope {
        Some(scope) => {
            widgets::Mount::all(&scope);
        }
        None => {
            widgets::Mount::page();
        }
    }
}
