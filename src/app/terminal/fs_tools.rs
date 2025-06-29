use std::sync::Arc;

use indextree::NodeId;
use leptos::prelude::*;
use leptos_router::components::*;

use super::command::{CommandRes, VfsCommand};
use super::components::{ColumnarView, TextContent};
use super::vfs::{FileContent, VfsError, VfsNode, VfsNodeType, VirtualFilesystem};

// Parse arguments to extract options & path arguments
pub fn parse_multitarget(args: Vec<&str>) -> (Vec<char>, Vec<&str>) {
    args.into_iter().fold(
        (Vec::<char>::new(), Vec::<&str>::new()),
        |(mut options, mut t), s| {
            if s.starts_with("-") {
                let mut opts = s.chars().filter(|c| *c != '-').collect::<Vec<char>>();
                options.append(&mut opts);
            } else if s.starts_with("~/") {
                t.push(&s[1..]);
            } else if s == "~" {
                t.push("/");
            } else {
                t.push(s);
            }
            (options, t)
        },
    )
}

#[derive(Debug, Clone)]
struct VfsItem {
    node: VfsNode,
    link_count: usize,    // Number of links to this item
    display_name: String, // Display name for the item
    path: String,         // Filesystem / URL path
}

impl TextContent for VfsItem {
    fn text_content(&self) -> &str {
        &self.display_name
    }
}

// VFS-based LsCommand for Phase 2 migration
pub struct LsCommand;

impl LsCommand {
    pub fn new() -> Self {
        Self
    }
}

