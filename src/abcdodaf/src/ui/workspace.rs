use crate::bpmn::elements::BpmnDiagram;
use crate::ui::bpmn_json_loader;
use crate::ui::diagram_converter::BpmnDiagramConverter;
use crate::ui::enhanced_nodes::EnhancedBpmnNode;
use egui_snarl::Snarl;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, error, info, warn};

pub type WorkflowId = usize;

/// Represents a single workflow document with metadata
#[derive(Clone)]
pub struct WorkflowDocument {
    pub id: WorkflowId,
    pub file_path: Option<PathBuf>,
    pub diagram: BpmnDiagram,           // Canonical BPMN storage format
    pub snarl: Snarl<EnhancedBpmnNode>, // Visual editor state
    pub is_modified: bool,
    pub validation_errors: Vec<crate::ui::validation::ValidationError>,
    pub name: String,
}

impl WorkflowDocument {
    pub fn new(id: WorkflowId, name: String) -> Self {
        // Create empty diagram
        let diagram = BpmnDiagram {
            id: format!("workflow_{}", id),
            name: Some(name.clone()),
            documentation: None,
            processes: Vec::new(),
            collaborations: Vec::new(),
            data_stores: Vec::new(),
            messages: Vec::new(),
            signals: Vec::new(),
            diagram_info: None,
        };

        Self {
            id,
            file_path: None,
            diagram,
            snarl: Snarl::new(),
            is_modified: false,
            validation_errors: Vec::new(),
            name,
        }
    }

    /// Synchronize diagram from snarl after editing
    ///
    /// Call this before saving to update the canonical BPMN format
    /// with changes made in the visual editor.
    pub fn sync_from_snarl(&mut self) -> Result<(), String> {
        debug!("Syncing diagram from snarl for workflow '{}'", self.name);
        self.diagram = BpmnDiagramConverter::from_snarl(&self.snarl, &self.diagram.id, &self.name)?;
        Ok(())
    }

