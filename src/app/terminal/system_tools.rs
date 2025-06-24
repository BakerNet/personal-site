use super::command::{CommandAlias, CommandRes, Executable};
use super::fs::{path_target_to_target_path, File, Target};
use super::simple_tools::MinesCommand;

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
                "help" | "ls" | "cat" | "clear" | "cp" | "date" | "mines" | "mkdir" | "mv"
                | "rm" | "touch" | "which" | "whoami" | "neofetch" | "uptime" | "ps" | "kill" => {
                    (format!("/usr/bin/{command}"), true)
                }

                // Unknown command
                _ => (format!("{command} not found"), false),
            }
        }
    }
}

impl Executable for WhichCommand {
    fn execute(
        &self,
        path: &str,
        args: Vec<&str>,
        _stdin: Option<&str>,
        _is_output_tty: bool,
    ) -> CommandRes {
        if args.is_empty() {
            return CommandRes::new()
                .with_error()
                .with_stderr("which: missing argument");
        }

        let mut is_err = false;
        let results: Vec<String> = args
            .iter()
            .map(|&command| {
                let (text, found) = self.get_which_result(path, command);
                if !found {
                    is_err = true;
                }
                text
            })
            .collect();

        let output = results.join("\n");
        let mut res = CommandRes::new().with_stdout_text(output);
        if is_err {
            res = res.with_error();
        }
        res
    }
}

pub struct UnknownCommand {
    blog_posts: Vec<String>,
    command_name: String,
}

impl UnknownCommand {
    pub fn new(blog_posts: Vec<String>, command_name: String) -> Self {
        Self {
            blog_posts,
            command_name,
        }
    }
}

impl Executable for UnknownCommand {
    fn execute(
        &self,
        path: &str,
        args: Vec<&str>,
        _stdin: Option<&str>,
        is_output_tty: bool,
    ) -> CommandRes {
        let target_string = self.command_name.clone();
        let target_path = path_target_to_target_path(path, &self.command_name, false);
        let target = Target::from_str(&target_path, &self.blog_posts);
        let is_executable = target.is_executable() && target_string.contains("/");
        if !args.is_empty() && !is_executable {
            // only mines.sh and nav.rs are executable, so only these can accept arguments
            let error_msg = format!("command not found: {target_string}");
            return CommandRes::new().with_error().with_stderr(error_msg);
        }
        match target {
            Target::Dir(_) => CommandRes::redirect(target_path),
            Target::File(f) => {
                if target_string.ends_with("/") {
                    let error_msg = format!("not a directory: {target_string}");
                    return CommandRes::new().with_error().with_stderr(error_msg);
                }
                match f {
                    File::Nav(s) => CommandRes::redirect(s),
                    File::MinesSh => {
                        if is_executable {
                            MinesCommand.execute(path, args, None, is_output_tty)
                        } else {
                            let error_msg = format!("command not found: {target_string}\nhint: try 'mines' or '/mines.sh'");
                            CommandRes::new().with_error().with_stderr(error_msg)
                        }
                    }
                    File::ThanksTxt | File::ZshRc => {
                        let error_msg = if target_string.contains("/") {
                            format!("permission denied: {target_string}")
                        } else {
                            format!("command not found: {target_string}")
                        };
                        CommandRes::new().with_error().with_stderr(error_msg)
                    }
                }
            }
            Target::Invalid => {
                let error_msg = format!("command not found: {target_string}");
                CommandRes::new().with_error().with_stderr(error_msg)
            }
        }
    }
}
