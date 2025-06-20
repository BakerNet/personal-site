use std::sync::Arc;

use leptos::prelude::*;

use super::command::{CommandRes, Executable, PipelineRes};
use crate::app::ascii::{AVATAR_BLOCK, INFO_BLOCK};
use chrono::prelude::*;

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

pub struct HelpCommand;

impl Executable for HelpCommand {
    fn execute(&self, _path: &str, _args: Vec<&str>) -> CommandRes {
        CommandRes::Output(Arc::new(move || HELP_TEXT.into_any()))
    }

    fn execute_pipeable(&self, _path: &str, _args: Vec<&str>, _stdin: &str) -> PipelineRes {
        PipelineRes::Output(HELP_TEXT.to_string())
    }
}

pub struct PwdCommand;

impl Executable for PwdCommand {
    fn execute(&self, path: &str, args: Vec<&str>) -> CommandRes {
        if !args.is_empty() {
            return CommandRes::Err(Arc::new(move || "pwd: too many arguments".into_any()));
        }
        let path = path.to_owned();
        CommandRes::Output(Arc::new(move || view! { {path.clone()} }.into_any()))
    }

    fn execute_pipeable(&self, path: &str, args: Vec<&str>, _stdin: &str) -> PipelineRes {
        if !args.is_empty() {
            return PipelineRes::Err("pwd: too many arguments".to_string());
        }
        PipelineRes::Output(path.to_string())
    }
}

pub struct WhoAmICommand;

impl Executable for WhoAmICommand {
    fn execute(&self, _path: &str, args: Vec<&str>) -> CommandRes {
        if !args.is_empty() {
            return CommandRes::Err(Arc::new(move || "useage: whoami".into_any()));
        }
        CommandRes::Output(Arc::new(move || "user".into_any()))
    }

    fn execute_pipeable(&self, _path: &str, args: Vec<&str>, _stdin: &str) -> PipelineRes {
        if !args.is_empty() {
            return PipelineRes::Err("useage: whoami".to_string());
        }
        PipelineRes::Output("user".to_string())
    }
}

pub struct ClearCommand;

impl Executable for ClearCommand {
    fn execute(&self, _path: &str, _args: Vec<&str>) -> CommandRes {
        CommandRes::Nothing
    }

    fn execute_pipeable(&self, _path: &str, _args: Vec<&str>, _stdin: &str) -> PipelineRes {
        PipelineRes::Nothing
    }
}

pub struct NeofetchCommand;

impl NeofetchCommand {
    fn as_text(&self) -> String {
        AVATAR_BLOCK
            .iter()
            .zip(INFO_BLOCK.iter())
            .map(|(a, b)| format!("{a}  {b}"))
            .fold(String::new(), |acc, s| {
                if acc.is_empty() {
                    s
                } else {
                    format!("{acc}\n{s}")
                }
            })
    }
}

impl Executable for NeofetchCommand {
    fn execute(&self, _path: &str, _args: Vec<&str>) -> CommandRes {
        let text = self.as_text();
        CommandRes::Output(Arc::new(move || {
            view! { <div class="leading-tight" inner_html=text.clone()></div> }.into_any()
        }))
    }

    fn execute_pipeable(&self, _path: &str, _args: Vec<&str>, _stdin: &str) -> PipelineRes {
        PipelineRes::Output(self.as_text())
    }
}

pub struct MinesCommand;

impl Executable for MinesCommand {
    fn execute(&self, _path: &str, _args: Vec<&str>) -> CommandRes {
        CommandRes::Redirect(MINES_URL.to_string())
    }

    fn execute_pipeable(&self, _path: &str, _args: Vec<&str>, _stdin: &str) -> PipelineRes {
        PipelineRes::Redirect(MINES_URL.to_string())
    }
}

pub struct SudoCommand;

impl Executable for SudoCommand {
    fn execute(&self, _path: &str, _args: Vec<&str>) -> CommandRes {
        CommandRes::Err(Arc::new(move || {
            "user is not in the sudoers file. This incident will be reported.".into_any()
        }))
    }

    fn execute_pipeable(&self, _path: &str, _args: Vec<&str>, _stdin: &str) -> PipelineRes {
        PipelineRes::Err(
            "user is not in the sudoers file. This incident will be reported.".to_string(),
        )
    }
}

pub struct EchoCommand;

