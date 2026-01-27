//! Tool use and function calling support
//!
//! Enables AI agents to call external tools and functions.

use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Tool registry
pub struct ToolRegistry {
    /// Registered tools
    tools: HashMap<String, ToolDefinition>,
    /// Tool executors
    executors: HashMap<String, Box<dyn ToolExecutor>>,
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Input parameters
    pub parameters: Vec<ParameterDefinition>,
    /// Return type description
    pub returns: String,
    /// Tool category
    pub category: Option<String>,
}

/// Parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDefinition {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: String,
    /// Parameter description
    pub description: String,
    /// Required flag
    pub required: bool,
    /// Default value
    pub default: Option<Value>,
}

/// Tool call request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Call ID
    pub id: String,
    /// Tool name
    pub tool_name: String,
    /// Arguments
    pub arguments: HashMap<String, Value>,
}

/// Tool call result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Call ID
    pub call_id: String,
    /// Success flag
    pub success: bool,
    /// Result data
    pub result: Value,
    /// Error message if failed
    pub error: Option<String>,
    /// Execution time in ms
    pub duration_ms: u64,
}

/// Tool executor trait
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    /// Execute the tool
    async fn execute(&self, arguments: HashMap<String, Value>) -> Result<Value>;
}

impl ToolRegistry {
    /// Create a new tool registry
    pub fn new() -> Self {
        Self { tools: HashMap::new(), executors: HashMap::new() }
    }

    /// Register a tool
    pub fn register(
        &mut self,
        definition: ToolDefinition,
        executor: Box<dyn ToolExecutor>,
    ) -> Result<()> {
        self.executors.insert(definition.name.clone(), executor);
        self.tools.insert(definition.name.clone(), definition);
        Ok(())
    }

    /// Get tool definition
    pub fn get_definition(&self, name: &str) -> Option<&ToolDefinition> {
        self.tools.get(name)
    }

    /// List all tools
    pub fn list_tools(&self) -> Vec<&ToolDefinition> {
        self.tools.values().collect()
    }

    /// Execute a tool call
    pub async fn execute(&self, call: &ToolCall) -> ToolResult {
        let start = std::time::Instant::now();

        let result = if let Some(executor) = self.executors.get(&call.tool_name) {
            match executor.execute(call.arguments.clone()).await {
                Ok(value) => ToolResult {
                    call_id: call.id.clone(),
                    success: true,
                    result: value,
                    error: None,
                    duration_ms: start.elapsed().as_millis() as u64,
                },
                Err(e) => ToolResult {
                    call_id: call.id.clone(),
                    success: false,
                    result: Value::Null,
                    error: Some(e.to_string()),
                    duration_ms: start.elapsed().as_millis() as u64,
                },
            }
        } else {
            ToolResult {
                call_id: call.id.clone(),
                success: false,
                result: Value::Null,
                error: Some(format!("Tool '{}' not found", call.tool_name)),
                duration_ms: start.elapsed().as_millis() as u64,
            }
        };

        result
    }

    /// Create with default tools
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();

        // Calculator tool
        let calc_def = ToolDefinition {
            name: "calculator".to_string(),
            description: "Perform arithmetic calculations".to_string(),
            parameters: vec![ParameterDefinition {
                name: "expression".to_string(),
                param_type: "string".to_string(),
                description: "Mathematical expression to evaluate".to_string(),
                required: true,
                default: None,
            }],
            returns: "number".to_string(),
            category: Some("math".to_string()),
        };

        struct CalculatorExecutor;
        #[async_trait]
        impl ToolExecutor for CalculatorExecutor {
            async fn execute(&self, arguments: HashMap<String, Value>) -> Result<Value> {
                // Simplified calculator
                if let Some(Value::String(expr)) = arguments.get("expression") {
                    // This is a placeholder - in production use a proper expression parser
                    Ok(Value::String(format!("Result of: {}", expr)))
                } else {
                    Ok(Value::Null)
                }
            }
        }

        let _ = registry.register(calc_def, Box::new(CalculatorExecutor));

        registry
    }
}

impl ToolDefinition {
    /// Create a new tool definition
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters: vec![],
            returns: "any".to_string(),
            category: None,
        }
    }

    /// Add parameter
    pub fn add_parameter(mut self, param: ParameterDefinition) -> Self {
        self.parameters.push(param);
        self
    }

    /// Set return type
    pub fn with_returns(mut self, returns: impl Into<String>) -> Self {
        self.returns = returns.into();
        self
    }

    /// Set category
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }
}

impl ParameterDefinition {
    /// Create a new parameter
    pub fn new(
        name: impl Into<String>,
        param_type: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            param_type: param_type.into(),
            description: description.into(),
            required: true,
            default: None,
        }
    }

    /// Make optional with default
    pub fn optional(mut self, default: Value) -> Self {
        self.required = false;
        self.default = Some(default);
        self
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_registry_creation() {
        let registry = ToolRegistry::new();
        assert_eq!(registry.list_tools().len(), 0);
    }

    #[test]
    fn test_tool_definition() {
        let tool = ToolDefinition::new("test", "Test tool")
            .add_parameter(ParameterDefinition::new("arg", "string", "Test arg"))
            .with_returns("string")
            .with_category("test");

        assert_eq!(tool.name, "test");
        assert_eq!(tool.parameters.len(), 1);
    }

    #[test]
    fn test_default_tools() {
        let registry = ToolRegistry::with_defaults();
        assert!(!registry.list_tools().is_empty());
        assert!(registry.get_definition("calculator").is_some());
    }
}
