#![allow(dead_code)]
use chrono::{DateTime, Local};
use indextree::{Arena, NodeId};

// Re-use the same static file contents from the original VFS
const MINES_SH: &str = r#"#!/bin/bash
set -e

# https://mines.hansbaker.com
# Minesweeper client with multiplayer, replay analysis, and stat tracking
mines
"#;

const THANKS_TXT: &str =
    "Thank you to my wife and my daughter for bringing immense joy to my life.";

const ZSHRC_CONTENT: &str = r#"# Simple zsh configuration
unsetopt beep

# Basic completion
autoload -Uz compinit
compinit

# plugins
plugins = (zsh-autosuggestions, zsh-history-substring-search)

# Aliases
alias ll='ls -la'
alias la='ls -a'
alias h='history'

# robbyrussell theme prompt
# Arrow changes color based on exit status, directory in cyan, git status
PROMPT='%(?:%{$fg_bold[green]%}➜ :%{$fg_bold[red]%}➜ )%{$fg[cyan]%}%c%{$reset_color%} $(git_prompt_info)'

ZSH_THEME_GIT_PROMPT_PREFIX="%{$fg_bold[blue]%}git:(%{$fg[red]%}"
ZSH_THEME_GIT_PROMPT_SUFFIX="%{$reset_color%} "
ZSH_THEME_GIT_PROMPT_DIRTY="%{$fg[blue]%}) %{$fg[yellow]%}✗"
ZSH_THEME_GIT_PROMPT_CLEAN="%{$fg[blue]%})"

# History settings
HISTFILE=window.localStorage["cmd_history"]
HISTSIZE=1000
SAVEHIST=1000
setopt SHARE_HISTORY
setopt APPEND_HISTORY

# zsh-history-substring-search configuration
bindkey '^[[A' history-substring-search-up # or '\eOA'
bindkey '^[[B' history-substring-search-down # or '\eOB'
HISTORY_SUBSTRING_SEARCH_ENSURE_UNIQUE=1
HISTORY_SUBSTRING_SEARCH_HIGHLIGHT_FOUND=0
HISTORY_SUBSTRING_SEARCH_HIGHLIGHT_NOT_FOUND=0
"#;

#[derive(Debug, Clone)]
pub struct VfsNode {
    pub name: String,
    pub node_type: VfsNodeType,
    pub permissions: Permissions,
    pub metadata: NodeMetadata,
}

impl VfsNode {
    pub fn long_meta_string(&self, link_count: usize) -> String {
        let is_directory = matches!(self.node_type, VfsNodeType::Directory);
        let is_executable = self.permissions.execute;
        // Generate permissions string (similar to Unix ls -l format)
        let permissions = format!(
            "{}{}{}{}{}{}{}{}{}{}",
            if is_directory { "d" } else { "-" },
            if self.permissions.read { "r" } else { "-" },
            if self.permissions.write { "w" } else { "-" },
            if is_executable { "x" } else { "-" },
            if self.permissions.read { "r" } else { "-" },
            if self.permissions.write { "w" } else { "-" },
            if is_executable { "x" } else { "-" },
            if self.permissions.read { "r" } else { "-" },
            if self.permissions.write { "w" } else { "-" },
            if is_executable { "x" } else { "-" },
        );
        {
            format!(
                "{} {:2} {:6} {:6} {:>6} ",
                permissions,
                link_count,
                self.metadata.owner,
                self.metadata.group,
                self.metadata.size
            )
        }
    }

    pub fn is_directory(&self) -> bool {
        matches!(self.node_type, VfsNodeType::Directory)
    }

    pub fn is_executable(&self) -> bool {
        self.permissions.execute
    }

    pub fn is_hidden(&self) -> bool {
        self.name.starts_with(".")
    }
}

#[derive(Debug, Clone)]
pub enum VfsNodeType {
    Directory,
    File { content: FileContent },
    Link { target: String },
}

#[derive(Debug, Clone)]
pub enum FileContent {
    Static(&'static str),
    Dynamic(String),
    NavFile(String),
}

#[derive(Debug, Clone)]
pub struct Permissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
    pub immutable: bool,
}

impl Default for Permissions {
    fn default() -> Self {
        Self {
            read: true,
            write: true,
            execute: false,
            immutable: false,
        }
    }
}

impl Permissions {
    pub fn read_only() -> Self {
        Self {
            read: true,
            write: false,
            execute: false,
            immutable: true,
        }
    }

    pub fn executable() -> Self {
        Self {
            read: true,
            write: false,
            execute: true,
            immutable: true,
        }
    }

