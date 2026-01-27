//! Standards and Rules Validation Framework
//!
//! Comprehensive validation system for enforcing DODAF, BPMN, and custom business rules.
//! Provides declarative rule definitions, automated compliance checking, and detailed reporting.

pub mod rules;
pub mod engine;
pub mod standards;
pub mod compliance;
pub mod violations;

pub use rules::{Rule, RuleSet, RuleDefinition, RuleSeverity, RuleCategory};
pub use engine::{ValidationEngine, ValidationContext, ValidationResult};
pub use standards::{Standard, StandardVersion, StandardsRegistry};
pub use compliance::{ComplianceReport, ComplianceScore, ComplianceLevel};
pub use violations::{Violation, ViolationType, ViolationSeverity};

/// Validation error types
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// Rule not found
    RuleNotFound(String),
    /// Standard not found
    StandardNotFound(String),
    /// Invalid rule definition
    InvalidRuleDefinition(String),
    /// Execution error
    ExecutionError(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RuleNotFound(id) => write!(f, "Rule not found: {}", id),
            Self::StandardNotFound(name) => write!(f, "Standard not found: {}", name),
            Self::InvalidRuleDefinition(msg) => write!(f, "Invalid rule definition: {}", msg),
            Self::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
        }
    }
}

impl std::error::Error for ValidationError {}

pub type ValidationOutcome<T> = Result<T, ValidationError>;
