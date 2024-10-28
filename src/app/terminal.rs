use std::{collections::VecDeque, sync::Arc};

use leptos::{logging, prelude::*};

pub struct Terminal {
    blog_posts: Vec<String>,
    history: Vec<String>,
    pointer: usize,
}

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
            Command::Help => CommandRes::Output(Arc::new(move || {
                {
                    view! {

                        r#"This is Hans Baker's personal website.  Use this terminal to navigate the site.
The commands should feel familiar:
    ls      list directory contents (sitemap)
    cd      change directory (navigate site)
    pwd     print name of the current/working directory (current URL path)
    clear   clear the terminal screen
    mines   minesweeper app"#
                    }
                    .into_any()
                }
            })),
            Command::Pwd => {
                let path = path.to_owned();
                CommandRes::Output(Arc::new(move || view! {{path.clone()}}.into_any()))
            }
            Command::Ls => self.handle_ls(path, parts.collect()),
            Command::Cd => todo!(),
            Command::Cat => todo!(),
            Command::Clear => CommandRes::Nothing,
            Command::Mines => CommandRes::Redirect("https://mines.hansbaker.com".to_string()),
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
        todo!()
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
        // TODO - handle multiple targets
        let target = if targets.len() == 0 { "" } else { targets[0] };
        let target = path_target_to_target(&path, target);
        match target.as_ref() {
            "" => CommandRes::Output(Arc::new(move || base_ls_view(all))),
            "blog" => CommandRes::Output(Arc::new({
                let posts = self.blog_posts.clone();
                move || blog_ls_view(&posts, all)
            })),
            "cv" => CommandRes::Output(Arc::new(move || empty_ls_view(all))),
            "mines.sh" => CommandRes::Output(Arc::new(move || "mines.sh".into_any())),
            "thanks.txt" => CommandRes::Output(Arc::new(move || "thanks.txt".into_any())),
            post if post.starts_with("blog/") && blog_post_exists(post, &self.blog_posts) => {
                CommandRes::Output(Arc::new(move || empty_ls_view(all)))
            }
            "index.rs" | "blog/index.rs" | "cv/index.rs" => {
                CommandRes::Output(Arc::new(move || "index.rs".into_any()))
            }
            post_index
                if post_index.starts_with("blog/")
                    && post_index.ends_with("/index.rs")
                    && blog_post_exists(&post_index[..post_index.len() - 9], &self.blog_posts) =>
            {
                CommandRes::Output(Arc::new(move || "index.rs".into_any()))
            }
            other => {
                let other = other.to_owned();
                CommandRes::Err(Arc::new(move || {
                    format!("ls: cannot access '{}': No such file or directory", other).into_any()
                }))
            }
        }
    }
}

fn path_target_to_target(path: &str, target: &str) -> String {
    let mut target = target;
    let mut parts = path.split("/").filter(|s| *s != "").collect::<Vec<_>>();
    while target.starts_with("./") {
        target = &target[2..];
    }
    if target.starts_with("/") {
        parts = Vec::new();
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
    parts.join("/")
}

fn base_ls_view(all: bool) -> AnyView {
    let dir_class = "text-blue-400";
    view!{{if all {".  ..  "} else {""}}<a href="/blog" class=dir_class>"blog"</a>"  "<a href="/cv" class=dir_class>"cv"</a>"  index.rs  mines.sh  thanks.txt"}.into_any()
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
    view!{{if all {".  ..  "} else {""}}{blog_posts.iter().map(|title| view!{<a href=format!("/blog/{}", title) class=dir_class>"first_post"</a>"  "}).collect_view()}"index.rs"}.into_any()
}

fn blog_post_exists(name: &str, blog_posts: &[String]) -> bool {
    let name = if name.starts_with("blog/") {
        &name[5..]
    } else {
        name
    };
    logging::log!("checking for blog_post: {}", name);
    blog_posts.iter().find(|s| *s == name).is_some()
}

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
            "" => Self::Base,
            "blog" => Self::Blog,
            "cv" => Self::CV,
            "mines.sh" => Self::MinesSh,
            "thanks.txt" => Self::ThanksTxt,
            post if post.starts_with("blog/") && blog_post_exists(post, blog_posts) => {
                Self::BlogPost(path.to_string())
            }
            "index.rs" | "blog/index.rs" | "cv/index.rs" => Self::Index(path.to_string()),
            post_index
                if post_index.starts_with("blog/")
                    && post_index.ends_with("/index.rs")
                    && blog_post_exists(&post_index[..post_index.len() - 9], blog_posts) =>
            {
                Self::Index(path.to_string())
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
            "mines.sh" => Self::Mines,
            _ => Self::Unknown,
        }
    }
}
