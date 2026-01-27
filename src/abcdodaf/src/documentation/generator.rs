//! Documentation generator for BPMN processes and workflows

use super::{ProcessDocumentation, Parameter, Participant, ErrorScenario};
use crate::bpmn::ProcessInstance;
use crate::bpmn::process::Process;
#[cfg(test)]
use crate::bpmn::process::ProcessBuilder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Documentation format types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentationFormat {
    /// Markdown format
    Markdown,
    /// HTML format
    Html,
    /// PlainText format
    PlainText,
    /// JSON format
    Json,
}

/// Documentation generation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationConfig {
    /// Include flow diagram
    pub include_diagram: bool,
    /// Include error handling documentation
    pub include_error_handling: bool,
    /// Include participant information
    pub include_participants: bool,
    /// Include input/output parameter documentation
    pub include_parameters: bool,
    /// Include DoDAF alignment
    pub include_dodaf_alignment: bool,
    /// Documentation format
    pub format: DocumentationFormat,
    /// Custom title
    pub title: Option<String>,
    /// Custom author
    pub author: Option<String>,
    /// Custom version
    pub version: Option<String>,
}

impl Default for DocumentationConfig {
    fn default() -> Self {
        Self {
            include_diagram: true,
            include_error_handling: true,
            include_participants: true,
            include_parameters: true,
            include_dodaf_alignment: true,
            format: DocumentationFormat::Markdown,
            title: None,
            author: None,
            version: None,
        }
    }
}

/// Main documentation generator
pub struct DocumentationGenerator {
    #[allow(dead_code)]
    config: DocumentationConfig,
}

impl DocumentationGenerator {
    /// Create a new documentation generator
    pub fn new(config: DocumentationConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self {
            config: DocumentationConfig::default(),
        }
    }

    /// Generate documentation from a process
    pub fn generate_from_process(
        &self,
        process: &Process,
    ) -> Result<ProcessDocumentation, String> {
        Ok(ProcessDocumentation {
            process_id: process.id.clone(),
            name: process.name.clone(),
            description: self.extract_process_description(process),
            purpose: self.extract_process_purpose(process),
            inputs: self.extract_inputs(process),
            outputs: self.extract_outputs(process),
            participants: self.extract_participants(process),
            flow_description: self.generate_flow_description(process),
            error_handling: self.extract_error_handling(process),
            dodaf_views: self.extract_dodaf_views(process),
            metadata: self.extract_metadata(process),
            generated_at: chrono::Utc::now(),
        })
    }

    /// Generate documentation from a process instance
    pub fn generate_from_instance(
        &self,
        instance: &ProcessInstance,
    ) -> Result<ProcessDocumentation, String> {
        // Extract execution metadata from instance
        let doc = ProcessDocumentation {
            process_id: instance.process_id.clone(),
            name: format!("Execution: {}", instance.process_id),
            description: format!("Process instance execution documentation for {}",
                instance.process_id),
            purpose: Some(format!("Instance {} executed at {}",
                instance.id, instance.started_at)),
            inputs: self.extract_instance_inputs(instance),
            outputs: self.extract_instance_outputs(instance),
            participants: vec![],
            flow_description: format!("Process state: {:?}", instance.state),
            error_handling: vec![],
            dodaf_views: vec![],
            metadata: self.extract_instance_metadata(instance),
            generated_at: chrono::Utc::now(),
        };

        Ok(doc)
    }

    // Helper methods

    fn extract_process_description(&self, process: &Process) -> String {
        process.description.clone().unwrap_or_else(|| {
            format!("Process: {}", process.name)
        })
    }

    fn extract_process_purpose(&self, _process: &Process) -> Option<String> {
        None // To be populated from process metadata
    }

    fn extract_inputs(&self, process: &Process) -> Vec<Parameter> {
        // Extract inputs from process tasks and metadata
        let mut inputs = Vec::new();

        // Add inputs from first task if available
        if let Some(first_task) = process.tasks.first() {
            inputs.push(Parameter {
                name: format!("{}_input", first_task.id),
                param_type: "any".to_string(),
                description: format!("Input for {}", first_task.name),
                required: true,
                default: None,
                constraints: None,
            });
        }

        inputs
    }

