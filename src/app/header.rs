use std::sync::{Arc, Mutex};

#[cfg(feature = "hydrate")]
use std::collections::VecDeque;

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

use super::terminal::fs::{DirContentItem, Target};
use super::terminal::{ColumnarView, CommandRes, Terminal};

#[component]
fn MobileFloatingButton(on_click: impl Fn() + 'static) -> impl IntoView {
    view! {
        <button
            class="fixed bottom-6 left-6 z-50 group active:scale-95"
            on:click=move |_| on_click()
            aria-label="Return to terminal input"
            title="Return to terminal input"
        >
            // Always-visible directional caret, but refined
            <div class="absolute -top-4 left-1/2 transform -translate-x-1/2">
                <svg
                    width="22"
                    height="14"
                    viewBox="0 0 22 14"
                    fill="none"
                    class="text-cyan/70"
                >
                    <path
                        d="M11 2L4 9L6 11L11 6L16 11L18 9L11 2Z"
                        fill="currentColor"
                    />
                </svg>
            </div>

            // Main button optimized for touch
            <div class="relative w-14 h-14 bg-background/95 border-2 border-cyan text-cyan rounded-2xl shadow-lg backdrop-blur-md transition-all duration-150 ease-out active:shadow-xl active:shadow-cyan/20 focus:outline-none focus:ring-2 focus:ring-cyan focus:ring-offset-2 focus:ring-offset-background">
                <div class="flex items-center justify-center w-full h-full">
                    <span class="text-lg font-mono font-bold">">_"</span>
                </div>
            </div>
        </button>
    }
}

