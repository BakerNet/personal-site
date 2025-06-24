use std::sync::Arc;

use leptos::prelude::*;
use leptos_router::components::*;

use crate::app::terminal::fs::DirContentItem;

use super::command::{CommandRes, Executable};
use super::components::{ColumnarView, TextContent};
use super::fs::{parse_multitarget, path_target_to_target_path, Dir, Target};
pub struct LsCommand {
    blog_posts: Vec<String>,
}

impl LsCommand {
    pub fn new(blog_posts: Vec<String>) -> Self {
        Self { blog_posts }
    }
}

impl Executable for LsCommand {
    fn execute(
        &self,
        path: &str,
        args: Vec<&str>,
        _stdin: Option<&str>,
        is_output_tty: bool,
    ) -> CommandRes {
        let mut all = false;
        let mut long_format = false;
        let (options, mut target_paths) = parse_multitarget(args);
        let invalid = options.iter().find(|c| **c != 'a' && **c != 'l');
        if let Some(c) = invalid {
            let c = c.to_owned();
            let error_msg = format!(
                r#"ls: invalid option -- '{c}'
This version of ls only supports options 'a' and 'l'"#
            );
            return CommandRes::new().with_error().with_stderr(error_msg);
        }
        for option in &options {
            match option {
                'a' => all = true,
                'l' => long_format = true,
                _ => unreachable!("Invalid options should be caught above"),
            }
        }
        if target_paths.is_empty() {
            target_paths = vec![""];
        }

        // Process targets and collect errors
        let mut stderr_parts = Vec::new();
        let mut file_targets: Vec<(String, Target)> = Vec::new();
        let mut dir_targets: Vec<(String, Dir)> = Vec::new();
        let mut has_error = false;

        for tp in target_paths.iter() {
            let target_string = tp.to_string();
            let target_path = path_target_to_target_path(path, tp, false);
            let target = Target::from_str(&target_path, &self.blog_posts);

            match target {
                Target::File(_) => file_targets.push((tp.to_string(), target)),
                Target::Dir(d) => dir_targets.push((tp.to_string(), d)),
                Target::Invalid => {
                    // If the target is empty, we treat it as the current directory
                    has_error = true;
                    stderr_parts.push(format!(
                        "ls: cannot access '{target_string}': No such file or directory"
                    ));
                    continue;
                }
            }
        }

        let mut result = CommandRes::new();

        if has_error {
            let stderr_text = stderr_parts.join("\n");
            result = result.with_error().with_stderr(stderr_text);
        }

        file_targets.sort_by(|a, b| a.0.cmp(&b.0));
        dir_targets.sort_by(|a, b| a.0.cmp(&b.0));

        if is_output_tty {
            let posts = self.blog_posts.clone();
            let is_multi =
                dir_targets.len() > 1 || !dir_targets.is_empty() && !file_targets.is_empty();
            let all_captured = all;
            let long_format_captured = long_format;
            let path_owned = path.to_owned();
            result = result.with_stdout_view(Arc::new(move || {
                let mut all_views = Vec::new();
                if !file_targets.is_empty() {
                    all_views.push(
                        LsView(LsViewProps {
                            items: file_targets
                                .iter()
                                .map(|(s, t)| DirContentItem(s.to_string(), t.to_owned()))
                                .collect(),
                            base: path_owned.clone(),
                            long_format: long_format_captured,
                            blog_post_count: posts.len(),
                        })
                        .into_any(),
                    );
                    if is_multi {
                        all_views.push(view! { <br /> }.into_any());
                    }
                }
                for (i, (tp, d)) in dir_targets.iter().enumerate() {
                    if is_multi {
                        if i > 0 {
                            all_views.push(view! { <br /> }.into_any());
                        }
                        all_views.push(
                            view! {
                                {format!("{tp}:")}
                                <br />
                            }
                            .into_any(),
                        );
                    }
                    all_views.push(
                        LsView(LsViewProps {
                            items: d.contents(&posts, all_captured),
                            base: d.base(),
                            long_format: long_format_captured,
                            blog_post_count: posts.len(),
                        })
                        .into_any(),
                    );
                }
                view! { {all_views} }.into_any()
            }))
        } else {
            let mut stdout_text = String::new();
            let is_multi =
                dir_targets.len() > 1 || !dir_targets.is_empty() && !file_targets.is_empty();

            // Handle file targets
            if !file_targets.is_empty() {
                for (_, target) in file_targets.iter() {
                    if let Target::File(f) = target {
                        if !stdout_text.is_empty() {
                            stdout_text.push('\n');
                        }
                        if long_format {
                            stdout_text.push_str(&format!(
                                "{} {:2} {:8} {:8} {:>6} {}",
                                target.full_permissions(),
                                target.link_count(self.blog_posts.len()),
                                target.owner(),
                                target.group(),
                                target.size(),
                                f.name()
                            ));
                        } else {
                            stdout_text.push_str(f.name());
                        }
                    }
                }
            }

            // Handle directory targets
            for (tp, d) in dir_targets.iter() {
                if is_multi {
                    if !stdout_text.is_empty() {
                        stdout_text.push_str("\n\n");
                    }
                    stdout_text.push_str(&format!("{tp}:\n"));
                }

                let items = d.contents(&self.blog_posts, all);
                for (i, item) in items.iter().enumerate() {
                    if i > 0 || (!is_multi && !stdout_text.is_empty()) {
                        stdout_text.push('\n');
                    }
                    if long_format {
                        stdout_text.push_str(&format!(
                            "{} {:2} {:8} {:8} {:>6} {}",
                            item.1.full_permissions(),
                            item.1.link_count(self.blog_posts.len()),
                            item.1.owner(),
                            item.1.group(),
                            item.1.size(),
                            item.text_content()
                        ));
                    } else {
                        stdout_text.push_str(item.text_content());
                    }
                }
            }

            if !stdout_text.is_empty() {
                result = result.with_stdout_text(stdout_text);
            }
        }

        result
    }
}

