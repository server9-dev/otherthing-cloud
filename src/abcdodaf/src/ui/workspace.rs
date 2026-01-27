use crate::ui::enhanced_nodes::EnhancedBpmnNode;
use egui_snarl::Snarl;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub type WorkflowId = usize;

/// Represents a single workflow document with metadata
#[derive(Clone)]
pub struct WorkflowDocument {
    pub id: WorkflowId,
    pub file_path: Option<PathBuf>,
    pub snarl: Snarl<EnhancedBpmnNode>,
    pub is_modified: bool,
    pub validation_errors: Vec<crate::ui::validation::ValidationError>,
    pub name: String,
}

impl WorkflowDocument {
    pub fn new(id: WorkflowId, name: String) -> Self {
        Self {
            id,
            file_path: None,
            snarl: Snarl::new(),
            is_modified: false,
            validation_errors: Vec::new(),
            name,
        }
    }

    pub fn display_name(&self) -> String {
        if let Some(path) = &self.file_path {
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Untitled")
                .to_string()
        } else {
            self.name.clone()
        }
    }

    pub fn display_title(&self) -> String {
        let name = self.display_name();
        if self.is_modified {
            format!("{}*", name)
        } else {
            name
        }
    }
}

/// Serializable format for saving/loading workflows
#[derive(Serialize, Deserialize)]
struct WorkflowFile {
    version: String,
    name: String,
    snarl: Snarl<EnhancedBpmnNode>,
}

/// Manages all open workflows in the IDE
pub struct Workspace {
    workflows: HashMap<WorkflowId, WorkflowDocument>,
    active_workflow: Option<WorkflowId>,
    next_id: usize,
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            workflows: HashMap::new(),
            active_workflow: None,
            next_id: 1,
        }
    }

    /// Create a new empty workflow
    pub fn create_new_workflow(&mut self) -> WorkflowId {
        let id = self.next_id;
        self.next_id += 1;

        let name = format!("Untitled {}", id);
        let doc = WorkflowDocument::new(id, name);
        self.workflows.insert(id, doc);
        self.active_workflow = Some(id);
        id
    }

    /// Open a workflow from a file
    pub fn open_workflow(&mut self, path: PathBuf) -> Result<WorkflowId, String> {
        // Check if already open
        for (id, doc) in &self.workflows {
            if doc.file_path.as_ref() == Some(&path) {
                self.active_workflow = Some(*id);
                return Ok(*id);
            }
        }

        // Load from file
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let workflow_file: WorkflowFile = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        // Create new workflow document
        let id = self.next_id;
        self.next_id += 1;

        let mut doc = WorkflowDocument::new(id, workflow_file.name.clone());
        doc.file_path = Some(path);
        doc.snarl = workflow_file.snarl;
        doc.is_modified = false;

        self.workflows.insert(id, doc);
        self.active_workflow = Some(id);

        Ok(id)
    }

    /// Save a workflow to disk
    pub fn save_workflow(&mut self, id: WorkflowId) -> Result<(), String> {
        let doc = self.workflows.get_mut(&id)
            .ok_or_else(|| "Workflow not found".to_string())?;

        let path = doc.file_path.clone()
            .ok_or_else(|| "No file path set. Use save_workflow_as instead.".to_string())?;

        self.save_workflow_to_path(id, path)
    }

    /// Save a workflow to a specific path
    pub fn save_workflow_as(&mut self, id: WorkflowId, path: PathBuf) -> Result<(), String> {
        self.save_workflow_to_path(id, path)
    }

    fn save_workflow_to_path(&mut self, id: WorkflowId, path: PathBuf) -> Result<(), String> {
        let doc = self.workflows.get_mut(&id)
            .ok_or_else(|| "Workflow not found".to_string())?;

        let workflow_file = WorkflowFile {
            version: "1.0".to_string(),
            name: doc.name.clone(),
            snarl: doc.snarl.clone(),
        };

        let json = serde_json::to_string_pretty(&workflow_file)
            .map_err(|e| format!("Failed to serialize: {}", e))?;

        fs::write(&path, json)
            .map_err(|e| format!("Failed to write file: {}", e))?;

        doc.file_path = Some(path);
        doc.is_modified = false;

        Ok(())
    }

    /// Close a workflow, returns false if cancelled due to unsaved changes
    pub fn close_workflow(&mut self, id: WorkflowId) -> bool {
        if let Some(doc) = self.workflows.get(&id) {
            if doc.is_modified {
                // In a real implementation, we'd show a dialog here
                // For now, just allow closing
                // Return false to indicate "user cancelled"
                // The caller should handle showing the dialog
            }
        }

        self.workflows.remove(&id);

        // Update active workflow
        if self.active_workflow == Some(id) {
            self.active_workflow = self.workflows.keys().next().copied();
        }

        true
    }

    /// Mark a workflow as modified
    pub fn mark_modified(&mut self, id: WorkflowId) {
        if let Some(doc) = self.workflows.get_mut(&id) {
            doc.is_modified = true;
        }
    }

    /// Get the active workflow (immutable)
    pub fn get_active_workflow(&self) -> Option<&WorkflowDocument> {
        self.active_workflow.and_then(|id| self.workflows.get(&id))
    }

    /// Get the active workflow (mutable)
    pub fn get_active_workflow_mut(&mut self) -> Option<&mut WorkflowDocument> {
        self.active_workflow.and_then(|id| self.workflows.get_mut(&id))
    }

    /// Get a specific workflow (immutable)
    pub fn get_workflow(&self, id: WorkflowId) -> Option<&WorkflowDocument> {
        self.workflows.get(&id)
    }

    /// Get a specific workflow (mutable)
    pub fn get_workflow_mut(&mut self, id: WorkflowId) -> Option<&mut WorkflowDocument> {
        self.workflows.get_mut(&id)
    }

    /// Set the active workflow
    pub fn set_active_workflow(&mut self, id: WorkflowId) {
        if self.workflows.contains_key(&id) {
            self.active_workflow = Some(id);
        }
    }

    /// Get all workflow IDs
    pub fn workflow_ids(&self) -> Vec<WorkflowId> {
        self.workflows.keys().copied().collect()
    }

    /// Get the active workflow ID
    pub fn active_workflow_id(&self) -> Option<WorkflowId> {
        self.active_workflow
    }

    /// Get count of open workflows
    pub fn workflow_count(&self) -> usize {
        self.workflows.len()
    }

    /// Check if a file is already open
    pub fn is_file_open(&self, path: &PathBuf) -> bool {
        self.workflows.values().any(|doc| doc.file_path.as_ref() == Some(path))
    }

    /// Get all workflows (for DoDAF aggregation)
    pub fn workflows(&self) -> impl Iterator<Item = &WorkflowDocument> {
        self.workflows.values()
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}
