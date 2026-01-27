use crate::ui::workspace::{Workspace, WorkflowId};
use std::path::PathBuf;
use std::fs;
use std::time::SystemTime;

#[derive(Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub last_modified: Option<SystemTime>,
    pub is_open: bool,
}

pub struct FileBrowser {
    root_dir: PathBuf,
    workflow_files: Vec<FileEntry>,
    selected_file: Option<PathBuf>,
    filter_text: String,
    needs_refresh: bool,
}

impl FileBrowser {
    pub fn new() -> Self {
        let root_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut browser = Self {
            root_dir: root_dir.clone(),
            workflow_files: Vec::new(),
            selected_file: None,
            filter_text: String::new(),
            needs_refresh: true,
        };
        browser.refresh_files();
        browser
    }

    pub fn with_root(root_dir: PathBuf) -> Self {
        let mut browser = Self {
            root_dir: root_dir.clone(),
            workflow_files: Vec::new(),
            selected_file: None,
            filter_text: String::new(),
            needs_refresh: true,
        };
        browser.refresh_files();
        browser
    }

    /// Scan directory for .json workflow files
    pub fn refresh_files(&mut self) {
        self.workflow_files.clear();

        if let Ok(entries) = fs::read_dir(&self.root_dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                // Only include .json files
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "json" {
                            let name = path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("Unknown")
                                .to_string();

                            let last_modified = entry.metadata().ok().and_then(|m| m.modified().ok());

                            self.workflow_files.push(FileEntry {
                                path: path.clone(),
                                name,
                                last_modified,
                                is_open: false,
                            });
                        }
                    }
                }
            }
        }

        // Sort by last modified (newest first)
        self.workflow_files.sort_by(|a, b| {
            b.last_modified
                .unwrap_or(SystemTime::UNIX_EPOCH)
                .cmp(&a.last_modified.unwrap_or(SystemTime::UNIX_EPOCH))
        });

        self.needs_refresh = false;
    }

    /// Update which files are currently open
    pub fn update_open_state(&mut self, workspace: &Workspace) {
        for file in &mut self.workflow_files {
            file.is_open = workspace.is_file_open(&file.path);
        }
    }

    /// Set the root directory and refresh
    pub fn set_root_dir(&mut self, path: PathBuf) {
        self.root_dir = path;
        self.needs_refresh = true;
        self.refresh_files();
    }

    /// Get the current root directory
    pub fn root_dir(&self) -> &PathBuf {
        &self.root_dir
    }

    /// Render the file browser UI
    pub fn render(&mut self, ui: &mut egui::Ui, workspace: &mut Workspace) -> Option<FileAction> {
        let mut action = None;

        ui.heading("📁 Files");
        ui.separator();

        // Root directory selector
        ui.horizontal(|ui| {
            if ui.button("📂 Open Folder").clicked() {
                // In a real implementation, use a native file dialog
                // For now, just show current directory
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.set_root_dir(path);
                }
            }
        });

        ui.label(format!("📍 {}", self.root_dir.display()));
        ui.separator();

        // Filter/search box
        ui.horizontal(|ui| {
            ui.label("🔍");
            if ui.text_edit_singleline(&mut self.filter_text).changed() {
                // Filter will be applied in the list below
            }
        });

        ui.separator();

        // Refresh if needed
        if self.needs_refresh {
            self.refresh_files();
        }

        // Update open state
        self.update_open_state(workspace);

        // File list
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let filtered_files: Vec<_> = self
                    .workflow_files
                    .iter()
                    .filter(|f| {
                        self.filter_text.is_empty()
                            || f.name.to_lowercase().contains(&self.filter_text.to_lowercase())
                    })
                    .collect();

                if filtered_files.is_empty() {
                    ui.label("No workflow files found");
                } else {
                    for file in filtered_files {
                        ui.horizontal(|ui| {
                            // File icon
                            let icon = if file.is_open { "📄" } else { "📃" };
                            ui.label(icon);

                            // File name (clickable)
                            let mut text = file.name.clone();
                            if file.is_open {
                                text = format!("{} ●", text);
                            }

                            let response = ui.selectable_label(
                                self.selected_file.as_ref() == Some(&file.path),
                                text,
                            );

                            // Single click to select
                            if response.clicked() {
                                self.selected_file = Some(file.path.clone());
                            }

                            // Double click to open
                            if response.double_clicked() {
                                action = Some(FileAction::OpenFile(file.path.clone()));
                            }

                            // Context menu
                            response.context_menu(|ui| {
                                if ui.button("Open").clicked() {
                                    action = Some(FileAction::OpenFile(file.path.clone()));
                                    ui.close_menu();
                                }

                                if ui.button("Delete").clicked() {
                                    action = Some(FileAction::DeleteFile(file.path.clone()));
                                    ui.close_menu();
                                }

                                if ui.button("Rename").clicked() {
                                    action = Some(FileAction::RenameFile(file.path.clone()));
                                    ui.close_menu();
                                }
                            });
                        });
                    }
                }
            });

        ui.separator();

        // New file button at bottom
        if ui.button("➕ New Workflow").clicked() {
            action = Some(FileAction::NewFile);
        }

        action
    }

    /// Mark that files need to be refreshed
    pub fn mark_dirty(&mut self) {
        self.needs_refresh = true;
    }
}

impl Default for FileBrowser {
    fn default() -> Self {
        Self::new()
    }
}

/// Actions that can be triggered from the file browser
#[derive(Debug, Clone)]
pub enum FileAction {
    NewFile,
    OpenFile(PathBuf),
    DeleteFile(PathBuf),
    RenameFile(PathBuf),
}
