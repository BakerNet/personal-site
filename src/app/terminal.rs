mod command;
mod simple_tools;
mod ps_tools;
mod fs;
mod components;

pub use command::CommandRes;
pub use components::ColumnarView;

use std::{collections::{HashMap}, sync::Arc};

use leptos::prelude::*;

use fs::{File, parse_multitarget, path_target_to_target_path, Target};
use command::{Command, CommandAlias, Executable, PipelineRes};
use simple_tools::{WhoAmICommand, PwdCommand, DateCommand, NeofetchCommand, MinesCommand, HelpCommand, UptimeCommand, EchoCommand, ClearCommand, SudoCommand};
use ps_tools::{Process, PsCommand, KillCommand};
use components::{LsView, LsViewProps};

pub struct WhichCommand {
    blog_posts: Vec<String>,
}

impl WhichCommand {
    pub fn new(blog_posts: Vec<String>) -> Self {
        Self { blog_posts }
    }
    
    fn get_which_result(&self, path: &str, command: &str) -> (String, bool) {
        // If the command contains a path separator, treat it as a file path
        if command.contains('/') {
            let target_path = path_target_to_target_path(path, command, false);
            let target = Target::from_str(&target_path, &self.blog_posts);
            
            // Check if it's an executable file
            let is_executable = target.is_executable();
            
            if is_executable {
                (command.to_string(), true)
            } else {
                (format!("{command} not found"), false)
            }
        } else if let Some(alias) = CommandAlias::from_str(command) {
            // Check if it's an alias first
            let expansion = alias.expand("");
            (format!("{command}: aliased to {expansion}"), true)
        } else {
            // Map commands to their simulated paths
            match command {
                // Shell builtins
                "cd" | "pwd" | "echo" | "history" => (format!("{command}: shell builtin"), true),
                
                // External commands (simulated paths)
                "help" | "ls" | "cat" | "clear" | "cp" | "date" | "mines" | "mkdir" | "mv" | "rm" | "touch" | "which" | "whoami" | "neofetch" | "uptime" | "ps" | "kill" => {
                    (format!("/usr/bin/{command}"), true)
                }
                
                // Unknown command
                _ => (format!("{command} not found"), false),
            }
        }
    }
}

impl Executable for WhichCommand {
    fn execute(&self, path: &str, args: Vec<&str>) -> CommandRes {
        if args.is_empty() {
            return CommandRes::Err(Arc::new(move || "which: missing argument".into_any()));
        }
        
        let mut is_err = false;
        let results: Vec<String> = args.iter().map(|&command| {
            let (text, found) = self.get_which_result(path, command);
            if !found {
                is_err = true;
            }
            text
        }).collect();
        
        let output = results.join("\n");
        CommandRes::Output(Arc::new(move || output.clone().into_any()))
    }
    
    fn execute_pipeable(&self, _path: &str, _args: Vec<&str>, _stdin: &str) -> PipelineRes {
        todo!()
    }
}

pub struct Terminal {
    blog_posts: Vec<String>,
    history: Vec<String>,
    env_vars: HashMap<String, String>,
    processes: Vec<Process>,
    commands: HashMap<Command, Box<dyn Executable>>,
}

impl Terminal {
    pub fn new(blog_posts: &[String], history: Option<Vec<String>>) -> Self {
        let history = history.unwrap_or_default();
        let mut env_vars = HashMap::new();
        env_vars.insert("USER".to_string(), "user".to_string());
        env_vars.insert("HOME".to_string(), "/".to_string());
        env_vars.insert("SITE".to_string(), "hansbaker.com".to_string());
        env_vars.insert("VERSION".to_string(), env!("CARGO_PKG_VERSION").to_string());
        
        let processes = Self::initialize_processes();
        let commands = HashMap::new(); // Will be populated after construction
        
        let mut terminal = Self {
            blog_posts: blog_posts.to_owned(),
            history,
            env_vars,
            processes,
            commands,
        };
        
        terminal.initialize_commands();
        terminal
    }

