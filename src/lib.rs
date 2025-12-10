pub mod app;
pub mod components;
pub mod db;
pub mod openai;
pub mod otro_upload;
pub mod storage;
#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    leptos::mount::hydrate_body(App);
}
