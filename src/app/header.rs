use std::sync::{Arc, Mutex};

use leptos::{either::*, ev::KeyboardEvent, html, prelude::*};
use leptos_router::{
    components::*,
    hooks::{use_location, use_navigate},
    NavigateOptions,
};

#[cfg(feature = "hydrate")]
use codee::string::JsonSerdeWasmCodec;
#[cfg(feature = "hydrate")]
use leptos_use::storage::use_local_storage;

use crate::blog::Assets;

use super::terminal::{ColumnarView, CommandRes, Terminal};

#[derive(Debug, Clone)]
struct TabState {
    cursor: usize,
    opts: Arc<Vec<String>>,
    index: Option<usize>,
}

#[derive(Debug, Clone)]
struct HistState {
    cursor: usize,
    opts: Arc<Vec<String>>,
    index: usize,
}

#[component]
pub fn Header() -> impl IntoView {
    // TODO - actually get blog posts
    let blog_posts = Assets::iter()
        .map(|s| s[..s.len() - 3].to_string())
        .collect::<Vec<_>>();
    let terminal = StoredValue::new(Arc::new(Mutex::new(Terminal::new(&blog_posts, None))));
    let input_ref = NodeRef::<html::Input>::new();
    let (output_history, set_output_history) =
        signal(Arc::new(Mutex::new(Vec::<ChildrenFn>::new())));
    let (is_err, set_is_err) = signal(false);
    let (tab_state, set_tab_state) = signal(None::<TabState>);
    let (hist_state, set_hist_state) = signal(None::<HistState>);

    #[cfg(feature = "hydrate")]
    let (cmd_history, set_cmd_history, _) =
        use_local_storage::<Vec<String>, JsonSerdeWasmCodec>("cmd_history");

    #[cfg(feature = "hydrate")]
    Effect::watch(
        || (),
        move |_, _, _| {
            let history = cmd_history.get_untracked();
            terminal.with_value(|t| {
                t.lock()
                    .expect("should be able to unlock terminal")
                    .set_history(history);
            });
        },
        true,
    );

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

    let handle_cmd = move |cmd: String, force_err: bool| {
        let history_vec = set_output_history.write();
        let mut history_vec = history_vec.lock().expect("should be able to acquire lock");

        if cmd.trim() != "clear" {
            // add copy of current ps1 to history
            let was_err = is_err.get_untracked();
            let prev_pathname = use_location().pathname.get();
            let prev_dir = dir_from_pathname(prev_pathname);
            let cmd = cmd.clone();
            history_vec.push(Arc::new(move || {
                view! {
                    <div>
                        <Ps1 is_err=was_err path=prev_dir.clone() with_links=false />
                        " "
                        {cmd.clone()}
                    </div>
                }
                .into_any()
            }));
        } else {
            history_vec.clear();
        }

        if force_err {
            // user used Ctrl+C
            set_is_err(true);
            return;
        }

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
            }
            CommandRes::Err(s) => {
                set_is_err(true);
                history_vec.push(s);
            }
            CommandRes::Redirect(s) => {
                set_is_err(false);
                let navigate = use_navigate();
                navigate(&s, NavigateOptions::default());
            }
            CommandRes::Output(s) => {
                set_is_err(false);
                history_vec.push(s);
            }
            CommandRes::Nothing => {
                set_is_err(false);
            }
        }

        #[cfg(feature = "hydrate")]
        terminal.with_value(|t| {
            set_cmd_history.set(
                t.lock()
                    .expect("should be able to unlock terminal")
                    .history(),
            );
        });
    };

    let tab_replace = move |val: &str, new: &str| {
        let new = if let Some(s) = new.strip_suffix("*") {
            s
        } else {
            new
        };
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
        let el = if let Some(el) = input_ref.get_untracked() {
            el
        } else {
            set_is_err(true);
            return;
        };
        if ev.ctrl_key() && ev.key() == "c" {
            handle_cmd(el.value(), true);
            el.set_value("");
            set_hist_state(None);
            set_tab_state(None);
            return;
        }
        if ev.ctrl_key() && ev.key() == "l" {
            handle_cmd("clear".to_string(), false);
            el.set_value("");
            set_hist_state(None);
            set_tab_state(None);
            return;
        }
        if ev.meta_key() || ev.alt_key() || ev.ctrl_key() {
            return;
        }

        let is_tabbing = tab_state.get_untracked().is_some();
        let is_cycling_hist = hist_state.get_untracked().is_some();

        match ev.key().as_ref() {
            "ArrowUp" => {
                // cycle history prev
                ev.prevent_default();
                if is_tabbing {
                    set_tab_state(None);
                }
                let HistState {
                    cursor,
                    opts,
                    index,
                } = if is_cycling_hist {
                    hist_state.get_untracked().unwrap()
                } else {
                    // initialize state
                    let val = el.value();
                    let v = terminal.with_value(|t| {
                        t.lock()
                            .expect("should be able to access terminal")
                            .handle_start_hist(&val)
                    });
                    let cursor = val.len();
                    let i = v.len();
                    HistState {
                        cursor,
                        opts: v.into(),
                        index: i,
                    }
                };
                if index == 0 {
                    return;
                }
                let index = index - 1;
                let new_val = &(*opts)[index];
                el.set_value(new_val);
                set_hist_state(Some(HistState {
                    cursor,
                    opts,
                    index,
                }));
            }
            "ArrowDown" => {
                // cycle history next
                if is_tabbing {
                    set_tab_state(None);
                }
                if !is_cycling_hist {
                    return;
                }
                ev.prevent_default();
                let HistState {
                    cursor,
                    opts,
                    index,
                } = hist_state.get_untracked().unwrap();
                let index = index + 1;
                if index == opts.len() {
                    let val = el.value();
                    el.set_value(&val[..cursor]);
                    set_hist_state(None);
                    return;
                }
                let new_val = &(*opts)[index];
                el.set_value(new_val);
                set_hist_state(Some(HistState {
                    cursor,
                    opts,
                    index,
                }));
            }
            "Tab" => {
                let val = el.value();
                if val.is_empty() {
                    return;
                }
                ev.prevent_default();
                let is_shift = ev.shift_key();
                if is_tabbing {
                    // cycle tab options
                    let TabState {
                        cursor,
                        opts,
                        index,
                    } = tab_state
                        .get_untracked()
                        .expect("is tabbing but no tab state");
                    let new_index = match (index, is_shift) {
                        (None, false) => 0,
                        (Some(i), false) => (i + 1) % opts.len(),
                        (None, true) | (Some(0), true) => opts.len() - 1,
                        (Some(i), true) => i - 1,
                    };
                    let new = tab_replace(&val[..cursor], &opts[new_index]);
                    el.set_value(&new);
                    set_tab_state(Some(TabState {
                        cursor,
                        opts,
                        index: Some(new_index),
                    }));
                } else {
                    // initialize state
                    let path = if let Some(p) = location_pathname() {
                        p
                    } else {
                        return;
                    };
                    let opts = terminal.with_value(|t| {
                        t.lock()
                            .expect("should be able to access terminal")
                            .handle_start_tab(&path, &val)
                    });
                    if opts.is_empty() {
                        return;
                    };
                    if opts.len() == 1 {
                        let new = tab_replace(&val, &opts[0]);
                        el.set_value(&new);
                        return;
                    }
                    let cursor = val.len();
                    let index = if is_shift {
                        let i = opts.len() - 1;
                        let new = tab_replace(&val[..cursor], &opts[i]);
                        el.set_value(&new);
                        Some(i)
                    } else {
                        None
                    };
                    set_tab_state(Some(TabState {
                        cursor,
                        opts: opts.into(),
                        index,
                    }));
                }
            }
            "Shift" => {} // don't reset state on empty shift
            _ => {
                if is_tabbing {
                    set_tab_state(None);
                }
                if is_cycling_hist {
                    set_hist_state(None);
                }
            }
        }
    };

    let auto_comp_item = |s: &str, active: bool| {
        let is_dir = s.ends_with("/");
        let is_ex = s.ends_with("*");
        let s = if !active && (is_dir || is_ex) {
            s[..s.len() - 1].to_string()
        } else {
            s.to_owned()
        };
        view! {
            <span class=if active {
                "bg-white text-black"
            } else {
                ""
            }>
                {if !active && is_dir {
                    EitherOf3::A(
                        view! {
                            <span class="text-blue">{s}</span>
                            "/"
                        },
                    )
                } else if !active && is_ex {
                    EitherOf3::B(
                        view! {
                            <span class="text-green">{s}</span>
                            "*"
                        },
                    )
                } else {
                    EitherOf3::C(s)
                }}
            </span>
        }
    };

    view! {
        <header class="shadow-lg">
            <div class="mx-auto px-4 sm:px-6 lg:px-8 py-4">
                {move || {
                    let history = output_history.get();
                    let views = {
                        let history = history.lock().expect("should be able to acquire lock");
                        history.iter().map(|s| s()).collect_view()
                    };
                    if views.is_empty() {
                        None
                    } else {
                        Some(
                            view! {
                                <div class="flex flex-col-reverse max-h-[480px] overflow-y-auto mb-2 p-2 rounded-md">
                                    <pre class="whitespace-pre-wrap">{views}</pre>
                                </div>
                            },
                        )
                    }
                }} <div class="flex flex-wrap items-center justify-between">
                    <div class="text-2xl font-bold mr-4">
                        {move || {
                            let err = is_err.get();
                            let pathname = use_location().pathname.get();
                            let dir = dir_from_pathname(pathname);
                            view! { <Ps1 is_err=err path=dir with_links=true /> }
                        }}

                    </div>
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
                            handle_cmd(el.value(), false);
                            el.set_value("");
                        }
                    >
                        <div class="relative">
                            <input
                                node_ref=input_ref
                                on:keydown=keydown_handler
                                type="text"
                                placeholder="Type a command (try 'help')"
                                // autocorrect="off"
                                autocapitalize="none"
                                class="w-full px-4 py-2 rounded-md border focus:outline-none focus:ring-2 focus:ring-brightBlack bg-background text-foreground"
                            />
                        </div>
                    </form>
                    <nav></nav>
                </div>
                {move || {
                    let tab_state = tab_state.get();
                    if tab_state.is_none() {
                        None
                    } else {
                        Some(
                            view! {
                                <div class="mt-2 p-2 rounded-md">
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
                                                }
                                            })}
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

#[component]
fn Ps1(is_err: bool, path: String, with_links: bool) -> impl IntoView {
    let path_git = view! {
        <span class="text-cyan">{path.to_string()}</span>
        " "
        <span class="text-blue">
            <span>"git:("</span>
            <span class="text-red">"main"</span>
            <span>")"</span>
        </span>
    };
    view! {
        <span class=move || { if is_err { "text-red" } else { "text-green" } }>"➜"</span>
        " "
        {if with_links {
            Either::Left(view! { <A href="/">{path_git}</A> })
        } else {
            Either::Right(path_git)
        }}
        ""
        <span class="text-yellow">"✗"</span>
    }
}