impl VfsCommand for LsCommand {
    fn execute(
        &self,
        vfs: &mut VirtualFilesystem,
        current_dir: NodeId,
        args: Vec<&str>,
        _stdin: Option<&str>,
        is_output_tty: bool,
    ) -> CommandRes {
        let mut all = false;
        let mut long_format = false;
        let (options, mut target_paths) = parse_multitarget(args);

        // Validate options
        let invalid = options.iter().find(|c| **c != 'a' && **c != 'l');
        if let Some(c) = invalid {
            let c = c.to_owned();
            let error_msg = format!(
                r#"ls: invalid option -- '{c}'
This version of ls only supports options 'a' and 'l'"#
            );
            return CommandRes::new().with_error().with_stderr(error_msg);
        }

        // Process options
        for option in &options {
            match option {
                'a' => all = true,
                'l' => long_format = true,
                _ => unreachable!("Invalid options should be caught above"),
            }
        }

        // Default to current directory if no targets specified
        if target_paths.is_empty() {
            target_paths = vec![""];
        }

        // Process targets using VFS and create VfsItems directly
        let mut stderr_parts = Vec::new();
        let mut file_items: Vec<VfsItem> = Vec::new();
        let mut dir_listings: Vec<(String, Vec<VfsItem>)> = Vec::new(); // (display_name, items)
        let mut has_error = false;

        let get_vfs_item = |node_id: NodeId, display_name: String| {
            let node = vfs
                .get_node(node_id)
                .expect("We should be in a resolved_path");
            let link_count = if matches!(node.node_type, VfsNodeType::Directory) {
                vfs.list_directory(node_id).map(|es| es.len()).unwrap_or(0) + 2
            } else {
                1
            };

            VfsItem {
                node: node.clone(),
                link_count,
                display_name,
                path: vfs.get_node_path(node_id),
            }
        };

        for tp in target_paths.iter() {
            let target_string = tp.to_string();

            let resolved_path = if tp.is_empty() {
                Ok(current_dir)
            } else {
                vfs.resolve_path(current_dir, tp)
            };

            let node_id = if let Ok(node_id) = resolved_path {
                node_id
            } else {
                has_error = true;
                stderr_parts.push(format!(
                    "ls: cannot access '{target_string}': No such file or directory"
                ));
                continue;
            };

            let node = vfs
                .get_node(node_id)
                .expect("Node should exist after resolve_path");
            let node_path = vfs.get_node_path(node_id);

            match &node.node_type {
                VfsNodeType::File { .. } => {
                    file_items.push(VfsItem {
                        node: node.clone(),
                        link_count: 0,
                        display_name: target_string.clone(),
                        path: node_path,
                    });
                }
                VfsNodeType::Directory => {
                    if let Ok(entries) = vfs.list_directory(node_id) {
                        let mut dir_items: Vec<VfsItem> = Vec::new();
                        let dir_link_count = entries.len() + 2; // +2 for . and ..

                        // Add . and .. entries when -a flag is used
                        if all {
                            // Add current directory entry
                            dir_items.push(VfsItem {
                                node: node.clone(),
                                link_count: dir_link_count,
                                display_name: ".".to_string(),
                                path: node_path,
                            });

                            // Add parent directory entry
                            let parent_id =
                                vfs.get_parent(node_id).unwrap_or_else(|| vfs.get_root());
                            dir_items.push(get_vfs_item(parent_id, "..".to_string()));
                        }

                        // Add regular entries
                        for entry in entries {
                            // Skip hidden files unless -a is specified
                            if !all && entry.name.starts_with('.') {
                                continue;
                            }
                            dir_items.push(get_vfs_item(entry.node_id, entry.name));
                        }

                        dir_items.sort_by(|a, b| a.display_name.cmp(&b.display_name));
                        dir_listings.push((tp.to_string(), dir_items));
                    } else {
                        has_error = true;
                        stderr_parts.push(format!(
                            "ls: cannot access '{target_string}': Permission denied"
                        ));
                    }
                }
                VfsNodeType::Link { .. } => {
                    has_error = true;
                    stderr_parts.push(format!(
                        "ls: cannot access '{target_string}': No such file or directory"
                    ));
                }
            }
        }

        let mut result = CommandRes::new();

        if has_error {
            let stderr_text = stderr_parts.join("\n");
            result = result.with_error().with_stderr(stderr_text);
        }

        file_items.sort_by(|a, b| a.display_name.cmp(&b.display_name));
        dir_listings.sort_by(|a, b| a.0.cmp(&b.0));

        if is_output_tty {
            let is_multi =
                dir_listings.len() > 1 || (!dir_listings.is_empty() && !file_items.is_empty());

            result = result.with_stdout_view(Arc::new(move || {
                let mut all_views = Vec::new();

                // Handle file targets
                if !file_items.is_empty() {
                    all_views.push(
                        VfsLsView(VfsLsViewProps {
                            items: file_items.clone(),
                            long_format,
                        })
                        .into_any(),
                    );

                    if is_multi {
                        all_views.push(view! { <br /> }.into_any());
                    }
                }

                // Handle directory targets
                for (i, (display_name, items)) in dir_listings.iter().enumerate() {
                    if is_multi {
                        if i > 0 {
                            all_views.push(view! { <br /> }.into_any());
                        }
                        all_views.push(
                            view! {
                                {format!("{display_name}:")}
                                <br />
                            }
                            .into_any(),
                        );
                    }

                    all_views.push(
                        VfsLsView(VfsLsViewProps {
                            items: items.clone(),
                            long_format,
                        })
                        .into_any(),
                    );
                }

                view! { {all_views} }.into_any()
            }))
        } else {
            // For non-TTY output, just return simple text
            // TODO - fix
            let mut text_output = Vec::new();

            for item in &file_items {
                text_output.push(item.display_name.clone());
            }

            for (_, items) in &dir_listings {
                for item in items {
                    text_output.push(item.display_name.clone());
                }
            }

            if !text_output.is_empty() {
                result = result.with_stdout_text(text_output.join("\n"));
            }
        }

        result
    }
}

/// VFS-based LsView component that works directly with VfsItem instead of DirContentItem
#[component]
fn VfsLsView(items: Vec<VfsItem>, #[prop(default = false)] long_format: bool) -> impl IntoView {
    let dir_class = "text-blue";
    let ex_class = "text-green";

    if long_format {
        let long_render_func = move |item: VfsItem| {
            let filename = item.display_name;
            let path = item.path;
            let is_directory = item.node.is_directory();
            let is_executable = item.node.is_executable();

            // Create the styled filename part
            let styled_filename = if is_directory {
                view! {
                    <A href=path attr:class=dir_class>
                        {filename.clone()}
                    </A>
                }
                .into_any()
            } else if is_executable {
                view! { <span class=ex_class>{filename.clone()}</span> }.into_any()
            } else {
                view! { <span>{filename.clone()}</span> }.into_any()
            };

            view! {
                <div class="whitespace-pre font-mono">
                    {item.node.long_meta_string(item.link_count)}
                    {styled_filename}
                </div>
            }
            .into_any()
        };

        view! {
            <div>
                {items
                    .into_iter()
                    .map(long_render_func)
                    .collect_view()}
            </div>
        }
        .into_any()
    } else {
        let short_render_func = move |item: VfsItem| {
            let display_name = item.display_name;
            let path = item.path;
            let is_directory = item.node.is_directory();
            let is_executable = item.node.is_executable();

            if is_directory {
                view! {
                    <A href=path attr:class=dir_class>
                        {display_name}
                    </A>
                }
                .into_any()
            } else if is_executable {
                view! { <span class=ex_class>{display_name}</span> }.into_any()
            } else {
                view! { <span>{display_name}</span> }.into_any()
            }
        };

        view! {
            <div>
                <ColumnarView items=items render_func=short_render_func />
            </div>
        }
        .into_any()
    }
}

