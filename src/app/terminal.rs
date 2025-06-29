mod command;
mod components;
mod fs_tools;
mod ps_tools;
mod simple_tools;
mod system_tools;
pub mod vfs;

pub use command::CommandRes;
pub use components::ColumnarView;

use std::collections::{HashMap, VecDeque};

use command::{Cmd, CmdAlias, Command, VfsCommand};
use components::TextContent;
use fs_tools::{
    CatCommand, CdCommand, CpCommand, LsCommand, MkdirCommand, MvCommand, RmCommand, TouchCommand,
};
use indextree::NodeId;
use ps_tools::{KillCommand, Process, PsCommand};
use simple_tools::{
    ClearCommand, DateCommand, EchoCommand, HelpCommand, HistoryCommand, MinesCommand,
    NeofetchCommand, PwdCommand, SudoCommand, UptimeCommand, WhoAmICommand,
};
use system_tools::{UnknownCommand, WhichCommand};
use vfs::VirtualFilesystem;

static HISTORY_SIZE: usize = 1000;

#[derive(Debug, Clone)]
pub struct TabCompletionItem {
    pub completion_text: String, // The text to insert when selected
    pub is_directory: bool,      // Whether this is a directory
    pub is_executable: bool,     // Whether this is an executable file
}

impl TextContent for TabCompletionItem {
    fn text_content(&self) -> &str {
        &self.completion_text
    }
}

pub struct Terminal {
    history: VecDeque<String>,
    env_vars: HashMap<String, String>,
    processes: Vec<Process>,
    commands: HashMap<Cmd, Box<dyn Command>>,
    vfs_commands: HashMap<Cmd, Box<dyn VfsCommand>>,
    vfs: VirtualFilesystem,
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
        let vfs_commands = HashMap::new(); // Will be populated after construction

        let vfs = VirtualFilesystem::new(blog_posts.to_owned());

        let mut terminal = Self {
            history,
            env_vars,
            processes,
            commands,
            vfs_commands,
            vfs,
        };

        terminal.initialize_commands();
        terminal.initialize_vfs_commands();
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
        self.commands.insert(Cmd::Help, Box::new(HelpCommand));
        self.commands.insert(Cmd::Pwd, Box::new(PwdCommand));
        self.commands.insert(Cmd::WhoAmI, Box::new(WhoAmICommand));
        self.commands.insert(Cmd::Clear, Box::new(ClearCommand));
        self.commands
            .insert(Cmd::Neofetch, Box::new(NeofetchCommand));
        self.commands.insert(Cmd::Mines, Box::new(MinesCommand));
        self.commands.insert(Cmd::Sudo, Box::new(SudoCommand));
        self.commands.insert(Cmd::Echo, Box::new(EchoCommand));
        self.commands.insert(Cmd::Date, Box::new(DateCommand));
        self.commands.insert(Cmd::Uptime, Box::new(UptimeCommand));

        // Process commands
        self.commands
            .insert(Cmd::Ps, Box::new(PsCommand::new(self.processes.clone())));
        self.commands.insert(
            Cmd::Kill,
            Box::new(KillCommand::new(self.processes.clone())),
        );

