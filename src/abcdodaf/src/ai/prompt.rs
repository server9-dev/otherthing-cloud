//! Prompt template management system
//!
//! Provides templating, variable substitution, and prompt engineering utilities.

use crate::error::{AbcdodafError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Prompt template with variable substitution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    /// Template ID
    pub id: String,
    /// Template name
    pub name: String,
    /// Template content with variables
    pub template: String,
    /// Variable definitions
    pub variables: Vec<PromptVariable>,
    /// Template category
    pub category: Option<String>,
    /// Template version
    pub version: String,
    /// Template metadata
    pub metadata: HashMap<String, String>,
}

/// Prompt variable definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptVariable {
    /// Variable name (used as {{name}} in template)
    pub name: String,
    /// Variable description
    pub description: String,
    /// Variable type
    pub var_type: VariableType,
    /// Default value
    pub default_value: Option<String>,
    /// Required flag
    pub required: bool,
}

/// Variable type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VariableType {
    /// String value
    String,
    /// Integer value
    Integer,
    /// Float value
    Float,
    /// Boolean value
    Boolean,
    /// List of strings
    List,
    /// JSON object
    Json,
}

/// Prompt manager for template storage and retrieval
pub struct PromptManager {
    templates: HashMap<String, PromptTemplate>,
}

impl PromptTemplate {
    /// Create a new prompt template
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        template: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            template: template.into(),
            variables: vec![],
            category: None,
            version: "1.0".to_string(),
            metadata: HashMap::new(),
        }
    }

    /// Add a variable definition
    pub fn add_variable(mut self, variable: PromptVariable) -> Self {
        self.variables.push(variable);
        self
    }

    /// Set category
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Set version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Add metadata
    pub fn add_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Render the template with provided values
    pub fn render(&self, values: &HashMap<String, String>) -> Result<String> {
        let mut rendered = self.template.clone();

        // Check required variables
        for var in &self.variables {
            if var.required && !values.contains_key(&var.name) && var.default_value.is_none() {
                return Err(AbcdodafError::Other(anyhow::anyhow!(
                    "Required variable '{}' not provided",
                    var.name
                )));
            }
        }

        // Substitute variables
        for var in &self.variables {
            let value = values
                .get(&var.name)
                .or(var.default_value.as_ref())
                .map(|s| s.as_str())
                .unwrap_or("");

            let placeholder = format!("{{{{{}}}}}", var.name);
            rendered = rendered.replace(&placeholder, value);
        }

        // Check for unreplaced variables
        if rendered.contains("{{") && rendered.contains("}}") {
            tracing::warn!("Template contains unreplaced variables: {}", rendered);
        }

        Ok(rendered)
    }

    /// Validate template syntax
    pub fn validate(&self) -> Result<()> {
        // Check for balanced braces
        let open_count = self.template.matches("{{").count();
        let close_count = self.template.matches("}}").count();

        if open_count != close_count {
            return Err(AbcdodafError::Other(anyhow::anyhow!(
                "Unbalanced template braces: {} open, {} close",
                open_count,
                close_count
            )));
        }

        // Extract variable names from template
        let mut template_vars = std::collections::HashSet::new();
        let re = regex::Regex::new(r"\{\{(\w+)\}\}").unwrap();
        for cap in re.captures_iter(&self.template) {
            if let Some(name) = cap.get(1) {
                template_vars.insert(name.as_str().to_string());
            }
        }

        // Check if all template variables are defined
        for var_name in &template_vars {
            if !self.variables.iter().any(|v| &v.name == var_name) {
                return Err(AbcdodafError::Other(anyhow::anyhow!(
                    "Template variable '{}' is not defined",
                    var_name
                )));
            }
        }

        Ok(())
    }

    /// Get required variables
    pub fn required_variables(&self) -> Vec<&PromptVariable> {
        self.variables.iter().filter(|v| v.required).collect()
    }

    /// Get optional variables
    pub fn optional_variables(&self) -> Vec<&PromptVariable> {
        self.variables.iter().filter(|v| !v.required).collect()
    }
}

impl PromptVariable {
    /// Create a new prompt variable
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        var_type: VariableType,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            var_type,
            default_value: None,
            required: true,
        }
    }

    /// Set as optional with default value
    pub fn optional(mut self, default: impl Into<String>) -> Self {
        self.default_value = Some(default.into());
        self.required = false;
        self
    }

    /// Set as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
}

impl PromptManager {
    /// Create a new prompt manager
    pub fn new() -> Self {
        Self { templates: HashMap::new() }
    }

    /// Register a prompt template
    pub fn register(&mut self, template: PromptTemplate) -> Result<()> {
        template.validate()?;
        self.templates.insert(template.id.clone(), template);
        Ok(())
    }

    /// Get a template by ID
    pub fn get(&self, id: &str) -> Option<&PromptTemplate> {
        self.templates.get(id)
    }

    /// Get a template by ID (mutable)
    pub fn get_mut(&mut self, id: &str) -> Option<&mut PromptTemplate> {
        self.templates.get_mut(id)
    }

    /// List all templates
    pub fn list(&self) -> Vec<&PromptTemplate> {
        self.templates.values().collect()
    }

    /// List templates by category
    pub fn list_by_category(&self, category: &str) -> Vec<&PromptTemplate> {
        self.templates
            .values()
            .filter(|t| t.category.as_deref() == Some(category))
            .collect()
    }

    /// Remove a template
    pub fn remove(&mut self, id: &str) -> Option<PromptTemplate> {
        self.templates.remove(id)
    }

