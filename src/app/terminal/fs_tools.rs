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
        is_tty: bool,
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

        if is_tty {
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
            // TODO - fix - not currently doing long format
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
        _is_tty: bool,
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
        _is_tty: bool,
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
        _is_tty: bool,
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
        _is_tty: bool,
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

pub struct CpCommand;

impl CpCommand {
    pub fn new() -> Self {
        Self
    }
}

impl VfsCommand for CpCommand {
    fn execute(
        &self,
        vfs: &mut VirtualFilesystem,
        current_dir: NodeId,
        args: Vec<&str>,
        _stdin: Option<&str>,
        _is_tty: bool,
    ) -> CommandRes {
        let (options, targets) = parse_multitarget(args);

        // Check for recursive option
        let recursive = options.contains(&'r');

        // Validate options
        let invalid = options.iter().find(|c| **c != 'r' && **c != 'f');
        if let Some(c) = invalid {
            let c = c.to_owned();
            let error_msg = format!(
                r#"cp: invalid option -- '{c}'
This version of cp only supports options 'r' and 'f'"#
            );
            return CommandRes::new().with_error().with_stderr(error_msg);
        }

        if targets.len() < 2 {
            return CommandRes::new()
                .with_error()
                .with_stderr("cp: missing destination file operand");
        }

        let destination = targets.last().unwrap();
        let sources = &targets[..targets.len() - 1];

        let mut stderr_parts = Vec::new();
        let mut has_error = false;

        for source in sources {
            match self.copy_item(vfs, current_dir, source, destination, recursive) {
                Ok(_) => {}
                Err(err_msg) => {
                    has_error = true;
                    stderr_parts.push(err_msg);
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

impl CpCommand {
    fn copy_item(
        &self,
        vfs: &mut VirtualFilesystem,
        current_dir: NodeId,
        source_path: &str,
        dest_path: &str,
        recursive: bool,
    ) -> Result<(), String> {
        // Resolve source path
        let source_id = vfs
            .resolve_path(current_dir, source_path)
            .map_err(|_| format!("cp: cannot stat '{source_path}': No such file or directory"))?;

        let source_node = vfs
            .get_node(source_id)
            .ok_or_else(|| format!("cp: cannot stat '{source_path}': No such file or directory"))?;

        // Check if source is a directory and recursive flag
        if source_node.is_directory() && !recursive {
            return Err(format!(
                "cp: omitting directory '{source_path}': use -r to copy directories"
            ));
        }

        // Determine destination
        let (dest_parent_id, dest_name) =
            self.resolve_destination(vfs, current_dir, dest_path, source_path)?;

        // Perform the copy
        match &source_node.node_type {
            VfsNodeType::File { content } => {
                self.copy_file(vfs, dest_parent_id, &dest_name, content.clone())
            }
            VfsNodeType::Directory => {
                // we already know recursive is true here
                self.copy_directory_recursive(vfs, source_id, dest_parent_id, &dest_name)
            }
            VfsNodeType::Link { .. } => Err(format!(
                "cp: cannot copy '{source_path}': Links not supported"
            )),
        }
    }

    fn resolve_destination(
        &self,
        vfs: &VirtualFilesystem,
        current_dir: NodeId,
        dest_path: &str,
        source_path: &str,
    ) -> Result<(NodeId, String), String> {
        // Try to resolve destination path
        match vfs.resolve_path(current_dir, dest_path) {
            Ok(dest_id) => {
                // Destination exists
                let dest_node = vfs.get_node(dest_id).ok_or_else(|| {
                    format!("cp: cannot access '{dest_path}': No such file or directory")
                })?;

                if dest_node.is_directory() {
                    // Copy into the directory with source filename
                    let source_name = source_path.split('/').next_back().unwrap_or(source_path);
                    Ok((dest_id, source_name.to_string()))
                } else {
                    // Destination is a file - get parent directory and use dest filename
                    let parent_id = vfs.get_parent(dest_id).ok_or_else(|| {
                        format!("cp: cannot access '{dest_path}': No such file or directory")
                    })?;
                    Ok((parent_id, dest_node.name.clone()))
                }
            }
            Err(_) => {
                // Destination doesn't exist - parse as parent/filename
                let (parent_path, filename) = if let Some(pos) = dest_path.rfind('/') {
                    (&dest_path[..pos], &dest_path[pos + 1..])
                } else {
                    ("", dest_path)
                };

                let parent_id = if parent_path.is_empty() {
                    Ok(current_dir)
                } else {
                    vfs.resolve_path(current_dir, parent_path).map_err(|_| {
                        format!("cp: cannot create '{dest_path}': No such file or directory")
                    })
                }?;

                Ok((parent_id, filename.to_string()))
            }
        }
    }

    fn copy_file(
        &self,
        vfs: &mut VirtualFilesystem,
        parent_id: NodeId,
        filename: &str,
        content: FileContent,
    ) -> Result<(), String> {
        vfs.create_file(parent_id, filename, content)
            .map_err(|err| match err {
                VfsError::AlreadyExists => format!("cp: cannot create '{filename}': File exists"),
                VfsError::PermissionDenied => {
                    format!("cp: cannot create '{filename}': Permission denied")
                }
                VfsError::NotADirectory => {
                    format!("cp: cannot create '{filename}': Not a directory")
                }
                _ => format!("cp: cannot create '{filename}': Unknown error"),
            })?;
        Ok(())
    }

    fn copy_directory_recursive(
        &self,
        vfs: &mut VirtualFilesystem,
        source_id: NodeId,
        dest_parent_id: NodeId,
        dest_name: &str,
    ) -> Result<(), String> {
        // Create destination directory
        let dest_dir_id =
            vfs.create_directory(dest_parent_id, dest_name)
                .map_err(|err| match err {
                    VfsError::AlreadyExists => {
                        format!("cp: cannot create directory '{dest_name}': File exists")
                    }
                    VfsError::PermissionDenied => {
                        format!("cp: cannot create directory '{dest_name}': Permission denied")
                    }
                    VfsError::NotADirectory => {
                        format!("cp: cannot create directory '{dest_name}': Not a directory")
                    }
                    _ => format!("cp: cannot create directory '{dest_name}': Unknown error"),
                })?;

        // Copy all entries from source directory
        let entries = vfs
            .list_directory(source_id)
            .map_err(|_| "cp: cannot read directory: Permission denied".to_string())?;

        for entry in entries {
            let child_source_id = entry.node_id;
            let child_node = vfs
                .get_node(child_source_id)
                .ok_or_else(|| "cp: cannot access child: No such file or directory".to_string())?;

            match &child_node.node_type {
                VfsNodeType::File { content } => {
                    self.copy_file(vfs, dest_dir_id, &entry.name, content.clone())?;
                }
                VfsNodeType::Directory => {
                    self.copy_directory_recursive(vfs, child_source_id, dest_dir_id, &entry.name)?;
                }
                VfsNodeType::Link { .. } => {
                    // Skip links for now
                }
            }
        }

        Ok(())
    }
}

pub struct MvCommand;

impl MvCommand {
    pub fn new() -> Self {
        Self
    }
}

impl VfsCommand for MvCommand {
    fn execute(
        &self,
        vfs: &mut VirtualFilesystem,
        current_dir: NodeId,
        args: Vec<&str>,
        _stdin: Option<&str>,
        _is_tty: bool,
    ) -> CommandRes {
        let (_, targets) = parse_multitarget(args);

        if targets.len() < 2 {
            return CommandRes::new()
                .with_error()
                .with_stderr("mv: missing destination file operand");
        }

        let destination = targets.last().unwrap();
        let sources = &targets[..targets.len() - 1];

        let mut stderr_parts = Vec::new();
        let mut has_error = false;

        for source in sources {
            match self.move_item(vfs, current_dir, source, destination) {
                Ok(_) => {}
                Err(err_msg) => {
                    has_error = true;
                    stderr_parts.push(err_msg);
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

impl MvCommand {
    fn move_item(
        &self,
        vfs: &mut VirtualFilesystem,
        current_dir: NodeId,
        source_path: &str,
        dest_path: &str,
    ) -> Result<(), String> {
        // First check if source exists and is not immutable
        let source_id = vfs
            .resolve_path(current_dir, source_path)
            .map_err(|_| format!("mv: cannot stat '{source_path}': No such file or directory"))?;

        let source_node = vfs
            .get_node(source_id)
            .ok_or_else(|| format!("mv: cannot stat '{source_path}': No such file or directory"))?;

        // Check if source is immutable (cannot be moved)
        if source_node.permissions.immutable {
            return Err(format!(
                "mv: cannot move '{source_path}': Permission denied"
            ));
        }

        let is_directory = source_node.is_directory();

        // As per Unix mv documentation:
        // rm -f destination_path && cp -pRP source_file destination && rm -rf source_file

        // Step 1: If destination exists and is a file, try to remove it first
        // (We handle directory destinations differently in cp)
        if let Ok(dest_id) = vfs.resolve_path(current_dir, dest_path) {
            if let Some(dest_node) = vfs.get_node(dest_id) {
                if !dest_node.is_directory() {
                    // Try to remove the destination file (ignore errors for now)
                    let _ = vfs.delete_node(dest_id);
                }
            }
        }

        // Step 2: Copy source to destination (with -r if directory)
        let cp_command = CpCommand::new();
        let cp_args = if is_directory {
            vec!["-r", source_path, dest_path]
        } else {
            vec![source_path, dest_path]
        };

        // Execute the copy
        let cp_result = cp_command.execute(vfs, current_dir, cp_args, None, false);
        if cp_result.is_error() {
            // Extract error message from cp command
            return Err(format!(
                "mv: copy failed: {}",
                match cp_result {
                    CommandRes::Output {
                        stderr_text: Some(ref msg),
                        ..
                    } => msg,
                    _ => "unknown error",
                }
            ));
        }

        // Step 3: Remove the source (with -r if directory)
        let rm_command = RmCommand::new();
        let rm_args = if is_directory {
            vec!["-r", source_path]
        } else {
            vec![source_path]
        };

        // Execute the removal
        let rm_result = rm_command.execute(vfs, current_dir, rm_args, None, false);
        if rm_result.is_error() {
            // The copy succeeded but removal failed - this is still an error
            return Err(format!(
                "mv: cannot remove '{source_path}': Permission denied"
            ));
        }

        Ok(())
    }
}
