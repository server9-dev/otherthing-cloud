//! Auto-documentation generator for BPMN models and workflows
//!
//! This module provides automatic documentation generation from BPMN process definitions,
//! including Markdown, HTML, and SVG diagram exports. Supports both operational and
//! architectural documentation views aligned with DoDAF 2.02.

pub mod builtin_templates;
pub mod diagrams;
pub mod exporters;
pub mod generator;
pub mod templates;

pub use builtin_templates::create_builtin_library;
pub use diagrams::{DiagramFormat, DiagramGenerator, SvgDiagram};
pub use exporters::{Exporter, HtmlExporter, MarkdownExporter, PlainTextExporter};
pub use generator::{DocumentationConfig, DocumentationFormat, DocumentationGenerator};
pub use templates::{Template, TemplateCategory, TemplateLibrary, TemplateMetadata};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Comprehensive process documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessDocumentation {
    /// Process ID
    pub process_id: String,
    /// Process name
    pub name: String,
    /// Process description
    pub description: String,
    /// Process purpose
    pub purpose: Option<String>,
    /// Process inputs
    pub inputs: Vec<Parameter>,
    /// Process outputs
    pub outputs: Vec<Parameter>,
    /// Process participants
    pub participants: Vec<Participant>,
    /// Process flow description
    pub flow_description: String,
    /// Exception handling documentation
    pub error_handling: Vec<ErrorScenario>,
    /// Related DoDAF views
    pub dodaf_views: Vec<String>,
    /// Tags and metadata
    pub metadata: HashMap<String, String>,
    /// Generated date
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

/// Parameter documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: String,
    /// Parameter description
    pub description: String,
    /// Required flag
    pub required: bool,
    /// Default value (if any)
    pub default: Option<String>,
    /// Constraints or validation rules
    pub constraints: Option<String>,
}

/// Process participant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    /// Participant ID
    pub id: String,
    /// Participant name
    pub name: String,
    /// Participant type (system, human, agent, etc.)
    pub participant_type: String,
    /// Participant role
    pub role: Option<String>,
    /// Description of responsibilities
    pub responsibilities: Vec<String>,
}

/// Error/exception scenario documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorScenario {
    /// Error condition
    pub condition: String,
    /// Error handling strategy
    pub handling: String,
    /// Recovery steps
    pub recovery_steps: Vec<String>,
    /// Impact assessment
    pub impact: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_documentation_creation() {
        let doc = ProcessDocumentation {
            process_id: "test_proc".to_string(),
            name: "Test Process".to_string(),
            description: "A test process".to_string(),
            purpose: None,
            inputs: vec![],
            outputs: vec![],
            participants: vec![],
            flow_description: "Simple flow".to_string(),
            error_handling: vec![],
            dodaf_views: vec![],
            metadata: HashMap::new(),
            generated_at: chrono::Utc::now(),
        };
        assert_eq!(doc.process_id, "test_proc");
    }
}
