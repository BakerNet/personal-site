use std::collections::VecDeque;

const LEN_OF_NAV: usize = 7;
const MINES_SH: &str = r#"#!/bin/bash
set -e

# https://mines.hansbaker.com
# Minesweeper client with multiplayer, replay analysis, and stat tracking
mines
"#;
const THANKS_TXT: &str =
    "Thank you to my wife and my daughter for bringing immense joy to my life.";
// TODO - actually implement HIST_SAVE_NO_DUPS and histsize 1000
// TODO - implement ls -l
const ZSHRC_CONTENT: &str = r#"# Simple zsh configuration
unsetopt beep
setopt HIST_SAVE_NO_DUPS

# Basic completion
autoload -Uz compinit
compinit

# plugins
plugins = (zsh-autosuggestions, zsh-history-substring-search)

# Aliases
alias ll='ls -la'
alias la='ls -a'
alias h='history'

# robbyrussell theme prompt
# Arrow changes color based on exit status, directory in cyan, git status
PROMPT='%(?:%{$fg_bold[green]%}➜ :%{$fg_bold[red]%}➜ )%{$fg[cyan]%}%c%{$reset_color%} $(git_prompt_info)'

ZSH_THEME_GIT_PROMPT_PREFIX="%{$fg_bold[blue]%}git:(%{$fg[red]%}"
ZSH_THEME_GIT_PROMPT_SUFFIX="%{$reset_color%} "
ZSH_THEME_GIT_PROMPT_DIRTY="%{$fg[blue]%}) %{$fg[yellow]%}✗"
ZSH_THEME_GIT_PROMPT_CLEAN="%{$fg[blue]%})"

# History settings
HISTFILE=window.localStorage
HISTSIZE=1000
SAVEHIST=1000
setopt SHARE_HISTORY
setopt APPEND_HISTORY

# zsh-history-substring-search configuration
bindkey '^[[A' history-substring-search-up # or '\eOA'
bindkey '^[[B' history-substring-search-down # or '\eOB'
HISTORY_SUBSTRING_SEARCH_ENSURE_UNIQUE=1
HISTORY_SUBSTRING_SEARCH_HIGHLIGHT_FOUND=0
HISTORY_SUBSTRING_SEARCH_HIGHLIGHT_NOT_FOUND=0
"#;

pub fn parse_multitarget(args: Vec<&str>) -> (Vec<char>, Vec<&str>) {
    args.into_iter().fold(
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
    )
}

pub fn path_target_to_target_path(path: &str, target: &str, preserve_dot: bool) -> String {
    let mut target = target;
    let ends_with_dot = target.ends_with(".");
    let mut parts = path
        .split("/")
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
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

#[derive(Debug, Clone)]
pub enum Dir {
    Base,
    Blog,
    CV,
    BlogPost(String),
}

impl Dir {
    pub fn contents(&self, blog_posts: &[String], all: bool) -> Vec<String> {
        let mut common = if all {
            vec!["./".to_string(), "../".to_string(), "nav.rs*".to_string()]
        } else {
            vec!["nav.rs*".to_string()]
        };
        match self {
            Dir::Base => {
                let mut items = vec![
                    "blog/".to_string(),
                    "cv/".to_string(),
                    "mines.sh*".to_string(),
                    "thanks.txt".to_string(),
                ];
                items.append(&mut common);
                items.sort();
                if all {
                    // './' should come before '../'
                    items.swap(0, 1);
                }
                items
            }
            Dir::Blog => {
                let mut posts = blog_posts
                    .iter()
                    .map(|bp| format!("{bp}/"))
                    .collect::<Vec<_>>();
                posts.append(&mut common);
                posts.sort();
                if all {
                    // './' should come before '../'
                    posts.swap(0, 1);
                }
                posts
            }
            Dir::CV => common,
            Dir::BlogPost(_) => common,
        }
    }

    pub fn base(&self) -> String {
        match self {
            Dir::Base => "/".into(),
            Dir::Blog => "/blog".into(),
            Dir::CV => "/cv".into(),
            Dir::BlogPost(s) => format!("/blog/{s}"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum File {
    MinesSh,
    ThanksTxt,
    ZshRc,
    ZshHistory,
    Nav(String),
}

impl File {
    pub fn name(&self) -> &'static str {
        match self {
            File::MinesSh => "mines.sh",
            File::ThanksTxt => "thanks.txt",
            File::ZshRc => ".zshrc"
            File::Nav(_) => "nav.rs",
        }
    }

    pub fn contents(&self) -> String {
        match self {
            File::MinesSh => MINES_SH.to_string(),
            File::ThanksTxt => THANKS_TXT.to_string(),
            File::ZshRc => ZSHRC_CONTENT.to_string(),
            File::Nav(s) => {
                let s = if s.is_empty() { "/" } else { s };
                format!(
                    r#"use leptos::prelude::*;
use leptos_router::{{hooks::use_navigate, UseNavigateOptions}};

func main() {{
    Effect::new((_) => {{
        let navigate = use_navigate();
        navigate("{s}", UseNavigateOptions::default);
    }})
}}
"#
                )
            }
        }
    }
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
pub enum Target {
    Dir(Dir),
    File(File),
    Invalid,
}

impl Target {
    pub fn from_str(path: &str, blog_posts: &[String]) -> Self {
        match path {
            "/" => Self::Dir(Dir::Base),
            "/blog" => Self::Dir(Dir::Blog),
            "/cv" => Self::Dir(Dir::CV),
            post if post.starts_with("/blog/") && blog_post_exists(post, blog_posts) => {
                let blog_post_name = post
                    .split("/")
                    .last()
                    .expect("all blog posts should contain a /");
                Self::Dir(Dir::BlogPost(blog_post_name.to_string()))
            }
            "/mines.sh" => Self::File(File::MinesSh),
            "/thanks.txt" => Self::File(File::ThanksTxt),
            "/nav.rs" => Self::File(File::Nav("/".to_string())),
            "/blog/nav.rs" | "/cv/nav.rs" => {
                Self::File(File::Nav(path[..path.len() - LEN_OF_NAV].to_string()))
            }
            post_nav
                if post_nav.starts_with("/blog/")
                    && post_nav.ends_with("/nav.rs")
                    && blog_post_exists(&post_nav[..post_nav.len() - LEN_OF_NAV], blog_posts) =>
            {
                Self::File(File::Nav(path[..path.len() - LEN_OF_NAV].to_string()))
            }
            _ => Self::Invalid,
        }
    }

    pub fn is_executable(&self) -> bool {
        matches!(self, Self::File(File::MinesSh | File::Nav(_)))
    }
}
