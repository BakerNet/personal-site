#[cfg(any(feature = "ssr", feature = "hydrate"))]
pub mod app;
pub mod blog;
#[cfg(any(feature = "ssr", feature = "rss"))]
mod highlight;
#[cfg(feature = "rss")]
pub mod rss;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