#[component]
fn LsView(
    items: Vec<DirContentItem>,
    base: String,
    #[prop(default = false)] long_format: bool,
    #[prop(default = 0)] blog_post_count: usize,
) -> impl IntoView {
    let dir_class = "text-blue";
    let ex_class = "text-green";

    // Create modified items for long format display if needed
    let display_items = if long_format {
        items
            .into_iter()
            .map(|item| {
                let long_info = format!(
                    "{} {:2} {:8} {:8} {:>6} {}",
                    item.1.full_permissions(),
                    item.1.link_count(blog_post_count),
                    item.1.owner(),
                    item.1.group(),
                    item.1.size(),
                    item.0
                );
                DirContentItem(long_info, item.1)
            })
            .collect::<Vec<_>>()
    } else {
        items
    };

    if long_format {
        let long_render_func = move |s: DirContentItem| {
            // Find the last space before the filename to preserve original formatting
            let last_space_pos = s.0.rfind(' ').unwrap();
            let metadata_with_spaces = s.0[..last_space_pos].to_string();
            let filename = s.0[last_space_pos + 1..].to_string();

            // Create the styled filename part
            let styled_filename = if matches!(s.1, Target::Dir(_)) {
                let base = if base == "/" { "" } else { &base };
                let href = if filename == "." {
                    base.to_string()
                } else {
                    format!("{}/{}", base, filename)
                };
                view! { <A href=href attr:class=dir_class>{filename}</A> }.into_any()
            } else if s.1.is_executable() {
                view! { <span class=ex_class>{filename}</span> }.into_any()
            } else {
                view! { <span>{filename}</span> }.into_any()
            };

            view! {
                <span>{metadata_with_spaces} " " {styled_filename}</span>
            }
            .into_any()
        };

        view! {
            <div>
                {display_items.into_iter().map(|item| {
                    view! {
                        {long_render_func(item)} "\n"
                    }
                }).collect_view()}
            </div>
        }
        .into_any()
    } else {
        let short_render_func = {
            move |s: DirContentItem| {
                if matches!(s.1, Target::Dir(_)) {
                    let base = if base == "/" { "" } else { &base };
                    let href = if s.0 == "." {
                        base.to_string()
                    } else {
                        format!("{}/{}", base, s.0)
                    };
                    view! {
                        <A href=href attr:class=dir_class>
                            {s.text_content().to_string()}
                        </A>
                    }
                    .into_any()
                } else if s.1.is_executable() {
                    view! { <span class=ex_class>{s.text_content()}</span> }.into_any()
                } else {
                    view! { <span>{s.text_content()}</span> }.into_any()
                }
            }
        };

        view! {
            <div>
                <ColumnarView items=display_items render_func=short_render_func />
            </div>
        }
        .into_any()
    }
}

pub struct CatCommand {
    blog_posts: Vec<String>,
}

impl CatCommand {
    pub fn new(blog_posts: Vec<String>) -> Self {
        Self { blog_posts }
    }
}

