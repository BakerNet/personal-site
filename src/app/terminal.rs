mod command;
mod components;
mod fs;
mod fs_tools;
mod ps_tools;
mod simple_tools;
mod system_tools;

pub use command::CommandRes;
pub use components::ColumnarView;

use std::{collections::HashMap, sync::Arc};

use leptos::prelude::*;

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
    pub fn history(&self) -> Vec<String> {
        if self.history.len() > 100 {
            self.history[self.history.len() - 100..].to_vec()
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
        self.history.push(input.to_string());

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
                    return CommandRes::new()
                        .with_stdout(
                            "history cleared",
                            Some(Arc::new(move || "history cleared".into_any())),
                        );
                }
                // For non-clear history commands, update the command with current history before executing
                HistoryCommand::new(&self.history).execute(path, args, None, true)
            }
            Command::Unknown => {
                let unknown_cmd =
                    UnknownCommand::new(self.blog_posts.clone(), cmd_text.to_string());
                unknown_cmd.execute(path, parts.collect(), None, true)
            }
            // All commands should now be handled by the trait system
            _ => {
                panic!("Command not implemented in trait system: {}", cmd_text);
            }
        }
    }

    pub fn handle_start_hist(&self, input: &str) -> Vec<String> {
        if input.trim().is_empty() {
            self.history.clone()
        } else {
            self.history
                .iter()
                .filter(|s| s.starts_with(input))
                .map(|s| s.to_string())
                .collect()
        }
    }

    pub fn handle_start_tab(&mut self, path: &str, input: &str) -> Vec<String> {
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
            Command::Cd => self
                .tab_opts(path, parts.last().unwrap_or_default())
                .into_iter()
                .filter(|s| s.ends_with("/"))
                .collect(),
            _ => self.tab_opts(path, parts.last().unwrap_or_default()),
        }
    }

    fn tab_opts(&self, path: &str, target_path: &str) -> Vec<String> {
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
                .filter(|s| s.starts_with(prefix) && s != prefix)
                .collect(),
            _ => Vec::new(),
        }
    }

    fn tab_commands(&self, cmd_text: &str) -> Vec<String> {
        let mut commands = Command::all()
            .into_iter()
            .filter(|s| s.starts_with(cmd_text))
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

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

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     fn create_test_terminal() -> Terminal {
//         Terminal::new(
//             &[],
//             Some(vec![
//                 "ls".to_string(),
//                 "pwd".to_string(),
//                 "echo hello".to_string(),
//             ]),
//         )
//     }
//
//     #[test]
//     fn test_environment_variables() {
//         let terminal = create_test_terminal();
//
//         // Test basic environment variable expansion
//         let expanded = terminal.expand_env_vars("/", "echo $USER");
//         assert_eq!(expanded, "echo user");
//
//         let expanded = terminal.expand_env_vars("/blog", "echo $PWD");
//         assert_eq!(expanded, "echo /blog");
//
//         let expanded = terminal.expand_env_vars("/", "echo $HOME");
//         assert_eq!(expanded, "echo /");
//
//         let expanded = terminal.expand_env_vars("/", "echo $SITE");
//         assert_eq!(expanded, "echo hansbaker.com");
//     }
//
//     #[test]
//     fn test_command_aliases() {
//         let terminal = create_test_terminal();
//
//         // Test alias processing
//         assert_eq!(terminal.process_aliases("ll"), "ls -la");
//         assert_eq!(terminal.process_aliases("la"), "ls -a");
//         assert_eq!(terminal.process_aliases("h"), "history");
//         assert_eq!(terminal.process_aliases("ll /blog"), "ls -la /blog");
//         assert_eq!(
//             terminal.process_aliases("regular_command"),
//             "regular_command"
//         );
//     }
//
//     #[test]
//     fn test_history_command() {
//         let mut terminal = create_test_terminal();
//
//         // Test basic history display
//         let result = terminal.handle_command("/", "history");
//         match result {
//             CommandRes::Output(_) => {} // Expected
//             _ => panic!("Expected output for history command"),
//         }
//
//         // Test history with count
//         let result = terminal.handle_command("/", "history 2");
//         match result {
//             CommandRes::Output(_) => {} // Expected
//             _ => panic!("Expected output for history 2 command"),
//         }
//
//         // Test history clear
//         let result = terminal.handle_command("/", "history -c");
//         match result {
//             CommandRes::Output(_) => {} // Expected
//             _ => panic!("Expected output for history -c command"),
//         }
//     }
//
//     #[test]
//     fn test_which_command() {
//         let mut terminal = create_test_terminal();
//
//         // Test which with known command
//         let result = terminal.handle_command("/", "which ls");
//         match result {
//             CommandRes::Output(_) => {} // Expected
//             _ => panic!("Expected output for which ls"),
//         }
//
//         // Test which with builtin
//         let result = terminal.handle_command("/", "which cd");
//         match result {
//             CommandRes::Output(_) => {} // Expected
//             _ => panic!("Expected output for which cd"),
//         }
//
//         // Test which with alias
//         let result = terminal.handle_command("/", "which ll");
//         match result {
//             CommandRes::Output(_) => {} // Expected
//             _ => panic!("Expected output for which ll"),
//         }
//
//         // Test which with unknown command
//         let result = terminal.handle_command("/", "which nonexistent");
//         match result {
//             CommandRes::Output(_) => {} // Expected
//             _ => panic!("Expected output for which nonexistent"),
//         }
//
//         // Test which with executable file path
//         let result = terminal.handle_command("/", "which ./mines.sh");
//         match result {
//             CommandRes::Output(_) => {} // Expected
//             _ => panic!("Expected output for which ./mines.sh"),
//         }
//
//         // Test which with non-executable file path
//         let result = terminal.handle_command("/", "which ./thanks.txt");
//         match result {
//             CommandRes::Output(_) => {} // Expected
//             _ => panic!("Expected output for which ./thanks.txt"),
//         }
//
//         // Test which with non-existent file path
//         let result = terminal.handle_command("/", "which ./nonexistent.sh");
//         match result {
//             CommandRes::Output(_) => {} // Expected
//             _ => panic!("Expected output for which ./nonexistent.sh"),
//         }
//
//         // Test which with multiple arguments
//         let result = terminal.handle_command("/", "which ls ll cd");
//         match result {
//             CommandRes::Output(_) => {} // Expected
//             _ => panic!("Expected output for which with multiple arguments"),
//         }
//     }
//
//     #[test]
//     fn test_date_command() {
//         let mut terminal = create_test_terminal();
//
//         // Test basic date command
//         let result = terminal.handle_command("/", "date");
//         match result {
//             CommandRes::Output(_) => {} // Expected
//             _ => panic!("Expected output for date command"),
//         }
//
//         // Test date with format
//         let result = terminal.handle_command("/", "date +%Y-%m-%d");
//         match result {
//             CommandRes::Output(_) => {} // Expected
//             _ => panic!("Expected output for date +%Y-%m-%d"),
//         }
//
//         // Test date with time format
//         let result = terminal.handle_command("/", "date \"+%H:%M:%S\"");
//         match result {
//             CommandRes::Output(_) => {} // Expected
//             _ => panic!("Expected output for date +%H:%M:%S"),
//         }
//     }
//
//     #[test]
//     fn test_tab_completion_includes_new_commands() {
//         let terminal = create_test_terminal();
//
//         // Test that new commands are included in tab completion
//         let commands = terminal.tab_commands("h");
//         assert!(commands.contains(&"history".to_string()));
//         assert!(commands.contains(&"h".to_string())); // alias
//
//         let commands = terminal.tab_commands("w");
//         assert!(commands.contains(&"which".to_string()));
//         assert!(commands.contains(&"whoami".to_string()));
//
//         let commands = terminal.tab_commands("d");
//         assert!(commands.contains(&"date".to_string()));
//
//         let commands = terminal.tab_commands("l");
//         assert!(commands.contains(&"ls".to_string()));
//         assert!(commands.contains(&"ll".to_string())); // alias
//         assert!(commands.contains(&"la".to_string())); // alias
//     }
//
//     #[test]
//     fn test_command_parsing() {
//         // Test that new commands are parsed correctly
//         assert!(matches!(Command::from("history"), Command::History));
//         assert!(matches!(Command::from("which"), Command::Which));
//         assert!(matches!(Command::from("date"), Command::Date));
//         assert!(matches!(Command::from("uptime"), Command::Uptime));
//         assert!(matches!(Command::from("ps"), Command::Ps));
//         assert!(matches!(Command::from("kill"), Command::Kill));
//         assert!(matches!(Command::from("unknown"), Command::Unknown));
//     }
//
//     #[test]
//     fn test_uptime_command() {
//         let mut terminal = create_test_terminal();
//
//         // Test basic uptime command
//         let result = terminal.handle_command("/", "uptime");
//         match result {
//             CommandRes::Output(_) => {} // Expected
//             _ => panic!("Expected output for uptime command"),
//         }
//     }
//
//     #[test]
//     fn test_ps_command() {
//         let mut terminal = create_test_terminal();
//
//         // Test basic ps command
//         let result = terminal.handle_command("/", "ps");
//         match result {
//             CommandRes::Output(_) => {} // Expected
//             _ => panic!("Expected output for ps command"),
//         }
//
//         // Test ps aux command
//         let result = terminal.handle_command("/", "ps aux");
//         match result {
//             CommandRes::Output(_) => {} // Expected
//             _ => panic!("Expected output for ps aux command"),
//         }
//
//         // Test ps with invalid argument
//         let result = terminal.handle_command("/", "ps invalid");
//         match result {
//             CommandRes::Err(_) => {} // Expected
//             _ => panic!("Expected error for ps with invalid argument"),
//         }
//     }
//
//     #[test]
//     fn test_kill_command() {
//         let mut terminal = create_test_terminal();
//
//         // Test kill with valid PID (should show permission denied)
//         let result = terminal.handle_command("/", "kill 1");
//         match result {
//             CommandRes::Err(_) => {} // Expected (permission denied)
//             _ => panic!("Expected error for kill 1"),
//         }
//
//         // Test kill with PID 42 easter egg
//         let result = terminal.handle_command("/", "kill 42");
//         match result {
//             CommandRes::Err(_) => {} // Expected (with easter egg message)
//             _ => panic!("Expected error for kill 42"),
//         }
//
//         // Test kill with force flag
//         let result = terminal.handle_command("/", "kill -9 99");
//         match result {
//             CommandRes::Err(_) => {} // Expected (permission denied)
//             _ => panic!("Expected error for kill -9 99"),
//         }
//
//         // Test kill with non-existent PID
//         let result = terminal.handle_command("/", "kill 999");
//         match result {
//             CommandRes::Err(_) => {} // Expected (no such process)
//             _ => panic!("Expected error for kill 999"),
//         }
//
//         // Test kill without arguments
//         let result = terminal.handle_command("/", "kill");
//         match result {
//             CommandRes::Err(_) => {} // Expected
//             _ => panic!("Expected error for kill without arguments"),
//         }
//
//         // Test kill with invalid PID
//         let result = terminal.handle_command("/", "kill abc");
//         match result {
//             CommandRes::Err(_) => {} // Expected
//             _ => panic!("Expected error for kill with invalid PID"),
//         }
//     }
//
//     #[test]
//     fn test_system_commands_in_tab_completion() {
//         let terminal = create_test_terminal();
//
//         // Test that new system commands are included in tab completion
//         let commands = terminal.tab_commands("u");
//         assert!(commands.contains(&"uptime".to_string()));
//
//         let commands = terminal.tab_commands("p");
//         assert!(commands.contains(&"ps".to_string()));
//
//         let commands = terminal.tab_commands("k");
//         assert!(commands.contains(&"kill".to_string()));
//     }
//
//     #[test]
//     fn test_system_commands_in_which() {
//         let mut terminal = create_test_terminal();
//
//         // Test which with new system commands
//         let result = terminal.handle_command("/", "which uptime");
//         match result {
//             CommandRes::Output(_) => {} // Expected
//             _ => panic!("Expected output for which uptime"),
//         }
//
//         let result = terminal.handle_command("/", "which ps kill");
//         match result {
//             CommandRes::Output(_) => {} // Expected
//             _ => panic!("Expected output for which ps kill"),
//         }
//     }
//
//     #[test]
//     fn test_error_handling() {
//         let mut terminal = create_test_terminal();
//
//         // Test which without arguments
//         let result = terminal.handle_command("/", "which");
//         match result {
//             CommandRes::Err(_) => {} // Expected
//             _ => panic!("Expected error for which without arguments"),
//         }
//
//         // Test history with invalid argument
//         let result = terminal.handle_command("/", "history invalid");
//         match result {
//             CommandRes::Err(_) => {} // Expected
//             _ => panic!("Expected error for history with invalid argument"),
//         }
//
//         // Test date with invalid format
//         let result = terminal.handle_command("/", "date invalid");
//         match result {
//             CommandRes::Err(_) => {} // Expected
//             _ => panic!("Expected error for date with invalid format"),
//         }
//     }
// }
