//! Template library system for workflows and documentation
//!
//! Provides pre-built workflow templates for common patterns including:
//! - Approval workflows
//! - Orchestration patterns
//! - ETL (Extract-Transform-Load)
//! - Human-in-the-loop workflows
//! - Error handling patterns

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Template categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum TemplateCategory {
    /// Approval workflows
    ApprovalWorkflow,
    /// Orchestration patterns
    Orchestration,
    /// ETL patterns
    DataTransformation,
    /// Human-in-the-loop patterns
    HumanInTheLoop,
    /// Error handling patterns
    ErrorHandling,
    /// Notification and alerting
    Notification,
    /// Scheduled/batch processing
    BatchProcessing,
    /// Decision making patterns
    DecisionLogic,
}

impl TemplateCategory {
    pub fn label(&self) -> &'static str {
        match self {
            TemplateCategory::ApprovalWorkflow => "Approval Workflows",
            TemplateCategory::Orchestration => "Orchestration Patterns",
            TemplateCategory::DataTransformation => "Data Transformation (ETL)",
            TemplateCategory::HumanInTheLoop => "Human-in-the-Loop",
            TemplateCategory::ErrorHandling => "Error Handling",
            TemplateCategory::Notification => "Notifications & Alerts",
            TemplateCategory::BatchProcessing => "Batch Processing",
            TemplateCategory::DecisionLogic => "Decision Logic",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            TemplateCategory::ApprovalWorkflow => "Templates for multi-level approval processes",
            TemplateCategory::Orchestration => "Patterns for orchestrating distributed services",
            TemplateCategory::DataTransformation => "Extract, transform, and load data workflows",
            TemplateCategory::HumanInTheLoop => "Workflows combining human and automated tasks",
            TemplateCategory::ErrorHandling => "Patterns for resilient error handling",
            TemplateCategory::Notification => "Notification and alert workflow patterns",
            TemplateCategory::BatchProcessing => "Scheduled and batch processing patterns",
            TemplateCategory::DecisionLogic => "Business rules and decision point patterns",
        }
    }
}

/// Template metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    /// Template ID
    pub id: String,
    /// Template name
    pub name: String,
    /// Template description
    pub description: String,
    /// Template category
    pub category: TemplateCategory,
    /// Template version
    pub version: String,
    /// Author/creator
    pub author: Option<String>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last modified timestamp
    pub modified_at: chrono::DateTime<chrono::Utc>,
    /// Tags for searching
    pub tags: Vec<String>,
    /// Complexity level (beginner, intermediate, advanced)
    pub complexity: String,
    /// Use cases this template is suitable for
    pub use_cases: Vec<String>,
    /// Related templates
    pub related_templates: Vec<String>,
}

/// Complete template with content and structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    /// Metadata
    pub metadata: TemplateMetadata,
    /// BPMN XML definition
    pub bpmn_definition: String,
    /// Process definition JSON
    pub process_definition: serde_json::Value,
    /// Documentation template
    pub documentation: String,
    /// Example usage
    pub example_usage: Option<String>,
    /// Configuration parameters
    pub parameters: HashMap<String, ParameterConfig>,
}

/// Configuration parameter for templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterConfig {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: String,
    /// Default value
    pub default: Option<String>,
    /// Parameter description
    pub description: String,
    /// Required flag
    pub required: bool,
}

