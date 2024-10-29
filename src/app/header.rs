use std::sync::{Arc, Mutex};

use leptos::{html, prelude::*};
use leptos_router::{
    hooks::{use_location, use_navigate},
    NavigateOptions,
};

use super::terminal::{CommandRes, Terminal};

#[component]
pub fn Header() -> impl IntoView {
    // TODO - actually get blog posts
    let blog_posts = vec!["first_post".to_string()];
    // TODO - fetch and store history in local storage
    let terminal = StoredValue::new(Arc::new(Mutex::new(Terminal::new(&blog_posts, None))));
    let input_ref = NodeRef::<html::Input>::new();
    let (text, set_text) = signal(None::<ChildrenFn>);
    let (is_err, set_is_err) = signal(false);

    let handle_cmd = move |cmd: String| {
        let res = terminal.with_value(|t| {
            if let Some(path) = location_pathname() {
                t.lock()
                    .expect("should be able to unlock terminal")
                    .handle_command(&path, &cmd)
            } else {
                CommandRes::EmptyErr
            }
        });
        match res {
            CommandRes::EmptyErr => {
                set_is_err(true);
                set_text(None);
            }
            CommandRes::Err(s) => {
                set_is_err(true);
                set_text(Some(s));
            }
            CommandRes::Redirect(s) => {
                set_is_err(false);
                set_text(None);
                let navigate = use_navigate();
                navigate(&s, NavigateOptions::default());
            }
            CommandRes::Output(s) => {
                set_is_err(false);
                set_text(Some(s));
            }
            CommandRes::Nothing => {
                set_is_err(false);
                set_text(None);
            }
        }
    };

    view! {
        <header class="bg-gray-800 shadow">
            <div class="mx-auto px-4 sm:px-6 lg:px-8 py-4">
                <div class="flex items-center justify-between">
                    <h1 class="text-2xl font-bold">

                        <span class=move || {
                            if is_err() { "text-red-500" } else { "text-green-500" }
                        }>"➜"</span>
                        " "
                        <a href="/">
                            <span class="text-teal-400">
                                {move || {
                                    let pathname = use_location().pathname.get();
                                    let dir = pathname
                                        .split("/")
                                        .last()
                                        .expect("There should be at least one / in path");
                                    if dir == "" {
                                        "hansbaker.com".to_string()
                                    } else {
                                        dir.to_string()
                                    }
                                }}
                            </span>
                        </a>
                        " "
                        <a href="https://github.com/BakerNet/personal-site">
                            <span class="text-blue-400">
                                <span>"git:("</span>
                                <span class="text-red-500">"main"</span>
                                <span>")"</span>
                            </span>
                        </a>
                        ""
                        <span class="text-yellow-400">"✗"</span>
                    </h1>
                    <form
                        class="flex-1 mx-4"
                        on:submit=move |ev| {
                            ev.prevent_default();
                            let el = if let Some(el) = input_ref.get_untracked() {
                                el
                            } else {
                                set_is_err(true);
                                return;
                            };
                            handle_cmd(el.value());
                            el.set_value("");
                        }
                    >
                        <div class="relative">
                            <input
                                node_ref=input_ref
                                type="text"
                                placeholder="Type a command (try 'help')"
                                class="w-full px-4 py-2 rounded-md border border-gray-700 bg-gray-800 focus:outline-none focus:ring-2 focus:ring-blue-500"
                            />
                        </div>
                    </form>
                    <nav></nav>
                </div>
                {move || {
                    text.get()
                        .map(|s| {
                            view! {
                                <div class="mt-2 mr-4 p-2 bg-gray-700 rounded-md">
                                    <pre class="whitespace-pre-wrap">{s()}</pre>
                                </div>
                            }
                        })
                }}
            </div>
        </header>
    }
}
