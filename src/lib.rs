pub mod app;
pub mod components;
pub mod otro_upload;
#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    leptos::mount::hydrate_body(App);
}
