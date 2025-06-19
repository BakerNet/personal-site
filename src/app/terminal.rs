use std::{collections::{HashMap, VecDeque}, sync::Arc};

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
    env_vars: HashMap<String, String>,
}

impl Terminal {
    pub fn new(blog_posts: &[String], history: Option<Vec<String>>) -> Self {
        let history = history.unwrap_or_default();
        let mut env_vars = HashMap::new();
        env_vars.insert("USER".to_string(), "user".to_string());
        env_vars.insert("HOME".to_string(), "/".to_string());
        env_vars.insert("SITE".to_string(), "hansbaker.com".to_string());
        env_vars.insert("VERSION".to_string(), env!("CARGO_PKG_VERSION").to_string());
        
        Self {
            blog_posts: blog_posts.to_owned(),
            history,
            env_vars,
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

    fn process_aliases(&self, input: &str) -> String {
        let trimmed = input.trim();
        
        for alias in CommandAlias::all() {
            let alias_str = alias.as_str();
            if trimmed.starts_with(alias_str) {
                if trimmed == alias_str {
                    return alias.expand("");
                } else if trimmed.starts_with(&format!("{} ", alias_str)) {
                    let args = &trimmed[alias_str.len()..];
                    return alias.expand(args);
                }
            }
        }
        
        input.to_string()
    }

    fn expand_env_vars(&self, path: &str, input: &str) -> String {
        let mut result = input.to_string();
        let mut env_vars = self.env_vars.clone();
        env_vars.insert("PWD".to_string(), path.to_string());
        
        while let Some(start) = result.find('$') {
            let remaining = &result[start + 1..];
            let end = remaining.find(|c: char| !c.is_alphanumeric() && c != '_').unwrap_or(remaining.len());
            let var_name = &remaining[..end];
            
            if let Some(value) = env_vars.get(var_name) {
                let var_ref = format!("${}", var_name);
                result = result.replace(&var_ref, value);
            } else {
                // Replace with empty string if variable not found
                let var_ref = format!("${}", var_name);
                result = result.replace(&var_ref, "");
            }
        }
        result
    }

    pub fn handle_command(&mut self, path: &str, input: &str) -> CommandRes {
        if input.trim().is_empty() {
            return CommandRes::EmptyErr
        }
        self.history.push(input.to_string());

        // Process command aliases first
        let aliased_input = self.process_aliases(input);
        
        // Expand environment variables in the input
        let expanded_input = self.expand_env_vars(path, &aliased_input);
        
        let mut parts = expanded_input.split_whitespace();
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
            Command::History => self.handle_history(parts.collect()),
            Command::Mv => self.handle_mv(path, parts.collect()),
            Command::Cp => self.handle_cp(path, parts.collect()),
            Command::Date => self.handle_date(parts.collect()),
            Command::Touch => self.handle_touch(path, parts.collect()),
            Command::Which => self.handle_which(path, parts.collect()),
            Command::WhoAmI => CommandRes::Output(Arc::new(move || "user".into_any())),
            Command::Neofetch => CommandRes::Output(Arc::new(move || {
                let text = AVATAR_BLOCK.iter().zip(INFO_BLOCK.iter()).map(|(a, b)| format!("{a}  {b}")).fold(String::new(), |acc, s| {
                    if acc.is_empty() {
                        s
                    } else {
                        format!("{acc}\n{s}")
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
            return CommandRes::Err(Arc::new(move || format!("command not found: {target_string}").into_any()));
        }
        match target {
            Target::Dir(_) => CommandRes::Redirect(target_path),
            Target::File(f) => {
                if target_string.ends_with("/") {
                    return CommandRes::Err(Arc::new(move || format!("not a directory: {target_string}").into_any()));
                }
                match f {
                    File::Nav(s) => {
                        CommandRes::Redirect(s)
                    }
                    File::MinesSh => {
                        if is_executable {
                            CommandRes::Redirect(MINES_URL.to_string())
                        } else {
                            CommandRes::Err(Arc::new(move || format!("command not found: {target_string}\nhint: try 'mines' or '/mines.sh'").into_any()))
                        }
                    }
                    File::ThanksTxt => {
                        if target_string.contains("/") {
                            CommandRes::Err(Arc::new(move || format!("permission denied: {target_string}").into_any()))
                        } else {
                            CommandRes::Err(Arc::new(move || format!("command not found: {target_string}").into_any()))
                        }
                    }
                }
            }
            Target::Invalid => CommandRes::Err(Arc::new(move || format!("command not found: {target_string}").into_any())),
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
                    r#"ls: invalid option -- '{c}'
This version of ls only supports option 'a'"#
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
                            format!("{name}:\n")
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
                                format!("ls: cannot access '{name}': No such file or directory")
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
                        format!("cd: not a directory: {other}").into_any()
                    }))
                }
                Target::Invalid => {
                    let other = target_string.clone();
                    CommandRes::Err(Arc::new(move || {
                        format!("cd: no such file or directory: {other}").into_any()
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
                    r#"cat: invalid option -- '{c}'
This version of cat doesn't support any options"#
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
                            Target::Dir(_) => format!("cat: {name}: Is a directory").into_any(),
                            Target::File(f) => f.contents().into_any(),
                            Target::Invalid => {
                                format!("cat: {name}: No such file or directory").into_any()
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
                    r#"cp: invalid option -- '{c}'
This version of cp only supports option 'r'"#
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
            return CommandRes::Err(Arc::new(move || format!("cp: missing destination file operand after {target}").into_any()));
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
                                   format!("cp: cannot create regular file '{name}{target_filename}': Permission denied").into_any()
                                }else {
                                   format!("cp: cannot create regular file '{name}/{target_filename}': Permission denied").into_any()
                                }
                            },
                            Target::File(_) => format!("cp: cannot create regular file '{name}': Permission denied").into_any(),
                            Target::Invalid => {
                                if name.ends_with("/") {
                                    format!("cp: cannot create regular file '{name}': Not a directory").into_any()
                                } else {
                                    match partial_ts {
                                        Target::Dir(_) | Target::File(_) => format!("cp: cannot create regular file '{name}': Permission denied").into_any(),
                                        Target::Invalid => format!("cp: cannot create regular file '{name}': No such file or directory").into_any(),
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
                    r#"mv: invalid option -- '{c}'
This version of mv only supports option 'r'"#
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
            return CommandRes::Err(Arc::new(move || format!("mv: missing destination file operand after {target}").into_any()));
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
                                   format!("mv: cannot create regular file '{name}{target_filename}': Permission denied").into_any()
                                }else {
                                   format!("mv: cannot create regular file '{name}/{target_filename}': Permission denied").into_any()
                                }
                            },
                            Target::File(_) => format!("mv: cannot create regular file '{name}': Permission denied").into_any(),
                            Target::Invalid => {
                                if name.ends_with("/") {
                                    format!("mv: cannot create regular file '{name}': Not a directory").into_any()
                                } else {
                                    match partial_ts {
                                        Target::Dir(_) | Target::File(_) => format!("mv: cannot create regular file '{name}': Permission denied").into_any(),
                                        Target::Invalid => format!("mv: cannot create regular file '{name}': No such file or directory").into_any(),
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
                    let base = format!("touch: cannot touch '{name}': ");
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
                    let base = format!("mkdir: cannot create directory '{name}': ");
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
                    r#"rm: invalid option -- '{c}'
This version of rm only supports option 'r'"#
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
                    let base = format!("rm: cannot remove '{name}': ");
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
        let message = args.iter().map(|s| s.replace("\"", "")).collect::<Vec<_>>().join(" ");
        
        // Check for unsupported command substitution
        if message.contains("$(") {
            return CommandRes::Err(Arc::new(move || "echo: command substitution not supported".into_any()));
        }
        
        CommandRes::Output(Arc::new(move || message.clone().into_any()))
    }

    fn handle_history(&mut self, args: Vec<&str>) -> CommandRes {
        if args.len() > 1 {
            return CommandRes::Err(Arc::new(move || "history: too many arguments".into_any()));
        }
        
        if let Some(arg) = args.first() {
            if *arg == "-c" {
                self.history.clear();
                return CommandRes::Output(Arc::new(move || "history cleared".into_any()));
            }
            
            if let Ok(n) = arg.parse::<usize>() {
                let history = self.history.clone();
                let count = n.min(history.len());
                let start_idx = if history.len() > count { history.len() - count } else { 0 };
                let limited_history = &history[start_idx..];
                
                let output = limited_history.iter().enumerate()
                    .map(|(i, cmd)| format!("{:4}  {}", start_idx + i + 1, cmd))
                    .collect::<Vec<_>>()
                    .join("\n");
                
                return CommandRes::Output(Arc::new(move || output.clone().into_any()));
            } else {
                return CommandRes::Err(Arc::new(move || "history: numeric argument required".into_any()));
            }
        }
        
        // Show all history with line numbers
        let history = self.history.clone();
        let output = history.iter().enumerate()
            .map(|(i, cmd)| format!("{:4}  {}", i + 1, cmd))
            .collect::<Vec<_>>()
            .join("\n");
        
        CommandRes::Output(Arc::new(move || output.clone().into_any()))
    }

    fn handle_which(&self, path: &str, args: Vec<&str>) -> CommandRes {
        if args.is_empty() {
            return CommandRes::Err(Arc::new(move || "which: missing argument".into_any()));
        }
        
        let results: Vec<String> = args.iter().map(|&command| {
            self.get_which_result(path, command)
        }).collect();
        
        let output = results.join("\n");
        CommandRes::Output(Arc::new(move || output.clone().into_any()))
    }
    
    fn get_which_result(&self, path: &str, command: &str) -> String {
        // If the command contains a path separator, treat it as a file path
        if command.contains('/') {
            let target_path = path_target_to_target_path(path, command, false);
            let target = Target::from_str(&target_path, &self.blog_posts);
            
            // Check if it's an executable file
            let is_executable = matches!(target, Target::File(File::MinesSh | File::Nav(_)));
            
            if is_executable {
                command.to_string()
            } else {
                format!("{} not found", command)
            }
        } else if let Some(alias) = CommandAlias::from_str(command) {
            // Check if it's an alias first
            let expansion = alias.expand("");
            format!("{}: aliased to {}", command, expansion)
        } else {
            // Map commands to their simulated paths
            match command {
                // Shell builtins
                "cd" | "pwd" | "echo" | "history" => format!("{}: shell builtin", command),
                
                // External commands (simulated paths)
                "help" | "ls" | "cat" | "clear" | "cp" | "date" | "mines" | "mkdir" | "mv" | "rm" | "touch" | "which" | "whoami" | "neofetch" => {
                    format!("/usr/bin/{}", command)
                }
                
                // Unknown command
                _ => format!("{} not found", command),
            }
        }
    }

    fn handle_date(&self, args: Vec<&str>) -> CommandRes {
        use chrono::prelude::*;
        
        let now = Local::now();
        
        if args.is_empty() {
            // Default format: Wed Dec 25 14:30:15 PST 2024
            let formatted = now.format("%a %b %d %H:%M:%S %Z %Y").to_string();
            return CommandRes::Output(Arc::new(move || formatted.clone().into_any()));
        }
        
        if args.len() > 1 {
            return CommandRes::Err(Arc::new(move || "date: too many arguments".into_any()));
        }
        
        let format_str = args[0].trim_matches('"');
        
        if !format_str.starts_with('+') {
            return CommandRes::Err(Arc::new(move || "date: invalid format (must start with +)".into_any()));
        }
        
        let format_str = &format_str[1..]; // Remove the + prefix
        
        // Handle common format strings
        let result = match format_str {
            "%Y-%m-%d" => now.format("%Y-%m-%d").to_string(),
            "%H:%M:%S" => now.format("%H:%M:%S").to_string(),
            "%Y-%m-%d %H:%M:%S" => now.format("%Y-%m-%d %H:%M:%S").to_string(),
            "%Y" => now.format("%Y").to_string(),
            "%m" => now.format("%m").to_string(),
            "%d" => now.format("%d").to_string(),
            "%H" => now.format("%H").to_string(),
            "%M" => now.format("%M").to_string(),
            "%S" => now.format("%S").to_string(),
            _ => {
                // Try to parse as a general format string
                let formatted = now.format(format_str).to_string();
                formatted
            }
        };
        
        CommandRes::Output(Arc::new(move || result.clone().into_any()))
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
        let mut commands = Command::all().into_iter().filter(|s| s.starts_with(cmd_text)).map(|s| s.to_string()).collect::<Vec<_>>();
        
        // Add aliases
        for alias in CommandAlias::all() {
            let alias_str = alias.as_str();
            if alias_str.starts_with(cmd_text) {
                commands.push(alias_str.to_string());
            }
        }
        
        commands.sort();
        commands
    }
}

fn parse_multitarget(args: Vec<&str>) -> (Vec<char>, Vec<&str>) {
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
                let mut posts = blog_posts.iter().map(|bp| format!("{bp}/")).collect::<Vec<_>>();
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
            Dir::BlogPost(s) => format!("/blog/{s}"),
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
        navigate("{s}", UseNavigateOptions::default);
    }})
}}
"#)
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
    Date,
    Echo,
    History,
    Mines,
    MkDir,
    Mv,
    Rm,
    Neofetch,
    Touch,
    Which,
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
            "date" => Self::Date,
            "echo" => Self::Echo,
            "history" => Self::History,
            "mines" => Self::Mines,
            "mkdir" => Self::MkDir,
            "mv" => Self::Mv,
            "rm" => Self::Rm,
            "touch" => Self::Touch,
            "which" => Self::Which,
            "whoami" => Self::WhoAmI,
            "neofetch" => Self::Neofetch,
            "sudo" => Self::Sudo,
            _ => Self::Unknown,
        }
    }
}

impl Command {
    fn all() -> Vec<&'static str> {
        vec!["help", "pwd", "ls", "cd", "cat", "clear", "cp", "date", "echo", "history", "mines", "mkdir", "mv", "rm", "touch", "which", "whoami", "neofetch"]
    }
}

#[derive(Debug, Clone)]
enum CommandAlias {
    Ll,
    La,
    H,
}

impl CommandAlias {
    fn all() -> Vec<CommandAlias> {
        vec![CommandAlias::Ll, CommandAlias::La, CommandAlias::H]
    }

    fn as_str(&self) -> &'static str {
        match self {
            CommandAlias::Ll => "ll",
            CommandAlias::La => "la", 
            CommandAlias::H => "h",
        }
    }

    fn expand(&self, args: &str) -> String {
        match self {
            CommandAlias::Ll => {
                if args.is_empty() {
                    "ls -la".to_string()
                } else {
                    format!("ls -la{}", args)
                }
            }
            CommandAlias::La => {
                if args.is_empty() {
                    "ls -a".to_string()
                } else {
                    format!("ls -a{}", args)
                }
            }
            CommandAlias::H => {
                if args.is_empty() {
                    "history".to_string()
                } else {
                    format!("history{}", args)
                }
            }
        }
    }


    fn from_str(s: &str) -> Option<CommandAlias> {
        match s {
            "ll" => Some(CommandAlias::Ll),
            "la" => Some(CommandAlias::La),
            "h" => Some(CommandAlias::H),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_terminal() -> Terminal {
        Terminal::new(&[], Some(vec!["ls".to_string(), "pwd".to_string(), "echo hello".to_string()]))
    }

    #[test]
    fn test_environment_variables() {
        let terminal = create_test_terminal();
        
        // Test basic environment variable expansion
        let expanded = terminal.expand_env_vars("/", "echo $USER");
        assert_eq!(expanded, "echo user");
        
        let expanded = terminal.expand_env_vars("/blog", "echo $PWD");
        assert_eq!(expanded, "echo /blog");
        
        let expanded = terminal.expand_env_vars("/", "echo $HOME");
        assert_eq!(expanded, "echo /");
        
        let expanded = terminal.expand_env_vars("/", "echo $SITE");
        assert_eq!(expanded, "echo hansbaker.com");
    }

    #[test]
    fn test_command_aliases() {
        let terminal = create_test_terminal();
        
        // Test alias processing
        assert_eq!(terminal.process_aliases("ll"), "ls -la");
        assert_eq!(terminal.process_aliases("la"), "ls -a");
        assert_eq!(terminal.process_aliases("h"), "history");
        assert_eq!(terminal.process_aliases("ll /blog"), "ls -la /blog");
        assert_eq!(terminal.process_aliases("regular_command"), "regular_command");
    }

    #[test]
    fn test_history_command() {
        let mut terminal = create_test_terminal();
        
        // Test basic history display
        let result = terminal.handle_command("/", "history");
        match result {
            CommandRes::Output(_) => {}, // Expected
            _ => panic!("Expected output for history command"),
        }
        
        // Test history with count
        let result = terminal.handle_command("/", "history 2");
        match result {
            CommandRes::Output(_) => {}, // Expected
            _ => panic!("Expected output for history 2 command"),
        }
        
        // Test history clear
        let result = terminal.handle_command("/", "history -c");
        match result {
            CommandRes::Output(_) => {}, // Expected
            _ => panic!("Expected output for history -c command"),
        }
    }

    #[test]
    fn test_which_command() {
        let mut terminal = create_test_terminal();
        
        // Test which with known command
        let result = terminal.handle_command("/", "which ls");
        match result {
            CommandRes::Output(_) => {}, // Expected
            _ => panic!("Expected output for which ls"),
        }
        
        // Test which with builtin
        let result = terminal.handle_command("/", "which cd");
        match result {
            CommandRes::Output(_) => {}, // Expected
            _ => panic!("Expected output for which cd"),
        }
        
        // Test which with alias
        let result = terminal.handle_command("/", "which ll");
        match result {
            CommandRes::Output(_) => {}, // Expected
            _ => panic!("Expected output for which ll"),
        }
        
        // Test which with unknown command
        let result = terminal.handle_command("/", "which nonexistent");
        match result {
            CommandRes::Output(_) => {}, // Expected
            _ => panic!("Expected output for which nonexistent"),
        }
        
        // Test which with executable file path
        let result = terminal.handle_command("/", "which ./mines.sh");
        match result {
            CommandRes::Output(_) => {}, // Expected
            _ => panic!("Expected output for which ./mines.sh"),
        }
        
        // Test which with non-executable file path
        let result = terminal.handle_command("/", "which ./thanks.txt");
        match result {
            CommandRes::Output(_) => {}, // Expected
            _ => panic!("Expected output for which ./thanks.txt"),
        }
        
        // Test which with non-existent file path
        let result = terminal.handle_command("/", "which ./nonexistent.sh");
        match result {
            CommandRes::Output(_) => {}, // Expected
            _ => panic!("Expected output for which ./nonexistent.sh"),
        }
        
        // Test which with multiple arguments
        let result = terminal.handle_command("/", "which ls ll cd");
        match result {
            CommandRes::Output(_) => {}, // Expected
            _ => panic!("Expected output for which with multiple arguments"),
        }
    }

    #[test]
    fn test_date_command() {
        let mut terminal = create_test_terminal();
        
        // Test basic date command
        let result = terminal.handle_command("/", "date");
        match result {
            CommandRes::Output(_) => {}, // Expected
            _ => panic!("Expected output for date command"),
        }
        
        // Test date with format
        let result = terminal.handle_command("/", "date +%Y-%m-%d");
        match result {
            CommandRes::Output(_) => {}, // Expected
            _ => panic!("Expected output for date +%Y-%m-%d"),
        }
        
        // Test date with time format
        let result = terminal.handle_command("/", "date \"+%H:%M:%S\"");
        match result {
            CommandRes::Output(_) => {}, // Expected
            _ => panic!("Expected output for date +%H:%M:%S"),
        }
    }

    #[test]
    fn test_tab_completion_includes_new_commands() {
        let terminal = create_test_terminal();
        
        // Test that new commands are included in tab completion
        let commands = terminal.tab_commands("h");
        assert!(commands.contains(&"history".to_string()));
        assert!(commands.contains(&"h".to_string())); // alias
        
        let commands = terminal.tab_commands("w");
        assert!(commands.contains(&"which".to_string()));
        assert!(commands.contains(&"whoami".to_string()));
        
        let commands = terminal.tab_commands("d");
        assert!(commands.contains(&"date".to_string()));
        
        let commands = terminal.tab_commands("l");
        assert!(commands.contains(&"ls".to_string()));
        assert!(commands.contains(&"ll".to_string())); // alias
        assert!(commands.contains(&"la".to_string())); // alias
    }

    #[test]
    fn test_command_parsing() {
        // Test that new commands are parsed correctly
        assert!(matches!(Command::from("history"), Command::History));
        assert!(matches!(Command::from("which"), Command::Which));
        assert!(matches!(Command::from("date"), Command::Date));
        assert!(matches!(Command::from("unknown"), Command::Unknown));
    }

    #[test]
    fn test_error_handling() {
        let mut terminal = create_test_terminal();
        
        // Test which without arguments
        let result = terminal.handle_command("/", "which");
        match result {
            CommandRes::Err(_) => {}, // Expected
            _ => panic!("Expected error for which without arguments"),
        }
        
        // Test history with invalid argument
        let result = terminal.handle_command("/", "history invalid");
        match result {
            CommandRes::Err(_) => {}, // Expected
            _ => panic!("Expected error for history with invalid argument"),
        }
        
        // Test date with invalid format
        let result = terminal.handle_command("/", "date invalid");
        match result {
            CommandRes::Err(_) => {}, // Expected
            _ => panic!("Expected error for date with invalid format"),
        }
    }
}
