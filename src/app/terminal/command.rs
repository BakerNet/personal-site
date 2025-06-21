#![allow(dead_code)]

use leptos::prelude::*;

pub trait Executable: Send + Sync {
    fn execute(&self, path: &str, args: Vec<&str>, stdin: Option<&str>, is_output_tty: bool) -> CommandRes;
}

pub enum CommandRes {
    Output {
        is_err: bool,                     // true if command failed (non-zero exit code)
        stdout_view: Option<ChildrenFn>,  // stdout content for display (only set if is_output_tty)
        stdout_text: Option<String>,      // stdout for piping
        stderr_text: Option<String>,      // stderr text (Header converts to view)
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
        if let Self::Output { is_err, .. } = &mut self { *is_err = true }
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
        if let Self::Output { stdout_text, .. } = &mut self { *stdout_text = Some(text.into()) }
        self
    }

    /// Add only stderr text (alias for with_stderr for consistency)
    pub fn with_stderr_text(self, text: impl Into<String>) -> Self {
        self.with_stderr(text)
    }

    /// Add only stdout view (no text)
    pub fn with_stdout_view(mut self, view: ChildrenFn) -> Self {
        if let Self::Output { stdout_view, .. } = &mut self { *stdout_view = Some(view) }
        self
    }

}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Command {
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
            "uptime" => Self::Uptime,
            "ps" => Self::Ps,
            "kill" => Self::Kill,
            _ => Self::Unknown,
        }
    }
}

impl Command {
    pub fn all() -> Vec<&'static str> {
        vec![
            "help", "pwd", "ls", "cd", "cat", "clear", "cp", "date", "echo", "history", "mines",
            "mkdir", "mv", "rm", "touch", "which", "whoami", "neofetch", "uptime", "ps", "kill",
        ]
    }
}

#[derive(Debug, Clone)]
pub enum CommandAlias {
    Ll,
    La,
    H,
}

impl CommandAlias {
    pub fn all() -> Vec<CommandAlias> {
        vec![CommandAlias::Ll, CommandAlias::La, CommandAlias::H]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CommandAlias::Ll => "ll",
            CommandAlias::La => "la",
            CommandAlias::H => "h",
        }
    }

    pub fn expand(&self, args: &str) -> String {
        match self {
            CommandAlias::Ll => {
                if args.is_empty() {
                    "ls -la".to_string()
                } else {
                    format!("ls -la{args}")
                }
            }
            CommandAlias::La => {
                if args.is_empty() {
                    "ls -a".to_string()
                } else {
                    format!("ls -a{args}")
                }
            }
            CommandAlias::H => {
                if args.is_empty() {
                    "history".to_string()
                } else {
                    format!("history{args}")
                }
            }
        }
    }

    pub fn from_str(s: &str) -> Option<CommandAlias> {
        match s {
            "ll" => Some(CommandAlias::Ll),
            "la" => Some(CommandAlias::La),
            "h" => Some(CommandAlias::H),
            _ => None,
        }
    }
}