    pub fn system_dir() -> Self {
        Self {
            read: true,
            write: true, // Allow file creation in system directories
            execute: true,
            immutable: true, // But prevent deletion of the directory itself
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodeMetadata {
    pub size: u64,
    pub owner: String,
    pub group: String,
    pub created: DateTime<Local>,
    pub modified: DateTime<Local>,
}

impl Default for NodeMetadata {
    fn default() -> Self {
        let now = Local::now();
        Self {
            size: 0,
            owner: "hans".to_string(),
            group: "staff".to_string(),
            created: now,
            modified: now,
        }
    }
}

#[derive(Debug, Clone)]
pub enum VfsError {
    NotFound,
    PermissionDenied,
    NotADirectory,
    NotAFile,
    AlreadyExists,
    QuotaExceeded,
    InvalidPath,
    SystemError(String),
}

pub struct VirtualFilesystem {
    arena: Arena<VfsNode>,
    root: NodeId,
}

impl VirtualFilesystem {
    pub fn new(blog_posts: Vec<String>) -> Self {
        let mut arena = Arena::new();

        // Create root directory (writable by users)
        let root_node = VfsNode {
            name: String::new(),
            node_type: VfsNodeType::Directory,
            permissions: Permissions::default(),
            metadata: NodeMetadata {
                size: 4096,
                ..Default::default()
            },
        };

        let root = arena.new_node(root_node);

        let mut vfs = Self { arena, root };

        // Initialize the filesystem structure
        vfs.initialize_system_structure(blog_posts);

        vfs
    }

    fn initialize_system_structure(&mut self, blog_posts: Vec<String>) {
        // Create system directories
        let blog_id = self.create_system_directory("blog").unwrap();
        let cv_id = self.create_system_directory("cv").unwrap();

        // Create system files
        self.create_system_file("mines.sh", FileContent::Static(MINES_SH), true)
            .unwrap();
        self.create_system_file("thanks.txt", FileContent::Static(THANKS_TXT), false)
            .unwrap();
        self.create_system_file(".zshrc", FileContent::Static(ZSHRC_CONTENT), false)
            .unwrap();
        self.create_system_file("nav.rs", FileContent::NavFile("/".to_string()), true)
            .unwrap();

        // Create blog directory files
        self.create_system_file_in(
            "nav.rs",
            FileContent::NavFile("/blog".to_string()),
            true,
            blog_id,
        )
        .unwrap();

        // Create blog post directories
        for post in blog_posts {
            let post_dir_id = self.create_system_directory_in(&post, blog_id).unwrap();
            let post_path = format!("/blog/{post}");
            self.create_system_file_in(
                "nav.rs",
                FileContent::NavFile(post_path),
                true,
                post_dir_id,
            )
            .unwrap();
        }

        // Create cv directory files
        self.create_system_file_in(
            "nav.rs",
            FileContent::NavFile("/cv".to_string()),
            true,
            cv_id,
        )
        .unwrap();
    }

    fn create_system_directory(&mut self, name: &str) -> Result<NodeId, VfsError> {
        let dir_node = VfsNode {
            name: name.to_string(),
            node_type: VfsNodeType::Directory,
            permissions: Permissions::system_dir(),
            metadata: NodeMetadata {
                size: 4096,
                ..Default::default()
            },
        };

        let dir_id = self.arena.new_node(dir_node);
        self.root.append(dir_id, &mut self.arena);
        Ok(dir_id)
    }

    fn create_system_directory_in(
        &mut self,
        name: &str,
        parent: NodeId,
    ) -> Result<NodeId, VfsError> {
        let dir_node = VfsNode {
            name: name.to_string(),
            node_type: VfsNodeType::Directory,
            permissions: Permissions::system_dir(),
            metadata: NodeMetadata {
                size: 4096,
                ..Default::default()
            },
        };

        let dir_id = self.arena.new_node(dir_node);
        parent.append(dir_id, &mut self.arena);
        Ok(dir_id)
    }

    fn create_system_file(
        &mut self,
        name: &str,
        content: FileContent,
        executable: bool,
    ) -> Result<NodeId, VfsError> {
        self.create_system_file_in(name, content, executable, self.root)
    }

    fn create_system_file_in(
        &mut self,
        name: &str,
        content: FileContent,
        executable: bool,
        parent: NodeId,
    ) -> Result<NodeId, VfsError> {
        let size = match &content {
            FileContent::Static(s) => s.len() as u64,
            FileContent::Dynamic(s) => s.len() as u64,
            FileContent::NavFile(_) => 512,
        };

        let permissions = if executable {
            Permissions::executable()
        } else {
            Permissions::read_only()
        };

        let group = if executable { "wheel" } else { "staff" };

        let file_node = VfsNode {
            name: name.to_string(),
            node_type: VfsNodeType::File { content },
            permissions,
            metadata: NodeMetadata {
                size,
                group: group.to_string(),
                ..Default::default()
            },
        };

        let file_id = self.arena.new_node(file_node);
        parent.append(file_id, &mut self.arena);
        Ok(file_id)
    }

    // Path resolution
    pub fn resolve_path(&self, base: NodeId, path: &str) -> Result<NodeId, VfsError> {
        if path.is_empty() || path == "." {
            return Ok(base);
        }

        // Handle ~ expansion
        let expanded_path = if path == "~" {
            return Ok(self.root); // ~ alone means home (root)
        } else if path.starts_with("~/") {
            path.strip_prefix('~').unwrap() // ~/foo becomes /foo
        } else {
            path
        };

        if expanded_path.starts_with('/') {
            let stripped = expanded_path.strip_prefix('/').unwrap();
            // Absolute path
            self.resolve_path_from(self.root, stripped)
        } else {
            // Relative path
            self.resolve_path_from(base, expanded_path)
        }
    }

    fn resolve_path_from(&self, mut current: NodeId, path: &str) -> Result<NodeId, VfsError> {
        if path.is_empty() {
            return Ok(current);
        }

        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        for part in parts {
            match part {
                "." => continue,
                ".." => {
                    // Use indextree's parent functionality!
                    if let Some(parent) = self.arena[current].parent() {
                        current = parent;
                    }
                    // If no parent (at root), stay at root
                }
                name => {
                    // Find child with matching name
                    let mut found = false;
                    for child_id in current.children(&self.arena) {
                        if let Some(child_node) = self.arena.get(child_id) {
                            if child_node.get().name == name {
                                current = child_id;
                                found = true;
                                break;
                            }
                        }
                    }
                    if !found {
                        return Err(VfsError::NotFound);
                    }
                }
            }
        }

        Ok(current)
    }

    // CRUD operations
    pub fn create_file(
        &mut self,
        parent: NodeId,
        name: &str,
        content: FileContent,
    ) -> Result<NodeId, VfsError> {
        // Validate parent is a directory
        let parent_node = self.arena.get(parent).ok_or(VfsError::NotFound)?;

        // Check permissions
        if !parent_node.get().permissions.write {
            return Err(VfsError::PermissionDenied);
        }

        // Ensure it's a directory
        if !matches!(parent_node.get().node_type, VfsNodeType::Directory) {
            return Err(VfsError::NotADirectory);
        }

        // Check if name already exists
        for child_id in parent.children(&self.arena) {
            if let Some(child_node) = self.arena.get(child_id) {
                if child_node.get().name == name {
                    return Err(VfsError::AlreadyExists);
                }
            }
        }

        // Calculate file size
        let size = match &content {
            FileContent::Static(s) => s.len() as u64,
            FileContent::Dynamic(s) => s.len() as u64,
            FileContent::NavFile(_) => 512,
        };

        // Create the file node
        let file_node = VfsNode {
            name: name.to_string(),
            node_type: VfsNodeType::File { content },
            permissions: Permissions::default(),
            metadata: NodeMetadata {
                size,
                owner: "user".to_string(),
                group: "user".to_string(),
                ..Default::default()
            },
        };

        let file_id = self.arena.new_node(file_node);
        parent.append(file_id, &mut self.arena);

        Ok(file_id)
    }

    pub fn create_directory(&mut self, parent: NodeId, name: &str) -> Result<NodeId, VfsError> {
        // Validate parent is a directory
        let parent_node = self.arena.get(parent).ok_or(VfsError::NotFound)?;

        // Check permissions
        if !parent_node.get().permissions.write {
            return Err(VfsError::PermissionDenied);
        }

        // Ensure it's a directory
        if !matches!(parent_node.get().node_type, VfsNodeType::Directory) {
            return Err(VfsError::NotADirectory);
        }

        // Check if name already exists
        for child_id in parent.children(&self.arena) {
            if let Some(child_node) = self.arena.get(child_id) {
                if child_node.get().name == name {
                    return Err(VfsError::AlreadyExists);
                }
            }
        }

        // Create the directory node
        let dir_node = VfsNode {
            name: name.to_string(),
            node_type: VfsNodeType::Directory,
            permissions: Permissions::default(),
            metadata: NodeMetadata {
                size: 4096,
                owner: "user".to_string(),
                group: "user".to_string(),
                ..Default::default()
            },
        };

        let dir_id = self.arena.new_node(dir_node);
        parent.append(dir_id, &mut self.arena);

        Ok(dir_id)
    }

    pub fn read_file(&self, node: NodeId) -> Result<String, VfsError> {
        let node_ref = self.arena.get(node).ok_or(VfsError::NotFound)?;
        let node_data = node_ref.get();

        // Check read permission
        if !node_data.permissions.read {
            return Err(VfsError::PermissionDenied);
        }

        match &node_data.node_type {
            VfsNodeType::File { content } => match content {
                FileContent::Static(s) => Ok(s.to_string()),
                FileContent::Dynamic(s) => Ok(s.clone()),
                FileContent::NavFile(path) => Ok(generate_nav_content(path)),
            },
            VfsNodeType::Directory => Err(VfsError::NotAFile),
            VfsNodeType::Link { target } => {
                // Follow the link and read the target
                let target_node = self.resolve_path(self.root, target)?;
                self.read_file(target_node)
            }
        }
    }

    pub fn list_directory(&self, node: NodeId) -> Result<Vec<DirEntry>, VfsError> {
        let node_ref = self.arena.get(node).ok_or(VfsError::NotFound)?;
        let node_data = node_ref.get();

        // Check read permission
        if !node_data.permissions.read {
            return Err(VfsError::PermissionDenied);
        }

        match &node_data.node_type {
            VfsNodeType::Directory => {
                let mut entries = Vec::new();

                for child_id in node.children(&self.arena) {
                    if let Some(child_ref) = self.arena.get(child_id) {
                        let child_data = child_ref.get();
                        entries.push(DirEntry {
                            name: child_data.name.clone(),
                            node_id: child_id,
                            is_directory: matches!(child_data.node_type, VfsNodeType::Directory),
                            is_executable: child_data.permissions.execute,
                        });
                    }
                }

                entries.sort_by(|a, b| a.name.cmp(&b.name));
                Ok(entries)
            }
            VfsNodeType::File { .. } => Err(VfsError::NotADirectory),
            VfsNodeType::Link { target } => {
                // Follow the link and list the target
                let target_node = self.resolve_path(self.root, target)?;
                self.list_directory(target_node)
            }
        }
    }

    pub fn delete_node(&mut self, node: NodeId) -> Result<(), VfsError> {
        // Can't delete root
        if node == self.root {
            return Err(VfsError::PermissionDenied);
        }

        // Check if node exists and get its info
        let node_ref = self.arena.get(node).ok_or(VfsError::NotFound)?;
        let node_data = node_ref.get();

        // Check permissions
        if node_data.permissions.immutable {
            return Err(VfsError::PermissionDenied);
        }

        // If it's a directory, ensure it's empty
        if matches!(node_data.node_type, VfsNodeType::Directory)
            && node.children(&self.arena).next().is_some()
        {
            return Err(VfsError::SystemError("Directory not empty".to_string()));
        }

        // Remove the node from the tree (indextree handles parent cleanup!)
        node.remove(&mut self.arena);

        Ok(())
    }

    pub fn delete_node_recursive(&mut self, node: NodeId) -> Result<(), VfsError> {
        // Can't delete root
        if node == self.root {
            return Err(VfsError::PermissionDenied);
        }

        // Check if node exists and get its info
        let node_ref = self.arena.get(node).ok_or(VfsError::NotFound)?;
        let node_data = node_ref.get();

        // Check permissions
        if node_data.permissions.immutable {
            return Err(VfsError::PermissionDenied);
        }

        // If it's a directory, recursively delete children first
        if matches!(node_data.node_type, VfsNodeType::Directory) {
            // Collect children to avoid borrowing issues
            let children: Vec<NodeId> = node.children(&self.arena).collect();

            for child_id in children {
                self.delete_node_recursive(child_id)?;
            }
        }

        // Remove the node from the tree (indextree handles parent cleanup!)
        node.remove(&mut self.arena);

        Ok(())
    }

    // Get the full path of a node - MUCH simpler with indextree!
    pub fn get_node_path(&self, node: NodeId) -> String {
        if node == self.root {
            return "/".to_string();
        }

        // Collect ancestors (excluding root)
        let mut path_parts = Vec::new();
        let mut current = node;

        while let Some(parent) = self.arena[current].parent() {
            if let Some(node_ref) = self.arena.get(current) {
                let node_data = node_ref.get();
                if !node_data.name.is_empty() {
                    // Skip root (empty name)
                    path_parts.push(node_data.name.clone());
                }
            }
            current = parent;
        }

        // Reverse to get path from root to node
        path_parts.reverse();

        if path_parts.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", path_parts.join("/"))
        }
    }

    pub fn get_node(&self, node: NodeId) -> Option<&VfsNode> {
        self.arena.get(node).map(|node_ref| node_ref.get())
    }

    pub fn get_root(&self) -> NodeId {
        self.root
    }

    pub fn get_parent(&self, node: NodeId) -> Option<NodeId> {
        self.arena[node].parent()
    }
}

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub node_id: NodeId,
    pub is_directory: bool,
    pub is_executable: bool,
}

// Helper function to create nav.rs content
fn generate_nav_content(path: &str) -> String {
    let path = if path.is_empty() { "/" } else { path };
    format!(
        r#"use leptos::prelude::*;
use leptos_router::{{hooks::use_navigate, UseNavigateOptions}};

func main() {{
    Effect::new((_) => {{
        let navigate = use_navigate();
        navigate("{path}", UseNavigateOptions::default);
    }})
}}
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vfs_initialization() {
        let blog_posts = vec!["post1".to_string(), "post2".to_string()];
        let vfs = VirtualFilesystem::new(blog_posts);

        // Test root directory exists
        let root = vfs.get_root();
        let root_node = vfs.get_node(root).expect("Root node should exist");
        assert!(matches!(root_node.node_type, VfsNodeType::Directory));

        // Test basic system directories exist
        assert!(vfs.resolve_path(root, "/blog").is_ok());
        assert!(vfs.resolve_path(root, "/cv").is_ok());

        // Test system files exist
        assert!(vfs.resolve_path(root, "/mines.sh").is_ok());
        assert!(vfs.resolve_path(root, "/thanks.txt").is_ok());
        assert!(vfs.resolve_path(root, "/.zshrc").is_ok());
        assert!(vfs.resolve_path(root, "/nav.rs").is_ok());
    }

    #[test]
    fn test_path_resolution() {
        let blog_posts = vec!["example-post".to_string()];
        let vfs = VirtualFilesystem::new(blog_posts);
        let root = vfs.get_root();

        // Test absolute paths
        assert!(vfs.resolve_path(root, "/").is_ok());
        assert!(vfs.resolve_path(root, "/blog").is_ok());
        assert!(vfs.resolve_path(root, "/blog/example-post").is_ok());

        // Test relative paths from root
        assert!(vfs.resolve_path(root, "blog").is_ok());
        assert!(vfs.resolve_path(root, "./blog").is_ok());

        // Test parent directory navigation
        let blog_id = vfs.resolve_path(root, "/blog").unwrap();
        let back_to_root = vfs.resolve_path(blog_id, "..").unwrap();
        assert_eq!(back_to_root, root);

        // Test invalid paths
        assert!(vfs.resolve_path(root, "/nonexistent").is_err());
        assert!(vfs.resolve_path(root, "/blog/nonexistent").is_err());
    }

    #[test]
    fn test_file_reading() {
        let blog_posts = vec![];
        let vfs = VirtualFilesystem::new(blog_posts);
        let root = vfs.get_root();

        // Test reading static files
        let mines_id = vfs.resolve_path(root, "/mines.sh").unwrap();
        let content = vfs.read_file(mines_id).unwrap();
        assert!(content.contains("mines"));

        let thanks_id = vfs.resolve_path(root, "/thanks.txt").unwrap();
        let content = vfs.read_file(thanks_id).unwrap();
        assert!(content.contains("Thank you"));

        // Test reading nav file (virtual content)
        let nav_id = vfs.resolve_path(root, "/nav.rs").unwrap();
        let content = vfs.read_file(nav_id).unwrap();
        assert!(content.contains("use_navigate"));
        assert!(content.contains("\"/\""));
    }

    #[test]
    fn test_directory_listing() {
        let blog_posts = vec!["post1".to_string(), "post2".to_string()];
        let vfs = VirtualFilesystem::new(blog_posts);
        let root = vfs.get_root();

        // Test root directory listing
        let entries = vfs.list_directory(root).unwrap();
        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();

        assert!(names.contains(&"blog"));
        assert!(names.contains(&"cv"));
        assert!(names.contains(&"mines.sh"));
        assert!(names.contains(&"thanks.txt"));
        assert!(names.contains(&".zshrc"));
        assert!(names.contains(&"nav.rs"));

        // Test blog directory listing
        let blog_id = vfs.resolve_path(root, "/blog").unwrap();
        let blog_entries = vfs.list_directory(blog_id).unwrap();
        let blog_names: Vec<&str> = blog_entries.iter().map(|e| e.name.as_str()).collect();

        assert!(blog_names.contains(&"post1"));
        assert!(blog_names.contains(&"post2"));
        assert!(blog_names.contains(&"nav.rs"));
    }

    #[test]
    fn test_user_file_creation() {
        let blog_posts = vec![];
        let mut vfs = VirtualFilesystem::new(blog_posts);
        let root = vfs.get_root();

        // Create a user file
        let file_id = vfs
            .create_file(
                root,
                "test.txt",
                FileContent::Dynamic("Hello, world!".to_string()),
            )
            .unwrap();

        // Verify it was created
        let content = vfs.read_file(file_id).unwrap();
        assert_eq!(content, "Hello, world!");

        // Verify it appears in directory listing
        let entries = vfs.list_directory(root).unwrap();
        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"test.txt"));
    }

