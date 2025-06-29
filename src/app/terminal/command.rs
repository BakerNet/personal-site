#![allow(dead_code)]

use super::vfs::VirtualFilesystem;
use indextree::NodeId;
use leptos::prelude::*;

pub trait Command: Send + Sync {
    fn execute(
        &self,
        path: &str,
        args: Vec<&str>,
        stdin: Option<&str>,
        is_output_tty: bool,
    ) -> CommandRes;
}

/// VFS-aware command trait for commands that need direct filesystem access
pub trait VfsCommand: Send + Sync {
    fn execute(
        &self,
        vfs: &mut VirtualFilesystem,
        current_dir: NodeId,
        args: Vec<&str>,
        stdin: Option<&str>,
        is_tty: bool,
    ) -> CommandRes;
}

pub enum CommandRes {
    Output {
        is_err: bool,                    // true if command failed (non-zero exit code)
        stdout_view: Option<ChildrenFn>, // stdout content for display (only set if is_output_tty)
        stdout_text: Option<String>,     // stdout for piping
        stderr_text: Option<String>,     // stderr text (Header converts to view)
    },
    Redirect(String),
}

impl CommandRes {
    /// Create a new empty CommandRes with default values
    pub fn new() -> Self {
        Self::Output {
            is_err: false,
            stdout_view: None,
            stdout_text: None,
            stderr_text: None,
        }
    }

    /// Create a redirect result
    pub fn redirect(url: String) -> Self {
        Self::Redirect(url)
    }

    /// Mark this result as an error
    pub fn with_error(mut self) -> Self {
        if let Self::Output { is_err, .. } = &mut self {
            *is_err = true
        }
        self
    }

    /// Add stderr content (text only - Header handles view conversion)
    pub fn with_stderr(mut self, text: impl Into<String>) -> Self {
        if let Self::Output { stderr_text, .. } = &mut self {
            *stderr_text = Some(text.into());
        }
        self
    }

    /// Add only stdout text (no view)
    pub fn with_stdout_text(mut self, text: impl Into<String>) -> Self {
        if let Self::Output { stdout_text, .. } = &mut self {
            *stdout_text = Some(text.into())
        }
        self
    }

    /// Add only stderr text (alias for with_stderr for consistency)
    pub fn with_stderr_text(self, text: impl Into<String>) -> Self {
        self.with_stderr(text)
    }

    /// Add only stdout view (no text)
    pub fn with_stdout_view(mut self, view: ChildrenFn) -> Self {
        if let Self::Output { stdout_view, .. } = &mut self {
            *stdout_view = Some(view)
        }
        self
    }

