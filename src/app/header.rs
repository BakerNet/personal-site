use std::sync::{Arc, Mutex};

use leptos::{
    either::*,
    ev::{Event, KeyboardEvent},
    html,
    prelude::*,
};
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
    let (current_input, set_current_input) = signal(String::new());
    let (cursor_position, set_cursor_position) = signal(0usize);
    let (ghost_text, set_ghost_text) = signal(None::<String>);

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
                CommandRes::new().with_error()
            }
        });

        match res {
            CommandRes::Output {
                is_err,
                stdout_view,
                stdout_text,
                stderr_text,
            } => {
                set_is_err(is_err);
                // Convert stderr text to view with consistent error styling
                if let Some(stderr_msg) = stderr_text {
                    if !stderr_msg.is_empty() {
                        let error_view = Arc::new(move || {
                            view! { <div class="text-red whitespace-pre-wrap">{stderr_msg.clone()}</div> }
                            .into_any()
                        });
                        history_vec.push(error_view);
                    }
                }
                // Use stdout_view if available, otherwise convert stdout_text to view
                if let Some(view) = stdout_view {
                    history_vec.push(view);
                } else if let Some(stdout_msg) = stdout_text {
                    if !stdout_msg.is_empty() {
                        let text_view = Arc::new(move || {
                            view! { <div class="whitespace-pre-wrap" inner_html=stdout_msg.clone()></div> }
                            .into_any()
                        });
                        history_vec.push(text_view);
                    }
                }
            }
            CommandRes::Redirect(s) => {
                set_is_err(false);
                let navigate = use_navigate();
                navigate(&s, NavigateOptions::default());
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
        if let Some(i) = val.rfind(['/', ' ']) {
            let prefix = &val[..i + 1];
            format!("{prefix}{new}")
        } else {
            new.to_string()
        }
    };

    #[cfg(not(feature = "hydrate"))]
    let input_handler = move |_ev: Event| {};
    // Handle input changes for ghost text suggestions
    #[cfg(feature = "hydrate")]
    let input_handler = {
        move |ev: Event| {
            let input_value = event_target_value(&ev);
            set_current_input.set(input_value.clone());

            // Track cursor position
            if let Some(input_el) = input_ref.get_untracked() {
                if let Some(pos) = input_el.selection_start().unwrap_or(None) {
                    set_cursor_position.set(pos as usize);
                }
            }

            if !input_value.is_empty() {
                let history = cmd_history.get_untracked();
                // Find the first command in history that starts with current input
                if let Some(suggestion) = history
                    .iter()
                    .rev()
                    .find(|cmd| cmd.starts_with(&input_value) && cmd.len() > input_value.len())
                {
                    // Show the full remaining part of the suggestion
                    let remaining = &suggestion[input_value.len()..];
                    set_ghost_text.set(Some(remaining.to_string()));
                } else {
                    set_ghost_text.set(None);
                }
            } else {
                set_ghost_text.set(None);
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
        if ev.ctrl_key() && ev.key() == "c" {
            handle_cmd(el.value(), true);
            el.set_value("");
            set_current_input(String::new());
            set_cursor_position(0);
            set_hist_state(None);
            set_tab_state(None);
            return;
        }
        if ev.ctrl_key() && ev.key() == "l" {
            handle_cmd("clear".to_string(), false);
            el.set_value("");
            set_current_input(String::new());
            set_cursor_position(0);
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
                set_ghost_text.set(None); // Clear ghost text during history navigation
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
                set_current_input.set(new_val.clone());
                set_cursor_position.set(new_val.len()); // Set cursor to end of new value
                set_hist_state(Some(HistState {
                    cursor,
                    opts,
                    index,
                }));
            }
            "ArrowDown" => {
                // cycle history next
                set_ghost_text.set(None); // Clear ghost text during history navigation
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
                    let truncated = &val[..cursor];
                    el.set_value(truncated);
                    set_current_input.set(truncated.to_string());
                    set_cursor_position.set(truncated.len()); // Set cursor to end of truncated value
                    set_hist_state(None);
                    return;
                }
                let new_val = &(*opts)[index];
                el.set_value(new_val);
                set_current_input.set(new_val.clone());
                set_cursor_position.set(new_val.len()); // Set cursor to end of new value
                set_hist_state(Some(HistState {
                    cursor,
                    opts,
                    index,
                }));
            }
            "ArrowRight" => {
                // Accept ghost text suggestion only if cursor is at end
                let ghost = ghost_text.get_untracked();
                let current_pos = cursor_position.get_untracked();
                let current_input_val = current_input.get_untracked();

                if ghost.is_some()
                    && !is_tabbing
                    && !is_cycling_hist
                    && current_pos >= current_input_val.len()
                {
                    ev.prevent_default();
                    let current_val = el.value();
                    let new_val = format!("{current_val}{}", ghost.unwrap());
                    el.set_value(&new_val);
                    set_current_input.set(new_val.clone());
                    set_cursor_position.set(new_val.len()); // Set cursor to end after accepting ghost text
                    set_ghost_text.set(None);
                }
            }
            "Tab" => {
                let val = el.value();
                if val.is_empty() {
                    return;
                }
                ev.prevent_default();
                set_ghost_text.set(None); // Clear ghost text during tab completion
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
                    set_current_input.set(new.clone());
                    set_cursor_position.set(new.len()); // Set cursor to end after tab completion
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
                        set_current_input.set(new.clone());
                        set_cursor_position.set(new.len()); // Set cursor to end after tab completion
                        return;
                    }
                    let cursor = val.len();
                    let index = if is_shift {
                        let i = opts.len() - 1;
                        let new = tab_replace(&val[..cursor], &opts[i]);
                        el.set_value(&new);
                        set_current_input.set(new.clone());
                        set_cursor_position.set(new.len()); // Set cursor to end after tab completion
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
            "/" => {
                // Special handling for '/' while tabbing on directories
                if is_tabbing {
                    let val = el.value();
                    if val.ends_with('/') {
                        // Current completion ends with '/', stop tabbing instead of adding another '/'
                        ev.prevent_default();
                        set_tab_state(None);
                        return;
                    }
                }
                // Default behavior for other cases
                if is_tabbing {
                    set_tab_state(None);
                }
                if is_cycling_hist {
                    set_hist_state(None);
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

    let auto_comp_item = {
        move |s: &str, active: bool| {
            let is_dir = s.ends_with("/");
            let is_ex = s.ends_with("*");
            let s_display = if !active && (is_dir || is_ex) {
                s[..s.len() - 1].to_string()
            } else {
                s.to_owned()
            };
            let s_completion = s.to_owned();

            let handle_click = {
                let tab_replace = tab_replace;
                let input_ref = input_ref;
                let set_tab_state = set_tab_state;
                let set_current_input = set_current_input;
                move |_| {
                    if let Some(el) = input_ref.get_untracked() {
                        let current_val = el.value();
                        let new_val = tab_replace(&current_val, &s_completion);
                        el.set_value(&new_val);
                        set_current_input.set(new_val); // Update cursor position
                        set_tab_state(None);
                        // Focus the input after completion
                        let _ = el.focus();
                    }
                }
            };

            view! {
                <span
                    class=if active {
                        "bg-cyan text-black px-2 py-1 rounded-sm shadow-md transition-all duration-150 cursor-pointer"
                    } else {
                        "hover:bg-brightBlack/50 px-2 py-1 rounded-sm transition-all duration-150 cursor-pointer hover:bg-cyan/20"
                    }
                    on:click=handle_click
                >
                    {if !active && is_dir {
                        EitherOf3::A(
                            view! {
                                <span class="text-blue font-medium">{s_display}</span>
                                <span class="text-muted">"/"</span>
                            },
                        )
                    } else if !active && is_ex {
                        EitherOf3::B(
                            view! {
                                <span class="text-green font-medium">{s_display}</span>
                                <span class="text-muted">"*"</span>
                            },
                        )
                    } else {
                        EitherOf3::C(
                            view! {
                                <span class=if active {
                                    "font-medium"
                                } else {
                                    "text-foreground"
                                }>{s_display}</span>
                            },
                        )
                    }}
                </span>
            }
        }
    };

    view! {
        <header class="shadow-lg border-b border-muted/30">
            <div class="mx-auto px-3 sm:px-4 md:px-6 lg:px-8 py-3 sm:py-4">
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
                                <div class="flex flex-col-reverse max-h-[480px] overflow-y-auto mb-2 p-3 rounded-md bg-black/20 border border-muted/30 backdrop-blur-sm">
                                    <pre class="whitespace-pre-wrap terminal-output leading-tight">
                                        {views}
                                    </pre>
                                </div>
                            },
                        )
                    }
                }} <div class="flex flex-col sm:flex-row sm:items-center gap-3 sm:gap-4">
                    <div class="text-lg sm:text-xl lg:text-2xl font-bold min-w-0 flex-shrink-0">
                        {move || {
                            let err = is_err.get();
                            let pathname = use_location().pathname.get();
                            let dir = dir_from_pathname(pathname);
                            view! { <Ps1 is_err=err path=dir with_links=true /> }
                        }}

                    </div>
                    <form
                        class="flex-1 min-w-0 sm:min-w-64"
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
                            set_current_input.set(String::new());
                            set_cursor_position.set(0);
                            set_ghost_text.set(None);
                        }
                    >
                        <div class="relative group">
                            <input
                                node_ref=input_ref
                                on:keydown=keydown_handler
                                on:input=input_handler
                                on:keyup=move |_| {
                                    if let Some(input_el) = input_ref.get_untracked() {
                                        if let Some(pos) = input_el
                                            .selection_start()
                                            .unwrap_or(None)
                                        {
                                            set_cursor_position.set(pos as usize);
                                        }
                                    }
                                }
                                on:click=move |_| {
                                    if let Some(input_el) = input_ref.get_untracked() {
                                        if let Some(pos) = input_el
                                            .selection_start()
                                            .unwrap_or(None)
                                        {
                                            set_cursor_position.set(pos as usize);
                                        }
                                    }
                                }
                                type="text"
                                placeholder="Type a command (try 'help')"
                                autocapitalize="none"
                                aria-label="Terminal command input"
                                aria-describedby="terminal-help"
                                class="w-full px-4 py-2 rounded-md border focus:outline-none focus:ring-2 focus:ring-cyan bg-background text-foreground placeholder-muted transition-all duration-200 ease-out hover:border-subtle focus:border-cyan focus:shadow-lg focus:shadow-cyan/20 font-mono caret-transparent empty-placeholder"
                            />
                            <div id="terminal-help" class="sr-only">
                                "Type terminal commands like 'help', 'ls', 'cd /blog', or 'neofetch'. Use Tab for autocomplete and arrow keys for history. Right arrow to accept suggestions."
                            </div>
                            {}
                            <div class="absolute inset-y-0 left-0 px-4 py-2 pointer-events-none overflow-hidden flex items-center text-foreground terminal-overlay whitespace-nowrap">
                                {move || {
                                    let curr = current_input.get();
                                    let cursor_pos = cursor_position.get();
                                    if curr.is_empty() {

                                        // Empty input, show cursor at start
                                        view! {
                                            <span class="relative empty-placeholder">
                                                <span class="absolute terminal-block-cursor bg-cyan font-mono">
                                                    " "
                                                </span>
                                            </span>
                                        }
                                            .into_any()
                                    } else {
                                        let before_cursor = &curr[..cursor_pos.min(curr.len())];
                                        let after_cursor = &curr[cursor_pos.min(curr.len())..];
                                        // Split text at cursor position

                                        view! {
                                            <>
                                                <span class="invisible font-mono whitespace-pre empty-placeholder">
                                                    {before_cursor}
                                                </span>
                                                <span class="relative empty-placeholder">
                                                    <span class="absolute terminal-block-cursor bg-cyan font-mono">
                                                        " "
                                                    </span>
                                                </span>
                                                <span class="invisible font-mono whitespace-pre empty-placeholder">
                                                    {after_cursor}
                                                </span>
                                                {move || {
                                                    if cursor_position.get() >= current_input.get().len() {
                                                        // Only show ghost text if cursor is at the end
                                                        view! {
                                                            <span class="text-muted/70 font-mono whitespace-pre empty-placeholder">
                                                                {ghost_text.get().unwrap_or_default()}
                                                            </span>
                                                        }
                                                            .into_any()
                                                    } else {
                                                        view! { <span></span> }.into_any()
                                                    }
                                                }}
                                            </>
                                        }
                                            .into_any()
                                    }
                                }}
                            </div>
                            <div class="absolute inset-y-0 right-3 flex items-center pointer-events-none">
                                <span class="text-muted text-sm opacity-60 group-hover:opacity-80 transition-opacity duration-200">
                                    "▶"
                                </span>
                            </div>
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
                                <div class="mt-2 p-3 rounded-md bg-black/30 border border-muted/40 backdrop-blur-sm">
                                    <pre class="whitespace-pre-wrap terminal-output">
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
        " "
        <span class="text-yellow">"✗"</span>
    }
}