// VFS-based CdCommand for Phase 2 migration
pub struct CdCommand;

impl CdCommand {
    pub fn new() -> Self {
        Self
    }
}

impl VfsCommand for CdCommand {
    fn execute(
        &self,
        vfs: &mut VirtualFilesystem,
        current_dir: NodeId,
        args: Vec<&str>,
        _stdin: Option<&str>,
        _is_tty: bool,
    ) -> CommandRes {
        // Validate arguments
        if args.len() >= 2 {
            let error_msg = "cd: too many arguments";
            return CommandRes::new().with_error().with_stderr(error_msg);
        }

        let target_path = if args.is_empty() { "/" } else { args[0] };
        let target_string = target_path.to_string();

        // Resolve path using VFS (~ expansion is now handled by resolve_path)
        let resolved_path = if target_path.is_empty() {
            Ok(vfs.get_root())
        } else {
            vfs.resolve_path(current_dir, target_path)
        };

        match resolved_path {
            Ok(node_id) => {
                if let Some(node) = vfs.get_node(node_id) {
                    match &node.node_type {
                        VfsNodeType::Directory => {
                            // If it's the same directory, no change needed
                            if node_id == current_dir {
                                CommandRes::new()
                            } else {
                                // Return redirect with the new path
                                let new_path = vfs.get_node_path(node_id);
                                CommandRes::Redirect(new_path)
                            }
                        }
                        VfsNodeType::File { .. } => {
                            let error_msg = format!("cd: not a directory: {target_string}");
                            CommandRes::new().with_error().with_stderr(error_msg)
                        }
                        VfsNodeType::Link { .. } => {
                            let error_msg = format!("cd: cannot follow link: {target_string}");
                            CommandRes::new().with_error().with_stderr(error_msg)
                        }
                    }
                } else {
                    let error_msg = format!("cd: no such file or directory: {target_string}");
                    CommandRes::new().with_error().with_stderr(error_msg)
                }
            }
            Err(_) => {
                let error_msg = format!("cd: no such file or directory: {target_string}");
                CommandRes::new().with_error().with_stderr(error_msg)
            }
        }
    }
}

// VFS-based CatCommand for Phase 2 migration
pub struct CatCommand;

impl CatCommand {
    pub fn new() -> Self {
        Self
    }
}

