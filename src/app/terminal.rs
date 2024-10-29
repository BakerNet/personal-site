use std::{collections::VecDeque, sync::Arc};

use leptos::{logging, prelude::*};

pub struct Terminal {
    blog_posts: Vec<String>,
    history: Vec<String>,
    pointer: usize,
}

const LEN_OF_INDEX: usize = 9;
const MINES_URL: &'static str = "https://mines.hansbaker.com";
const HELP_TEXT: &'static str = r#"This is Hans Baker's personal website.  Use this terminal to navigate the site.
The commands should feel familiar:
    ls      list directory contents (sitemap)
    cd      change directory (navigate site)
    pwd     print name of the current/working directory (current URL path)
    clear   clear the terminal screen
    mines   minesweeper app"#;

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

    pub fn handle_command(&mut self, path: &str, cmd: &str) -> CommandRes {
        self.history.push(cmd.to_string());
        self.pointer = self.history.len();

        let mut parts = cmd.split_whitespace();
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
            Command::Cat => todo!(),
            Command::Clear => CommandRes::Nothing,
            Command::Mines => CommandRes::Redirect(MINES_URL.to_string()),
            Command::Unknown => self.handle_unknown(path, cmd_text),
        }
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
            Some(self.history[self.pointer - 1].clone())
        } else {
            None
        }
    }

    fn handle_unknown(&self, path: &str, target: &str) -> CommandRes {
        let target_string = target.to_owned();
        let target_path = path_target_to_target_path(&path, target);
        let target = Target::from_str(&target_path, &self.blog_posts);
        logging::log!("unknown: {}", target_string);
        match target {
            Target::Base | Target::Blog | Target::CV => CommandRes::Redirect(target_path),
            Target::BlogPost(s) => CommandRes::Redirect(s),
            Target::Index(s) => {
                if target_string.ends_with("/") {
                    CommandRes::Err(Arc::new(move || format!("not a directory: {}", target_string).into_any()))
                } else {
                    CommandRes::Redirect(s)
                }
            }
            Target::MinesSh => {
                if target_string.ends_with("/") {
                    CommandRes::Err(Arc::new(move || format!("not a directory: {}", target_string).into_any()))
                } else if target_string.contains("/") {
                    CommandRes::Redirect(MINES_URL.to_string())
                } else {
                    CommandRes::Err(Arc::new(move || format!("command not found: {}\nhint: try 'mines' or '/mines.sh'", target_string).into_any()))
                }
            }
            Target::ThanksTxt => {
                if target_string.ends_with("/") {
                    CommandRes::Err(Arc::new(move || format!("not a directory: {}", target_string).into_any()))
                } else if target_string.contains("/") {
                    CommandRes::Err(Arc::new(move || format!("permission denied: {}", target_string).into_any()))
                } else {
                    CommandRes::Err(Arc::new(move || format!("command not found: {}", target_string).into_any()))
                }
            }
            Target::Invalid => CommandRes::Err(Arc::new(move || format!("command not found: {}", target_string).into_any())),
        }
    }

    fn handle_ls(&self, path: &str, args: Vec<&str>) -> CommandRes {
        let mut all = false;
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
        if targets.len() < 2 {
            let target_path = if targets.len() == 0 { "" } else { targets[0] };
            let target_string = target_path.to_owned();
            let target_path = path_target_to_target_path(&path, target_path);
            let target = Target::from_str(&target_path, &self.blog_posts);
            match target {
                Target::Base => CommandRes::Output(Arc::new(move || base_ls_view(all))),
                Target::Blog => CommandRes::Output(Arc::new({
                    let posts = self.blog_posts.clone();
                    move || blog_ls_view(&posts, all)
                })),
                Target::CV => CommandRes::Output(Arc::new(move || empty_ls_view(all))),
                Target::MinesSh => CommandRes::Output(Arc::new(move || "mines.sh".into_any())),
                Target::ThanksTxt => CommandRes::Output(Arc::new(move || "thanks.txt".into_any())),
                Target::BlogPost(_) => CommandRes::Output(Arc::new(move || empty_ls_view(all))),
                Target::Index(_) => CommandRes::Output(Arc::new(move || "index.rs".into_any())),
                Target::Invalid => {
                    let other = target_string.clone();
                    CommandRes::Err(Arc::new(move || {
                        format!("ls: cannot access '{}': No such file or directory", other).into_any()
                    }))
                }
            }
        } else {
            let (targets, is_err) = targets.into_iter().fold(
                (Vec::new(), false), 
                |(mut ts, is_err), tp| {
                    let target_string = tp.to_owned();
                    let target_path = path_target_to_target_path(&path, tp);
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
                            {if !is_invalid { format!("{}:\n", name) } else { "".to_string() }}
                            {match ts {
                                Target::Base => base_ls_view(all),
                                Target::Blog => blog_ls_view(&posts, all),
                                Target::CV => empty_ls_view(all),
                                Target::MinesSh => "mines.sh".into_any(),
                                Target::ThanksTxt => "thanks.txt".into_any(),
                                Target::BlogPost(_) => empty_ls_view(all),
                                Target::Index(_) => "index.rs".into_any(),
                                Target::Invalid => {
                                    format!(
                                        "ls: cannot access '{}': No such file or directory",
                                        name,
                                    )
                                        .into_any()
                                }
                            }}
                            {if i != last && !is_invalid { "\n\n" } else { "\n" }}
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
    }

    fn handle_cd(&self, path: &str, args: Vec<&str>) -> CommandRes {
        if args.len() < 2 {
            let target_path = if args.len() == 0 { "/" } else { args[0] };
            let target_string = target_path.to_owned();
            let target_path = path_target_to_target_path(&path, target_path);
            let target = Target::from_str(&target_path, &self.blog_posts);
            if target_path == path {
                return CommandRes::Nothing;
            }
            match target {
                Target::MinesSh | Target::ThanksTxt | Target::Index(_) => {
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
                Target::BlogPost(s) => CommandRes::Redirect(s),
                _ => CommandRes::Redirect(target_path),
            }
        } else {
            CommandRes::EmptyErr
        }
    }
}

fn path_target_to_target_path(path: &str, target: &str) -> String {
    let mut target = target;
    let mut parts = path.split("/").filter(|s| *s != "").collect::<Vec<_>>();
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
        .filter(|s| *s != "" && *s != ".")
        .collect::<VecDeque<_>>();
    while target.len() > 0 {
        let p = target.pop_front().unwrap();
        match p {
            ".." if parts.len() > 0 => {
                let _ = parts.pop();
            }
            ".." => {}
            other => parts.push(other),
        }
    }
    logging::log!("parts: {:?}", parts);
    logging::log!("target: {:?}", target);
    format!("/{}", parts.join("/"))
}

fn base_ls_view(all: bool) -> AnyView {
    let dir_class = "text-blue-400";
    view! {
        {if all { ".  ..  " } else { "" }}
        <a href="/blog" class=dir_class>
            "blog"
        </a>
        "  "
        <a href="/cv" class=dir_class>
            "cv"
        </a>
        "  index.rs  mines.sh  thanks.txt"
    }.into_any()
}

fn empty_ls_view(all: bool) -> AnyView {
    if all {
        ".  ..  index.rs".into_any()
    } else {
        "index.rs".into_any()
    }
}

fn blog_ls_view(blog_posts: &[String], all: bool) -> AnyView {
    let dir_class = "text-blue-400";
    view! {
        {if all { ".  ..  " } else { "" }}
        {blog_posts
            .iter()
            .map(|title| {
                view! {
                    <a href=format!("/blog/{}", title) class=dir_class>
                        "first_post"
                    </a>
                    "  "
                }
            })
            .collect_view()}
        "index.rs"
    }.into_any()
}

fn blog_post_exists(name: &str, blog_posts: &[String]) -> bool {
    let name = if name.starts_with("/blog/") {
        &name[6..]
    } else {
        name
    };
    logging::log!("checking for blog_post: {}", name);
    blog_posts.iter().find(|s| *s == name).is_some()
}

// TODO - refactor to file or directory & enum for each
#[derive(Debug, Clone)]
enum Target {
    Base,
    Blog,
    BlogPost(String),
    CV,
    MinesSh,
    ThanksTxt,
    Index(String),
    Invalid,
}

impl Target {
    fn from_str(path: &str, blog_posts: &[String]) -> Self {
        match path {
            "/" => Self::Base,
            "/blog" => Self::Blog,
            "/cv" => Self::CV,
            "/mines.sh" => Self::MinesSh,
            "/thanks.txt" => Self::ThanksTxt,
            post if post.starts_with("/blog/") && blog_post_exists(post, blog_posts) => {
                Self::BlogPost(path.to_string())
            }
            "/index.rs" | "/blog/index.rs" | "/cv/index.rs" => Self::Index(path[..path.len() -LEN_OF_INDEX].to_string()),
            post_index
                if post_index.starts_with("/blog/")
                    && post_index.ends_with("/index.rs")
                    && blog_post_exists(&post_index[..post_index.len() - LEN_OF_INDEX], blog_posts) =>
            {
                Self::Index(path[..path.len()-LEN_OF_INDEX].to_string())
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
            _ => Self::Unknown,
        }
    }
}