    /// Render a template by ID
    pub fn render(&self, id: &str, values: &HashMap<String, String>) -> Result<String> {
        let template = self
            .get(id)
            .ok_or_else(|| AbcdodafError::Other(anyhow::anyhow!("Template '{}' not found", id)))?;

        template.render(values)
    }

    /// Create a prompt manager with default templates
    pub fn with_defaults() -> Self {
        let mut manager = Self::new();

        // Code generation template
        let code_gen = PromptTemplate::new(
            "code_generation",
            "Code Generation",
            r#"You are an expert {{language}} programmer. Generate {{language}} code for the following task:

Task: {{task}}

Requirements:
{{requirements}}

Generate clean, well-documented, and idiomatic {{language}} code."#,
        )
        .add_variable(PromptVariable::new("language", "Programming language", VariableType::String))
        .add_variable(PromptVariable::new("task", "Task description", VariableType::String))
        .add_variable(
            PromptVariable::new("requirements", "Specific requirements", VariableType::String)
                .optional("No specific requirements"),
        )
        .with_category("code");

        // Code review template
        let code_review = PromptTemplate::new(
            "code_review",
            "Code Review",
            r#"You are an experienced code reviewer. Review the following {{language}} code and provide feedback:

Code:
```{{language}}
{{code}}
```

Focus areas:
- Code quality and readability
- Best practices and idioms
- Potential bugs or issues
- Performance considerations
- Security concerns

Provide constructive feedback."#,
        )
        .add_variable(PromptVariable::new("language", "Programming language", VariableType::String))
        .add_variable(PromptVariable::new("code", "Code to review", VariableType::String))
        .with_category("code");

        // Data analysis template
        let data_analysis = PromptTemplate::new(
            "data_analysis",
            "Data Analysis",
            r#"You are a data analyst. Analyze the following data and provide insights:

Data:
{{data}}

Analysis goals:
{{goals}}

Provide:
1. Summary statistics
2. Key patterns and trends
3. Insights and recommendations
4. Visualizations (if applicable)"#,
        )
        .add_variable(PromptVariable::new("data", "Data to analyze", VariableType::String))
        .add_variable(PromptVariable::new("goals", "Analysis goals", VariableType::String))
        .with_category("analysis");

        // Q&A template
        let qa = PromptTemplate::new(
            "question_answering",
            "Question Answering",
            r#"Answer the following question accurately and concisely:

Question: {{question}}

{{context}}

Provide a clear, factual answer."#,
        )
        .add_variable(PromptVariable::new("question", "User question", VariableType::String))
        .add_variable(
            PromptVariable::new("context", "Additional context", VariableType::String).optional(""),
        )
        .with_category("qa");

        // Summarization template
        let summarize = PromptTemplate::new(
            "summarization",
            "Text Summarization",
            r#"Summarize the following text in {{length}} format:

Text:
{{text}}

Provide a {{style}} summary."#,
        )
        .add_variable(PromptVariable::new("text", "Text to summarize", VariableType::String))
        .add_variable(
            PromptVariable::new("length", "Summary length", VariableType::String).optional("brief"),
        )
        .add_variable(
            PromptVariable::new("style", "Summary style", VariableType::String)
                .optional("concise and clear"),
        )
        .with_category("text");

        // Register all templates
        let _ = manager.register(code_gen);
        let _ = manager.register(code_review);
        let _ = manager.register(data_analysis);
        let _ = manager.register(qa);
        let _ = manager.register(summarize);

        manager
    }
}

impl Default for PromptManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_template_creation() {
        let template = PromptTemplate::new("test", "Test Template", "Hello {{name}}!")
            .add_variable(PromptVariable::new("name", "User name", VariableType::String));

        assert_eq!(template.id, "test");
        assert_eq!(template.variables.len(), 1);
    }

    #[test]
    fn test_template_rendering() {
        let template = PromptTemplate::new("greeting", "Greeting", "Hello {{name}}!")
            .add_variable(PromptVariable::new("name", "User name", VariableType::String));

        let mut values = HashMap::new();
        values.insert("name".to_string(), "Alice".to_string());

        let rendered = template.render(&values).unwrap();
        assert_eq!(rendered, "Hello Alice!");
    }

    #[test]
    fn test_default_values() {
        let template = PromptTemplate::new("greeting", "Greeting", "Hello {{name}}!").add_variable(
            PromptVariable::new("name", "User name", VariableType::String).optional("World"),
        );

        let values = HashMap::new();
        let rendered = template.render(&values).unwrap();
        assert_eq!(rendered, "Hello World!");
    }

    #[test]
    fn test_required_variable_missing() {
        let template = PromptTemplate::new("greeting", "Greeting", "Hello {{name}}!")
            .add_variable(PromptVariable::new("name", "User name", VariableType::String));

        let values = HashMap::new();
        assert!(template.render(&values).is_err());
    }

    #[test]
    fn test_prompt_manager() {
        let mut manager = PromptManager::new();

        let template = PromptTemplate::new("test", "Test", "Hello {{name}}!")
            .add_variable(PromptVariable::new("name", "Name", VariableType::String));

        manager.register(template).unwrap();

        assert!(manager.get("test").is_some());
        assert_eq!(manager.list().len(), 1);
    }

    #[test]
    fn test_default_templates() {
        let manager = PromptManager::with_defaults();
        assert!(!manager.list().is_empty());
        assert!(manager.get("code_generation").is_some());
        assert!(manager.get("code_review").is_some());
    }
}