        // History command and Unknown commands handled separately
    }

    fn initialize_vfs_commands(&mut self) {
        // VFS-aware commands that need direct filesystem access
        self.vfs_commands
            .insert(Cmd::Ls, Box::new(LsCommand::new()));
        self.vfs_commands
            .insert(Cmd::Cd, Box::new(CdCommand::new()));
        self.vfs_commands
            .insert(Cmd::Cat, Box::new(CatCommand::new()));
        self.vfs_commands
            .insert(Cmd::Touch, Box::new(TouchCommand::new()));
        self.vfs_commands
            .insert(Cmd::MkDir, Box::new(MkdirCommand::new()));
        self.vfs_commands
            .insert(Cmd::Rm, Box::new(RmCommand::new()));
        self.vfs_commands
            .insert(Cmd::Which, Box::new(WhichCommand::new()));
        self.vfs_commands
            .insert(Cmd::Cp, Box::new(CpCommand::new()));
        self.vfs_commands
            .insert(Cmd::Mv, Box::new(MvCommand::new()));
    }

    #[cfg(feature = "hydrate")]
    pub fn history(&self) -> VecDeque<String> {
        self.history.clone()
    }

    fn process_aliases(&self, input: &str) -> String {
        let trimmed = input.trim();

        for alias in CmdAlias::all() {
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
        let cmd = Cmd::from(cmd_text);

        // Try VFS commands first (they have priority)
        if let Some(vfs_command) = self.vfs_commands.get(&cmd) {
            let current_node = if let Ok(node_id) = self.vfs.resolve_path(self.vfs.get_root(), path)
            {
                node_id
            } else {
                self.vfs.get_root()
            };
            return vfs_command.execute(&mut self.vfs, current_node, parts.collect(), None, true);
        }

        // Try non-VFS commands
        if let Some(command) = self.commands.get(&cmd) {
            // For now, assume not piped and output to TTY
            return command.execute(path, parts.collect(), None, true);
        }

        // Fall back to special command handling for some commands
        match cmd {
            // Special handling for history command due to mutable state requirements
            // The history -c flag requires mutable access to clear the terminal's history Vec,
            // which cannot be provided through the immutable Executable trait interface.
            // Therefore, we handle -c here in the terminal and update the HistoryCommand
            // with current history for other operations.
            Cmd::History => {
                let args: Vec<&str> = parts.collect();
                if args.len() == 1 && args[0] == "-c" {
                    self.history.clear();
                    return CommandRes::new().with_stdout_text("history cleared");
                }
                self.history.make_contiguous();
                // For non-clear history commands, update the command with current history before executing
                HistoryCommand::new(self.history.as_slices().0).execute(path, args, None, true)
            }
            Cmd::Unknown => {
                // Handle unknown commands through VFS
                let unknown_cmd = UnknownCommand::new(cmd_text.to_string());
                let current_node =
                    if let Ok(node_id) = self.vfs.resolve_path(self.vfs.get_root(), path) {
                        node_id
                    } else {
                        self.vfs.get_root()
                    };
                unknown_cmd.execute(&mut self.vfs, current_node, parts.collect(), None, true)
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

    pub fn handle_start_tab(&mut self, path: &str, input: &str) -> Vec<TabCompletionItem> {
        let mut parts = input.split_whitespace();
        let cmd_text = if let Some(word) = parts.next() {
            word
        } else {
            return Vec::new();
        };
        let cmd = Cmd::from(cmd_text);
        let mut parts = parts.peekable();

        // Get current directory in VFS
        let current_dir = match self.vfs.resolve_path(self.vfs.get_root(), path) {
            Ok(node_id) => node_id,
            Err(_) => self.vfs.get_root(),
        };

        match cmd {
            Cmd::Unknown if parts.peek().is_none() && !input.ends_with(" ") => {
                if cmd_text.contains("/") {
                    self.vfs_tab_opts(current_dir, cmd_text)
                } else {
                    self.tab_commands(cmd_text)
                }
            }
            _ if parts.peek().is_none() && !input.ends_with(" ") => Vec::new(),
            Cmd::Cd => self.vfs_tab_dirs(current_dir, parts.last().unwrap_or_default()),
            _ => self.vfs_tab_opts(current_dir, parts.last().unwrap_or_default()),
        }
    }

    fn tab_commands(&self, cmd_text: &str) -> Vec<TabCompletionItem> {
        let mut commands = Cmd::all()
            .into_iter()
            .filter(|s| s.starts_with(cmd_text))
            .map(|s| TabCompletionItem {
                completion_text: s.to_string(),
                is_directory: false,
                is_executable: true, // Commands are executable
            })
            .collect::<Vec<_>>();

        // Add aliases
        for alias in CmdAlias::all() {
            let alias_str = alias.as_str();
            if alias_str.starts_with(cmd_text) {
                commands.push(TabCompletionItem {
                    completion_text: alias_str.to_string(),
                    is_directory: false,
                    is_executable: true,
                });
            }
        }

        commands.sort_by(|a, b| a.completion_text.cmp(&b.completion_text));
        commands
    }

    // VFS-based tab completion methods
    fn vfs_tab_opts(&self, current_dir: NodeId, target_path: &str) -> Vec<TabCompletionItem> {
        let no_prefix = target_path.ends_with("/") || target_path.is_empty();

        // Split the path to get directory and prefix for completion
        let (dir_path, prefix) = if no_prefix {
            (target_path, "")
        } else if let Some(pos) = target_path.rfind('/') {
            (&target_path[..=pos], &target_path[pos + 1..])
        } else {
            ("", target_path)
        };

        // Resolve the directory path
        let target_dir = if dir_path.is_empty() {
            Ok(current_dir)
        } else {
            self.vfs.resolve_path(current_dir, dir_path).or_else(|_| {
                // Try without trailing slash
                let trimmed = dir_path.trim_end_matches('/');
                if trimmed.is_empty() {
                    Ok(self.vfs.get_root())
                } else {
                    self.vfs.resolve_path(current_dir, trimmed)
                }
            })
        };

        let target_dir = match target_dir {
            Ok(node_id) => node_id,
            Err(_) => return Vec::new(),
        };

        // Get entries from the directory
        let entries = match self.vfs.list_directory(target_dir) {
            Ok(entries) => entries,
            Err(_) => return Vec::new(),
        };

        // Filter and convert entries
        let mut results = Vec::new();
        for entry in entries {
            if !prefix.is_empty() && !entry.name.starts_with(prefix) {
                continue;
            }

            // Skip hidden files unless prefix starts with '.'
            if entry.name.starts_with('.') && !prefix.starts_with('.') {
                continue;
            }

            // The completion text should just be the entry name, not the full path
            // Add appropriate suffix for display
            let completion_text = if entry.is_directory {
                format!("{}/", entry.name)
            } else if entry.is_executable {
                format!("{}*", entry.name)
            } else {
                entry.name.clone()
            };

            results.push(TabCompletionItem {
                completion_text,
                is_directory: entry.is_directory,
                is_executable: entry.is_executable,
            });
        }

        results.sort_by(|a, b| a.completion_text.cmp(&b.completion_text));
        results
    }

    fn vfs_tab_dirs(&self, current_dir: NodeId, target_path: &str) -> Vec<TabCompletionItem> {
        // Similar to vfs_tab_opts but only returns directories
        let results = self.vfs_tab_opts(current_dir, target_path);
        results
            .into_iter()
            .filter(|item| item.is_directory)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to extract stdout text from CommandRes
    fn get_stdout_text(res: &CommandRes) -> Option<String> {
        match res {
            CommandRes::Output { stdout_text, .. } => stdout_text.clone(),
            _ => None,
        }
    }

    // Helper function to extract stderr text from CommandRes
    fn get_stderr_text(res: &CommandRes) -> Option<String> {
        match res {
            CommandRes::Output { stderr_text, .. } => stderr_text.clone(),
            _ => None,
        }
    }

    // Helper function to check if a file exists in VFS
    fn vfs_file_exists(terminal: &mut Terminal, path: &str) -> bool {
        // Create a temporary VFS command to access the filesystem
        let result = terminal.handle_command("/", &format!("cat {}", path));

        // If cat succeeds and no "Is a directory" error, it's a file
        if !result.is_error() {
            return true;
        }

        // Check if error is "Is a directory" (meaning path exists but is dir)
        if let Some(stderr) = get_stderr_text(&result) {
            if stderr.contains("Is a directory") {
                return false; // Path exists but is a directory
            }
            if stderr.contains("No such file or directory") {
                return false; // Path doesn't exist
            }
        }

        false
    }

    // Helper function to check if a directory exists in VFS
    fn vfs_dir_exists(terminal: &mut Terminal, path: &str) -> bool {
        // Try to cd to the directory and back
        let current_path = "/"; // We'll assume we're testing from root
        let cd_result = terminal.handle_command(current_path, &format!("cd {}", path));

        match cd_result {
            CommandRes::Redirect(_) => true,
            _ => {
                // Check if it's a file by trying cat
                let cat_result = terminal.handle_command(current_path, &format!("cat {}", path));
                if let Some(stderr) = get_stderr_text(&cat_result) {
                    stderr.contains("Is a directory")
                } else {
                    false
                }
            }
        }
    }

    // Helper function to check if a specific item exists in a directory
    fn vfs_contains_file(terminal: &mut Terminal, dir_path: &str, filename: &str) -> bool {
        let full_path = if dir_path == "/" {
            format!("/{}", filename)
        } else {
            format!("{}/{}", dir_path, filename)
        };
        vfs_file_exists(terminal, &full_path) || vfs_dir_exists(terminal, &full_path)
    }

    #[test]
    fn test_cd_integration() {
        let blog_posts = vec!["test-post".to_string(), "another-post".to_string()];
        let mut terminal = Terminal::new(&blog_posts, None);

        // Test root directory contents via VFS
        assert!(vfs_dir_exists(&mut terminal, "/blog"));
        assert!(vfs_dir_exists(&mut terminal, "/cv"));
        assert!(vfs_file_exists(&mut terminal, "/mines.sh"));

        // Change to blog directory
        let cd_result = terminal.handle_command("/", "cd blog");
        if let CommandRes::Redirect(path) = cd_result {
            assert_eq!(path, "/blog");
        } else {
            panic!("cd should return a redirect");
        }

        // Verify blog directory contents by checking individual files exist
        assert!(vfs_contains_file(&mut terminal, "/blog", "test-post"));
        assert!(vfs_contains_file(&mut terminal, "/blog", "another-post"));
        assert!(vfs_contains_file(&mut terminal, "/blog", "nav.rs"));

        // Test relative navigation
        let relative_cd = terminal.handle_command("/blog", "cd ..");
        if let CommandRes::Redirect(path) = relative_cd {
            assert_eq!(path, "/");
        } else {
            panic!("cd .. should return a redirect to root");
        }

        // Test cd to specific blog post
        let post_cd = terminal.handle_command("/blog", "cd test-post");
        if let CommandRes::Redirect(path) = post_cd {
            assert_eq!(path, "/blog/test-post");
        } else {
            panic!("cd test-post should redirect");
        }
    }

    #[test]
    fn test_path_navigation() {
        let blog_posts = vec!["my-post".to_string()];
        let mut terminal = Terminal::new(&blog_posts, None);

        // Test absolute path navigation
        let abs_cd = terminal.handle_command("/", "cd /blog/my-post");
        if let CommandRes::Redirect(path) = abs_cd {
            assert_eq!(path, "/blog/my-post");
        } else {
            panic!("absolute cd should work");
        }

        // Test .. from deep directory
        let up_cd = terminal.handle_command("/blog/my-post", "cd ../..");
        if let CommandRes::Redirect(path) = up_cd {
            assert_eq!(path, "/");
        } else {
            panic!("cd ../.. should go to root");
        }

        // Test ~ expansion
        let home_cd = terminal.handle_command("/blog", "cd ~");
        if let CommandRes::Redirect(path) = home_cd {
            assert_eq!(path, "/");
        } else {
            panic!("cd ~ should go to root");
        }

        // Test ~/path expansion
        let home_path_cd = terminal.handle_command("/blog", "cd ~/cv");
        if let CommandRes::Redirect(path) = home_path_cd {
            assert_eq!(path, "/cv");
        } else {
            panic!("cd ~/cv should work");
        }
    }

    #[test]
    fn test_cat_command() {
        let blog_posts = vec!["test-post".to_string()];
        let mut terminal = Terminal::new(&blog_posts, None);

        // Test cat on existing file
        let cat_result = terminal.handle_command("/", "cat thanks.txt");
        assert!(!cat_result.is_error());
        let cat_output = get_stdout_text(&cat_result).unwrap_or_default();
        assert!(cat_output.contains("Thank you"));

        // Test cat on directory (should fail)
        let cat_dir = terminal.handle_command("/", "cat blog");
        assert!(cat_dir.is_error());
        let error_msg = get_stderr_text(&cat_dir).unwrap_or_default();
        assert!(error_msg.contains("Is a directory"));

        // Test cat on non-existent file
        let cat_missing = terminal.handle_command("/", "cat nonexistent.txt");
        assert!(cat_missing.is_error());
        let error_msg = get_stderr_text(&cat_missing).unwrap_or_default();
        assert!(error_msg.contains("No such file or directory"));
    }

    #[test]
    fn test_file_creation_deletion() {
        let blog_posts = vec!["test-post".to_string()];
        let mut terminal = Terminal::new(&blog_posts, None);

        // Create a file with touch
        let touch_result = terminal.handle_command("/", "touch newfile.txt");
        assert!(!touch_result.is_error());

        // Verify file exists via VFS
        assert!(vfs_file_exists(&mut terminal, "/newfile.txt"));

        // Remove the file
        let rm_result = terminal.handle_command("/", "rm newfile.txt");
        assert!(!rm_result.is_error());

        // Verify file is gone via VFS
        assert!(!vfs_file_exists(&mut terminal, "/newfile.txt"));
    }

    #[test]
    fn test_directory_operations() {
        let blog_posts = vec!["test-post".to_string()];
        let mut terminal = Terminal::new(&blog_posts, None);

        // Create a directory
        let mkdir_result = terminal.handle_command("/", "mkdir testdir");
        assert!(!mkdir_result.is_error());

        // Verify directory exists via VFS
        assert!(vfs_dir_exists(&mut terminal, "/testdir"));

        // Create a file in the directory
        let touch_in_dir = terminal.handle_command("/", "touch testdir/file.txt");
        assert!(!touch_in_dir.is_error());

        // Verify file exists in directory via VFS
        assert!(vfs_file_exists(&mut terminal, "/testdir/file.txt"));
        assert!(vfs_contains_file(&mut terminal, "/testdir", "file.txt"));

        // Try to remove non-empty directory without -r (should fail)
        let rm_fail = terminal.handle_command("/", "rm testdir");
        assert!(rm_fail.is_error());
        let error_msg = get_stderr_text(&rm_fail).unwrap_or_default();
        assert!(error_msg.contains("Is a directory"));

        // Remove directory with -r
        let rm_recursive = terminal.handle_command("/", "rm -r testdir");
        assert!(!rm_recursive.is_error());

        // Verify directory is gone via VFS
        assert!(!vfs_dir_exists(&mut terminal, "/testdir"));
    }

    #[test]
    fn test_cp_mv_operations() {
        let blog_posts = vec!["test-post".to_string()];
        let mut terminal = Terminal::new(&blog_posts, None);

        // Create a test file
        terminal.handle_command("/", "touch original.txt");
        assert!(vfs_file_exists(&mut terminal, "/original.txt"));

        // Copy the file
        let cp_result = terminal.handle_command("/", "cp original.txt copy.txt");
        assert!(!cp_result.is_error());

        // Verify both files exist via VFS
        assert!(vfs_file_exists(&mut terminal, "/original.txt"));
        assert!(vfs_file_exists(&mut terminal, "/copy.txt"));

        // Move the copy
        let mv_result = terminal.handle_command("/", "mv copy.txt renamed.txt");
        assert!(!mv_result.is_error());

        // Verify move worked via VFS
        assert!(vfs_file_exists(&mut terminal, "/original.txt"));
        assert!(!vfs_file_exists(&mut terminal, "/copy.txt"));
        assert!(vfs_file_exists(&mut terminal, "/renamed.txt"));

        // Test directory copy
        terminal.handle_command("/", "mkdir sourcedir");
        terminal.handle_command("/", "touch sourcedir/file1.txt");
        terminal.handle_command("/", "touch sourcedir/file2.txt");

        // Verify source directory setup via VFS
        assert!(vfs_dir_exists(&mut terminal, "/sourcedir"));
        assert!(vfs_file_exists(&mut terminal, "/sourcedir/file1.txt"));
        assert!(vfs_file_exists(&mut terminal, "/sourcedir/file2.txt"));

        let cp_dir_result = terminal.handle_command("/", "cp -r sourcedir destdir");
        assert!(!cp_dir_result.is_error());

        // Verify directory was copied via VFS
        assert!(vfs_dir_exists(&mut terminal, "/destdir"));
        assert!(vfs_file_exists(&mut terminal, "/destdir/file1.txt"));
        assert!(vfs_file_exists(&mut terminal, "/destdir/file2.txt"));
        assert!(vfs_contains_file(&mut terminal, "/destdir", "file1.txt"));
        assert!(vfs_contains_file(&mut terminal, "/destdir", "file2.txt"));
    }

    #[test]
    fn test_permission_errors() {
        let blog_posts = vec!["test-post".to_string()];
        let mut terminal = Terminal::new(&blog_posts, None);

        // Verify system file exists first
        assert!(vfs_file_exists(&mut terminal, "/mines.sh"));

        // Try to remove system file (should fail)
        let rm_system = terminal.handle_command("/", "rm mines.sh");
        assert!(rm_system.is_error());
        let error_msg = get_stderr_text(&rm_system).unwrap_or_default();
        assert!(error_msg.contains("Permission denied"));

        // Verify system file still exists after failed removal
        assert!(vfs_file_exists(&mut terminal, "/mines.sh"));

        // Try to move system directory (should fail)
        let mv_system = terminal.handle_command("/", "mv blog renamed_blog");
        assert!(mv_system.is_error());
        let mv_error = get_stderr_text(&mv_system).unwrap_or_default();
        assert!(mv_error.contains("Permission denied"));

        // Verify blog directory still exists at original location
        assert!(vfs_dir_exists(&mut terminal, "/blog"));
        assert!(!vfs_dir_exists(&mut terminal, "/renamed_blog"));

        // We can create files in immutable directories
        let touch_in_blog = terminal.handle_command("/", "touch blog/userfile.txt");
        if touch_in_blog.is_error() {
            let error = get_stderr_text(&touch_in_blog).unwrap_or_default();
            panic!("touch in blog failed with error: {}", error);
        }

        // Verify the file was created
        assert!(vfs_file_exists(&mut terminal, "/blog/userfile.txt"));

        // But we can't delete the directory itself
        let rm_blog = terminal.handle_command("/", "rm -r blog");
        assert!(rm_blog.is_error());

        // Verify blog directory still exists
        assert!(vfs_dir_exists(&mut terminal, "/blog"));
    }

    #[test]
    fn test_ls_options() {
        let blog_posts = vec!["test-post".to_string()];
        let mut terminal = Terminal::new(&blog_posts, None);

        // Test that hidden files exist in VFS
        assert!(vfs_file_exists(&mut terminal, "/.zshrc"));

        // Test ls -a functionality (we just verify command doesn't error)
        let ls_all = terminal.handle_command("/", "ls -a");
        assert!(!ls_all.is_error());

        // Test ls without -a
        let ls_normal = terminal.handle_command("/", "ls");
        assert!(!ls_normal.is_error());

        // Verify contents of blog and cv directories via individual file checks
        assert!(vfs_contains_file(&mut terminal, "/blog", "test-post"));
        assert!(vfs_contains_file(&mut terminal, "/blog", "nav.rs"));
        assert!(vfs_contains_file(&mut terminal, "/cv", "nav.rs"));

        // Test ls with multiple targets
        let ls_multi = terminal.handle_command("/", "ls blog cv");
        assert!(!ls_multi.is_error());
    }

    #[test]
    fn test_command_errors() {
        let blog_posts = vec!["test-post".to_string()];
        let mut terminal = Terminal::new(&blog_posts, None);

        // Test invalid cd path
        let cd_invalid = terminal.handle_command("/", "cd nonexistent");
        assert!(cd_invalid.is_error());
        let cd_error = get_stderr_text(&cd_invalid).unwrap_or_default();
        assert!(cd_error.contains("no such file or directory"));

        // Test cd to file
        let cd_file = terminal.handle_command("/", "cd mines.sh");
        assert!(cd_file.is_error());
        let cd_file_error = get_stderr_text(&cd_file).unwrap_or_default();
        assert!(cd_file_error.contains("not a directory"));

        // Test touch with no arguments
        let touch_no_args = terminal.handle_command("/", "touch");
        assert!(touch_no_args.is_error());
        let touch_error = get_stderr_text(&touch_no_args).unwrap_or_default();
        assert!(touch_error.contains("missing file operand"));

        // Test cp with missing destination
        let cp_no_dest = terminal.handle_command("/", "cp file.txt");
        assert!(cp_no_dest.is_error());
        let cp_error = get_stderr_text(&cp_no_dest).unwrap_or_default();
        assert!(cp_error.contains("missing destination"));
    }

    #[test]
    fn test_which_command() {
        let blog_posts = vec!["test-post".to_string()];
        let mut terminal = Terminal::new(&blog_posts, None);

        // Test which for builtin command
        let which_cd = terminal.handle_command("/", "which cd");
        assert!(!which_cd.is_error());
        let cd_output = get_stdout_text(&which_cd).unwrap_or_default();
        assert!(cd_output.contains("shell builtin"));

        // Test which for external command
        let which_ls = terminal.handle_command("/", "which ls");
        assert!(!which_ls.is_error());
        let ls_output = get_stdout_text(&which_ls).unwrap_or_default();
        assert!(ls_output.contains("/bin/ls"));

        // Test which for alias
        let which_ll = terminal.handle_command("/", "which ll");
        assert!(!which_ll.is_error());
        let ll_output = get_stdout_text(&which_ll).unwrap_or_default();
        assert!(ll_output.contains("aliased to"));

        // Test which for non-existent command
        let which_fake = terminal.handle_command("/", "which fakecmd");
        assert!(which_fake.is_error());
        let fake_output = get_stdout_text(&which_fake).unwrap_or_default();
        assert!(fake_output.contains("not found"));
    }

    #[test]
    fn test_echo_pwd_commands() {
        let blog_posts = vec!["test-post".to_string()];
        let mut terminal = Terminal::new(&blog_posts, None);

        // Test pwd at root
        let pwd_root = terminal.handle_command("/", "pwd");
        assert!(!pwd_root.is_error());
        let pwd_output = get_stdout_text(&pwd_root).unwrap_or_default();
        assert_eq!(pwd_output.trim(), "/");

        // Test pwd in subdirectory
        let pwd_blog = terminal.handle_command("/blog", "pwd");
        assert!(!pwd_blog.is_error());
        let pwd_blog_output = get_stdout_text(&pwd_blog).unwrap_or_default();
        assert_eq!(pwd_blog_output.trim(), "/blog");

        // Test echo
        let echo_result = terminal.handle_command("/", "echo hello world");
        assert!(!echo_result.is_error());
        let echo_output = get_stdout_text(&echo_result).unwrap_or_default();
        assert_eq!(echo_output.trim(), "hello world");

        // Test echo with no args
        let echo_empty = terminal.handle_command("/", "echo");
        assert!(!echo_empty.is_error());
        let empty_output = get_stdout_text(&echo_empty).unwrap_or_default();
        assert_eq!(empty_output.trim(), "");
    }

    #[test]
    fn test_cp_mv_basic_functionality() {
        let blog_posts = vec!["hello_world".to_string(), "rust_tips".to_string()];
        let mut terminal = Terminal::new(&blog_posts, None);

        // Test touch to create a file
        let touch_result = terminal.handle_command("/", "touch testfile.txt");
        assert!(!touch_result.is_error());
        assert!(vfs_file_exists(&mut terminal, "/testfile.txt"));

        // Test cp to copy the file
        let cp_result = terminal.handle_command("/", "cp testfile.txt testfile_copy.txt");
        assert!(!cp_result.is_error());

        // Verify both files exist via VFS
        assert!(vfs_file_exists(&mut terminal, "/testfile.txt"));
        assert!(vfs_file_exists(&mut terminal, "/testfile_copy.txt"));

        // Test mv to rename a file
        let mv_result = terminal.handle_command("/", "mv testfile_copy.txt renamed_file.txt");
        assert!(!mv_result.is_error());

        // Verify move worked via VFS
        assert!(vfs_file_exists(&mut terminal, "/testfile.txt")); // Original should still exist
        assert!(!vfs_file_exists(&mut terminal, "/testfile_copy.txt")); // Copy should be gone
        assert!(vfs_file_exists(&mut terminal, "/renamed_file.txt")); // New name should exist

        // Test mkdir and cp with directories
        let mkdir_result = terminal.handle_command("/", "mkdir testdir");
        assert!(!mkdir_result.is_error());
        assert!(vfs_dir_exists(&mut terminal, "/testdir"));

        let cp_to_dir_result = terminal.handle_command("/", "cp testfile.txt testdir/");
        assert!(!cp_to_dir_result.is_error());
        assert!(vfs_file_exists(&mut terminal, "/testdir/testfile.txt"));

        // Test recursive directory copy
        let cp_recursive_result = terminal.handle_command("/", "cp -r testdir testdir_copy");
        assert!(!cp_recursive_result.is_error());
        assert!(vfs_dir_exists(&mut terminal, "/testdir_copy"));
        assert!(vfs_file_exists(&mut terminal, "/testdir_copy/testfile.txt"));
    }

    #[test]
    fn test_cp_file_to_file_happy_path() {
        let blog_posts = vec![];
        let mut terminal = Terminal::new(&blog_posts, None);

        // Create source file with content
        terminal.handle_command("/", "touch source.txt");
        assert!(vfs_file_exists(&mut terminal, "/source.txt"));

        // Copy file to new name
        let cp_result = terminal.handle_command("/", "cp source.txt destination.txt");
        assert!(!cp_result.is_error());

        // Verify both files exist
        assert!(vfs_file_exists(&mut terminal, "/source.txt"));
        assert!(vfs_file_exists(&mut terminal, "/destination.txt"));

        // Verify we can read both files
        let cat_source = terminal.handle_command("/", "cat source.txt");
        assert!(!cat_source.is_error());
        let cat_dest = terminal.handle_command("/", "cat destination.txt");
        assert!(!cat_dest.is_error());
    }

    #[test]
    fn test_cp_file_to_directory_happy_path() {
        let blog_posts = vec![];
        let mut terminal = Terminal::new(&blog_posts, None);

        // Create source file and target directory
        terminal.handle_command("/", "touch file.txt");
        terminal.handle_command("/", "mkdir targetdir");
        assert!(vfs_file_exists(&mut terminal, "/file.txt"));
        assert!(vfs_dir_exists(&mut terminal, "/targetdir"));

        // Copy file to directory
        let cp_result = terminal.handle_command("/", "cp file.txt targetdir/");
        assert!(!cp_result.is_error());

        // Verify original file still exists and copy exists in target directory
        assert!(vfs_file_exists(&mut terminal, "/file.txt"));
        assert!(vfs_file_exists(&mut terminal, "/targetdir/file.txt"));
    }

    #[test]
    fn test_cp_directory_recursive_happy_path() {
        let blog_posts = vec![];
        let mut terminal = Terminal::new(&blog_posts, None);

        // Create source directory structure
        terminal.handle_command("/", "mkdir sourcedir");
        terminal.handle_command("/", "touch sourcedir/file1.txt");
        terminal.handle_command("/", "touch sourcedir/file2.txt");
        terminal.handle_command("/", "mkdir sourcedir/subdir");
        terminal.handle_command("/", "touch sourcedir/subdir/nested.txt");

        // Verify source structure
        assert!(vfs_dir_exists(&mut terminal, "/sourcedir"));
        assert!(vfs_file_exists(&mut terminal, "/sourcedir/file1.txt"));
        assert!(vfs_file_exists(&mut terminal, "/sourcedir/file2.txt"));
        assert!(vfs_dir_exists(&mut terminal, "/sourcedir/subdir"));
        assert!(vfs_file_exists(
            &mut terminal,
            "/sourcedir/subdir/nested.txt"
        ));

        // Recursively copy directory
        let cp_result = terminal.handle_command("/", "cp -r sourcedir targetdir");
        assert!(!cp_result.is_error());

        // Verify original structure still exists
        assert!(vfs_dir_exists(&mut terminal, "/sourcedir"));
        assert!(vfs_file_exists(&mut terminal, "/sourcedir/file1.txt"));
        assert!(vfs_file_exists(&mut terminal, "/sourcedir/file2.txt"));
        assert!(vfs_dir_exists(&mut terminal, "/sourcedir/subdir"));
        assert!(vfs_file_exists(
            &mut terminal,
            "/sourcedir/subdir/nested.txt"
        ));

        // Verify copied structure exists
        assert!(vfs_dir_exists(&mut terminal, "/targetdir"));
        assert!(vfs_file_exists(&mut terminal, "/targetdir/file1.txt"));
        assert!(vfs_file_exists(&mut terminal, "/targetdir/file2.txt"));
        assert!(vfs_dir_exists(&mut terminal, "/targetdir/subdir"));
        assert!(vfs_file_exists(
            &mut terminal,
            "/targetdir/subdir/nested.txt"
        ));
    }

    #[test]
    fn test_mv_file_rename_happy_path() {
        let blog_posts = vec![];
        let mut terminal = Terminal::new(&blog_posts, None);

        // Create source file
        terminal.handle_command("/", "touch original.txt");
        assert!(vfs_file_exists(&mut terminal, "/original.txt"));

        // Move/rename file
        let mv_result = terminal.handle_command("/", "mv original.txt renamed.txt");
        assert!(!mv_result.is_error());

        // Verify original is gone and new name exists
        assert!(!vfs_file_exists(&mut terminal, "/original.txt"));
        assert!(vfs_file_exists(&mut terminal, "/renamed.txt"));

        // Verify we can read the renamed file
        let cat_result = terminal.handle_command("/", "cat renamed.txt");
        assert!(!cat_result.is_error());
    }

    #[test]
    fn test_mv_file_to_directory_happy_path() {
        let blog_posts = vec![];
        let mut terminal = Terminal::new(&blog_posts, None);

        // Create source file and target directory
        terminal.handle_command("/", "touch moveme.txt");
        terminal.handle_command("/", "mkdir targetdir");
        assert!(vfs_file_exists(&mut terminal, "/moveme.txt"));
        assert!(vfs_dir_exists(&mut terminal, "/targetdir"));

        // Move file to directory
        let mv_result = terminal.handle_command("/", "mv moveme.txt targetdir/");
        assert!(!mv_result.is_error());

        // Verify original is gone and file exists in target directory
        assert!(!vfs_file_exists(&mut terminal, "/moveme.txt"));
        assert!(vfs_file_exists(&mut terminal, "/targetdir/moveme.txt"));
    }

    #[test]
    fn test_mv_directory_rename_happy_path() {
        let blog_posts = vec![];
        let mut terminal = Terminal::new(&blog_posts, None);

        // Create source directory with contents
        terminal.handle_command("/", "mkdir olddir");
        terminal.handle_command("/", "touch olddir/file.txt");
        assert!(vfs_dir_exists(&mut terminal, "/olddir"));
        assert!(vfs_file_exists(&mut terminal, "/olddir/file.txt"));

        // Move/rename directory
        let mv_result = terminal.handle_command("/", "mv olddir newdir");
        assert!(!mv_result.is_error());

        // Verify original is gone and new name exists with contents
        assert!(!vfs_dir_exists(&mut terminal, "/olddir"));
        assert!(vfs_dir_exists(&mut terminal, "/newdir"));
        assert!(vfs_file_exists(&mut terminal, "/newdir/file.txt"));
    }

    #[test]
    fn test_cp_mv_error_cases() {
        let blog_posts = vec!["hello_world".to_string()];
        let mut terminal = Terminal::new(&blog_posts, None);

        // Test cp with missing source
        let cp_missing_result = terminal.handle_command("/", "cp nonexistent.txt copy.txt");
        assert!(cp_missing_result.is_error());
        let error_msg = get_stderr_text(&cp_missing_result).unwrap_or_default();
        assert!(error_msg.contains("No such file or directory"));

        // Test cp directory without -r flag
        let cp_dir_result = terminal.handle_command("/", "cp blog copy_blog");
        assert!(cp_dir_result.is_error());
        let error_msg = get_stderr_text(&cp_dir_result).unwrap_or_default();
        assert!(
            error_msg.contains("Is a directory")
                || error_msg.contains("-r not specified")
                || error_msg.contains("omitting directory")
        );

        // Test mv with missing source
        let mv_missing_result = terminal.handle_command("/", "mv nonexistent.txt moved.txt");
        assert!(mv_missing_result.is_error());
        let error_msg = get_stderr_text(&mv_missing_result).unwrap_or_default();
        assert!(error_msg.contains("No such file or directory"));

        // Test mv system directory (should fail due to immutable permissions)
        let mv_system_result = terminal.handle_command("/", "mv blog moved_blog");
        assert!(mv_system_result.is_error());
        let error_msg = get_stderr_text(&mv_system_result).unwrap_or_default();
        assert!(error_msg.contains("Permission denied"));

        // Verify system directory still exists after failed move
        assert!(vfs_dir_exists(&mut terminal, "/blog"));
        assert!(!vfs_dir_exists(&mut terminal, "/moved_blog"));

        // Test cp with missing destination arguments
        let cp_no_dest = terminal.handle_command("/", "cp onlyarg");
        assert!(cp_no_dest.is_error());
        let error_msg = get_stderr_text(&cp_no_dest).unwrap_or_default();
        assert!(error_msg.contains("missing destination"));

        // Test mv with missing destination arguments
        let mv_no_dest = terminal.handle_command("/", "mv onlyarg");
        assert!(mv_no_dest.is_error());
        let error_msg = get_stderr_text(&mv_no_dest).unwrap_or_default();
        assert!(error_msg.contains("missing destination") || error_msg.contains("missing operand"));
    }

    #[test]
    fn test_cp_mv_complex_paths_happy_path() {
        let blog_posts = vec![];
        let mut terminal = Terminal::new(&blog_posts, None);

        // Create nested directory structure
        terminal.handle_command("/", "mkdir src");
        terminal.handle_command("/", "mkdir src/utils");
        terminal.handle_command("/", "mkdir dest");
        terminal.handle_command("/", "mkdir dest/backup");
        terminal.handle_command("/", "touch src/main.rs");
        terminal.handle_command("/", "touch src/utils/helper.rs");

        // Verify setup
        assert!(vfs_dir_exists(&mut terminal, "/src"));
        assert!(vfs_dir_exists(&mut terminal, "/src/utils"));
        assert!(vfs_dir_exists(&mut terminal, "/dest"));
        assert!(vfs_dir_exists(&mut terminal, "/dest/backup"));
        assert!(vfs_file_exists(&mut terminal, "/src/main.rs"));
        assert!(vfs_file_exists(&mut terminal, "/src/utils/helper.rs"));

        // Copy file with relative path
        let cp_result = terminal.handle_command("/src", "cp main.rs ../dest/");
        assert!(!cp_result.is_error());
        assert!(vfs_file_exists(&mut terminal, "/dest/main.rs"));
        assert!(vfs_file_exists(&mut terminal, "/src/main.rs")); // Original should remain

        // Move file from nested location
        let mv_result = terminal.handle_command("/src/utils", "mv helper.rs ../../dest/backup/");
        assert!(!mv_result.is_error());
        assert!(!vfs_file_exists(&mut terminal, "/src/utils/helper.rs")); // Original should be gone
        assert!(vfs_file_exists(&mut terminal, "/dest/backup/helper.rs"));

        // Copy entire directory structure
        let cp_recursive = terminal.handle_command("/", "cp -r src src_backup");
        assert!(!cp_recursive.is_error());
        assert!(vfs_dir_exists(&mut terminal, "/src_backup"));
        assert!(vfs_dir_exists(&mut terminal, "/src_backup/utils"));
        assert!(vfs_file_exists(&mut terminal, "/src_backup/main.rs"));
        // Note: helper.rs was moved out, so it shouldn't be in the backup
    }

    #[test]
    fn test_cp_mv_overwrite_behavior() {
        let blog_posts = vec![];
        let mut terminal = Terminal::new(&blog_posts, None);

        // Create a file and copy it to a new location
        terminal.handle_command("/", "touch file1.txt");
        assert!(vfs_file_exists(&mut terminal, "/file1.txt"));

        // Copy file1 to a new name (this should work)
        let cp_result = terminal.handle_command("/", "cp file1.txt file2.txt");
        assert!(!cp_result.is_error());

        // Both files should exist
        assert!(vfs_file_exists(&mut terminal, "/file1.txt"));
        assert!(vfs_file_exists(&mut terminal, "/file2.txt"));

        // Try to copy file1 to file2 again (should fail - file exists)
        let cp_overwrite = terminal.handle_command("/", "cp file1.txt file2.txt");
        assert!(cp_overwrite.is_error());
        let error_msg = get_stderr_text(&cp_overwrite).unwrap_or_default();
        assert!(error_msg.contains("File exists"));

        // Create a third file and move it to overwrite file1 (mv should work)
        terminal.handle_command("/", "touch file3.txt");
        assert!(vfs_file_exists(&mut terminal, "/file3.txt"));

        let mv_result = terminal.handle_command("/", "mv file3.txt file1.txt");
        assert!(!mv_result.is_error());

        // file3 should be gone, file1 should still exist (overwritten by mv)
        assert!(!vfs_file_exists(&mut terminal, "/file3.txt"));
        assert!(vfs_file_exists(&mut terminal, "/file1.txt"));
    }
}
