use std::{collections::VecDeque, sync::Arc};

use leptos::{either::*, prelude::*};
use leptos_router::components::*;

pub struct Terminal {
    blog_posts: Vec<String>,
    history: Vec<String>,
    pointer: usize,
}

const LEN_OF_INDEX: usize = 9;
const CHAR_WIDTH: usize = 9;
const TERMINAL_MARGINS: usize = 65;
const MINES_URL: &str = "https://mines.hansbaker.com";
const HELP_TEXT: &str = r#"This is Hans Baker's personal website.  Use this terminal to navigate the site.
The commands should feel familiar:
    ls      list directory contents (sitemap)
    cd      change directory (navigate site)
    pwd     print name of the current/working directory (current URL path)
    clear   clear the terminal screen
    mines   minesweeper app"#;
const MINES_SH: &str = r#"#!/bin/bash
set -e

# https://mines.hansbaker.com
# Minesweeper client with multiplayer, replay analysis, and stat tracking
mines
"#;
const THANKS_TXT: &str = "Thank you to my wife and my daughter for bringing immense joy to my life.";

impl Terminal {
    pub fn new(blog_posts: &[String], history: Option<Vec<String>>) -> Self {
        let history = history.unwrap_or_default();
        let pointer = history.len();
        Self {
            blog_posts: blog_posts.to_owned(),
            history,
            pointer,
        }
    }

    pub fn handle_command(&mut self, path: &str, input: &str) -> CommandRes {
        self.history.push(input.to_string());
        self.reset_pointer();

        let mut parts = input.split_whitespace();
        let cmd_text = if let Some(word) = parts.next() {
            word
        } else {
            return CommandRes::EmptyErr;
        };
        let cmd = Command::from(cmd_text);
        match cmd {
            Command::Help => CommandRes::Output(Arc::new(move || HELP_TEXT.into_any())),
            Command::Pwd => {
                let path = path.to_owned();
                CommandRes::Output(Arc::new(move || view! { {path.clone()} }.into_any()))
            }
            Command::Ls => self.handle_ls(path, parts.collect()),
            Command::Cd => self.handle_cd(path, parts.collect()),
            Command::Cat => self.handle_cat(path, parts.collect()),
            Command::Clear => CommandRes::Nothing,
            Command::Mines => CommandRes::Redirect(MINES_URL.to_string()),
            Command::WhoAmI => CommandRes::Output(Arc::new(move || "user".into_any())),
            Command::Unknown => self.handle_unknown(path, cmd_text, parts.collect()),
        }
    }

    pub fn handle_tab(&mut self, path: &str, input: &str) -> Vec<String> {
        let mut parts = input.split_whitespace();
        let cmd_text = if let Some(word) = parts.next() {
            word
        } else {
            return Vec::new()
        };
        let cmd = Command::from(cmd_text);
        let mut parts = parts.peekable();
        match cmd {
            Command::Unknown if parts.peek().is_none() && !input.ends_with(" ") => if cmd_text.contains("/") {
                self.tab_opts(path, cmd_text)
            } else {
                self.tab_commands(cmd_text)
            },
            _ if parts.peek().is_none() && !input.ends_with(" ") => Vec::new(),
            Command::Cd => self.tab_opts(path, parts.last().unwrap_or_default()).into_iter().filter(|s| s.ends_with("/")).collect(),
            _ => self.tab_opts(path, parts.last().unwrap_or_default())
        }
    }

    pub fn reset_pointer(&mut self) {
        self.pointer = self.history.len();
    }

    pub fn handle_up(&mut self) -> Option<String> {
        if self.pointer > 0 {
            self.pointer -= 1;
            Some(self.history[self.pointer].clone())
        } else {
            None
        }
    }

    pub fn handle_down(&mut self) -> Option<String> {
        if self.pointer < self.history.len() {
            self.pointer += 1;
        }
        if self.pointer < self.history.len() {
            Some(self.history[self.pointer].clone())
        } else {
            None
        }
    }