    #[cfg(feature = "hydrate")]
    pub fn set_history(&mut self, history: Vec<String>) {
        self.history = history;
    }

    fn initialize_processes() -> Vec<Process> {
        vec![
            Process {
                pid: 1,
                user: "root".to_string(),
                cpu_percent: 2.3,
                mem_percent: 15.2,
                command: "leptos-server".to_string(),
            },
            Process {
                pid: 42,
                user: "app".to_string(),
                cpu_percent: 0.1,
                mem_percent: 8.7,
                command: "blog-renderer".to_string(),
            },
            Process {
                pid: 99,
                user: "app".to_string(),
                cpu_percent: 0.2,
                mem_percent: 3.1,
                command: "terminal-sim".to_string(),
            },
            Process {
                pid: 128,
                user: "app".to_string(),
                cpu_percent: 0.0,
                mem_percent: 2.5,
                command: "wasm-hydrator".to_string(),
            },
            Process {
                pid: 256,
                user: "app".to_string(),
                cpu_percent: 0.0,
                mem_percent: 1.8,
                command: "rss-generator".to_string(),
            },
        ]
    }

    fn initialize_commands(&mut self) {
        // Simple commands (no context needed)
        self.commands.insert(Command::Help, Box::new(HelpCommand));
        self.commands.insert(Command::Pwd, Box::new(PwdCommand));
        self.commands.insert(Command::WhoAmI, Box::new(WhoAmICommand));
        self.commands.insert(Command::Clear, Box::new(ClearCommand));
        self.commands.insert(Command::Neofetch, Box::new(NeofetchCommand));
        self.commands.insert(Command::Mines, Box::new(MinesCommand));
        self.commands.insert(Command::Sudo, Box::new(SudoCommand));
        self.commands.insert(Command::Echo, Box::new(EchoCommand));
        self.commands.insert(Command::Date, Box::new(DateCommand));
        self.commands.insert(Command::Uptime, Box::new(UptimeCommand));

        // Process commands
        self.commands.insert(Command::Ps, Box::new(PsCommand::new(self.processes.clone())));
        self.commands.insert(Command::Kill, Box::new(KillCommand::new(self.processes.clone())));
        
        // Filesystem commands
        self.commands.insert(Command::Which, Box::new(WhichCommand::new(self.blog_posts.clone())));

        // Note: History command left in legacy system due to mutable requirements
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
            if let Some(args) = trimmed.strip_prefix(alias_str) {
                if trimmed == alias_str {
                    return alias.expand("");
                } else if trimmed.starts_with(&format!("{alias_str} ")) {
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
                let var_ref = format!("${var_name}");
                result = result.replace(&var_ref, value);
            } else {
                // Replace with empty string if variable not found
                let var_ref = format!("${var_name}");
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
        // Convert string to Command enum for type-safe lookup
        let cmd = Command::from(cmd_text);
        
        // Try to find command in the new trait-based registry first
        if let Some(command) = self.commands.get(&cmd) {
            return command.execute(path, parts.collect());
        }
        
        // Fall back to legacy command handling for unimplemented commands
        match cmd {
            Command::Ls => self.handle_ls(path, parts.collect()),
            Command::Cd => self.handle_cd(path, parts.collect()),
            Command::Cat => self.handle_cat(path, parts.collect()),
            Command::MkDir => self.handle_mkdir(path, parts.collect()),
            Command::Rm => self.handle_rm(path, parts.collect()),
            Command::History => self.handle_history(parts.collect()),
            Command::Mv => self.handle_mv(path, parts.collect()),
            Command::Cp => self.handle_cp(path, parts.collect()),
            Command::Touch => self.handle_touch(path, parts.collect()),
            Command::Unknown => self.handle_unknown(path, cmd_text, parts.collect()),
            // Trait-based commands are handled above
            _ => self.handle_unknown(path, cmd_text, parts.collect()),
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
        let is_executable = target.is_executable() && target_string.contains("/"); 
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
                            MinesCommand.execute(path, args)
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
        assert!(matches!(Command::from("uptime"), Command::Uptime));
        assert!(matches!(Command::from("ps"), Command::Ps));
        assert!(matches!(Command::from("kill"), Command::Kill));
        assert!(matches!(Command::from("unknown"), Command::Unknown));
    }

    #[test]
    fn test_uptime_command() {
        let mut terminal = create_test_terminal();
        
        // Test basic uptime command
        let result = terminal.handle_command("/", "uptime");
        match result {
            CommandRes::Output(_) => {}, // Expected
            _ => panic!("Expected output for uptime command"),
        }
    }

    #[test]
    fn test_ps_command() {
        let mut terminal = create_test_terminal();
        
        // Test basic ps command
        let result = terminal.handle_command("/", "ps");
        match result {
            CommandRes::Output(_) => {}, // Expected
            _ => panic!("Expected output for ps command"),
        }
        
        // Test ps aux command
        let result = terminal.handle_command("/", "ps aux");
        match result {
            CommandRes::Output(_) => {}, // Expected
            _ => panic!("Expected output for ps aux command"),
        }
        
        // Test ps with invalid argument
        let result = terminal.handle_command("/", "ps invalid");
        match result {
            CommandRes::Err(_) => {}, // Expected
            _ => panic!("Expected error for ps with invalid argument"),
        }
    }

    #[test]
    fn test_kill_command() {
        let mut terminal = create_test_terminal();
        
        // Test kill with valid PID (should show permission denied)
        let result = terminal.handle_command("/", "kill 1");
        match result {
            CommandRes::Err(_) => {}, // Expected (permission denied)
            _ => panic!("Expected error for kill 1"),
        }
        
        // Test kill with PID 42 easter egg
        let result = terminal.handle_command("/", "kill 42");
        match result {
            CommandRes::Err(_) => {}, // Expected (with easter egg message)
            _ => panic!("Expected error for kill 42"),
        }
        
        // Test kill with force flag
        let result = terminal.handle_command("/", "kill -9 99");
        match result {
            CommandRes::Err(_) => {}, // Expected (permission denied)
            _ => panic!("Expected error for kill -9 99"),
        }
        
        // Test kill with non-existent PID
        let result = terminal.handle_command("/", "kill 999");
        match result {
            CommandRes::Err(_) => {}, // Expected (no such process)
            _ => panic!("Expected error for kill 999"),
        }
        
        // Test kill without arguments
        let result = terminal.handle_command("/", "kill");
        match result {
            CommandRes::Err(_) => {}, // Expected
            _ => panic!("Expected error for kill without arguments"),
        }
        
        // Test kill with invalid PID
        let result = terminal.handle_command("/", "kill abc");
        match result {
            CommandRes::Err(_) => {}, // Expected
            _ => panic!("Expected error for kill with invalid PID"),
        }
    }

    #[test]
    fn test_system_commands_in_tab_completion() {
        let terminal = create_test_terminal();
        
        // Test that new system commands are included in tab completion
        let commands = terminal.tab_commands("u");
        assert!(commands.contains(&"uptime".to_string()));
        
        let commands = terminal.tab_commands("p");
        assert!(commands.contains(&"ps".to_string()));
        
        let commands = terminal.tab_commands("k");
        assert!(commands.contains(&"kill".to_string()));
    }

    #[test]
    fn test_system_commands_in_which() {
        let mut terminal = create_test_terminal();
        
        // Test which with new system commands
        let result = terminal.handle_command("/", "which uptime");
        match result {
            CommandRes::Output(_) => {}, // Expected
            _ => panic!("Expected output for which uptime"),
        }
        
        let result = terminal.handle_command("/", "which ps kill");
        match result {
            CommandRes::Output(_) => {}, // Expected
            _ => panic!("Expected output for which ps kill"),
        }
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