impl VfsCommand for CatCommand {
    fn execute(
        &self,
        vfs: &mut VirtualFilesystem,
        current_dir: NodeId,
        args: Vec<&str>,
        _stdin: Option<&str>,
        _is_output_tty: bool,
    ) -> CommandRes {
        let (options, targets) = parse_multitarget(args);

        // Validate options
        if !options.is_empty() {
            let c = options[0].to_owned();
            let error_msg = format!(
                r#"cat: invalid option -- '{c}'
This version of cat doesn't support any options"#
            );
            return CommandRes::new().with_error().with_stderr(error_msg);
        }

        if targets.is_empty() {
            return CommandRes::new()
                .with_error()
                .with_stderr("cat: missing operand");
        }

        // Process targets and collect outputs
        let mut stdout_parts = Vec::new();
        let mut stderr_parts = Vec::new();
        let mut has_error = false;

        for tp in targets.iter() {
            let target_string = tp.to_string();

            let resolved_path = if tp.is_empty() {
                Err(())
            } else {
                vfs.resolve_path(current_dir, tp).map_err(|_| ())
            };

            let node_id = match resolved_path {
                Ok(node_id) => node_id,
                Err(_) => {
                    has_error = true;
                    stderr_parts.push(format!("cat: {target_string}: No such file or directory"));
                    continue;
                }
            };

            let node = match vfs.get_node(node_id) {
                Some(node) => node,
                None => {
                    has_error = true;
                    stderr_parts.push(format!("cat: {target_string}: No such file or directory"));
                    continue;
                }
            };

            match &node.node_type {
                VfsNodeType::File { .. } => match vfs.read_file(node_id) {
                    Ok(file_content) => {
                        stdout_parts.push(file_content);
                    }
                    Err(_) => {
                        has_error = true;
                        stderr_parts.push(format!("cat: {target_string}: Permission denied"));
                    }
                },
                VfsNodeType::Directory => {
                    has_error = true;
                    stderr_parts.push(format!("cat: {target_string}: Is a directory"));
                }
                VfsNodeType::Link { .. } => {
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
            result = result.with_stderr(stderr_text);
        }

        if !stdout_text.is_empty() {
            result = result.with_stdout_text(stdout_text);
        }

        result
    }
}

// VFS-based TouchCommand for Phase 2 migration
pub struct TouchCommand;

impl TouchCommand {
    pub fn new() -> Self {
        Self
    }
}

impl VfsCommand for TouchCommand {
    fn execute(
        &self,
        vfs: &mut VirtualFilesystem,
        current_dir: NodeId,
        args: Vec<&str>,
        _stdin: Option<&str>,
        _is_output_tty: bool,
    ) -> CommandRes {
        let (_, targets) = parse_multitarget(args);

        if targets.is_empty() {
            return CommandRes::new()
                .with_error()
                .with_stderr("touch: missing file operand");
        }

        let mut stderr_parts = Vec::new();
        let mut has_error = false;

        for target in targets {
            // Split the path to get parent directory and filename
            let (parent_path, filename) = if let Some(pos) = target.rfind('/') {
                (&target[..pos], &target[pos + 1..])
            } else {
                ("", target)
            };

            // Don't allow empty filename
            if filename.is_empty() {
                has_error = true;
                stderr_parts.push(format!(
                    "touch: cannot touch '{target}': No such file or directory"
                ));
                continue;
            }

            // Resolve parent directory
            let parent_id = if parent_path.is_empty() {
                Ok(current_dir)
            } else {
                vfs.resolve_path(current_dir, parent_path)
            };

            let parent_id = match parent_id {
                Ok(id) => id,
                Err(_) => {
                    has_error = true;
                    stderr_parts.push(format!(
                        "touch: cannot touch '{target}': No such file or directory"
                    ));
                    continue;
                }
            };

            // Check if file already exists
            let mut file_exists = false;
            if let Ok(entries) = vfs.list_directory(parent_id) {
                for entry in entries {
                    if entry.name == filename {
                        file_exists = true;
                        break;
                    }
                }
            }

            // If file doesn't exist, create it
            if !file_exists {
                match vfs.create_file(parent_id, filename, FileContent::Dynamic(String::new())) {
                    Ok(_) => {
                        // File created successfully
                    }
                    Err(VfsError::PermissionDenied) => {
                        has_error = true;
                        stderr_parts
                            .push(format!("touch: cannot touch '{target}': Permission denied"));
                    }
                    Err(_) => {
                        has_error = true;
                        stderr_parts.push(format!(
                            "touch: cannot touch '{target}': No such file or directory"
                        ));
                    }
                }
            }
            // If file exists, touch would normally update timestamps, but we don't have that functionality yet
        }

        let mut result = CommandRes::new();
        if has_error {
            result = result.with_error();
            let stderr_text = stderr_parts.join("\n");
            result = result.with_stderr(stderr_text);
        }

        result
    }
}

// VFS-based MkdirCommand for Phase 2 migration
pub struct MkdirCommand;

impl MkdirCommand {
    pub fn new() -> Self {
        Self
    }
}

impl VfsCommand for MkdirCommand {
    fn execute(
        &self,
        vfs: &mut VirtualFilesystem,
        current_dir: NodeId,
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

        let mut stderr_parts = Vec::new();
        let mut has_error = false;

        for target in targets {
            // Split the path to get parent directory and dirname
            let (parent_path, dirname) = if let Some(pos) = target.rfind('/') {
                (&target[..pos], &target[pos + 1..])
            } else {
                ("", target)
            };

            // Don't allow empty dirname
            if dirname.is_empty() {
                has_error = true;
                stderr_parts.push(format!(
                    "mkdir: cannot create directory '{target}': No such file or directory"
                ));
                continue;
            }

            // Resolve parent directory
            let parent_id = if parent_path.is_empty() {
                Ok(current_dir)
            } else {
                vfs.resolve_path(current_dir, parent_path)
            };

            let parent_id = match parent_id {
                Ok(id) => id,
                Err(_) => {
                    has_error = true;
                    stderr_parts.push(format!(
                        "mkdir: cannot create directory '{target}': No such file or directory"
                    ));
                    continue;
                }
            };

            // Create the directory
            match vfs.create_directory(parent_id, dirname) {
                Ok(_) => {
                    // Directory created successfully
                }
                Err(VfsError::AlreadyExists) => {
                    has_error = true;
                    stderr_parts.push(format!(
                        "mkdir: cannot create directory '{target}': File exists"
                    ));
                }
                Err(VfsError::PermissionDenied) => {
                    has_error = true;
                    stderr_parts.push(format!(
                        "mkdir: cannot create directory '{target}': Permission denied"
                    ));
                }
                Err(VfsError::NotADirectory) => {
                    has_error = true;
                    stderr_parts.push(format!(
                        "mkdir: cannot create directory '{target}': Not a directory"
                    ));
                }
                Err(_) => {
                    has_error = true;
                    stderr_parts.push(format!(
                        "mkdir: cannot create directory '{target}': No such file or directory"
                    ));
                }
            }
        }

        let mut result = CommandRes::new();
        if has_error {
            result = result.with_error();
            let stderr_text = stderr_parts.join("\n");
            result = result.with_stderr(stderr_text);
        }

        result
    }
}

// VFS-based RmCommand for Phase 2 migration
pub struct RmCommand;

impl RmCommand {
    pub fn new() -> Self {
        Self
    }
}

impl VfsCommand for RmCommand {
    fn execute(
        &self,
        vfs: &mut VirtualFilesystem,
        current_dir: NodeId,
        args: Vec<&str>,
        _stdin: Option<&str>,
        _is_output_tty: bool,
    ) -> CommandRes {
        let (options, targets) = parse_multitarget(args);

        // Check for recursive option
        let recursive = options.contains(&'r');

        // Validate options
        let invalid = options.iter().find(|c| **c != 'r' && **c != 'f');
        if let Some(c) = invalid {
            let c = c.to_owned();
            let error_msg = format!(
                r#"rm: invalid option -- '{c}'
This version of rm only supports options 'r' and 'f'"#
            );
            return CommandRes::new().with_error().with_stderr(error_msg);
        }

        if targets.is_empty() {
            return CommandRes::new()
                .with_error()
                .with_stderr("rm: missing operand");
        }

        let mut stderr_parts = Vec::new();
        let mut has_error = false;

        for target in targets {
            // Resolve the target path
            let node_id = match vfs.resolve_path(current_dir, target) {
                Ok(id) => id,
                Err(_) => {
                    has_error = true;
                    stderr_parts.push(format!(
                        "rm: cannot remove '{target}': No such file or directory"
                    ));
                    continue;
                }
            };

            // Check if it's a directory
            if let Some(node) = vfs.get_node(node_id) {
                let is_directory = matches!(node.node_type, VfsNodeType::Directory);

                if is_directory && !recursive {
                    has_error = true;
                    stderr_parts.push(format!("rm: cannot remove '{target}': Is a directory"));
                    continue;
                }
            }

            // Try to delete the node
            let delete_result = if recursive {
                vfs.delete_node_recursive(node_id)
            } else {
                vfs.delete_node(node_id)
            };

            match delete_result {
                Ok(_) => {
                    // Node deleted successfully
                }
                Err(VfsError::PermissionDenied) => {
                    has_error = true;
                    stderr_parts.push(format!("rm: cannot remove '{target}': Permission denied"));
                }
                Err(VfsError::SystemError(msg)) if msg.contains("not empty") => {
                    has_error = true;
                    stderr_parts.push(format!("rm: cannot remove '{target}': Directory not empty"));
                }
                Err(_) => {
                    has_error = true;
                    stderr_parts.push(format!("rm: cannot remove '{target}': Permission denied"));
                }
            }
        }

        let mut result = CommandRes::new();
        if has_error {
            result = result.with_error();
            let stderr_text = stderr_parts.join("\n");
            result = result.with_stderr(stderr_text);
        }

        result
    }
}
