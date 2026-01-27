//! DMN error types and results

use thiserror::Error;

/// DMN operation result type
pub type DmnResult<T> = Result<T, DmnError>;

/// Errors that can occur during DMN operations
#[derive(Error, Debug)]
pub enum DmnError {
    /// FEEL expression parsing error
    #[error("FEEL parsing error: {0}")]
    FeelParsingError(String),

    /// FEEL evaluation error
    #[error("FEEL evaluation error: {0}")]
    FeelEvaluationError(String),

    /// Invalid decision table configuration
    #[error("Invalid decision table: {0}")]
    InvalidDecisionTable(String),

    /// No matching rules found in decision table
    #[error("No matching rules found for inputs")]
    NoMatchingRules,

    /// Multiple conflicting rules matched
    #[error("Ambiguous rules matched: {0}")]
    AmbiguousRules(String),

    /// Missing required input
    #[error("Missing required input: {0}")]
    MissingInput(String),

    /// Invalid input value
    #[error("Invalid input value: {0}")]
    InvalidInput(String),

    /// Type mismatch
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },

    /// Decision not found
    #[error("Decision not found: {0}")]
    DecisionNotFound(String),

    /// Invalid decision graph
    #[error("Invalid decision graph: {0}")]
    InvalidDecisionGraph(String),

    /// Circular dependency in decisions
    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    /// XML parsing error
    #[error("XML parsing error: {0}")]
    XmlParsingError(String),

    /// XML generation error
    #[error("XML generation error: {0}")]
    XmlGenerationError(String),

    /// Unsupported DMN version
    #[error("Unsupported DMN version: {0}")]
    UnsupportedVersion(String),

    /// Generic operation error
    #[error("Operation failed: {0}")]
    OperationError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON serialization error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// YAML serialization error
    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = DmnError::NoMatchingRules;
        assert_eq!(err.to_string(), "No matching rules found for inputs");
    }

    #[test]
    fn test_type_mismatch_error() {
        let err = DmnError::TypeMismatch {
            expected: "String".to_string(),
            actual: "Number".to_string(),
        };
        assert!(err.to_string().contains("Type mismatch"));
        assert!(err.to_string().contains("String"));
        assert!(err.to_string().contains("Number"));
    }
}