    fn handle_unknown(&self, path: &str, target: &str, args: Vec<&str>) -> CommandRes {
        let target_string = target.to_owned();
        let target_path = path_target_to_target_path(path, target, false);
        let target = Target::from_str(&target_path, &self.blog_posts);
        let is_executable = matches!(target, Target::File(File::MinesSh | File::Index(_))) && target_string.contains("/"); 
        if !args.is_empty() && !is_executable {
            // only mines.sh and index.rs are executable, so only these can accept arguments
            return CommandRes::Err(Arc::new(move || format!("command not found: {}", target_string).into_any()));
        }
        match target {
            Target::Dir(_) => CommandRes::Redirect(target_path),
            Target::File(f) => {
                if target_string.ends_with("/") {
                    return CommandRes::Err(Arc::new(move || format!("not a directory: {}", target_string).into_any()));
                }
                match f {
                    File::Index(s) => {
                        CommandRes::Redirect(s)
                    }
                    File::MinesSh => {
                        if is_executable {
                            CommandRes::Redirect(MINES_URL.to_string())
                        } else {
                            CommandRes::Err(Arc::new(move || format!("command not found: {}\nhint: try 'mines' or '/mines.sh'", target_string).into_any()))
                        }
                    }
                    File::ThanksTxt => {
                        if target_string.contains("/") {
                            CommandRes::Err(Arc::new(move || format!("permission denied: {}", target_string).into_any()))
                        } else {
                            CommandRes::Err(Arc::new(move || format!("command not found: {}", target_string).into_any()))
                        }
                    }
                }
            }
            Target::Invalid => CommandRes::Err(Arc::new(move || format!("command not found: {}", target_string).into_any())),
        }
    }

    fn handle_ls(&self, path: &str, args: Vec<&str>) -> CommandRes {
        let mut all = false;
        let (options, mut targets) = args.into_iter().fold(
            (Vec::<char>::new(), Vec::<&str>::new()),
            |(mut options, mut t), s| {
                if s.starts_with("-") {
                    let mut opts = s.chars().filter(|c| *c != '-').collect::<Vec<char>>();
                    options.append(&mut opts);
                } else {
                    t.push(s);
                }
                (options, t)
            },
        );
        let invalid = options.iter().find(|c| **c != 'a');
        if let Some(c) = invalid {
            let c = c.to_owned();
            return CommandRes::Err(Arc::new(move || {
                format!(
                    r#"ls: invalid option -- '{}'
This version of ls only supports option 'a'"#,
                    c
                )
                .into_any()
            }));
        }
        if !options.is_empty() {
            all = true;
        }
        if targets.is_empty() {
            targets = vec!("");
        }
        let (targets, is_err) = targets.into_iter().fold(
            (Vec::new(), false), 
            |(mut ts, is_err), tp| {
                let target_string = tp.to_owned();
                let target_path = path_target_to_target_path(path, tp, false);
                let target = Target::from_str(&target_path, &self.blog_posts);
                let is_err = is_err || matches!(target, Target::Invalid);
                ts.push((target_string, target));
                (ts, is_err)
            },
        );
        let last = targets.len() - 1;
        let posts = self.blog_posts.clone();
        let callback = {
            let targets = targets.clone();
            let posts = posts.clone();
            move || {
                targets.iter().enumerate().map(|(i, (name, ts))| {
                    let is_invalid = matches!(ts, Target::Invalid);
                    view! {
                        {if !is_invalid && last != 0 {
                            format!("{}:\n", name)
                        } else {
                            "".to_string()
                        }}
                        {match ts {
                            Target::Dir(d) => {
                                LsView(LsViewProps {
                                        items: d.contents(&posts, all),
                                        base: d.base(),
                                    })
                                    .into_any()
                            }
                            Target::File(f) => f.name().into_any(),
                            Target::Invalid => {
                                format!("ls: cannot access '{}': No such file or directory", name)
                                    .into_any()
                            }
                        }}
                        {if i != last { if !is_invalid { "\n\n" } else { "\n" } } else { "" }}
                    } 
                }).collect_view().into_any()
            }
        };
        if is_err {
            CommandRes::Err(Arc::new(callback))
        } else {
            CommandRes::Output(Arc::new(callback))
        }
    }