impl Executable for CatCommand {
    fn execute(
        &self,
        path: &str,
        args: Vec<&str>,
        _stdin: Option<&str>,
        _is_output_tty: bool,
    ) -> CommandRes {
        let (options, targets) = parse_multitarget(args);
        if !options.is_empty() {
            let c = options[0].to_owned();
            let error_msg = format!(
                r#"cat: invalid option -- '{c}'
This version of cat doesn't support any options"#
            );
            return CommandRes::new().with_error().with_stderr(error_msg);
        }
        if targets.is_empty() {
            return CommandRes::new().with_error();
        }

        // Process targets and collect outputs
        let mut stdout_parts = Vec::new();
        let mut stderr_parts = Vec::new();
        let mut has_error = false;

        for tp in targets.iter() {
            let target_string = tp.to_string();
            let target_path = path_target_to_target_path(path, tp, false);
            let target = Target::from_str(&target_path, &self.blog_posts);

            match target {
                Target::File(f) => {
                    stdout_parts.push(f.contents().to_string());
                }
                Target::Dir(_) => {
                    has_error = true;
                    stderr_parts.push(format!("cat: {target_string}: Is a directory"));
                }
                Target::Invalid => {
                    has_error = true;
                    stderr_parts.push(format!("cat: {target_string}: No such file or directory"));
                }
            }
        }

        let stdout_text = stdout_parts.join("\n");
        let stderr_text = stderr_parts.join("\n");

        let mut result = CommandRes::new();
        if has_error {
            result = result.with_error();
        }

        if !stdout_text.is_empty() {
            result = result.with_stdout_text(stdout_text);
        }

        if !stderr_text.is_empty() {
            result = result.with_stderr(stderr_text);
        }

        result
    }
}

pub struct CdCommand {
    blog_posts: Vec<String>,
}

impl CdCommand {
    pub fn new(blog_posts: Vec<String>) -> Self {
        Self { blog_posts }
    }
}

impl Executable for CdCommand {
    fn execute(
        &self,
        path: &str,
        args: Vec<&str>,
        _stdin: Option<&str>,
        _is_output_tty: bool,
    ) -> CommandRes {
        if args.len() >= 2 {
            let error_msg = "cd: too many arguments";
            return CommandRes::new().with_error().with_stderr(error_msg);
        }
        let target_path = if args.is_empty() { "/" } else { args[0] };
        let target_string = target_path.to_owned();
        let target_path = path_target_to_target_path(path, target_path, false);
        let target = Target::from_str(&target_path, &self.blog_posts);
        if target_path == path {
            return CommandRes::new();
        }
        match target {
            Target::File(_) => {
                let error_msg = format!("cd: not a directory: {target_string}");
                CommandRes::new().with_error().with_stderr(error_msg)
            }
            Target::Dir(_) => CommandRes::Redirect(target_path),
            Target::Invalid => {
                let error_msg = format!("cd: no such file or directory: {target_string}");
                CommandRes::new().with_error().with_stderr(error_msg)
            }
        }
    }
}

pub struct TouchCommand {
    blog_posts: Vec<String>,
}

impl TouchCommand {
    pub fn new(blog_posts: Vec<String>) -> Self {
        Self { blog_posts }
    }
}

