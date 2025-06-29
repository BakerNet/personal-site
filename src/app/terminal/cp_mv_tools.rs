use indextree::NodeId;

use super::command::{CommandRes, VfsCommand};
use super::fs_tools::{parse_multitarget, VfsRmCommand};
use super::vfs::{FileContent, VfsError, VfsNodeType, VirtualFilesystem};

// VFS-based CpCommand for Phase 2 migration
pub struct VfsCpCommand;

impl VfsCpCommand {
    pub fn new() -> Self {
        Self
    }
}

impl VfsCommand for VfsCpCommand {
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

impl VfsCpCommand {
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

// VFS-based MvCommand for Phase 2 migration
pub struct VfsMvCommand;

impl VfsMvCommand {
    pub fn new() -> Self {
        Self
    }
}

impl VfsCommand for VfsMvCommand {
    fn execute(
        &self,
        vfs: &mut VirtualFilesystem,
        current_dir: NodeId,
        args: Vec<&str>,
        _stdin: Option<&str>,
        _is_output_tty: bool,
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

impl VfsMvCommand {
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
        let cp_command = VfsCpCommand::new();
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
        let rm_command = VfsRmCommand::new();
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
