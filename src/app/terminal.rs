mod command;
mod components;
pub mod fs;
mod fs_tools;
mod ps_tools;
mod simple_tools;
mod system_tools;

pub use command::CommandRes;
pub use components::ColumnarView;

use std::collections::{HashMap, VecDeque};

use command::{Command, CommandAlias, Executable};
use fs::{path_target_to_target_path, Target};
use fs_tools::{
    CatCommand, CdCommand, CpCommand, LsCommand, MkdirCommand, MvCommand, RmCommand, TouchCommand,
};
use ps_tools::{KillCommand, Process, PsCommand};
use simple_tools::{
    ClearCommand, DateCommand, EchoCommand, HelpCommand, HistoryCommand, MinesCommand,
    NeofetchCommand, PwdCommand, SudoCommand, UptimeCommand, WhoAmICommand,
};
use system_tools::{UnknownCommand, WhichCommand};

static HISTORY_SIZE: usize = 1000;

pub struct Terminal {
    blog_posts: Vec<String>,
    history: VecDeque<String>,
    env_vars: HashMap<String, String>,
    processes: Vec<Process>,
    commands: HashMap<Command, Box<dyn Executable>>,
}

impl Terminal {
    pub fn new(blog_posts: &[String], history: Option<VecDeque<String>>) -> Self {
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
    pub fn set_history(&mut self, history: VecDeque<String>) {
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
        self.commands
            .insert(Command::WhoAmI, Box::new(WhoAmICommand));
        self.commands.insert(Command::Clear, Box::new(ClearCommand));
        self.commands
            .insert(Command::Neofetch, Box::new(NeofetchCommand));
        self.commands.insert(Command::Mines, Box::new(MinesCommand));
        self.commands.insert(Command::Sudo, Box::new(SudoCommand));
        self.commands.insert(Command::Echo, Box::new(EchoCommand));
        self.commands.insert(Command::Date, Box::new(DateCommand));
        self.commands
            .insert(Command::Uptime, Box::new(UptimeCommand));

        // Process commands
        self.commands.insert(
            Command::Ps,
            Box::new(PsCommand::new(self.processes.clone())),
        );
        self.commands.insert(
            Command::Kill,
            Box::new(KillCommand::new(self.processes.clone())),
        );

        // Filesystem commands
        self.commands.insert(
            Command::Which,
            Box::new(WhichCommand::new(self.blog_posts.clone())),
        );
        self.commands.insert(
            Command::Ls,
            Box::new(LsCommand::new(self.blog_posts.clone())),
        );
        self.commands.insert(
            Command::Cat,
            Box::new(CatCommand::new(self.blog_posts.clone())),
        );
        self.commands.insert(
            Command::Cd,
            Box::new(CdCommand::new(self.blog_posts.clone())),
        );
        self.commands.insert(
            Command::Touch,
            Box::new(TouchCommand::new(self.blog_posts.clone())),
        );
        self.commands.insert(
            Command::MkDir,
            Box::new(MkdirCommand::new(self.blog_posts.clone())),
        );
        self.commands.insert(
            Command::Rm,
            Box::new(RmCommand::new(self.blog_posts.clone())),
        );
        self.commands.insert(
            Command::Cp,
            Box::new(CpCommand::new(self.blog_posts.clone())),
        );
        self.commands.insert(
            Command::Mv,
            Box::new(MvCommand::new(self.blog_posts.clone())),
        );

        // History command and Unknown commands handled separately
    }

    #[cfg(feature = "hydrate")]
    pub fn history(&self) -> VecDeque<String> {
        self.history.clone()
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
            let end = remaining
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .unwrap_or(remaining.len());
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
            return CommandRes::new();
        }
        self.history.push_back(input.to_string());
        if self.history.len() > HISTORY_SIZE {
            self.history.pop_front();
        }

        // Process command aliases first
        let aliased_input = self.process_aliases(input);

        // Expand environment variables in the input
        let expanded_input = self.expand_env_vars(path, &aliased_input);

        let mut parts = expanded_input.split_whitespace();
        let cmd_text = if let Some(word) = parts.next() {
            word
        } else {
            unreachable!("Should have returned early if empty");
        };
        // Convert string to Command enum for type-safe lookup
        let cmd = Command::from(cmd_text);

        // Try to find command in the new trait-based registry first
        if let Some(command) = self.commands.get(&cmd) {
            // For now, assume not piped and output to TTY
            return command.execute(path, parts.collect(), None, true);
        }

        // Fall back to legacy command handling for unimplemented commands
        match cmd {
            // Special handling for history command due to mutable state requirements
            // The history -c flag requires mutable access to clear the terminal's history Vec,
            // which cannot be provided through the immutable Executable trait interface.
            // Therefore, we handle -c here in the terminal and update the HistoryCommand
            // with current history for other operations.
            Command::History => {
                let args: Vec<&str> = parts.collect();
                if args.len() == 1 && args[0] == "-c" {
                    self.history.clear();
                    return CommandRes::new().with_stdout_text("history cleared");
                }
                self.history.make_contiguous();
                // For non-clear history commands, update the command with current history before executing
                HistoryCommand::new(self.history.as_slices().0).execute(path, args, None, true)
            }
            Command::Unknown => {
                let unknown_cmd =
                    UnknownCommand::new(self.blog_posts.clone(), cmd_text.to_string());
                unknown_cmd.execute(path, parts.collect(), None, true)
            }
            // All commands should now be handled by the trait system
            _ => {
                panic!("Command not implemented in trait system: {cmd_text}");
            }
        }
    }

