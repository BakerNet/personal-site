use std::sync::Arc;

use leptos::prelude::*;

use crate::app::terminal::command::{CommandRes, Executable, PipelineRes};

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
    fn execute(&self, _path: &str, args: Vec<&str>) -> CommandRes {
        // Check for supported options
        if args.len() > 1 {
            return CommandRes::Err(Arc::new(move || "ps: too many arguments".into_any()));
        }

        let detailed = if args.is_empty() {
            false
        } else if args[0] == "aux" {
            true
        } else {
            let arg = args[0].to_string();
            return CommandRes::Err(Arc::new(move || {
                format!("ps: invalid argument -- '{arg}'\nUsage: ps [aux]").into_any()
            }));
        };

        let output = self.get_processes(detailed);
        CommandRes::Output(Arc::new(move || output.clone().into_any()))
    }

    fn execute_pipeable(&self, _path: &str, args: Vec<&str>, _stdin: &str) -> PipelineRes {
        // Check for supported options
        if args.len() > 1 {
            return PipelineRes::Err("ps: too many arguments".to_string());
        }

        let detailed = if args.is_empty() {
            false
        } else if args[0] == "aux" {
            true
        } else {
            let arg = args[0].to_string();
            return PipelineRes::Err(format!("ps: invalid argument -- '{arg}'\nUsage: ps [aux]"));
        };

        let output = self.get_processes(detailed);

        PipelineRes::Output(output)
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
    fn execute(&self, _path: &str, args: Vec<&str>) -> CommandRes {
        if args.is_empty() {
            return CommandRes::Err(Arc::new(move || "kill: not enough arguments".into_any()));
        }

        let pids = if args[0].starts_with("-") {
            let signal_name = &args[0][1..];
            if !SIGS.contains(&signal_name) {
                if signal_name.chars().all(|c| c.is_ascii_alphabetic()) {
                    let signal_name = signal_name.to_uppercase();
                    return CommandRes::Err(Arc::new(move || {
                        format!("kill: unknown signal: SIG{}", signal_name).into_any()
                    }));
                } else {
                    return CommandRes::Err(Arc::new(move || {
                        "kill: usage: kill [-n signum] pid".into_any()
                    }));
                }
            }
            &args[1..]
        } else {
            &args
        };

        if pids.is_empty() {
            return CommandRes::Err(Arc::new(move || "kill: not enough arguments".into_any()));
        }

        // TODO - loop pids
        let pid_str = pids[0];

        let pid = match pid_str.parse::<u32>() {
            Ok(p) => p,
            Err(_) => {
                let pid_str = pid_str.to_string();
                return CommandRes::Err(Arc::new(move || {
                    format!("kill: illegal pid: {pid_str}").into_any()
                }));
            }
        };

        // Check if process exists
        let process_exists = self.get_process_by_pid(pid).is_some();

        if !process_exists {
            return CommandRes::Err(Arc::new(move || {
                format!("kill: kill {pid} failed: no such process").into_any()
            }));
        }

        // Handle special PID 42 with easter egg
        if pid == 42 {
            let message =
                "Answer to everything terminated\nkill: kill 42 failed: operation not permitted";
            return CommandRes::Err(Arc::new(move || message.into_any()));
        }

        // All core services show permission denied
        let core_services = vec![1, 42, 99, 128, 256];
        if core_services.contains(&pid) {
            return CommandRes::Err(Arc::new(move || {
                format!("kill: kill {pid} failed: operation not permitted").into_any()
            }));
        }

        // This shouldn't be reached with our current process list, but included for completeness
        CommandRes::Err(Arc::new(move || {
            format!("kill: kill {pid} failed: operation not permitted").into_any()
        }))
    }

    fn execute_pipeable(&self, _path: &str, _args: Vec<&str>, _stdin: &str) -> PipelineRes {
        todo!()
    }
}
