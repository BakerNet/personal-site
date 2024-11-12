use leptos::prelude::*;

use super::ascii::{AVATAR_BLOCK, INFO_BLOCK};

#[component]
pub fn Avatar() -> impl IntoView {
    view! {
        <pre
            class="text-md xl:text-lg leading-tight xl:leading-tight"
            inner_html=AVATAR_BLOCK.join("\n")
        ></pre>
    }
}

#[component]
pub fn InfoBlock() -> impl IntoView {
    view! {
        <pre
            class="text-md xl:text-lg leading-tight xl:leading-tight"
            inner_html=INFO_BLOCK.join("\n")
        ></pre>
    }
}
