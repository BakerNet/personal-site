use super::command::{Cmd, CmdAlias, Command, CommandRes, VfsCommand};
use super::simple_tools::MinesCommand;
use super::vfs::{FileContent, VfsNodeType, VirtualFilesystem};
use indextree::NodeId;

pub struct WhichCommand;

impl WhichCommand {
    pub fn new() -> Self {
        Self
    }

    fn get_which_result(
        &self,
        vfs: &VirtualFilesystem,
        current_dir: NodeId,
        command: &str,
    ) -> (String, bool) {
        // If the command contains a path separator, treat it as a file path
        if command.contains('/') {
            // Resolve the path using VFS
            let node_id = if let Ok(node_id) = vfs.resolve_path(current_dir, command) {
                node_id
            } else {
                return (format!("{command} not found"), false);
            };
            match vfs.get_node(node_id) {
                Some(node) if node.is_executable() => (command.to_string(), true),
                _ => (format!("{command} not found"), false),
            }
        } else if let Some(alias) = CmdAlias::from_str(command) {
            // Check if it's an alias first
            let expansion = alias.expand("");
            (format!("{command}: aliased to {expansion}"), true)
        } else if let Some(cmd) = Cmd::from_str(command) {
            // Known command - get its simulated path or mark as builtin
            if cmd.is_builtin() {
                (format!("{command}: shell builtin"), true)
            } else if let Some(path) = cmd.simulated_path() {
                (path, true)
            } else {
                (format!("{command} not found"), false)
            }
        } else {
            // Unknown command
            (format!("{command} not found"), false)
        }
    }
}

impl VfsCommand for WhichCommand {
    fn execute(
        &self,
        vfs: &mut VirtualFilesystem,
        current_dir: NodeId,
        args: Vec<&str>,
        _stdin: Option<&str>,
        _is_tty: bool,
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
                let (text, found) = self.get_which_result(vfs, current_dir, command);
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
    command_name: String,
}

impl UnknownCommand {
    pub fn new(command_name: String) -> Self {
        Self { command_name }
    }
}

impl VfsCommand for UnknownCommand {
    fn execute(
        &self,
        vfs: &mut VirtualFilesystem,
        current_dir: NodeId,
        args: Vec<&str>,
        _stdin: Option<&str>,
        is_tty: bool,
    ) -> CommandRes {
        let target_string = &self.command_name;

        // Try to resolve the path using VFS
        let node_id = match vfs.resolve_path(current_dir, target_string) {
            Ok(node_id) => node_id,
            Err(_) => {
                let error_msg = format!("command not found: {target_string}");
                return CommandRes::new().with_error().with_stderr(error_msg);
            }
        };

        let node = match vfs.get_node(node_id) {
            Some(node) => node,
            None => {
                let error_msg = format!("command not found: {target_string}");
                return CommandRes::new().with_error().with_stderr(error_msg);
            }
        };

        let is_executable = node.is_executable() && target_string.contains("/");

        // Check if arguments are allowed for non-executable files
        if !args.is_empty() && !is_executable {
            let error_msg = format!("command not found: {target_string}");
            return CommandRes::new().with_error().with_stderr(error_msg);
        }

        if node.name == "mines.sh" && is_executable {
            let path = vfs.get_node_path(current_dir);
            return MinesCommand.execute(&path, args, None, is_tty);
        }

        match &node.node_type {
            VfsNodeType::Directory => {
                let target_path = vfs.get_node_path(node_id);
                CommandRes::redirect(target_path)
            }
            VfsNodeType::File { content } => {
                // Check for directory syntax on file
                if target_string.ends_with("/") {
                    let error_msg = format!("not a directory: {target_string}");
                    return CommandRes::new().with_error().with_stderr(error_msg);
                }

                match content {
                    FileContent::NavFile(s) => CommandRes::redirect(s.clone()),
                    FileContent::Static(_) | FileContent::Dynamic(_) => {
                        let error_msg = if target_string.contains("/") {
                            format!("permission denied: {target_string}")
                        } else {
                            format!("command not found: {target_string}")
                        };
                        CommandRes::new().with_error().with_stderr(error_msg)
                    }
                }
            }
            VfsNodeType::Link { .. } => {
                let error_msg = format!("command not found: {target_string}");
                CommandRes::new().with_error().with_stderr(error_msg)
            }
        }
    }
}
