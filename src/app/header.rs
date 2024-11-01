use std::sync::{Arc, Mutex};

use leptos::{either::Either, ev::KeyboardEvent, html, logging, prelude::*};
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
    let (last_cmd, set_last_cmd) = signal(None::<ChildrenFn>);
    let (text, set_text) = signal(None::<ChildrenFn>);
    let (is_err, set_is_err) = signal(false);
    let (tab_state, set_tab_state) = signal(None::<(Vec<String>, usize)>);

    let dir_from_pathname = |pathname: String| {
        let dir = pathname
            .split("/")
            .last()
            .expect("There should be at least one / in path");
        if dir.is_empty() {
            "hansbaker.com".to_string()
        } else {
            dir.to_string()
        }
    };

    let ps1 = move |is_err: bool, path: &str, with_links: bool| {
        view! {
            <span class=move || {
                if is_err { "text-red-500" } else { "text-green-500" }
            }>"➜"</span>
            " "
            {if with_links {
                Either::Left(
                    view! {
                        <a href="/">
                            <span class="text-teal-400">{path.to_string()}</span>
                        </a>
                    },
                )
            } else {
                Either::Right(view! { <span class="text-teal-400">{path.to_string()}</span> })
            }}
            " "

            {if with_links {
                Either::Left(
                    view! {
                        <a href="https://github.com/BakerNet/personal-site">
                            <span class="text-blue-400">
                                <span>"git:("</span>
                                <span class="text-red-500">"main"</span>
                                <span>")"</span>
                            </span>
                        </a>
                    },
                )
            } else {
                Either::Right(
                    view! {
                        <span class="text-blue-400">
                            <span>"git:("</span>
                            <span class="text-red-500">"main"</span>
                            <span>")"</span>
                        </span>
                    },
                )
            }}
            ""
            <span class="text-yellow-400">"✗"</span>
        }
    };

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

        let was_err = is_err.get_untracked();
        let prev_pathname = use_location().pathname.get();
        let prev_dir = dir_from_pathname(prev_pathname);

        if cmd.trim() != "clear" {
            set_last_cmd(Some(Arc::new(move || {
                view! {
                    {ps1(was_err, &prev_dir, false)}
                    " "
                    {cmd.to_string()}
                }
                .into_any()
            })));
        } else {
            set_last_cmd(None);
        }

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

    let keydown_handler = move |ev: KeyboardEvent| {
        let el = if let Some(el) = input_ref.get_untracked() {
            el
        } else {
            set_is_err(true);
            return;
        };
        match ev.key().as_ref() {
            "ArrowUp" => {
                ev.prevent_default();
                let new_text = terminal.with_value(|t| {
                    t.lock()
                        .expect("should be able to access terminal")
                        .handle_up()
                });
                if let Some(nt) = new_text {
                    el.set_value(&nt);
                }
            }
            "ArrowDown" => {
                ev.prevent_default();
                let new_text = terminal.with_value(|t| {
                    t.lock()
                        .expect("should be able to access terminal")
                        .handle_down()
                });
                if let Some(nt) = new_text {
                    el.set_value(&nt);
                } else {
                    el.set_value("");
                }
            }
            "Tab" => {
                if let Some((opts, pointer)) = tab_state.get_untracked() {
                    ev.prevent_default();
                    // todo
                } else {
                    let val = el.value();
                    if val == "" {
                        return;
                    }
                    let path = if let Some(p) = location_pathname() {
                        p
                    } else {
                        return;
                    };
                    ev.prevent_default();
                    let opts = terminal.with_value(|t| {
                        t.lock()
                            .expect("should be able to access terminal")
                            .handle_tab(&path, &val)
                    });
                    logging::log!("{:?}", opts);
                    // todo
                }
            }
            _ => terminal.with_value(|t| {
                t.lock()
                    .expect("should be able to access terminal")
                    .reset_pointer();
            }),
        }
    };

    view! {
        <header class="bg-gray-800 shadow">
            <div class="mx-auto px-4 sm:px-6 lg:px-8 py-4">
                <div class="flex items-center justify-between">
                    <h1 class="text-2xl font-bold">
                        {move || {
                            let err = is_err.get();
                            let pathname = use_location().pathname.get();
                            let dir = dir_from_pathname(pathname);
                            ps1(err, &dir, true)
                        }}

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
                                on:keydown=keydown_handler
                                type="text"
                                placeholder="Type a command (try 'help')"
                                class="w-full px-4 py-2 rounded-md border border-gray-700 bg-gray-800 focus:outline-none focus:ring-2 focus:ring-blue-500"
                            />
                        </div>
                    </form>
                    <nav></nav>
                </div>
                {move || {
                    let text = text.get();
                    let last_cmd = last_cmd.get();
                    if text.is_none() && last_cmd.is_none() {
                        None
                    } else {
                        Some(
                            view! {
                                <div class="mt-2 mr-4 p-2 bg-gray-700 rounded-md">
                                    <pre class="whitespace-pre-wrap">
                                        {last_cmd.map(|s| { s() })} <br /> {text.map(|s| { s() })}

                                    </pre>
                                </div>
                            },
                        )
                    }
                }}
            </div>
        </header>
    }
}