    pub fn display_name(&self) -> String {
        if let Some(path) = &self.file_path {
            path.file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Untitled".to_string())
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
///
/// Uses BpmnDiagram as the canonical storage format for:
/// - Full BPMN XML import/export compatibility
/// - Runtime engine compatibility
/// - DoDAF metadata preservation
/// - Round-trip integrity with no data loss
#[derive(Serialize, Deserialize)]
struct WorkflowFile {
    version: String,
    diagram: BpmnDiagram,
}

/// File format detection result
#[derive(Debug)]
enum FileFormat {
    /// BpmnDiagram format (version 2.0+)
    BpmnDiagram,
    /// BPMN JSON format with workflow_steps
    BpmnJson,
    /// Serialized Snarl format (legacy)
    SnarlFormat,
    /// XML BPMN format
    Xml,
}

/// Manages all open workflows in the IDE
pub struct Workspace {
    workflows: HashMap<WorkflowId, WorkflowDocument>,
    active_workflow: Option<WorkflowId>,
    next_id: usize,
}

impl Workspace {
    pub fn new() -> Self {
        Self { workflows: HashMap::new(), active_workflow: None, next_id: 1 }
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
    ///
    /// Supports multiple file formats:
    /// - BPMN Diagram format (with "diagram" field)
    /// - BPMN JSON format (with "workflow_steps" field)
    /// - Serialized Snarl format (with "snarl" field)
    /// - XML BPMN format (starts with "<?xml")
    pub fn open_workflow(&mut self, path: PathBuf) -> Result<WorkflowId, String> {
        debug!("Opening workflow from path: {:?}", path);

        // Check if already open
        for (id, doc) in &self.workflows {
            if doc.file_path.as_ref() == Some(&path) {
                debug!("Workflow already open with id: {}", id);
                self.active_workflow = Some(*id);
                return Ok(*id);
            }
        }

        // Load from file
        debug!("Reading file from disk...");
        let content = fs::read_to_string(&path).map_err(|e| {
            error!("Failed to read file {:?}: {}", path, e);
            format!("Failed to read file: {}", e)
        })?;

        debug!("File read successfully, {} bytes", content.len());

        // Detect file format
        let format = Self::detect_file_format(&content)?;
        debug!("Detected file format: {:?}", format);

        // Load based on format
        let (diagram, snarl, name) = match format {
            FileFormat::BpmnDiagram => self.load_bpmn_diagram_format(&content)?,
            FileFormat::BpmnJson => self.load_bpmn_json_format(&content, &path)?,
            FileFormat::SnarlFormat => self.load_snarl_format(&content)?,
            FileFormat::Xml => {
                return Err(
                    "XML BPMN format not yet supported. Please convert to JSON format first."
                        .to_string(),
                );
            },
        };

        // Create new workflow document
        let id = self.next_id;
        self.next_id += 1;

        let mut doc = WorkflowDocument::new(id, name.clone());
        doc.file_path = Some(path);
        doc.diagram = diagram;
        doc.snarl = snarl;
        doc.is_modified = false;

        self.workflows.insert(id, doc);
        self.active_workflow = Some(id);

        info!("Workflow opened successfully: {}", name);
        Ok(id)
    }

    /// Detect the file format from content
    fn detect_file_format(content: &str) -> Result<FileFormat, String> {
        let content_trimmed = content.trim();

        // Check for XML format
        if content_trimmed.starts_with("<?xml") || content_trimmed.starts_with("<bpmn") {
            return Ok(FileFormat::Xml);
        }

        // Try to parse as JSON
        let json_value: serde_json::Value =
            serde_json::from_str(content).map_err(|e| format!("Not a valid JSON file: {}", e))?;

        // Check for BPMN JSON format (has workflow_steps)
        if json_value.get("workflow_steps").is_some() || json_value.get("bpmn_process").is_some() {
            debug!("Detected BPMN JSON format (workflow_steps field present)");
            return Ok(FileFormat::BpmnJson);
        }

        // Check for BpmnDiagram format (has diagram field)
        if json_value.get("diagram").is_some() {
            debug!("Detected BpmnDiagram format (diagram field present)");
            return Ok(FileFormat::BpmnDiagram);
        }

        // Check for Snarl format (has snarl field)
        if json_value.get("snarl").is_some() {
            debug!("Detected Snarl format (snarl field present)");
            return Ok(FileFormat::SnarlFormat);
        }

        Err("Unknown file format. Expected 'diagram', 'workflow_steps', or 'snarl' field."
            .to_string())
    }

    /// Load BpmnDiagram format (version 2.0+)
    fn load_bpmn_diagram_format(
        &self,
        content: &str,
    ) -> Result<(BpmnDiagram, Snarl<EnhancedBpmnNode>, String), String> {
        debug!("Parsing BpmnDiagram format...");
        let workflow_file: WorkflowFile = serde_json::from_str(content).map_err(|e| {
            error!("Failed to parse BpmnDiagram JSON: {}", e);
            format!("Failed to parse BpmnDiagram JSON: {}", e)
        })?;

        let diagram_name =
            workflow_file.diagram.name.clone().unwrap_or_else(|| "Untitled".to_string());
        debug!("BpmnDiagram parsed successfully, workflow name: {}", diagram_name);

        // Convert BpmnDiagram to Snarl for editing
        debug!("Converting BpmnDiagram to Snarl...");
        let snarl = BpmnDiagramConverter::to_snarl(&workflow_file.diagram).map_err(|e| {
            error!("Failed to convert diagram to snarl: {}", e);
            format!("Failed to convert diagram: {}", e)
        })?;

        Ok((workflow_file.diagram, snarl, diagram_name))
    }

    /// Load BPMN JSON format (with workflow_steps)
    fn load_bpmn_json_format(
        &self,
        content: &str,
        path: &PathBuf,
    ) -> Result<(BpmnDiagram, Snarl<EnhancedBpmnNode>, String), String> {
        debug!("Parsing BPMN JSON format (workflow_steps)...");

        // Use BpmnJsonConverter to load and convert to Snarl
        let converter = bpmn_json_loader::BpmnJsonConverter::new();
        let workflow_file = converter.load_from_string(content).map_err(|e| {
            error!("Failed to convert BPMN JSON: {}", e);
            format!("Failed to convert BPMN JSON: {}", e)
        })?;

        info!("BPMN JSON loaded successfully: {}", workflow_file.name);

        // Convert the Snarl back to BpmnDiagram for canonical storage
        debug!("Converting Snarl to BpmnDiagram for canonical storage...");
        let diagram_id = path.file_stem().and_then(|s| s.to_str()).unwrap_or("workflow");

        let diagram =
            BpmnDiagramConverter::from_snarl(&workflow_file.snarl, diagram_id, &workflow_file.name)
                .map_err(|e| {
                    error!("Failed to convert Snarl to BpmnDiagram: {}", e);
                    format!("Failed to convert to BpmnDiagram: {}", e)
                })?;

        Ok((diagram, workflow_file.snarl, workflow_file.name))
    }

    /// Load Snarl format (legacy)
    fn load_snarl_format(
        &self,
        content: &str,
    ) -> Result<(BpmnDiagram, Snarl<EnhancedBpmnNode>, String), String> {
        debug!("Parsing Snarl format (legacy)...");
        warn!("Loading legacy Snarl format. Consider converting to BpmnDiagram format.");

        // Parse as legacy WorkflowFile with Snarl
        #[derive(Deserialize)]
        struct LegacyWorkflowFile {
            #[allow(dead_code)]
            version: String,
            name: String,
            snarl: Snarl<EnhancedBpmnNode>,
        }

        let legacy_file: LegacyWorkflowFile = serde_json::from_str(content).map_err(|e| {
            error!("Failed to parse legacy Snarl format: {}", e);
            format!("Failed to parse Snarl format: {}", e)
        })?;

        debug!("Snarl format parsed successfully, workflow name: {}", legacy_file.name);

        // Convert Snarl to BpmnDiagram
        debug!("Converting Snarl to BpmnDiagram...");
        let diagram = BpmnDiagramConverter::from_snarl(
            &legacy_file.snarl,
            &format!("legacy_{}", self.next_id),
            &legacy_file.name,
        )
        .map_err(|e| {
            error!("Failed to convert Snarl to BpmnDiagram: {}", e);
            format!("Failed to convert to BpmnDiagram: {}", e)
        })?;

        Ok((diagram, legacy_file.snarl, legacy_file.name))
    }

    /// Save a workflow to disk
    pub fn save_workflow(&mut self, id: WorkflowId) -> Result<(), String> {
        let doc = self.workflows.get_mut(&id).ok_or_else(|| "Workflow not found".to_string())?;

        let path = doc
            .file_path
            .clone()
            .ok_or_else(|| "No file path set. Use save_workflow_as instead.".to_string())?;

        self.save_workflow_to_path(id, path)
    }

    /// Save a workflow to a specific path
    pub fn save_workflow_as(&mut self, id: WorkflowId, path: PathBuf) -> Result<(), String> {
        self.save_workflow_to_path(id, path)
    }

    fn save_workflow_to_path(&mut self, id: WorkflowId, path: PathBuf) -> Result<(), String> {
        let doc = self.workflows.get_mut(&id).ok_or_else(|| {
            error!("Attempted to save non-existent workflow with id: {}", id);
            "Workflow not found".to_string()
        })?;

        debug!("Saving workflow {} to {:?}", doc.name, path);

        // Sync diagram from snarl before saving
        debug!("Syncing diagram from snarl changes...");
        doc.sync_from_snarl().map_err(|e| {
            error!("Failed to sync diagram from snarl: {}", e);
            format!("Failed to sync diagram: {}", e)
        })?;

        let workflow_file = WorkflowFile {
            version: "2.0".to_string(), // Updated version for new format
            diagram: doc.diagram.clone(),
        };

        let json = serde_json::to_string_pretty(&workflow_file).map_err(|e| {
            error!("Failed to serialize workflow {}: {}", doc.name, e);
            format!("Failed to serialize: {}", e)
        })?;

        fs::write(&path, json).map_err(|e| {
            error!("Failed to write workflow {} to {:?}: {}", doc.name, path, e);
            format!("Failed to write file: {}", e)
        })?;

        doc.file_path = Some(path.clone());
        doc.is_modified = false;

        info!("Workflow saved successfully to {:?}", path);
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