/// Template library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateLibrary {
    /// Library name
    pub name: String,
    /// Library version
    pub version: String,
    /// All templates
    pub templates: HashMap<String, Template>,
    /// Category index
    pub categories: HashMap<TemplateCategory, Vec<String>>,
    /// Search index (tag -> template IDs)
    pub search_index: HashMap<String, Vec<String>>,
    /// Last updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl TemplateLibrary {
    /// Create a new template library
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            templates: HashMap::new(),
            categories: HashMap::new(),
            search_index: HashMap::new(),
            updated_at: chrono::Utc::now(),
        }
    }

    /// Add a template to the library
    pub fn add_template(&mut self, template: Template) -> Result<(), String> {
        let id = template.metadata.id.clone();
        let category = template.metadata.category;

        // Add to main templates map
        self.templates.insert(id.clone(), template.clone());

        // Add to category index
        self.categories.entry(category).or_insert_with(Vec::new).push(id.clone());

        // Add to search index
        for tag in &template.metadata.tags {
            self.search_index.entry(tag.clone()).or_insert_with(Vec::new).push(id.clone());
        }

        self.updated_at = chrono::Utc::now();
        Ok(())
    }

    /// Get a template by ID
    pub fn get_template(&self, id: &str) -> Option<&Template> {
        self.templates.get(id)
    }

    /// Get all templates in a category
    pub fn get_by_category(&self, category: TemplateCategory) -> Vec<&Template> {
        self.categories
            .get(&category)
            .map(|ids| ids.iter().filter_map(|id| self.templates.get(id)).collect())
            .unwrap_or_default()
    }

    /// Search templates by tag
    pub fn search(&self, query: &str) -> Vec<&Template> {
        let query_lower = query.to_lowercase();
        self.templates
            .values()
            .filter(|t| {
                t.metadata.name.to_lowercase().contains(&query_lower)
                    || t.metadata.description.to_lowercase().contains(&query_lower)
                    || t.metadata.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    /// Get templates by complexity level
    pub fn get_by_complexity(&self, level: &str) -> Vec<&Template> {
        self.templates.values().filter(|t| t.metadata.complexity == level).collect()
    }

    /// List all template IDs
    pub fn list_ids(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }

    /// Get library statistics
    pub fn get_statistics(&self) -> LibraryStatistics {
        LibraryStatistics {
            total_templates: self.templates.len(),
            templates_by_category: self
                .categories
                .iter()
                .map(|(cat, ids)| (format!("{:?}", cat), ids.len()))
                .collect(),
            total_categories: self.categories.len(),
        }
    }
}

/// Library statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryStatistics {
    /// Total number of templates
    pub total_templates: usize,
    /// Templates grouped by category
    pub templates_by_category: HashMap<String, usize>,
    /// Total number of categories
    pub total_categories: usize,
}

impl Template {
    /// Create a new template
    pub fn new(metadata: TemplateMetadata, bpmn_definition: String) -> Self {
        Self {
            metadata,
            bpmn_definition,
            process_definition: serde_json::json!({}),
            documentation: String::new(),
            example_usage: None,
            parameters: HashMap::new(),
        }
    }

    /// Add a parameter to the template
    pub fn add_parameter(&mut self, name: impl Into<String>, config: ParameterConfig) {
        self.parameters.insert(name.into(), config);
    }

    /// Clone the template with new ID
    pub fn clone_with_new_id(&self) -> Self {
        let mut cloned = self.clone();
        cloned.metadata.id = Uuid::new_v4().to_string();
        cloned.metadata.created_at = chrono::Utc::now();
        cloned.metadata.modified_at = chrono::Utc::now();
        cloned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_sample_metadata() -> TemplateMetadata {
        TemplateMetadata {
            id: Uuid::new_v4().to_string(),
            name: "Sample Approval Workflow".to_string(),
            description: "A simple approval workflow template".to_string(),
            category: TemplateCategory::ApprovalWorkflow,
            version: "1.0.0".to_string(),
            author: Some("System".to_string()),
            created_at: chrono::Utc::now(),
            modified_at: chrono::Utc::now(),
            tags: vec!["approval".to_string(), "workflow".to_string()],
            complexity: "beginner".to_string(),
            use_cases: vec!["Request approval".to_string()],
            related_templates: vec![],
        }
    }

    #[test]
    fn test_template_creation() {
        let metadata = create_sample_metadata();
        let template = Template::new(metadata.clone(), "<bpmn/>".to_string());
        assert_eq!(template.metadata.name, "Sample Approval Workflow");
    }

    #[test]
    fn test_template_library_creation() {
        let library = TemplateLibrary::new("Test Library", "1.0.0");
        assert_eq!(library.name, "Test Library");
        assert_eq!(library.templates.len(), 0);
    }

    #[test]
    fn test_add_template_to_library() {
        let mut library = TemplateLibrary::new("Test Library", "1.0.0");
        let metadata = create_sample_metadata();
        let template = Template::new(metadata, "<bpmn/>".to_string());

        assert!(library.add_template(template).is_ok());
        assert_eq!(library.templates.len(), 1);
    }

    #[test]
    fn test_search_templates() {
        let mut library = TemplateLibrary::new("Test Library", "1.0.0");
        let metadata = create_sample_metadata();
        let template = Template::new(metadata, "<bpmn/>".to_string());

        library.add_template(template).unwrap();
        let results = library.search("approval");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_get_by_category() {
        let mut library = TemplateLibrary::new("Test Library", "1.0.0");
        let metadata = create_sample_metadata();
        let template = Template::new(metadata, "<bpmn/>".to_string());

        library.add_template(template).unwrap();
        let results = library.get_by_category(TemplateCategory::ApprovalWorkflow);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_template_category_labels() {
        assert_eq!(TemplateCategory::ApprovalWorkflow.label(), "Approval Workflows");
        assert_eq!(TemplateCategory::Orchestration.label(), "Orchestration Patterns");
    }
}
