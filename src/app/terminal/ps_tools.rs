use std::sync::Arc;

use leptos::prelude::*;

use crate::app::terminal::command::{CommandRes, Executable};

#[derive(Debug, Clone)]
pub struct Process {
    pub pid: u32,
    pub user: String,
    pub cpu_percent: f32,
    pub mem_percent: f32,
    pub command: String,
}

pub struct PsCommand {
    processes: Vec<Process>,
}

impl PsCommand {
    pub fn new(processes: Vec<Process>) -> Self {
        Self { processes }
    }

    fn get_processes(&self, detailed: bool) -> String {
        if detailed {
            // Detailed format: USER PID %CPU %MEM COMMAND
            let mut lines = vec!["USER       PID %CPU %MEM COMMAND".to_string()];
            self.processes.iter().for_each(|process| {
                lines.push(format!(
                    "{:<8} {:>5} {:>4.1} {:>4.1} {}",
                    process.user,
                    process.pid,
                    process.cpu_percent,
                    process.mem_percent,
                    process.command
                ));
            });
            lines.join("\n")
        } else {
            // Basic format: PID COMMAND
            let mut lines = vec!["  PID COMMAND".to_string()];
            self.processes.iter().for_each(|process| {
                lines.push(format!("{:>5} {}", process.pid, process.command));
            });
            lines.join("\n")
        }
    }
}

impl Executable for PsCommand {
    fn execute(&self, _path: &str, args: Vec<&str>, _stdin: Option<&str>, _is_output_tty: bool) -> CommandRes {
        // Check for supported options
        if args.len() > 1 {
            return CommandRes::new()
                .with_error()
                .with_stderr("ps: too many arguments");
        }

        let detailed = if args.is_empty() {
            false
        } else if args[0] == "aux" {
            true
        } else {
            let arg = args[0].to_string();
            let error_msg = format!("ps: invalid argument -- '{arg}'\nUsage: ps [aux]");
            return CommandRes::new()
                .with_error()
                .with_stderr(error_msg);
        };

        let output = self.get_processes(detailed);
        let output_clone = output.clone();
        CommandRes::new().with_stdout(
            output,
            if _is_output_tty {
                Some(Arc::new(move || output_clone.clone().into_any()))
            } else {
                None
            },
        )
    }

}

pub struct KillCommand {
    processes: Vec<Process>,
}

impl KillCommand {
    pub fn new(processes: Vec<Process>) -> Self {
        Self { processes }
    }

    fn get_process_by_pid(&self, pid: u32) -> Option<&Process> {
        self.processes.iter().find(|p| p.pid == pid)
    }
}

static SIGS: [&str; 18] = [
    "HUP", "INT", "QUIT", "ILL", "TRAP", "ABRT", "EMT", "FPE", "KILL", "1", "2", "3", "4", "5",
    "6", "7", "8", "9",
];

impl Executable for KillCommand {
    fn execute(&self, _path: &str, args: Vec<&str>, _stdin: Option<&str>, _is_output_tty: bool) -> CommandRes {
        if args.is_empty() {
            return CommandRes::new()
                .with_error()
                .with_stderr("kill: not enough arguments");
        }

        let pids = if args[0].starts_with("-") {
            let signal_name = args[0][1..].to_uppercase();
            if !SIGS.contains(&signal_name.as_str()) {
                if signal_name.chars().all(|c| c.is_ascii_alphabetic()) {
                    let error_msg = format!("kill: unknown signal: SIG{signal_name}");
                    return CommandRes::new()
                        .with_error()
                        .with_stderr(error_msg);
                } else {
                    let error_msg = "kill: usage: kill [-n signum] pid";
                    return CommandRes::new()
                        .with_error()
                        .with_stderr(error_msg);
                }
            }
            &args[1..]
        } else {
            &args
        };

        if pids.is_empty() {
            return CommandRes::new()
                .with_error()
                .with_stderr("kill: not enough arguments");
        }

        // TODO - loop pids
        let pid_str = pids[0];

        let pid = match pid_str.parse::<u32>() {
            Ok(p) => p,
            Err(_) => {
                let pid_str = pid_str.to_string();
                let error_msg = format!("kill: illegal pid: {pid_str}");
                return CommandRes::new()
                    .with_error()
                    .with_stderr(error_msg);
            }
        };

        // Check if process exists
        let process_exists = self.get_process_by_pid(pid).is_some();

        if !process_exists {
            let error_msg = format!("kill: kill {pid} failed: no such process");
            return CommandRes::new()
                .with_error()
                .with_stderr(error_msg);
        }

        // Handle special PID 42 with easter egg
        if pid == 42 {
            let message =
                "Answer to everything terminated\nkill: kill 42 failed: operation not permitted";
            return CommandRes::new()
                .with_error()
                .with_stderr(message);
        }

        // All core services show permission denied
        let core_services = [1, 42, 99, 128, 256];
        if core_services.contains(&pid) {
            let error_msg = format!("kill: kill {pid} failed: operation not permitted");
            return CommandRes::new()
                .with_error()
                .with_stderr(error_msg);
        }

        // This shouldn't be reached with our current process list, but included for completeness
        let error_msg = format!("kill: kill {pid} failed: operation not permitted");
        CommandRes::new()
            .with_error()
            .with_stderr(error_msg)
    }

}
