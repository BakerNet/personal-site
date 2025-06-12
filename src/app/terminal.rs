use std::{collections::VecDeque, sync::Arc};

use leptos::{either::*, prelude::*};
use leptos_router::components::*;

use super::ascii::{AVATAR_BLOCK, INFO_BLOCK};

const LEN_OF_NAV: usize = 7;
const CHAR_WIDTH: usize = 9;
const TERMINAL_MARGINS: usize = 65;
const MINES_URL: &str = "https://mines.hansbaker.com";
const HELP_TEXT: &str = r#"This is Hans Baker's personal website.  Use this terminal to navigate the site.
The commands should feel familiar:
    cat     concatenate files and print to the standard output
    cd      change directory (navigate site)
    clear   clear the terminal screen
    ls      list directory contents (sitemap)
    mines   minesweeper app
    pwd     print name of the current/working directory (current URL path)
"#;
const MINES_SH: &str = r#"#!/bin/bash
set -e

# https://mines.hansbaker.com
# Minesweeper client with multiplayer, replay analysis, and stat tracking
mines
"#;
const THANKS_TXT: &str = "Thank you to my wife and my daughter for bringing immense joy to my life.";

pub struct Terminal {
    blog_posts: Vec<String>,
    history: Vec<String>,
}

impl Terminal {
    pub fn new(blog_posts: &[String], history: Option<Vec<String>>) -> Self {
        let history = history.unwrap_or_default();
        Self {
            blog_posts: blog_posts.to_owned(),
            history,
        }
    }

    #[cfg(feature = "hydrate")]
    pub fn set_history(&mut self, history: Vec<String>) {
        self.history = history;
    }

    #[cfg(feature = "hydrate")]
    pub fn history(&self) -> Vec<String> {
        if self.history.len() > 100 {
            self.history[self.history.len()-100..].to_vec()
        } else {
            self.history.clone()
        }
    }

