use std::sync::{Arc, Mutex};

use leptos::{either::Either, ev::KeyboardEvent, html, prelude::*};
use leptos_router::{
    components::*,
    hooks::{use_location, use_navigate},
    NavigateOptions,
};

use super::terminal::{ColumnarView, CommandRes, Terminal};

#[derive(Debug, Clone)]
struct TabState {
    cursor: usize,
    opts: Arc<Vec<String>>,
    index: Option<usize>,
}

#[component]
pub fn Header() -> impl IntoView {
    // TODO - actually get blog posts
    let blog_posts = vec![
        "first_post".to_string(),
        "second_post".to_string(),
        "third_post".to_string(),
        "fourth_post".to_string(),
        "fifth_post".to_string(),
        "sixth_with_long_name_post".to_string(),
        "seventh_with_also_long_name_post".to_string(),
        "eighth_post".to_string(),
        "ninth_post".to_string(),
        "tenth_post".to_string(),
        "eleventh_post".to_string(),
        "twelfth_woohoo_post".to_string(),
        "thirteenth_post".to_string(),
        "fourteenth_with_the_longest_name_post".to_string(),
        "fifteenth_post".to_string(),
        "sixteenth_post".to_string(),
        "last_post".to_string(),
    ];
    // TODO - fetch and store history in local storage
    let terminal = StoredValue::new(Arc::new(Mutex::new(Terminal::new(&blog_posts, None))));
    let input_ref = NodeRef::<html::Input>::new();
    let (last_cmd, set_last_cmd) = signal(None::<ChildrenFn>);
    let (text, set_text) = signal(None::<ChildrenFn>);
    let (is_err, set_is_err) = signal(false);
    let (is_tabbing, set_is_tabbing) = signal(false);
    let (tab_state, set_tab_state) = signal(None::<TabState>);

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
                Either::Left({
                    let path = path.to_string();
                    view! {
                        <A href="/" attr:class="text-teal-400">
                            {path}
                        </A>
                    }
                })
            } else {
                Either::Right(view! { <span class="text-teal-400">{path.to_string()}</span> })
            }}
            " "

            {if with_links {
                Either::Left(
                    view! {
                        <A href="https://github.com/BakerNet/personal-site">
                            <span class="text-blue-400">
                                <span>"git:("</span>
                                <span class="text-red-500">"main"</span>
                                <span>")"</span>
                            </span>
                        </A>
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

    let tab_replace = move |val: &str, new: &str| {
        if let Some(i) = val.rfind("/") {
            let prefix = &val[..i + 1];
            format!("{}{}", prefix, new)
        } else if let Some(i) = val.rfind(" ") {
            let prefix = &val[..i + 1];
            format!("{}{}", prefix, new)
        } else {
            new.to_string()
        }
    };

    let keydown_handler = move |ev: KeyboardEvent| {
        if ev.meta_key() || ev.alt_key() || ev.ctrl_key() {
            return;
        }
        let el = if let Some(el) = input_ref.get_untracked() {
            el
        } else {
            set_is_err(true);
            return;
        };

        match ev.key().as_ref() {
            "ArrowUp" => {
                if is_tabbing.get_untracked() {
                    set_is_tabbing(false);
                    set_tab_state(None);
                }
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
                if is_tabbing.get_untracked() {
                    set_is_tabbing(false);
                    set_tab_state(None);
                }
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
                if is_tabbing.get_untracked() {
                    let val = el.value();
                    if val.is_empty() {
                        return;
                    }
                    ev.prevent_default();
                    let TabState {
                        cursor,
                        opts,
                        index,
                    } = tab_state
                        .get_untracked()
                        .expect("is tabbing but no tab state");
                    let new_index = match index {
                        None => 0,
                        Some(i) => (i + 1) % opts.len(),
                    };
                    let new = tab_replace(&val[..cursor], &opts[new_index]);
                    el.set_value(&new);
                    set_tab_state(Some(TabState {
                        cursor,
                        opts,
                        index: Some(new_index),
                    }));
                } else {
                    let val = el.value();
                    if val.is_empty() {
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
                    if opts.is_empty() {
                        return;
                    };
                    if opts.len() == 1 {
                        let new = tab_replace(&val, &opts[0]);
                        el.set_value(&new);
                        return;
                    }
                    set_is_tabbing(true);
                    let cursor = val.len();
                    set_tab_state(Some(TabState {
                        cursor,
                        opts: opts.into(),
                        index: None,
                    }));
                }
            }
            _ => terminal.with_value(|t| {
                if is_tabbing.get_untracked() {
                    set_is_tabbing(false);
                    set_tab_state(None);
                }
                t.lock()
                    .expect("should be able to access terminal")
                    .reset_pointer();
            }),
        }
    };

    let auto_comp_item = |s: &str, active: bool| {
        let is_dir = s.ends_with("/");
        let s = if !active && is_dir {
            s[..s.len() - 1].to_string()
        } else {
            s.to_owned()
        };
        view! {
            <span class=if active {
                "bg-gray-200 text-gray-900"
            } else {
                ""
            }>
                {if !active && is_dir {
                    Either::Left(
                        view! {
                            <span class="text-blue-400">{s}</span>
                            "/"
                        },
                    )
                } else {
                    Either::Right(s)
                }}
            </span>
        }
    };

    view! {
        <header class="bg-gray-800 shadow">
            <div class="mx-auto px-4 sm:px-6 lg:px-8 py-4">
                <div class="flex flex-wrap items-center justify-between">
                    <h1 class="text-2xl font-bold mr-4">
                        {move || {
                            let err = is_err.get();
                            let pathname = use_location().pathname.get();
                            let dir = dir_from_pathname(pathname);
                            ps1(err, &dir, true)
                        }}

                    </h1>
                    <form
                        class="flex-1 min-w-64"
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
                    let tab_state = tab_state.get();
                    if text.is_none() && last_cmd.is_none() && tab_state.is_none() {
                        None
                    } else {
                        Some(
                            view! {
                                <div class="mt-2 p-2 bg-gray-700 rounded-md">
                                    <pre class="whitespace-pre-wrap">
                                        {tab_state
                                            .map(|ts| {
                                                let selected = ts
                                                    .opts
                                                    .iter()
                                                    .enumerate()
                                                    .find_map(|(vi, s)| {
                                                        if Some(vi) == ts.index { Some(s.to_owned()) } else { None }
                                                    });
                                                let render_func = move |s: String| {
                                                    let is_sel = Some(&s) == selected.as_ref();
                                                    auto_comp_item(&s, is_sel).into_any()
                                                };
                                                view! {
                                                    <ColumnarView items=ts.opts.to_vec() render_func />
                                                    <br />
                                                }
                                            })}
                                        {last_cmd
                                            .map(|s| {
                                                view! {
                                                    {s()}
                                                    <br />
                                                }
                                            })} {text.map(|s| { s() })}

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