    fn extract_outputs(&self, process: &Process) -> Vec<Parameter> {
        // Extract outputs from process tasks and metadata
        let mut outputs = Vec::new();

        // Add outputs from last task if available
        if let Some(last_task) = process.tasks.last() {
            outputs.push(Parameter {
                name: format!("{}_output", last_task.id),
                param_type: "any".to_string(),
                description: format!("Output from {}", last_task.name),
                required: false,
                default: None,
                constraints: None,
            });
        }

        outputs
    }

    fn extract_participants(&self, _process: &Process) -> Vec<Participant> {
        // Extract participants from lanes or process metadata
        // TODO: implement based on BPMN lane definitions
        vec![]
    }

    fn generate_flow_description(&self, process: &Process) -> String {
        let mut flow = String::new();
        flow.push_str("Process Flow:\n");

        if !process.tasks.is_empty() {
            let task_names: Vec<String> = process.tasks.iter().map(|t| t.name.clone()).collect();
            flow.push_str(&format!("- Tasks: {}\n", task_names.join(" → ")));
        }

        if !process.flows.is_empty() {
            flow.push_str(&format!("- Connections: {} flows defined\n", process.flows.len()));
        }

        if !process.gateways.is_empty() {
            flow.push_str(&format!("- Decision Points: {} gateways\n", process.gateways.len()));
        }

        flow
    }

    fn extract_error_handling(&self, _process: &Process) -> Vec<ErrorScenario> {
        // Extract error handling from process event handlers
        vec![]
    }

    fn extract_dodaf_views(&self, process: &Process) -> Vec<String> {
        // Map BPMN elements to DoDAF views
        let mut views = vec!["OV-6c - Systems Interface Description".to_string()];

        if !process.tasks.is_empty() {
            views.push("OV-1 - High-Level Operational Concept Graphic".to_string());
            views.push("OV-5b - Operational Activity Model".to_string());
        }

        if !process.gateways.is_empty() {
            views.push("OV-6a - All Processes and Interactions".to_string());
        }

        views
    }

    fn extract_metadata(&self, process: &Process) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("process_id".to_string(), process.id.clone());
        metadata.insert("process_name".to_string(), process.name.clone());
        if let Some(ref desc) = process.description {
            metadata.insert("description".to_string(), desc.clone());
        }
        metadata
    }

    fn extract_instance_inputs(&self, instance: &ProcessInstance) -> Vec<Parameter> {
        instance.variables.iter().map(|(k, v)| {
            Parameter {
                name: k.clone(),
                param_type: self.infer_type(v),
                description: format!("Process variable: {}", k),
                required: false,
                default: Some(v.to_string()),
                constraints: None,
            }
        }).collect()
    }

    fn extract_instance_outputs(&self, _instance: &ProcessInstance) -> Vec<Parameter> {
        // Same as inputs for now, in production this would be more sophisticated
        vec![]
    }

    fn extract_instance_metadata(&self, instance: &ProcessInstance) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("instance_id".to_string(), instance.id.to_string());
        metadata.insert("process_id".to_string(), instance.process_id.clone());
        metadata.insert("state".to_string(), format!("{:?}", instance.state));
        metadata.insert("started_at".to_string(), instance.started_at.to_rfc3339());
        if let Some(completed) = instance.completed_at {
            metadata.insert("completed_at".to_string(), completed.to_rfc3339());
        }
        metadata
    }

    fn infer_type(&self, value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::Null => "null",
            serde_json::Value::Bool(_) => "boolean",
            serde_json::Value::Number(_) => "number",
            serde_json::Value::String(_) => "string",
            serde_json::Value::Array(_) => "array",
            serde_json::Value::Object(_) => "object",
        }.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_documentation_generator_creation() {
        let gen = DocumentationGenerator::default();
        assert_eq!(gen.config.format, DocumentationFormat::Markdown);
    }

    #[test]
    fn test_documentation_config_defaults() {
        let config = DocumentationConfig::default();
        assert!(config.include_diagram);
        assert!(config.include_participants);
    }

    #[test]
    fn test_generate_from_process() {
        let process = ProcessBuilder::new("test", "Test Process")
            .add_user_task("task1", "Test Task")
            .build()
            .expect("Failed to build process");

        let gen = DocumentationGenerator::default();
        let doc = gen.generate_from_process(&process)
            .expect("Failed to generate documentation");

        assert_eq!(doc.process_id, "test");
        assert_eq!(doc.name, "Test Process");
    }
}