    fn handle_cd(&self, path: &str, args: Vec<&str>) -> CommandRes {
        if args.len() < 2 {
            let target_path = if args.is_empty() { "/" } else { args[0] };
            let target_string = target_path.to_owned();
            let target_path = path_target_to_target_path(path, target_path, false);
            let target = Target::from_str(&target_path, &self.blog_posts);
            if target_path == path {
                return CommandRes::Nothing;
            }
            match target {
                Target::File(_) => {
                    let other = target_string.clone();
                    CommandRes::Err(Arc::new(move || {
                        format!("cd: not a directory: {}", other).into_any()
                    }))
                }
                Target::Invalid => {
                    let other = target_string.clone();
                    CommandRes::Err(Arc::new(move || {
                        format!("cd: no such file or directory: {}", other).into_any()
                    }))
                }
                _ => CommandRes::Redirect(target_path),
            }
        } else {
            CommandRes::EmptyErr
        }
    }

    fn handle_cat(&self, path: &str, args: Vec<&str>) -> CommandRes {
        let (options, targets) = args.into_iter().fold(
            (Vec::<char>::new(), Vec::<&str>::new()),
            |(mut options, mut t), s| {
                if s.starts_with("-") {
                    let mut opts = s.chars().filter(|c| *c != '-').collect::<Vec<char>>();
                    options.append(&mut opts);
                } else {
                    t.push(s);
                }
                (options, t)
            },
        );
        if !options.is_empty() {
            let c = options[0].to_owned();
            return CommandRes::Err(Arc::new(move || {
                format!(
                    r#"ls: invalid option -- '{}'
This version of cat doesn't support any options"#,
                    c
                )
                .into_any()
            }));
        }
        if targets.is_empty() {
            return CommandRes::EmptyErr;
        }
        let (targets, is_err) = targets.into_iter().fold(
            (Vec::new(), false), 
            |(mut ts, is_err), tp| {
                let target_string = tp.to_owned();
                let target_path = path_target_to_target_path(path, tp, false);
                let target = Target::from_str(&target_path, &self.blog_posts);
                let is_err = is_err || matches!(target, Target::Invalid | Target::Dir(_)) || target_string.ends_with("/");
                ts.push((target_string, target));
                (ts, is_err)
            },
        );
        let last = targets.len() -1;
        let callback = {
            let targets = targets.clone();
            move || {
                targets.iter().enumerate().map(|(i, (name, ts))| {
                    view! {
                        {match ts {
                            Target::Dir(_) => format!("cat: {}: Is a directory", name).into_any(),
                            Target::File(f) => f.contents().into_any(),
                            Target::Invalid => {
                                format!("cat: {}: No such file or directory", name).into_any()
                            }
                        }}
                        {if i != last { "\n" } else { "" }}
                    } 
                }).collect_view().into_any()
            }
        };
        if is_err {
            CommandRes::Err(Arc::new(callback))
        } else {
            CommandRes::Output(Arc::new(callback))
        }
    }

    fn tab_opts(&self, path: &str, target_path: &str) -> Vec<String> {
        let no_prefix = target_path.ends_with("/") || target_path.is_empty();
        let target_path = path_target_to_target_path(path, target_path, true);
        let (target_path, prefix) = if no_prefix {
            (target_path.as_ref(), "")
        } else if let Some(pos) = target_path.rfind("/") {
            let new_target_path = &target_path[..pos];
            let new_target_path = if new_target_path.is_empty() {"/"} else {new_target_path};
            (new_target_path, &target_path[pos+1..])
        } else {
            return Vec::new()
        };
        let target = Target::from_str(target_path, &self.blog_posts);
        match target {
            Target::Dir(d) => d.contents(&self.blog_posts, prefix.starts_with(".")).into_iter().filter(|s| s.starts_with(prefix) && s != prefix).collect(),
            _ => Vec::new(),
        }
    }

    fn tab_commands(&self, cmd_text: &str) -> Vec<String> {
        Command::all().into_iter().filter(|s| s.starts_with(cmd_text)).map(|s| s.to_string()).collect()
    }
}

