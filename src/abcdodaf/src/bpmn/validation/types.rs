//! Validation error types and result structures
//!
//! Defines the error types, severity levels, and result types used throughout
//! the BPMN validation system.

use serde::{Deserialize, Serialize};

/// Validation error with detailed context
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error severity level
    pub severity: ErrorSeverity,
    /// Error category
    pub category: ErrorCategory,
    /// Human-readable error message
    pub message: String,
    /// Context information (node ID, field name, etc.)
    pub context: Option<String>,
    /// Suggested fix or additional information
    pub suggestion: Option<String>,
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Critical error - workflow cannot be loaded or executed
    Error,
    /// Warning - workflow may have issues but can be loaded
    Warning,
    /// Informational - best practice suggestions
    Info,
}

/// Error categories for easier filtering and handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// Missing required field
    MissingField,
    /// Invalid field value
    InvalidValue,
    /// Reference to non-existent element
    BrokenReference,
    /// Duplicate identifier
    DuplicateId,
    /// Structural issue (disconnected nodes, no start/end, etc.)
    StructuralIssue,
    /// Type-specific validation
    TypeValidation,
}

impl ValidationError {
    /// Create a new error
    pub fn error(category: ErrorCategory, message: impl Into<String>) -> Self {
        Self {
            severity: ErrorSeverity::Error,
            category,
            message: message.into(),
            context: None,
            suggestion: None,
        }
    }

    /// Create a new warning
    pub fn warning(category: ErrorCategory, message: impl Into<String>) -> Self {
        Self {
            severity: ErrorSeverity::Warning,
            category,
            message: message.into(),
            context: None,
            suggestion: None,
        }
    }

    /// Create a new info message
    pub fn info(category: ErrorCategory, message: impl Into<String>) -> Self {
        Self {
            severity: ErrorSeverity::Info,
            category,
            message: message.into(),
            context: None,
            suggestion: None,
        }
    }

    /// Add context to the error
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Add a suggestion for fixing the error
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

/// Validation result type
pub type ValidationResult = Result<(), Vec<ValidationError>>;

/// Validation summary helper
pub struct ValidationSummary {
    pub errors: usize,
    pub warnings: usize,
    pub info: usize,
}

impl ValidationSummary {
    /// Create a summary from a list of validation errors
    pub fn from_errors(errors: &[ValidationError]) -> Self {
        let mut summary = Self { errors: 0, warnings: 0, info: 0 };

        for error in errors {
            match error.severity {
                ErrorSeverity::Error => summary.errors += 1,
                ErrorSeverity::Warning => summary.warnings += 1,
                ErrorSeverity::Info => summary.info += 1,
            }
        }

        summary
    }

    /// Check if there are any errors (not warnings or info)
    pub fn has_errors(&self) -> bool {
        self.errors > 0
    }

    /// Format the summary as a string
    pub fn format(&self) -> String {
        format!(
            "Validation summary: {} error(s), {} warning(s), {} info message(s)",
            self.errors, self.warnings, self.info
        )
    }
}
