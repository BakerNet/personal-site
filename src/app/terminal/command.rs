#![allow(dead_code)]
use std::sync::Arc;

use leptos::prelude::*;

#[derive(Debug, Clone)]
pub enum PipelineRes {
    EmptyErr,
    Err(String),
    Redirect(String),
    Output(String),
    Nothing,
}

impl PipelineRes {
    pub fn to_command_res(self) -> CommandRes {
        match self {
            PipelineRes::EmptyErr => CommandRes::EmptyErr,
            PipelineRes::Err(msg) => CommandRes::Err(Arc::new(move || msg.clone().into_any())),
            PipelineRes::Redirect(url) => CommandRes::Redirect(url),
            PipelineRes::Output(text) => {
                CommandRes::Output(Arc::new(move || text.clone().into_any()))
            }
            PipelineRes::Nothing => CommandRes::Nothing,
        }
    }
}

pub trait Executable: Send + Sync {
    fn execute(&self, path: &str, args: Vec<&str>) -> CommandRes;
    fn execute_pipeable(&self, path: &str, args: Vec<&str>, stdin: &str) -> PipelineRes;
}

pub enum CommandRes {
    EmptyErr,
    Err(ChildrenFn),
    Redirect(String),
    Output(ChildrenFn),
    Nothing,
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

    pub fn from_str(s: &str) -> Option<CommandAlias> {
        match s {
            "ll" => Some(CommandAlias::Ll),
            "la" => Some(CommandAlias::La),
            "h" => Some(CommandAlias::H),
            _ => None,
        }
    }
}