impl Executable for TouchCommand {
    fn execute(
        &self,
        path: &str,
        args: Vec<&str>,
        _stdin: Option<&str>,
        _is_output_tty: bool,
    ) -> CommandRes {
        let (_, targets) = parse_multitarget(args);
        if targets.is_empty() {
            return CommandRes::new()
                .with_error()
                .with_stderr("touch: missing operand");
        }
        let targets = targets.into_iter().fold(Vec::new(), |mut ts, tp| {
            let target_string = tp.to_owned();
            let tp = if tp.contains("/") {
                tp.rsplit_once("/").unwrap().0
            } else {
                ""
            };
            let target_path = path_target_to_target_path(path, tp, false);
            let target = Target::from_str(&target_path, &self.blog_posts);
            ts.push((target_string, target));
            ts
        });
        let error_messages = targets
            .iter()
            .map(|(name, ts)| {
                let base = format!("touch: cannot touch '{name}': ");
                match ts {
                    Target::Dir(_) => base + "Permission denied",
                    Target::File(_) => base + "Not a directory",
                    Target::Invalid => base + "No such file or directory",
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        CommandRes::new().with_error().with_stderr(error_messages)
    }
}

pub struct MkdirCommand {
    blog_posts: Vec<String>,
}

impl MkdirCommand {
    pub fn new(blog_posts: Vec<String>) -> Self {
        Self { blog_posts }
    }
}

impl Executable for MkdirCommand {
    fn execute(
        &self,
        path: &str,
        args: Vec<&str>,
        _stdin: Option<&str>,
        _is_output_tty: bool,
    ) -> CommandRes {
        let (_, targets) = parse_multitarget(args);
        if targets.is_empty() {
            return CommandRes::new()
                .with_error()
                .with_stderr("mkdir: missing operand");
        }
        let targets = targets.into_iter().fold(Vec::new(), |mut ts, tp| {
            let target_string = tp.to_owned();
            let tp = if tp.contains("/") {
                tp.rsplit_once("/").unwrap().0
            } else {
                ""
            };
            let target_path = path_target_to_target_path(path, tp, false);
            let target = Target::from_str(&target_path, &self.blog_posts);
            ts.push((target_string, target));
            ts
        });
        let error_messages = targets
            .iter()
            .map(|(name, ts)| {
                let base = format!("mkdir: cannot create directory '{name}': ");
                match ts {
                    Target::Dir(_) => base + "Permission denied",
                    Target::File(_) => base + "Not a directory",
                    Target::Invalid => base + "No such file or directory",
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        CommandRes::new().with_error().with_stderr(error_messages)
    }
}

pub struct RmCommand {
    blog_posts: Vec<String>,
}

impl RmCommand {
    pub fn new(blog_posts: Vec<String>) -> Self {
        Self { blog_posts }
    }
}

impl Executable for RmCommand {
    fn execute(
        &self,
        path: &str,
        args: Vec<&str>,
        _stdin: Option<&str>,
        _is_output_tty: bool,
    ) -> CommandRes {
        let (_, targets) = parse_multitarget(args);
        if targets.is_empty() {
            return CommandRes::new()
                .with_error()
                .with_stderr("rm: missing operand");
        }
        let targets = targets.into_iter().fold(Vec::new(), |mut ts, tp| {
            let target_string = tp.to_owned();
            let target_path = path_target_to_target_path(path, tp, false);
            let target = Target::from_str(&target_path, &self.blog_posts);
            ts.push((target_string, target));
            ts
        });
        let error_messages = targets
            .iter()
            .map(|(name, ts)| {
                let base = format!("rm: cannot remove '{name}': ");
                match ts {
                    Target::Dir(_) => base + "Permission denied",
                    Target::File(_) => base + "Permission denied",
                    Target::Invalid => base + "No such file or directory",
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        CommandRes::new().with_error().with_stderr(error_messages)
    }
}

pub struct CpCommand {
    blog_posts: Vec<String>,
}

impl CpCommand {
    pub fn new(blog_posts: Vec<String>) -> Self {
        Self { blog_posts }
    }
}

impl Executable for CpCommand {
    fn execute(
        &self,
        path: &str,
        args: Vec<&str>,
        _stdin: Option<&str>,
        _is_output_tty: bool,
    ) -> CommandRes {
        let (options, targets) = parse_multitarget(args);
        let mut recursive = false;
        let invalid = options.iter().find(|c| **c != 'r');
        if let Some(c) = invalid {
            let c = c.to_owned();
            let error_msg = format!(
                r#"cp: invalid option -- '{c}'
This version of cp only supports option 'r'"#
            );
            return CommandRes::new().with_error().with_stderr(error_msg);
        }
        if !options.is_empty() {
            recursive = true;
        }
        if targets.is_empty() {
            return CommandRes::new()
                .with_error()
                .with_stderr("cp: missing file operand");
        }
        if targets.len() < 2 {
            let target = targets[0].to_owned();
            let error_msg = format!("cp: missing destination file operand after {target}");
            return CommandRes::new().with_error().with_stderr(error_msg);
        }
        let targets = targets
            .into_iter()
            .enumerate()
            .fold(Vec::new(), |mut ts, (i, tp)| {
                let target_string = tp.to_owned();
                let target_path = path_target_to_target_path(path, tp, false);
                let full_target = Target::from_str(&target_path, &self.blog_posts);
                let tp = if i != 0 && tp.contains("/") {
                    tp.rsplit_once("/").unwrap().0
                } else {
                    ""
                };
                let target_path = path_target_to_target_path(path, tp, false);
                let partial_target = Target::from_str(&target_path, &self.blog_posts);
                ts.push((target_string, full_target, partial_target));
                ts
            });
        let target_filename = match (recursive, &targets[0].1) {
            (false, Target::Dir(_)) => {
                let error_msg = format!(
                    "cp: -r not specified; omitting directory '{}'",
                    targets[0].0
                );
                return CommandRes::new().with_error().with_stderr(error_msg);
            }
            (_, Target::Invalid) => {
                let error_msg = format!(
                    "cp: cannot stat '{}': No such file or directory",
                    targets[0].0
                );
                return CommandRes::new().with_error().with_stderr(error_msg);
            }
            _ => {
                let target = &targets[0].0;
                let target = if target.ends_with("/") {
                    &target[..target.len() - 1]
                } else {
                    &target[..]
                };
                target
                    .split("/")
                    .last()
                    .expect("Should have a last element")
                    .to_string()
            }
        };
        let error_messages = targets.iter().skip(1).map(|(name, full_ts, partial_ts)| {
            match full_ts {
                Target::Dir(_) => {
                    if name.ends_with("/") {
                        format!("cp: cannot create regular file '{name}{target_filename}': Permission denied")
                    } else {
                        format!("cp: cannot create regular file '{name}/{target_filename}': Permission denied")
                    }
                },
                Target::File(_) => format!("cp: cannot create regular file '{name}': Permission denied"),
                Target::Invalid => {
                    if name.ends_with("/") {
                        format!("cp: cannot create regular file '{name}': Not a directory")
                    } else {
                        match partial_ts {
                            Target::Dir(_) | Target::File(_) => format!("cp: cannot create regular file '{name}': Permission denied"),
                            Target::Invalid => format!("cp: cannot create regular file '{name}': No such file or directory"),
                        }
                    }
                }
            }
        }).collect::<Vec<_>>().join("\n");

        CommandRes::new().with_error().with_stderr(error_messages)
    }
}

pub struct MvCommand {
    blog_posts: Vec<String>,
}

impl MvCommand {
    pub fn new(blog_posts: Vec<String>) -> Self {
        Self { blog_posts }
    }
}

impl Executable for MvCommand {
    fn execute(
        &self,
        path: &str,
        args: Vec<&str>,
        _stdin: Option<&str>,
        _is_output_tty: bool,
    ) -> CommandRes {
        let (options, targets) = parse_multitarget(args);
        let invalid = options.iter().find(|c| **c != 'f');
        if let Some(c) = invalid {
            let c = c.to_owned();
            let error_msg = format!(
                r#"mv: invalid option -- '{c}'
This version of mv only supports option 'f'"#
            );
            return CommandRes::new().with_error().with_stderr(error_msg);
        }
        if targets.is_empty() {
            return CommandRes::new()
                .with_error()
                .with_stderr("mv: missing file operand");
        }
        if targets.len() < 2 {
            let target = targets[0].to_owned();
            let error_msg = format!("mv: missing destination file operand after {target}");
            return CommandRes::new().with_error().with_stderr(error_msg);
        }
        let targets = targets
            .into_iter()
            .enumerate()
            .fold(Vec::new(), |mut ts, (i, tp)| {
                let target_string = tp.to_owned();
                let target_path = path_target_to_target_path(path, tp, false);
                let full_target = Target::from_str(&target_path, &self.blog_posts);
                let tp = if i != 0 && tp.contains("/") {
                    tp.rsplit_once("/").unwrap().0
                } else {
                    ""
                };
                let target_path = path_target_to_target_path(path, tp, false);
                let partial_target = Target::from_str(&target_path, &self.blog_posts);
                ts.push((target_string, full_target, partial_target));
                ts
            });
        let target_filename = match &targets[0].1 {
            Target::Invalid => {
                let error_msg = format!(
                    "mv: cannot stat '{}': No such file or directory",
                    targets[0].0
                );
                return CommandRes::new().with_error().with_stderr(error_msg);
            }
            _ => {
                let target = &targets[0].0;
                let target = if target.ends_with("/") {
                    &target[..target.len() - 1]
                } else {
                    &target[..]
                };
                target
                    .split("/")
                    .last()
                    .expect("Should have a last element")
                    .to_string()
            }
        };
        let error_messages = targets.iter().skip(1).map(|(name, full_ts, partial_ts)| {
            match full_ts {
                Target::Dir(_) => {
                    if name.ends_with("/") {
                        format!("mv: cannot move '{name}': Permission denied")
                    } else {
                        format!("mv: cannot move '{name}/{target_filename}' to '{name}': Permission denied")
                    }
                },
                Target::File(_) => format!("mv: cannot move '{name}': Permission denied"),
                Target::Invalid => {
                    if name.ends_with("/") {
                        format!("mv: cannot move '{name}': Not a directory")
                    } else {
                        match partial_ts {
                            Target::Dir(_) | Target::File(_) => format!("mv: cannot move '{name}': Permission denied"),
                            Target::Invalid => format!("mv: cannot move '{name}': No such file or directory"),
                        }
                    }
                }
            }
        }).collect::<Vec<_>>().join("\n");

        CommandRes::new().with_error().with_stderr(error_messages)
    }
}