    pub fn handle_command(&mut self, path: &str, input: &str) -> CommandRes {
        if input.trim().is_empty() {
            return CommandRes::EmptyErr
        }
        self.history.push(input.to_string());

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
            Command::MkDir => self.handle_mkdir(path, parts.collect()),
            Command::Rm => self.handle_rm(path, parts.collect()),
            Command::Echo => self.handle_echo(parts.collect()),
            Command::Mv => self.handle_mv(path, parts.collect()),
            Command::Cp => self.handle_cp(path, parts.collect()),
            Command::Touch => self.handle_touch(path, parts.collect()),
            Command::WhoAmI => CommandRes::Output(Arc::new(move || "user".into_any())),
            Command::Neofetch => CommandRes::Output(Arc::new(move || {
                let text = AVATAR_BLOCK.iter().zip(INFO_BLOCK.iter()).map(|(a, b)| format!("{}  {}", a, b)).fold(String::new(), |acc, s| {
                    if acc.is_empty() {
                        s
                    } else {
                        format!("{}\n{}", acc, s)
                    }
                });
                view! { <div class="leading-tight" inner_html=text></div> }.into_any()
            })),
            Command::Sudo => CommandRes::Err(Arc::new(move || "user is not in the sudoers file. This incident will be reported.".into_any())),
            Command::Unknown => self.handle_unknown(path, cmd_text, parts.collect()),
        }
    }

    pub fn handle_start_hist(&self, input: &str) -> Vec<String> {
        if input.trim().is_empty() {
            self.history.clone()
        } else {
            self.history.iter().filter(|s| s.starts_with(input)).map(|s| s.to_string()).collect()
        }
    }

    pub fn handle_start_tab(&mut self, path: &str, input: &str) -> Vec<String> {
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

    fn handle_unknown(&self, path: &str, target: &str, args: Vec<&str>) -> CommandRes {
        let target_string = target.to_owned();
        let target_path = path_target_to_target_path(path, target, false);
        let target = Target::from_str(&target_path, &self.blog_posts);
        let is_executable = matches!(target, Target::File(File::MinesSh | File::Nav(_))) && target_string.contains("/"); 
        if !args.is_empty() && !is_executable {
            // only mines.sh and nav.rs are executable, so only these can accept arguments
            return CommandRes::Err(Arc::new(move || format!("command not found: {}", target_string).into_any()));
        }
        match target {
            Target::Dir(_) => CommandRes::Redirect(target_path),
            Target::File(f) => {
                if target_string.ends_with("/") {
                    return CommandRes::Err(Arc::new(move || format!("not a directory: {}", target_string).into_any()));
                }
                match f {
                    File::Nav(s) => {
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
        let (options, mut targets) = parse_multitarget(args);
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
        let (options, targets) = parse_multitarget(args);
        if !options.is_empty() {
            let c = options[0].to_owned();
            return CommandRes::Err(Arc::new(move || {
                format!(
                    r#"cat: invalid option -- '{}'
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

    fn handle_cp(&self, path: &str, args: Vec<&str>) -> CommandRes {
        let (options, targets) = parse_multitarget(args);
        let mut recursive = false;
        let invalid = options.iter().find(|c| **c != 'r');
        if let Some(c) = invalid {
            let c = c.to_owned();
            return CommandRes::Err(Arc::new(move || {
                format!(
                    r#"cp: invalid option -- '{}'
This version of cp only supports option 'r'"#,
                    c
                )
                .into_any()
            }));
        }
        if !options.is_empty() {
            recursive = true;
        }
        if targets.is_empty() {
            return CommandRes::Err(Arc::new(move || "cp: missing file operand".into_any()));
        }
        if targets.len() < 2 {
            let target = targets[0].to_owned();
            return CommandRes::Err(Arc::new(move || format!("cp: missing destination file operand after {}", target).into_any()));
        }
        let targets = targets.into_iter().enumerate().fold(
            Vec::new(), 
            |mut ts, (i, tp)| {
                let target_string = tp.to_owned();
                let target_path = path_target_to_target_path(path, tp, false);
                let full_target = Target::from_str(&target_path, &self.blog_posts);
                let tp = if i != 0 && tp.contains("/") {
                    tp.rsplit_once("/").unwrap().0
                } else {
                    ""
                };
                let target_path = path_target_to_target_path(path, tp, false);
                let partial_target = Target::from_str(&target_path, &self.blog_posts);
                ts.push((target_string, full_target, partial_target));
                ts
            },
        );
        let target_filename = match (recursive, &targets[0].1) {
            (false, Target::Dir(_)) => {
                return CommandRes::Err(Arc::new(move || format!("cp: -r not specified; omitting directory '{}'", targets[0].0).into_any()));
            },
            (_ , Target::Invalid) => {
                return CommandRes::Err(Arc::new(move || format!("cp: cannot stat '{}': No such file or directory", targets[0].0).into_any()));
            },
            _ => {
                let target = &targets[0].0;
                let target = if target.ends_with("/") {
                    &target[..target.len()-1]
                } else {
                    &target[..]
                };
                target.split("/").last().expect("Should have a last element").to_string()
            }
        };
        let last = targets.len() -1;
        let callback = {
            let targets = targets.clone();
            move || {
                targets.iter().enumerate().skip(1).map(|(i, (name, full_ts, partial_ts))| {
                    view! {
                        {match full_ts {
                            Target::Dir(_) => {
                                if name.ends_with("/") {
                                   format!("cp: cannot create regular file '{}{}': Permission denied", name, target_filename).into_any()
                                }else {
                                   format!("cp: cannot create regular file '{}/{}': Permission denied", name, target_filename).into_any()
                                }
                            },
                            Target::File(_) => format!("cp: cannot create regular file '{}': Permission denied", name).into_any(),
                            Target::Invalid => {
                                if name.ends_with("/") {
                                    format!("cp: cannot create regular file '{}': Not a directory", name).into_any()
                                } else {
                                    match partial_ts {
                                        Target::Dir(_) | Target::File(_) => format!("cp: cannot create regular file '{}': Permission denied", name).into_any(),
                                        Target::Invalid => format!("cp: cannot create regular file '{}': No such file or directory", name).into_any(),
                                    }
                                }
                            }
                        }}
                        {if i != last { "\n" } else { "" }}
                    } 
                }).collect_view().into_any()
            }
        };
        CommandRes::Err(Arc::new(callback))
    }

    fn handle_mv(&self, path: &str, args: Vec<&str>) -> CommandRes {
        let (options, targets) = parse_multitarget(args);
        let mut recursive = false;
        let invalid = options.iter().find(|c| **c != 'r');
        if let Some(c) = invalid {
            let c = c.to_owned();
            return CommandRes::Err(Arc::new(move || {
                format!(
                    r#"mv: invalid option -- '{}'
This version of mv only supports option 'r'"#,
                    c
                )
                .into_any()
            }));
        }
        if !options.is_empty() {
            recursive = true;
        }
        if targets.is_empty() {
            return CommandRes::Err(Arc::new(move || "mv: missing file operand".into_any()));
        }
        if targets.len() < 2 {
            let target = targets[0].to_owned();
            return CommandRes::Err(Arc::new(move || format!("mv: missing destination file operand after {}", target).into_any()));
        }
        let targets = targets.into_iter().enumerate().fold(
            Vec::new(), 
            |mut ts, (i, tp)| {
                let target_string = tp.to_owned();
                let target_path = path_target_to_target_path(path, tp, false);
                let full_target = Target::from_str(&target_path, &self.blog_posts);
                let tp = if i != 0 && tp.contains("/") {
                    tp.rsplit_once("/").unwrap().0
                } else {
                    ""
                };
                let target_path = path_target_to_target_path(path, tp, false);
                let partial_target = Target::from_str(&target_path, &self.blog_posts);
                ts.push((target_string, full_target, partial_target));
                ts
            },
        );
        let target_filename = match (recursive, &targets[0].1) {
            (false, Target::Dir(_)) => {
                return CommandRes::Err(Arc::new(move || format!("mv: -r not specified; omitting directory '{}'", targets[0].0).into_any()));
            },
            (_ , Target::Invalid) => {
                return CommandRes::Err(Arc::new(move || format!("mv: cannot stat '{}': No such file or directory", targets[0].0).into_any()));
            },
            _ => {
                let target = &targets[0].0;
                let target = if target.ends_with("/") {
                    &target[..target.len()-1]
                } else {
                    &target[..]
                };
                target.split("/").last().expect("Should have a last element").to_string()
            }
        };
        let last = targets.len() -1;
        let callback = {
            let targets = targets.clone();
            move || {
                targets.iter().enumerate().skip(1).map(|(i, (name, full_ts, partial_ts))| {
                    view! {
                        {match full_ts {
                            Target::Dir(_) => {
                                if name.ends_with("/") {
                                   format!("mv: cannot create regular file '{}{}': Permission denied", name, target_filename).into_any()
                                }else {
                                   format!("mv: cannot create regular file '{}/{}': Permission denied", name, target_filename).into_any()
                                }
                            },
                            Target::File(_) => format!("mv: cannot create regular file '{}': Permission denied", name).into_any(),
                            Target::Invalid => {
                                if name.ends_with("/") {
                                    format!("mv: cannot create regular file '{}': Not a directory", name).into_any()
                                } else {
                                    match partial_ts {
                                        Target::Dir(_) | Target::File(_) => format!("mv: cannot create regular file '{}': Permission denied", name).into_any(),
                                        Target::Invalid => format!("mv: cannot create regular file '{}': No such file or directory", name).into_any(),
                                    }
                                }
                            }
                        }}
                        {if i != last { "\n" } else { "" }}
                    } 
                }).collect_view().into_any()
            }
        };
        CommandRes::Err(Arc::new(callback))
    }

    fn handle_touch(&self, path: &str, args: Vec<&str>) -> CommandRes {
        let (_, targets) = parse_multitarget(args);
        if targets.is_empty() {
            return CommandRes::Err(Arc::new(move || "touch: missing operand".into_any()));
        }
        let targets = targets.into_iter().fold(
            Vec::new(), 
            |mut ts, tp| {
                let target_string = tp.to_owned();
                let tp = if tp.contains("/") {
                    tp.rsplit_once("/").unwrap().0
                } else {
                    ""
                };
                let target_path = path_target_to_target_path(path, tp, false);
                let target = Target::from_str(&target_path, &self.blog_posts);
                ts.push((target_string, target));
                ts
            },
        );
        let last = targets.len() -1;
        let callback = {
            let targets = targets.clone();
            move || {
                targets.iter().enumerate().map(|(i, (name, ts))| {
                    let base = format!("touch: cannot touch '{}': ", name);
                    view! {
                        {match ts {
                            Target::Dir(_) => (base + "Permission denied").into_any(),
                            Target::File(_) => (base + "Not a directory").into_any(),
                            Target::Invalid => {
                                (base + "No such file or directory").into_any()
                            }
                        }}
                        {if i != last { "\n" } else { "" }}
                    } 
                }).collect_view().into_any()
            }
        };
        CommandRes::Err(Arc::new(callback))
    }

    fn handle_mkdir(&self, path: &str, args: Vec<&str>) -> CommandRes {
        let (_, targets) = parse_multitarget(args);
        if targets.is_empty() {
            return CommandRes::Err(Arc::new(move || "mkdir: missing operand".into_any()));
        }
        let targets = targets.into_iter().fold(
            Vec::new(), 
            |mut ts, tp| {
                let target_string = tp.to_owned();
                let tp = if tp.contains("/") {
                    tp.rsplit_once("/").unwrap().0
                } else {
                    ""
                };
                let target_path = path_target_to_target_path(path, tp, false);
                let target = Target::from_str(&target_path, &self.blog_posts);
                ts.push((target_string, target));
                ts
            },
        );
        let last = targets.len() -1;
        let callback = {
            let targets = targets.clone();
            move || {
                targets.iter().enumerate().map(|(i, (name, ts))| {
                    let base = format!("mkdir: cannot create directory '{}': ", name);
                    view! {
                        {match ts {
                            Target::Dir(_) => (base + "Permission denied").into_any(),
                            Target::File(_) => (base + "Not a directory").into_any(),
                            Target::Invalid => {
                                (base + "No such file or directory").into_any()
                            }
                        }}
                        {if i != last { "\n" } else { "" }}
                    } 
                }).collect_view().into_any()
            }
        };
        CommandRes::Err(Arc::new(callback))
    }

    fn handle_rm(&self, path: &str, args: Vec<&str>) -> CommandRes {
        let (options, targets) = parse_multitarget(args);
        if targets.is_empty() {
            return CommandRes::Err(Arc::new(move || "rm: missing operand".into_any()));
        }
        let mut recursive = false;
        let invalid = options.iter().find(|c| **c != 'r');
        if let Some(c) = invalid {
            let c = c.to_owned();
            return CommandRes::Err(Arc::new(move || {
                format!(
                    r#"rm: invalid option -- '{}'
This version of rm only supports option 'r'"#,
                    c
                )
                .into_any()
            }));
        }
        if !options.is_empty() {
            recursive = true;
        }
        let targets = targets.into_iter().fold(
            Vec::new(), 
            |mut ts, tp| {
                let target_string = tp.to_owned();
                let target_path = path_target_to_target_path(path, tp, false);
                let target = Target::from_str(&target_path, &self.blog_posts);
                ts.push((target_string, target));
                ts
            },
        );
        let last = targets.len() -1;
        let callback = {
            let targets = targets.clone();
            move || {
                targets.iter().enumerate().map(|(i, (name, ts))| {
                    let base = format!("rm: cannot remove '{}': ", name);
                    view! {
                        {match ts {
                            Target::File(_) => (base + "Permission denied").into_any(),
                            Target::Dir(_) if recursive => (base + "Permission denied").into_any(),
                            Target::Dir(_) => (base + "Is a directory").into_any(),
                            Target::Invalid => {
                                (base + "No such file or directory").into_any()
                            }
                        }}
                        {if i != last { "\n" } else { "" }}
                    } 
                }).collect_view().into_any()
            }
        };
        CommandRes::Err(Arc::new(callback))
    }

    fn handle_echo(&self, args: Vec<&str>) -> CommandRes {
        let mut message = args.iter().map(|s| s.replace("\"", "").to_string() + " ").collect::<String>();
        while let Some(i) = message.find("$") {
            if &message[i+1..i+2] == "(" {
                return CommandRes::Err(Arc::new(move || "echo: command substitution not supported".into_any()));
            };
            let end = message[i..].find(" ").unwrap_or(message.len());
            let var = &message[i+1..i+end];
            let val = match var {
                "USER" => "user",
                "HOME" => "/",
                _ => "",
            };
            message = message.replace(&message[i..i+end], val);
        }
        CommandRes::Output(Arc::new(move || message.clone().into_any()))
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

fn parse_multitarget<'a>(args: Vec<&'a str>) -> (Vec<char>, Vec<&'a str>) {
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
    let dir_class = "text-blue";
    let ex_class = "text-green";
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
                EitherOf3::A(view! {
                    <A href=href attr:class=dir_class>
                        {s}
                    </A>
                    " "
                })
            } else if s.ends_with("*") {
                let s = s[..s.len()-1].to_string();
                // note - adding extra space because trimming trailing '*'
                EitherOf3::B(view! {
                    <span class=ex_class>{s}</span>
                    " "
                })
            } else {
                EitherOf3::C(view! { <span>{s}</span> })
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
    F: Fn(String) -> AnyView + 'static
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
    let views = (0..rows).map(|row| 
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
            vec!["./".to_string(), "../".to_string(), "nav.rs*".to_string()]
        } else {
            vec!["nav.rs*".to_string()]
        };
        match self {
            Dir::Base => {
                let mut items = vec!["blog/".to_string(), "cv/".to_string(), "mines.sh*".to_string(), "thanks.txt".to_string()];
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
    Nav(String)
}

impl File {
    fn name(&self) -> &'static str {
        match self {
            File::MinesSh => "mines.sh",
            File::ThanksTxt => "thanks.txt",
            File::Nav(_) => "nav.rs",
        }
    }

    fn contents(&self) -> String {
        match self {
            File::MinesSh => MINES_SH.to_string(),
            File::ThanksTxt => THANKS_TXT.to_string(),
            File::Nav(s) => {
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
            "/nav.rs" => Self::File(File::Nav("/".to_string())),
            "/blog/nav.rs" | "/cv/nav.rs" => Self::File(File::Nav(path[..path.len() - LEN_OF_NAV].to_string())),
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
    Cp,
    Echo,
    Mines,
    MkDir,
    Mv,
    Rm,
    Neofetch,
    Touch,
    WhoAmI,
    Sudo,
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
            "cp" => Self::Cp,
            "mines" => Self::Mines,
            "mkdir" => Self::MkDir,
            "mv" => Self::Mv,
            "rm" => Self::Rm,
            "echo" => Self::Echo,
            "touch" => Self::Touch,
            "whoami" => Self::WhoAmI,
            "neofetch" => Self::Neofetch,
            "sudo" => Self::Sudo,
            _ => Self::Unknown,
        }
    }
}

impl Command {
    fn all() -> Vec<&'static str> {
        vec!["help", "pwd", "ls", "cd", "cat", "clear", "cp", "echo", "mines", "mkdir", "mv", "rm", "touch", "whoami", "neofetch"]
    }
}
