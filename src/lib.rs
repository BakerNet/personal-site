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
    // initializes logging using the `log` crate
    #[cfg(debug_assertions)]
    let log_level = log::Level::Debug;
    #[cfg(not(debug_assertions))]
    let log_level = log::Level::Warn;
    _ = console_log::init_with_level(log_level);
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