    pub fn handle_start_hist(&self, input: &str) -> Vec<String> {
        if input.trim().is_empty() {
            self.history.iter().cloned().collect()
        } else {
            self.history
                .iter()
                .filter(|s| s.starts_with(input))
                .cloned()
                .collect()
        }
    }

    pub fn handle_start_tab(&mut self, path: &str, input: &str) -> Vec<fs::DirContentItem> {
        let mut parts = input.split_whitespace();
        let cmd_text = if let Some(word) = parts.next() {
            word
        } else {
            return Vec::new();
        };
        let cmd = Command::from(cmd_text);
        let mut parts = parts.peekable();
        match cmd {
            Command::Unknown if parts.peek().is_none() && !input.ends_with(" ") => {
                if cmd_text.contains("/") {
                    self.tab_opts(path, cmd_text)
                } else {
                    self.tab_commands(cmd_text)
                }
            }
            _ if parts.peek().is_none() && !input.ends_with(" ") => Vec::new(),
            Command::Cd => self.tab_dirs(path, parts.last().unwrap_or_default()),
            _ => self.tab_opts(path, parts.last().unwrap_or_default()),
        }
    }

    fn tab_opts(&self, path: &str, target_path: &str) -> Vec<fs::DirContentItem> {
        let no_prefix = target_path.ends_with("/") || target_path.is_empty();
        let target_path = path_target_to_target_path(path, target_path, true);
        let (target_path, prefix) = if no_prefix {
            (target_path.as_ref(), "")
        } else if let Some(pos) = target_path.rfind("/") {
            let new_target_path = &target_path[..pos];
            let new_target_path = if new_target_path.is_empty() {
                "/"
            } else {
                new_target_path
            };
            (new_target_path, &target_path[pos + 1..])
        } else {
            return Vec::new();
        };
        let target = Target::from_str(target_path, &self.blog_posts);
        match target {
            Target::Dir(d) => d
                .contents(&self.blog_posts, prefix.starts_with("."))
                .into_iter()
                .filter(|item| {
                    item.0.starts_with(prefix)
                        && (item.0 != prefix || matches!(item.1, Target::Dir(_)))
                })
                .map(|item| {
                    // Add appropriate suffix for display
                    let display_name = match &item.1 {
                        Target::Dir(_) => format!("{}/", item.0),
                        Target::File(_) if item.1.is_executable() => format!("{}*", item.0),
                        _ => item.0.clone(),
                    };
                    fs::DirContentItem(display_name, item.1)
                })
                .collect(),
            _ => Vec::new(),
        }
    }

    fn tab_dirs(&self, path: &str, target_path: &str) -> Vec<fs::DirContentItem> {
        let no_prefix = target_path.ends_with("/") || target_path.is_empty();
        let target_path = path_target_to_target_path(path, target_path, true);
        let (target_path, prefix) = if no_prefix {
            (target_path.as_ref(), "")
        } else if let Some(pos) = target_path.rfind("/") {
            let new_target_path = &target_path[..pos];
            let new_target_path = if new_target_path.is_empty() {
                "/"
            } else {
                new_target_path
            };
            (new_target_path, &target_path[pos + 1..])
        } else {
            return Vec::new();
        };
        let target = Target::from_str(target_path, &self.blog_posts);
        match target {
            Target::Dir(d) => d
                .contents(&self.blog_posts, prefix.starts_with("."))
                .into_iter()
                .filter_map(|item| {
                    // Only include directories
                    if matches!(item.1, Target::Dir(_)) && item.0.starts_with(prefix) {
                        // Add "/" suffix to indicate it's a directory
                        let display_name = format!("{}/", item.0);
                        Some(fs::DirContentItem(display_name, item.1))
                    } else {
                        None
                    }
                })
                .collect(),
            _ => Vec::new(),
        }
    }

    fn tab_commands(&self, cmd_text: &str) -> Vec<fs::DirContentItem> {
        let mut commands = Command::all()
            .into_iter()
            .filter(|s| s.starts_with(cmd_text))
            .map(|s| fs::DirContentItem(s.to_string(), Target::File(fs::File::MinesSh))) // Use executable as dummy type
            .collect::<Vec<_>>();

        // Add aliases
        for alias in CommandAlias::all() {
            let alias_str = alias.as_str();
            if alias_str.starts_with(cmd_text) {
                commands.push(fs::DirContentItem(
                    alias_str.to_string(),
                    Target::File(fs::File::MinesSh),
                ));
            }
        }

        commands.sort_by(|a, b| a.0.cmp(&b.0));
        commands
    }
}