fn path_target_to_target_path(path: &str, target: &str, preserve_dot: bool) -> String {
    let mut target = target;
    let ends_with_dot = target.ends_with(".");
    let mut parts = path.split("/").filter(|s| !s.is_empty()).collect::<Vec<_>>();
    while target.starts_with("./") {
        target = &target[2..];
    }
    if target.starts_with("/") {
        parts = Vec::new();
    }
    if target == "~" || target.starts_with("~/") {
        parts = Vec::new();
        target = &target[1..];
    }
    while target.ends_with("/") {
        target = &target[..target.len() - 1];
    }
    let mut target = target
        .split("/")
        .filter(|s| !s.is_empty() && *s != ".")
        .collect::<VecDeque<_>>();
    if ends_with_dot && preserve_dot {
        target.push_back(".");
    }
    while !target.is_empty() {
        let p = target.pop_front().unwrap();
        match p {
            ".." if !parts.is_empty() => {
                let _ = parts.pop();
            }
            ".." => {}
            other => parts.push(other),
        }
    }
    format!("/{}", parts.join("/"))
}

#[component]
fn LsView(items: Vec<String>, base: String) -> impl IntoView {
    let dir_class = "text-blue-400";
    let item_clone = items.clone();
    let render_func = {
        let base = base.to_owned();
        move |s: String| {
            if s.ends_with("/") {
                let s = s[..s.len()-1].to_string();
                let base = if base == "/" {
                    ""
                } else {
                    &base
                };
                let href = if s == "." {
                    base.to_string()
                } else {
                    format!("{}/{}", base.to_owned(), s)
                };
                // note - adding extra space because trimming trailing '/'
                Either::Left(view! {
                    <A href=href attr:class=dir_class>
                        {s.clone()}
                    </A>
                    " "
                })
            } else {
                Either::Right(view! { <span>{s}</span> })
            }.into_any()
        }
    };
    view! {
        <div>
            <ColumnarView items=item_clone render_func />
        </div>
    }
}

fn num_rows(num_items: usize, cols: usize) -> usize {
    let items_per_row = num_items / cols;
    if num_items % cols > 0 {
        items_per_row + 1
    } else {items_per_row}
}

#[component]
pub fn ColumnarView<F>(items: Vec<String>, render_func: F) -> impl IntoView 
where
    F: Fn(String) -> AnyView
{
    let available_space = window().inner_width().expect("should be able to get window width").as_f64().expect("window width should be a number").round() as usize - TERMINAL_MARGINS;
    let available_space = available_space / CHAR_WIDTH;
    let total_len = items.iter().map(|s| s.len() + 2).sum::<usize>();
    if total_len < available_space {
        return view! {
            {items
                .into_iter()
                .map(|s| {
                    view! {
                        {render_func(s)}
                        "  "
                    }
                })
                .collect_view()}
        }.into_any();
    }
    let max_cols = 10.min(items.len());
    let mut cols = 1;
    for n in 0..max_cols {
        let n = max_cols - n;
        let per_col = num_rows(items.len(), n);
        let total_len = items.chunks(per_col).map(|v| v.iter().map(|s| s.len() +2).max().expect("there should be a max len for each column")).sum::<usize>();
        if total_len < available_space {
            cols = n;
            break;
        }
    }
    let rows = num_rows(items.len(), cols);
    let item_cols = items.chunks(rows).map(|x| x.to_vec()).collect::<Vec<Vec<String>>>();
    let col_lens = item_cols.iter().map(|v| v.iter().map(|s| s.len() +2).max().expect("there should be a max len for each column")).collect::<Vec<_>>();
    let views = (0..rows).into_iter().map(|row| 
        view! {
            <div>
                {item_cols
                    .iter()
                    .zip(col_lens.iter())
                    .filter(|(v, _)| row < v.len())
                    .map(|(v, l)| (&v[row], l))
                    .map(|(s, l)| {
                        let fill = l - s.len();
                        view! {
                            {render_func(s.to_string())}
                            {" ".repeat(fill)}
                        }
                    })
                    .collect_view()}
            </div>
        }
    ).collect::<Vec<_>>();
    view! { {views} }.into_any()
}

fn blog_post_exists(name: &str, blog_posts: &[String]) -> bool {
    let name = if let Some(stripped) = name.strip_prefix("/blog/") {
        stripped
    } else {
        name
    };
    blog_posts.iter().any(|s| *s == name)
}

#[derive(Debug, Clone)]
enum Dir {
    Base,
    Blog,
    CV,
    BlogPost(String),
}