    #[test]
    fn test_user_directory_creation() {
        let blog_posts = vec![];
        let mut vfs = VirtualFilesystem::new(blog_posts);
        let root = vfs.get_root();

        // Create a user directory
        let dir_id = vfs.create_directory(root, "mydir").unwrap();

        // Verify it was created and is a directory
        let node = vfs.get_node(dir_id).unwrap();
        assert!(matches!(node.node_type, VfsNodeType::Directory));

        // Verify it appears in directory listing
        let entries = vfs.list_directory(root).unwrap();
        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"mydir"));

        // Create a file inside the directory
        vfs.create_file(
            dir_id,
            "nested.txt",
            FileContent::Dynamic("nested content".to_string()),
        )
        .unwrap();

        // Verify the nested file can be accessed
        let nested_id = vfs.resolve_path(root, "/mydir/nested.txt").unwrap();
        let content = vfs.read_file(nested_id).unwrap();
        assert_eq!(content, "nested content");
    }

    #[test]
    fn test_system_file_immutability() {
        let blog_posts = vec![];
        let mut vfs = VirtualFilesystem::new(blog_posts);
        let root = vfs.get_root();

        // We can now create files in system directories (they have write permission)
        let blog_id = vfs.resolve_path(root, "/blog").unwrap();
        let result = vfs.create_file(
            blog_id,
            "user_content.txt",
            FileContent::Dynamic("user created content".to_string()),
        );

        // File creation should succeed
        assert!(result.is_ok());
        let _file_id = result.unwrap();

        // But we still can't delete system directories
        let delete_result = vfs.delete_node(blog_id);
        assert!(delete_result.is_err());
        assert!(matches!(
            delete_result.unwrap_err(),
            VfsError::PermissionDenied
        ));

        // System files (like mines.sh) still can't be deleted
        let mines_id = vfs.resolve_path(root, "mines.sh").unwrap();
        let delete_mines = vfs.delete_node(mines_id);
        assert!(delete_mines.is_err());
        assert!(matches!(
            delete_mines.unwrap_err(),
            VfsError::PermissionDenied
        ));
    }

    #[test]
    fn test_file_deletion() {
        let blog_posts = vec![];
        let mut vfs = VirtualFilesystem::new(blog_posts);
        let root = vfs.get_root();

        // Create a user file
        let file_id = vfs
            .create_file(
                root,
                "deleteme.txt",
                FileContent::Dynamic("temporary content".to_string()),
            )
            .unwrap();

        // Verify it exists
        assert!(vfs.read_file(file_id).is_ok());

        // Delete it
        vfs.delete_node(file_id).unwrap();

        // Verify it's gone
        assert!(vfs.resolve_path(root, "/deleteme.txt").is_err());
    }

    #[test]
    fn test_directory_deletion() {
        let blog_posts = vec![];
        let mut vfs = VirtualFilesystem::new(blog_posts);
        let root = vfs.get_root();

        // Create a user directory
        let dir_id = vfs.create_directory(root, "tempdir").unwrap();

        // Verify it exists
        assert!(vfs.resolve_path(root, "/tempdir").is_ok());

        // Delete it
        vfs.delete_node(dir_id).unwrap();

        // Verify it's gone
        assert!(vfs.resolve_path(root, "/tempdir").is_err());
    }

    #[test]
    fn test_cannot_delete_non_empty_directory() {
        let blog_posts = vec![];
        let mut vfs = VirtualFilesystem::new(blog_posts);
        let root = vfs.get_root();

        // Create a directory with a file
        let dir_id = vfs.create_directory(root, "nonempty").unwrap();
        vfs.create_file(
            dir_id,
            "file.txt",
            FileContent::Dynamic("content".to_string()),
        )
        .unwrap();

        // Try to delete the directory (should fail)
        let result = vfs.delete_node(dir_id);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VfsError::SystemError(_)));
    }

    #[test]
    fn test_cannot_delete_system_files() {
        let blog_posts = vec![];
        let mut vfs = VirtualFilesystem::new(blog_posts);
        let root = vfs.get_root();

        // Try to delete a system file
        let mines_id = vfs.resolve_path(root, "/mines.sh").unwrap();
        let result = vfs.delete_node(mines_id);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VfsError::PermissionDenied));
    }

    #[test]
    fn test_cannot_delete_root() {
        let blog_posts = vec![];
        let mut vfs = VirtualFilesystem::new(blog_posts);
        let root = vfs.get_root();

        // Try to delete root
        let result = vfs.delete_node(root);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VfsError::PermissionDenied));
    }

    #[test]
    fn test_duplicate_name_prevention() {
        let blog_posts = vec![];
        let mut vfs = VirtualFilesystem::new(blog_posts);
        let root = vfs.get_root();

        // Create a file
        vfs.create_file(
            root,
            "duplicate.txt",
            FileContent::Dynamic("first".to_string()),
        )
        .unwrap();

        // Try to create another file with the same name
        let result = vfs.create_file(
            root,
            "duplicate.txt",
            FileContent::Dynamic("second".to_string()),
        );
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VfsError::AlreadyExists));

        // Same for directories
        vfs.create_directory(root, "dupdir").unwrap();
        let result = vfs.create_directory(root, "dupdir");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VfsError::AlreadyExists));
    }

    #[test]
    fn test_path_generation() {
        let blog_posts = vec!["example-post".to_string()];
        let vfs = VirtualFilesystem::new(blog_posts);
        let root = vfs.get_root();

        // Test root path
        assert_eq!(vfs.get_node_path(root), "/");

        // Test simple paths
        let blog_id = vfs.resolve_path(root, "/blog").unwrap();
        assert_eq!(vfs.get_node_path(blog_id), "/blog");

        let cv_id = vfs.resolve_path(root, "/cv").unwrap();
        assert_eq!(vfs.get_node_path(cv_id), "/cv");

        // Test nested paths
        let post_id = vfs.resolve_path(root, "/blog/example-post").unwrap();
        assert_eq!(vfs.get_node_path(post_id), "/blog/example-post");
    }

    #[test]
    fn test_complex_path_navigation() {
        let blog_posts = vec!["post1".to_string()];
        let vfs = VirtualFilesystem::new(blog_posts);
        let root = vfs.get_root();

        // Navigate to deep path then back up
        let post_id = vfs.resolve_path(root, "/blog/post1").unwrap();
        let back_to_blog = vfs.resolve_path(post_id, "..").unwrap();
        let back_to_root = vfs.resolve_path(back_to_blog, "..").unwrap();

        assert_eq!(back_to_root, root);
        assert_eq!(vfs.get_node_path(back_to_blog), "/blog");

        // Test relative path navigation - go to blog dir, then up and over to cv
        let blog_id = vfs.resolve_path(root, "/blog").unwrap();
        let cv_via_relative = vfs.resolve_path(blog_id, "../cv").unwrap();
        assert_eq!(vfs.get_node_path(cv_via_relative), "/cv");
    }

    #[test]
    fn test_empty_and_dot_paths() {
        let blog_posts = vec![];
        let vfs = VirtualFilesystem::new(blog_posts);
        let root = vfs.get_root();

        // Test empty path resolves to base
        assert_eq!(vfs.resolve_path(root, "").unwrap(), root);

        // Test dot path resolves to base
        assert_eq!(vfs.resolve_path(root, ".").unwrap(), root);

        // Test from subdirectory
        let blog_id = vfs.resolve_path(root, "/blog").unwrap();
        assert_eq!(vfs.resolve_path(blog_id, "").unwrap(), blog_id);
        assert_eq!(vfs.resolve_path(blog_id, ".").unwrap(), blog_id);
    }

    #[test]
    fn test_complex_relative_paths() {
        let blog_posts = vec!["post1".to_string(), "post2".to_string()];
        let mut vfs = VirtualFilesystem::new(blog_posts);
        let root = vfs.get_root();

        // Create nested structure for testing
        let user_dir = vfs.create_directory(root, "user").unwrap();
        let projects_dir = vfs.create_directory(user_dir, "projects").unwrap();
        let deep_dir = vfs.create_directory(projects_dir, "deep").unwrap();

        // Test multiple .. segments
        let back_to_root = vfs.resolve_path(deep_dir, "../../..").unwrap();
        assert_eq!(back_to_root, root);

        // Test multiple .. with final directory
        let back_to_user = vfs.resolve_path(deep_dir, "../..").unwrap();
        assert_eq!(back_to_user, user_dir);

        // Test .. beyond root (should stay at root)
        let still_root = vfs.resolve_path(root, "../../..").unwrap();
        assert_eq!(still_root, root);

        // Test complex path with mixed . and ..
        let mixed_path = vfs
            .resolve_path(deep_dir, "./../../../user/./projects")
            .unwrap();
        assert_eq!(mixed_path, projects_dir);

        // Test relative path from deep location to system directory
        let to_blog = vfs.resolve_path(deep_dir, "../../../blog").unwrap();
        let blog_id = vfs.resolve_path(root, "/blog").unwrap();
        assert_eq!(to_blog, blog_id);

        // Test relative path with intermediate directory traversal
        let complex_nav = vfs
            .resolve_path(deep_dir, "../../projects/../projects/deep")
            .unwrap();
        assert_eq!(complex_nav, deep_dir);
    }

    #[test]
    fn test_relative_paths_with_files() {
        let blog_posts = vec!["example-post".to_string()];
        let mut vfs = VirtualFilesystem::new(blog_posts);
        let root = vfs.get_root();

        // Create test structure
        let test_dir = vfs.create_directory(root, "test").unwrap();
        let sub_dir = vfs.create_directory(test_dir, "sub").unwrap();
        vfs.create_file(
            sub_dir,
            "file.txt",
            FileContent::Dynamic("test content".to_string()),
        )
        .unwrap();

        // Navigate to file via complex relative path
        let file_via_complex = vfs
            .resolve_path(root, "test/./sub/../sub/./file.txt")
            .unwrap();
        let content = vfs.read_file(file_via_complex).unwrap();
        assert_eq!(content, "test content");

        // Navigate from file back to root via complex path
        let back_to_root = vfs.resolve_path(file_via_complex, "../../..").unwrap();
        assert_eq!(back_to_root, root);

        // Navigate from file to system file via complex relative path
        let to_mines = vfs
            .resolve_path(file_via_complex, "../../.././mines.sh")
            .unwrap();
        let mines_content = vfs.read_file(to_mines).unwrap();
        assert!(mines_content.contains("mines"));
    }

    #[test]
    fn test_paths_with_redundant_separators() {
        let blog_posts = vec!["post1".to_string()];
        let vfs = VirtualFilesystem::new(blog_posts);
        let root = vfs.get_root();

        // Test paths with multiple consecutive dots and slashes
        let blog_id = vfs.resolve_path(root, "/blog").unwrap();

        // These should all resolve to the same location
        let path1 = vfs.resolve_path(blog_id, "../././blog/./post1").unwrap();
        let path2 = vfs.resolve_path(blog_id, ".././blog/post1").unwrap();
        let path3 = vfs
            .resolve_path(blog_id, "././../blog/./././post1")
            .unwrap();
        let direct_path = vfs.resolve_path(root, "/blog/post1").unwrap();

        assert_eq!(path1, direct_path);
        assert_eq!(path2, direct_path);
        assert_eq!(path3, direct_path);
    }

    #[test]
    fn test_relative_navigation_from_deep_blog_posts() {
        let blog_posts = vec!["deep-post".to_string(), "other-post".to_string()];
        let vfs = VirtualFilesystem::new(blog_posts);
        let root = vfs.get_root();

        // Verify both blog posts exist
        let deep_post = vfs.resolve_path(root, "/blog/deep-post").unwrap();
        let other_post = vfs.resolve_path(root, "/blog/other-post").unwrap();
        let nav_file = vfs.resolve_path(deep_post, "nav.rs").unwrap();

        // Navigate to CV from deep blog location using nav.rs as starting point
        // nav.rs -> .. (to post dir) -> .. (to blog dir) -> .. (to root) -> cv
        let cv_via_relative = vfs.resolve_path(nav_file, "../../../cv").unwrap();
        let cv_direct = vfs.resolve_path(root, "/cv").unwrap();
        assert_eq!(cv_via_relative, cv_direct);

        // Navigate from blog directory to other post (not from nav.rs file)
        let blog_dir = vfs.resolve_path(root, "/blog").unwrap();
        let other_via_relative = vfs.resolve_path(blog_dir, "other-post").unwrap();
        assert_eq!(other_via_relative, other_post);

        // Complex navigation: blog post -> root -> back to same blog post
        let roundtrip = vfs
            .resolve_path(nav_file, "../../.././blog/deep-post")
            .unwrap();
        assert_eq!(roundtrip, deep_post);
    }

    #[test]
    fn test_edge_case_relative_paths() {
        let blog_posts = vec![];
        let mut vfs = VirtualFilesystem::new(blog_posts);
        let root = vfs.get_root();

        // Create nested structure for edge case testing
        let a = vfs.create_directory(root, "a").unwrap();
        let b = vfs.create_directory(a, "b").unwrap();
        let c = vfs.create_directory(b, "c").unwrap();

        // Test many consecutive dots and parent traversals
        let many_dots = vfs.resolve_path(c, "./././././.").unwrap();
        assert_eq!(many_dots, c);

        // Test excessive parent traversal (should clamp at root)
        let excessive_parents = vfs.resolve_path(c, "../../../../../../../..").unwrap();
        assert_eq!(excessive_parents, root);

        // Test mixed excessive traversal with final valid path
        let mixed_excessive = vfs.resolve_path(c, "../../../../../../../../blog").unwrap();
        let blog_direct = vfs.resolve_path(root, "/blog").unwrap();
        assert_eq!(mixed_excessive, blog_direct);

        // Test path that goes up and down multiple times
        let zigzag = vfs.resolve_path(c, "../../../a/b/../b/c/../c").unwrap();
        assert_eq!(zigzag, c);
    }
}