    /// Check if this result represents an error
    pub fn is_error(&self) -> bool {
        match self {
            Self::Output { is_err, .. } => *is_err,
            Self::Redirect(_) => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cmd {
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
    Uptime,
    Ps,
    Kill,
    Unknown,
}

impl From<&str> for Cmd {
    fn from(value: &str) -> Self {
        Self::from_str(value).unwrap_or(Self::Unknown)
    }
}

impl Cmd {
    pub fn all() -> Vec<&'static str> {
        vec![
            "help", "pwd", "ls", "cd", "cat", "clear", "cp", "date", "echo", "history", "mines",
            "mkdir", "mv", "rm", "touch", "which", "whoami", "neofetch", "uptime", "ps", "kill",
        ]
    }

    pub fn from_str(s: &str) -> Option<Cmd> {
        match s {
            "help" => Some(Self::Help),
            "pwd" => Some(Self::Pwd),
            "ls" => Some(Self::Ls),
            "cd" => Some(Self::Cd),
            "cat" => Some(Self::Cat),
            "clear" => Some(Self::Clear),
            "cp" => Some(Self::Cp),
            "date" => Some(Self::Date),
            "echo" => Some(Self::Echo),
            "history" => Some(Self::History),
            "mines" => Some(Self::Mines),
            "mkdir" => Some(Self::MkDir),
            "mv" => Some(Self::Mv),
            "rm" => Some(Self::Rm),
            "touch" => Some(Self::Touch),
            "which" => Some(Self::Which),
            "whoami" => Some(Self::WhoAmI),
            "neofetch" => Some(Self::Neofetch),
            "sudo" => Some(Self::Sudo),
            "uptime" => Some(Self::Uptime),
            "ps" => Some(Self::Ps),
            "kill" => Some(Self::Kill),
            _ => None,
        }
    }

    /// Returns the simulated filesystem path for this command
    pub fn simulated_path(&self) -> Option<String> {
        match self {
            // Shell builtins don't have paths
            Self::Pwd | Self::Cd | Self::Echo | Self::History => None,

            // Core system utilities (typically in /bin)
            Self::Ls | Self::Cat | Self::Cp | Self::Mv | Self::Rm | Self::MkDir | Self::Touch => {
                Some(format!("/bin/{}", self.as_str()))
            }

            // System administration and process tools (typically in /usr/bin)
            Self::Ps | Self::Kill | Self::WhoAmI | Self::Which | Self::Uptime => {
                Some(format!("/usr/bin/{}", self.as_str()))
            }

            // Terminal/display utilities (typically in /usr/bin)
            Self::Clear | Self::Date => {
                Some(format!("/usr/bin/{}", self.as_str()))
            }

            // Custom/third-party applications (typically in /usr/local/bin)
            Self::Neofetch | Self::Mines => {
                Some(format!("/usr/local/bin/{}", self.as_str()))
            }

            // Documentation/help (typically in /usr/bin)
            Self::Help => Some(format!("/usr/bin/{}", self.as_str())),

            // System utilities (typically in /usr/bin)
            Self::Sudo => Some(format!("/usr/bin/{}", self.as_str())),

            // Unknown commands don't have paths
            Self::Unknown => None,
        }
    }

    /// Returns the command name as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Help => "help",
            Self::Pwd => "pwd",
            Self::Ls => "ls",
            Self::Cd => "cd",
            Self::Cat => "cat",
            Self::Clear => "clear",
            Self::Cp => "cp",
            Self::Date => "date",
            Self::Echo => "echo",
            Self::History => "history",
            Self::Mines => "mines",
            Self::MkDir => "mkdir",
            Self::Mv => "mv",
            Self::Rm => "rm",
            Self::Touch => "touch",
            Self::Which => "which",
            Self::WhoAmI => "whoami",
            Self::Neofetch => "neofetch",
            Self::Sudo => "sudo",
            Self::Uptime => "uptime",
            Self::Ps => "ps",
            Self::Kill => "kill",
            Self::Unknown => "unknown",
        }
    }

    /// Returns true if this command is a shell builtin
    pub fn is_builtin(&self) -> bool {
        matches!(self, Self::Pwd | Self::Cd | Self::Echo | Self::History)
    }
}

#[derive(Debug, Clone)]
pub enum CmdAlias {
    Ll,
    La,
    H,
}

impl CmdAlias {
    pub fn all() -> Vec<CmdAlias> {
        vec![CmdAlias::Ll, CmdAlias::La, CmdAlias::H]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CmdAlias::Ll => "ll",
            CmdAlias::La => "la",
            CmdAlias::H => "h",
        }
    }

    pub fn expand(&self, args: &str) -> String {
        match self {
            CmdAlias::Ll => {
                if args.is_empty() {
                    "ls -la".to_string()
                } else {
                    format!("ls -la{args}")
                }
            }
            CmdAlias::La => {
                if args.is_empty() {
                    "ls -a".to_string()
                } else {
                    format!("ls -a{args}")
                }
            }
            CmdAlias::H => {
                if args.is_empty() {
                    "history".to_string()
                } else {
                    format!("history{args}")
                }
            }
        }
    }

    pub fn from_str(s: &str) -> Option<CmdAlias> {
        match s {
            "ll" => Some(CmdAlias::Ll),
            "la" => Some(CmdAlias::La),
            "h" => Some(CmdAlias::H),
            _ => None,
        }
    }
}