impl Dir {
    fn contents(&self, blog_posts: &[String], all: bool) -> Vec<String> {
        let mut common = if all {
            vec!["./".to_string(), "../".to_string(), "index.rs".to_string()]
        } else {
            vec!["index.rs".to_string()]
        };
        match self {
            Dir::Base => {
                let mut items = vec!["blog/".to_string(), "cv/".to_string(), "mines.sh".to_string(), "thanks.txt".to_string()];
                items.append(&mut common);
                items.sort();
                if all {
                    // './' should come before '../'
                    items.swap(0,1);
                }
                items
            }
            Dir::Blog => {
                let mut posts = blog_posts.iter().map(|bp| format!("{}/", bp)).collect::<Vec<_>>();
                posts.append(&mut common);
                posts.sort();
                if all {
                    // './' should come before '../'
                    posts.swap(0,1);
                }
                posts
            },
            Dir::CV => common,
            Dir::BlogPost(_) => common,
        }
    }

    fn base(&self) -> String {
        match self {
            Dir::Base => "/".into(),
            Dir::Blog => "/blog".into(),
            Dir::CV => "/cv".into(),
            Dir::BlogPost(s) => format!("/blog/{}", s),
        }
    }
}

#[derive(Debug, Clone)]
enum File {
    MinesSh,
    ThanksTxt,
    Index(String)
}

impl File {
    fn name(&self) -> &'static str {
        match self {
            File::MinesSh => "mines.sh",
            File::ThanksTxt => "thanks.txt",
            File::Index(_) => "index.rs",
        }
    }

    fn contents(&self) -> String {
        match self {
            File::MinesSh => MINES_SH.to_string(),
            File::ThanksTxt => THANKS_TXT.to_string(),
            File::Index(s) => {
                let s = if s.is_empty() {"/"} else{s};
                format!(r#"use leptos::prelude::*;
use leptos_router::{{hooks::use_navigate, UseNavigateOptions}};

func main() {{
    Effect::new((_) => {{
        let navigate = use_navigate();
        navigate("{}", UseNavigateOptions::default);
    }})
}}
"#, s)
            }
        }
    }
}

// TODO - refactor to file or directory & enum for each
#[derive(Debug, Clone)]
enum Target {
    Dir(Dir),
    File(File),
    Invalid,
}

impl Target {
    fn from_str(path: &str, blog_posts: &[String]) -> Self {
        match path {
            "/" => Self::Dir(Dir::Base),
            "/blog" => Self::Dir(Dir::Blog),
            "/cv" => Self::Dir(Dir::CV),
            post if post.starts_with("/blog/") && blog_post_exists(post, blog_posts) => {
                let blog_post_name = post.split("/").last().expect("all blog posts should contain a /");
                Self::Dir(Dir::BlogPost(blog_post_name.to_string()))
            }
            "/mines.sh" => Self::File(File::MinesSh),
            "/thanks.txt" => Self::File(File::ThanksTxt),
            "/index.rs" | "/blog/index.rs" | "/cv/index.rs" => Self::File(File::Index(path[..path.len() -LEN_OF_INDEX].to_string())),
            post_index
                if post_index.starts_with("/blog/")
                    && post_index.ends_with("/index.rs")
                    && blog_post_exists(&post_index[..post_index.len() - LEN_OF_INDEX], blog_posts) =>
            {
                Self::File(File::Index(path[..path.len()-LEN_OF_INDEX].to_string()))
            }
            _ => Self::Invalid,
        }
    }
}

pub enum CommandRes {
    EmptyErr,
    Err(ChildrenFn),
    Redirect(String),
    Output(ChildrenFn),
    Nothing,
}

enum Command {
    Help,
    Pwd,
    Ls,
    Cd,
    Cat,
    Clear,
    Mines,
    WhoAmI,
    Unknown,
}

impl From<&str> for Command {
    fn from(value: &str) -> Self {
        match value {
            "help" => Self::Help,
            "pwd" => Self::Pwd,
            "ls" => Self::Ls,
            "cd" => Self::Cd,
            "cat" => Self::Cat,
            "clear" => Self::Clear,
            "mines" => Self::Mines,
            "whoami" => Self::WhoAmI,
            _ => Self::Unknown,
        }
    }
}

impl Command {
    fn all() -> Vec<&'static str> {
        vec!["help", "pwd", "ls", "cd", "cat", "clear", "mines", "whoami"]
    }
}