impl Executable for EchoCommand {
    fn execute(&self, _path: &str, args: Vec<&str>) -> CommandRes {
        let message = args
            .iter()
            .map(|s| s.replace("\"", ""))
            .collect::<Vec<_>>()
            .join(" ");

        // Check for unsupported command substitution
        if message.contains("$(") {
            return CommandRes::Err(Arc::new(move || {
                "echo: command substitution not supported".into_any()
            }));
        }

        CommandRes::Output(Arc::new(move || message.clone().into_any()))
    }

    fn execute_pipeable(&self, _path: &str, args: Vec<&str>, _stdin: &str) -> PipelineRes {
        let message = args
            .iter()
            .map(|s| s.replace("\"", ""))
            .collect::<Vec<_>>()
            .join(" ");

        // Check for unsupported command substitution
        if message.contains("$(") {
            return PipelineRes::Err("echo: command substitution not supported".to_string());
        }

        PipelineRes::Output(message)
    }
}

pub struct DateCommand;

impl DateCommand {
    fn get_date(&self, format_str: Option<&str>) -> String {
        let now = Local::now();

        if format_str.is_none() {
            // Default format: Wed Dec 25 14:30:15 PST 2024
            return now.format("%a %b %d %H:%M:%S %Z %Y").to_string();
        }
        let format_str = format_str.unwrap();

        // Handle common format strings
        match format_str {
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
        }
    }
}

impl Executable for DateCommand {
    fn execute(&self, _path: &str, args: Vec<&str>) -> CommandRes {
        if args.len() > 1 {
            return CommandRes::Err(Arc::new(move || "date: too many arguments".into_any()));
        }

        let format_str = if args.len() == 1 {
            let arg_str = args[0].trim_matches('"');

            if !arg_str.starts_with('+') {
                return CommandRes::Err(Arc::new(move || {
                    "date: invalid format (must start with +)".into_any()
                }));
            }
            Some(&arg_str[1..]) // Remove the + prefix
        } else {
            None
        };

        let result = self.get_date(format_str);
        CommandRes::Output(Arc::new(move || result.clone().into_any()))
    }

    fn execute_pipeable(&self, _path: &str, args: Vec<&str>, _stdin: &str) -> PipelineRes {
        if args.len() > 1 {
            return PipelineRes::Err("date: too many arguments".to_string());
        }

        let format_str = if args.len() == 1 {
            let arg_str = args[0].trim_matches('"');

            if !arg_str.starts_with('+') {
                return PipelineRes::Err("date: invalid format (must start with +)".to_string());
            }
            Some(&arg_str[1..]) // Remove the + prefix
        } else {
            None
        };

        let result = self.get_date(format_str);
        PipelineRes::Output(result)
    }
}

pub struct UptimeCommand;

impl UptimeCommand {
    fn get_uptime(&self) -> String {
        let now = Local::now();
        let current_time = now.format("%H:%M:%S").to_string();

        // Use the actual build time as the start point for uptime
        let build_time_str = env!("BUILD_TIME");
        let build_time = match DateTime::parse_from_rfc3339(build_time_str) {
            Ok(dt) => dt.with_timezone(&Local),
            Err(_) => {
                // Fallback to a reasonable default if parsing fails
                now - chrono::Duration::days(42)
                    - chrono::Duration::hours(13)
                    - chrono::Duration::minutes(37)
            }
        };

        let uptime_duration = now.signed_duration_since(build_time);
        let uptime_days = uptime_duration.num_days();
        let uptime_hours = uptime_duration.num_hours() % 24;
        let uptime_minutes = uptime_duration.num_minutes() % 60;

        // Generate slightly varying load averages based on current time
        let base_seed = (now.timestamp() / 300) as f64; // Change every 5 minutes
        let load_1 = 0.08 + (base_seed * 0.001).sin() * 0.02;
        let load_5 = 0.12 + (base_seed * 0.0015).cos() * 0.03;
        let load_15 = 0.15 + (base_seed * 0.002).sin() * 0.02;

        format!(
            "{current_time} up {uptime_days} days, {uptime_hours}:{uptime_minutes:02}, load average: {load_1:.2}, {load_5:.2}, {load_15:.2}"
        )
    }
}

impl Executable for UptimeCommand {
    fn execute(&self, _path: &str, _args: Vec<&str>) -> CommandRes {
        let output = self.get_uptime();
        CommandRes::Output(Arc::new(move || output.clone().into_any()))
    }

    fn execute_pipeable(&self, _path: &str, _args: Vec<&str>, _stdin: &str) -> PipelineRes {
        let output = self.get_uptime();
        PipelineRes::Output(output)
    }
}