#[component]
fn InputSection(
    input_ref: NodeRef<html::Input>,
    input_value: ReadSignal<String>,
    set_input_value: WriteSignal<String>,
    cursor_position: ReadSignal<usize>,
    set_cursor_position: WriteSignal<usize>,
    ghost_text: ReadSignal<Option<String>>,
    is_err: ReadSignal<bool>,
    keydown_handler: impl Fn(KeyboardEvent) + 'static,
    input_handler: impl Fn(Event) + 'static,
    submit_handler: impl Fn() + 'static,
    aria_describedby: &'static str,
) -> impl IntoView {
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

    view! {
        <div class="flex flex-col sm:flex-row sm:items-center gap-3 sm:gap-4">
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
                    submit_handler();
                }
            >
                <div class="relative group">
                    <input
                        node_ref=input_ref
                        prop:value=move || input_value.get()
                        on:keydown=keydown_handler
                        on:input=move |ev| {
                            let value = event_target_value(&ev);
                            set_input_value.set(value);
                            input_handler(ev);
                        }
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
                        aria-describedby=aria_describedby
                        class="w-full px-4 py-2 rounded-md border focus:outline-none focus:ring-2 focus:ring-cyan bg-background text-foreground placeholder-muted transition-all duration-200 ease-out hover:border-subtle focus:border-cyan focus:shadow-lg focus:shadow-cyan/20 font-mono caret-transparent empty-placeholder"
                    />
                    <div id=aria_describedby class="sr-only">
                        "Type terminal commands like 'help', 'ls', 'cd /blog', or 'neofetch'. Use Tab for autocomplete and arrow keys for history. Right arrow to accept suggestions."
                    </div>
                    <div class="absolute inset-y-0 left-0 px-4 py-2 pointer-events-none overflow-hidden flex items-center text-foreground terminal-overlay whitespace-nowrap">
                        {move || {
                            let curr = input_value.get();
                            let cursor_pos = cursor_position.get();
                            if curr.is_empty() {
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
                                            if cursor_position.get() >= input_value.get().len() {
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
    }
}

#[derive(Debug, Clone)]
struct TabState {
    cursor: usize,
    opts: Arc<Vec<DirContentItem>>,
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
    let floating_input_ref = NodeRef::<html::Input>::new();
    let header_ref = NodeRef::<html::Header>::new();
    let (output_history, set_output_history) =
        signal(Arc::new(Mutex::new(Vec::<ChildrenFn>::new())));
    let (is_err, set_is_err) = signal(false);
    let (tab_state, set_tab_state) = signal(None::<TabState>);
    let (hist_state, set_hist_state) = signal(None::<HistState>);
    let (input_value, set_input_value) = signal(String::new());
    let (cursor_position, set_cursor_position) = signal(0usize);
    let (ghost_text, set_ghost_text) = signal(None::<String>);
    #[allow(unused_variables)]
    let (is_sticky, set_is_sticky) = signal(false);

    // Mobile detection (narrow screens < 768px OR short screens < 600px)
    #[allow(unused_variables)]
    let (is_mobile, set_is_mobile) = signal(false);

    #[cfg(feature = "hydrate")]
    {
        // Check initial screen size
        let check_mobile_size = move || {
            let win = window();
            if let (Ok(width), Ok(height)) = (win.inner_width(), win.inner_height()) {
                if let (Some(width), Some(height)) = (width.as_f64(), height.as_f64()) {
                    // Mobile if narrow (< 768px) OR short (< 600px) - covers landscape phones
                    let is_mobile_size = width < 768.0 || height < 600.0;
                    set_is_mobile.set(is_mobile_size);
                }
            }
        };

        check_mobile_size();

        // Listen for window resize
        let resize_handle = window_event_listener(leptos::ev::resize, move |_| {
            check_mobile_size();
        });
        on_cleanup(move || resize_handle.remove());
    }

    #[cfg(feature = "hydrate")]
    let (cmd_history, set_cmd_history, _) =
        use_local_storage::<VecDeque<String>, JsonSerdeWasmCodec>("cmd_history");

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

    // Scroll detection for floating header
    #[cfg(feature = "hydrate")]
    {
        let handle = window_event_listener(leptos::ev::scroll, move |_| {
            if let Some(input_el) = input_ref.get() {
                // Check if the original input field is scrolled off the top
                let input_rect = input_el.get_bounding_client_rect();

                // Show floating header when input would be off-screen
                // Small buffer to make transition feel natural
                let should_be_sticky = input_rect.top() < 10.0;
                set_is_sticky.set(should_be_sticky);
            }
        });
        on_cleanup(move || handle.remove());
    }

    // Function to scroll to top
    let scroll_to_top = move || {
        #[cfg(feature = "hydrate")]
        {
            window().scroll_to_with_x_and_y(0.0, 0.0);
        }
    };

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

    // Shared submit handler
    let shared_submit_handler = move || {
        // Scroll to top if in sticky mode to show output
        if is_sticky.get_untracked() {
            scroll_to_top();
        }
        handle_cmd(input_value.get_untracked(), false);
        // Clear input
        set_input_value(String::new());
        set_cursor_position(0);
        set_ghost_text(None);
    };

    // Shared floating submit handler
    let floating_submit_handler = move || {
        // Scroll to top when submitting from floating header
        scroll_to_top();

        // Focus the static input after scroll
        #[cfg(feature = "hydrate")]
        if let Some(static_input) = input_ref.get_untracked() {
            let _ = static_input.focus();
        }

        handle_cmd(input_value.get_untracked(), false);
        // Clear input
        set_input_value(String::new());
        set_cursor_position(0);
        set_ghost_text(None);
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
        move |_ev: Event| {
            let input_val = input_value.get_untracked();

            // Track cursor position
            if let Some(input_el) = input_ref.get_untracked() {
                if let Some(pos) = input_el.selection_start().unwrap_or(None) {
                    set_cursor_position.set(pos as usize);
                }
            }

            if !input_val.is_empty() {
                let history = cmd_history.get_untracked();
                // Find the first command in history that starts with current input
                if let Some(suggestion) = history
                    .iter()
                    .rev()
                    .find(|cmd| cmd.starts_with(&input_val) && cmd.len() > input_val.len())
                {
                    // Show the full remaining part of the suggestion
                    let remaining = &suggestion[input_val.len()..];
                    set_ghost_text.set(Some(remaining.to_string()));
                } else {
                    set_ghost_text.set(None);
                }
            } else {
                set_ghost_text.set(None);
            }
        }
    };

    // Helper function to handle scroll and focus transfer from floating to static input
    let handle_scroll_and_focus_transfer = move || {
        // Check if we're in floating mode and need to transfer focus
        #[allow(unused_variables)]
        let is_floating_focused = floating_input_ref
            .get_untracked()
            .map(|floating_el| {
                #[cfg(feature = "hydrate")]
                {
                    document().active_element().as_ref() == Some(floating_el.as_ref())
                }
                #[cfg(not(feature = "hydrate"))]
                {
                    false
                }
            })
            .unwrap_or(false);

        // Scroll to top if in sticky mode
        if is_sticky.get_untracked() {
            scroll_to_top();

            // If we were focused on floating input, transfer focus to static input
            if is_floating_focused {
                #[cfg(feature = "hydrate")]
                if let Some(static_input) = input_ref.get_untracked() {
                    let _ = static_input.focus();
                }
            }
        }
    };

    let keydown_handler = move |ev: KeyboardEvent| {
        // Get the currently focused input (original or floating)
        let el = if let Some(floating_el) = floating_input_ref.get_untracked() {
            if document().active_element().as_ref() == Some(floating_el.as_ref()) {
                floating_el
            } else if let Some(original_el) = input_ref.get_untracked() {
                original_el
            } else {
                set_is_err(true);
                return;
            }
        } else if let Some(original_el) = input_ref.get_untracked() {
            original_el
        } else {
            set_is_err(true);
            return;
        };
        if ev.ctrl_key() && ev.key() == "c" {
            handle_cmd(el.value(), true);
            el.set_value("");
            set_input_value(String::new());
            set_cursor_position(0);
            set_hist_state(None);
            set_tab_state(None);
            return;
        }
        if ev.ctrl_key() && ev.key() == "l" {
            handle_cmd("clear".to_string(), false);
            el.set_value("");
            set_input_value(String::new());
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

                handle_scroll_and_focus_transfer();
                let current_val = el.value();
                let HistState {
                    cursor,
                    opts,
                    mut index,
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

                // Find the next different command going backwards
                let mut new_val = &(*opts)[index - 1];
                index -= 1;
                while index > 0 && new_val == &current_val {
                    index -= 1;
                    new_val = &(*opts)[index];
                }

                // If we only found duplicates all the way to the start, don't move
                if new_val == &current_val && index == 0 {
                    return;
                }

                el.set_value(new_val);
                set_input_value.set(new_val.clone());
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

                handle_scroll_and_focus_transfer();
                if !is_cycling_hist {
                    return;
                }
                ev.prevent_default();
                let current_val = el.value();
                let HistState {
                    cursor,
                    opts,
                    mut index,
                } = hist_state.get_untracked().unwrap();

                // Find the next different command going forwards
                index += 1;
                while index < opts.len() && (*opts)[index] == current_val {
                    index += 1;
                }

                if index >= opts.len() {
                    let val = el.value();
                    let truncated = &val[..cursor];
                    el.set_value(truncated);
                    set_input_value.set(truncated.to_string());
                    set_cursor_position.set(truncated.len()); // Set cursor to end of truncated value
                    set_hist_state(None);
                    return;
                }

                let new_val = &(*opts)[index];
                el.set_value(new_val);
                set_input_value.set(new_val.clone());
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
                let input_value_val = input_value.get_untracked();

                if ghost.is_some()
                    && !is_tabbing
                    && !is_cycling_hist
                    && current_pos >= input_value_val.len()
                {
                    ev.prevent_default();
                    let current_val = el.value();
                    let new_val = format!("{current_val}{}", ghost.unwrap());
                    el.set_value(&new_val);
                    set_input_value.set(new_val.clone());
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

                handle_scroll_and_focus_transfer();
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
                    let new = tab_replace(&val[..cursor], &opts[new_index].0);
                    el.set_value(&new);
                    set_input_value.set(new.clone());
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
                        let new = tab_replace(&val, &opts[0].0);
                        el.set_value(&new);
                        set_input_value.set(new.clone());
                        set_cursor_position.set(new.len()); // Set cursor to end after tab completion
                        return;
                    }
                    let cursor = val.len();
                    let index = if is_shift {
                        let i = opts.len() - 1;
                        let new = tab_replace(&val[..cursor], &opts[i].0);
                        el.set_value(&new);
                        set_input_value.set(new.clone());
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
        move |item: &DirContentItem, active: bool| {
            let s = &item.0;
            let target = &item.1;
            let is_dir = matches!(target, Target::Dir(_));
            let is_ex = target.is_executable();
            let has_suffix = s.ends_with("/") || s.ends_with("*");

            let s_display = if !active && has_suffix {
                s[..s.len() - 1].to_string()
            } else {
                s.to_owned()
            };
            let s_completion = s.to_owned();

            let handle_click = {
                let tab_replace = tab_replace;
                let input_ref = input_ref;
                let set_tab_state = set_tab_state;
                let set_input_value = set_input_value;
                move |_| {
                    if let Some(el) = input_ref.get_untracked() {
                        let current_val = el.value();
                        let new_val = tab_replace(&current_val, &s_completion);
                        el.set_value(&new_val);
                        set_input_value.set(new_val); // Update cursor position
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
        <>
            <header
                node_ref=header_ref
                class="shadow-lg border-b border-muted/30"
            >
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
                }} <InputSection
                        input_ref=input_ref
                        input_value=input_value
                        set_input_value=set_input_value
                        cursor_position=cursor_position
                        set_cursor_position=set_cursor_position
                        ghost_text=ghost_text
                        is_err=is_err
                        keydown_handler=keydown_handler
                        input_handler=input_handler
                        submit_handler=shared_submit_handler
                        aria_describedby="terminal-help"
                    />
                {move || {
                    let tab_state = tab_state.get();
                    tab_state
                        .map(|ts| {
                            let selected = ts
                                .opts
                                .iter()
                                .enumerate()
                                .find_map(|(vi, item)| {
                                    if Some(vi) == ts.index { Some(item.to_owned()) } else { None }
                                });
                            let render_func = move |item: DirContentItem| {
                                let is_sel = selected.as_ref().map(|s| &s.0) == Some(&item.0);
                                auto_comp_item(&item, is_sel).into_any()
                            };
                            view! {
                                <div class="mt-2 p-3 rounded-md bg-black/30 border border-muted/40 backdrop-blur-sm">
                                    <pre class="whitespace-pre-wrap terminal-output">
                                        <ColumnarView items=ts.opts.to_vec() render_func />
                                    </pre>
                                </div>
                            }
                        })
                }}
            </div>
            </header>

            // Floating overlay - mobile button or desktop header
            {move || {
                let sticky = is_sticky.get();
                let mobile = is_mobile.get();

                if !sticky {
                    // Not scrolled, don't show anything
                    EitherOf3::A(())
                } else if mobile {
                    // Mobile: show floating button
                    EitherOf3::B(view! {
                        <MobileFloatingButton
                            on_click=move || {
                                scroll_to_top();
                                #[cfg(feature = "hydrate")]
                                if let Some(static_input) = input_ref.get_untracked() {
                                    let _ = static_input.focus();
                                }
                            }
                        />
                    })
                } else {
                    // Desktop: show full floating header
                    EitherOf3::C(view! {
                        <div class="fixed top-0 left-0 right-0 z-50 bg-background/95 backdrop-blur-md shadow-lg border-b border-muted/30">
                            <div class="mx-auto px-3 sm:px-4 md:px-6 lg:px-8 py-3 sm:py-4">
                                <InputSection
                                    input_ref=floating_input_ref
                                    input_value=input_value
                                    set_input_value=set_input_value
                                    cursor_position=cursor_position
                                    set_cursor_position=set_cursor_position
                                    ghost_text=ghost_text
                                    is_err=is_err
                                    keydown_handler=keydown_handler
                                    input_handler=input_handler
                                    submit_handler=floating_submit_handler
                                    aria_describedby="terminal-help-floating"
                                />
                            </div>
                        </div>
                    })
                }
            }}
        </>
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
