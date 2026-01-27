//! Violation tracking and reporting

use super::rules::RuleSeverity;
use serde::{Deserialize, Serialize};

/// Type of violation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ViolationType {
    /// Rule violation
    RuleViolation,
    /// Missing required element
    MissingElement,
    /// Invalid value
    InvalidValue,
    /// Constraint violation
    ConstraintViolation,
    /// Security violation
    SecurityViolation,
}

/// Violation severity
pub type ViolationSeverity = RuleSeverity;

/// Detailed violation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    /// Rule ID that was violated
    pub rule_id: String,
    /// Rule name
    pub rule_name: String,
    /// Violation severity
    pub severity: RuleSeverity,
    /// Type of violation
    pub violation_type: ViolationType,
    /// Element ID where violation occurred
    pub element_id: Option<String>,
    /// Element type
    pub element_type: Option<String>,
    /// Violation message
    pub message: String,
    /// Location in the model
    pub location: String,
    /// Fix suggestion
    pub fix_suggestion: Option<String>,
}

impl Violation {
    /// Create a new violation
    pub fn new(
        rule_id: impl Into<String>,
        rule_name: impl Into<String>,
        severity: RuleSeverity,
        message: impl Into<String>,
    ) -> Self {
        Self {
            rule_id: rule_id.into(),
            rule_name: rule_name.into(),
            severity,
            violation_type: ViolationType::RuleViolation,
            element_id: None,
            element_type: None,
            message: message.into(),
            location: String::new(),
            fix_suggestion: None,
        }
    }

    /// Set element context
    pub fn with_element(mut self, id: impl Into<String>, element_type: impl Into<String>) -> Self {
        self.element_id = Some(id.into());
        self.element_type = Some(element_type.into());
        self
    }

    /// Set location
    pub fn at_location(mut self, location: impl Into<String>) -> Self {
        self.location = location.into();
        self
    }

    /// Set fix suggestion
    pub fn with_fix(mut self, suggestion: impl Into<String>) -> Self {
        self.fix_suggestion = Some(suggestion.into());
        self
    }

    /// Get formatted display string
    pub fn display_string(&self) -> String {
        format!(
            "[{}] {} at {}: {}",
            match self.severity {
                RuleSeverity::Critical => "CRITICAL",
                RuleSeverity::Error => "ERROR",
                RuleSeverity::Warning => "WARNING",
                RuleSeverity::Info => "INFO",
            },
            self.rule_name,
            if self.location.is_empty() { "root" } else { &self.location },
            self.message
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_violation_creation() {
        let violation = Violation::new(
            "DODAF_001",
            "Missing operational activity",
            RuleSeverity::Error,
            "Process must have operational activity metadata",
        )
        .with_element("process_1", "BpmnProcess")
        .at_location("processes[0]")
        .with_fix("Add metadata.dodaf.operational_activity field");

        assert_eq!(violation.rule_id, "DODAF_001");
        assert_eq!(violation.severity, RuleSeverity::Error);
        assert!(violation.element_id.is_some());
        assert!(violation.fix_suggestion.is_some());
    }

    #[test]
    fn test_violation_display() {
        let violation =
            Violation::new("TEST_001", "Test Rule", RuleSeverity::Warning, "Test message")
                .at_location("test.path");

        let display = violation.display_string();
        assert!(display.contains("WARNING"));
        assert!(display.contains("test.path"));
    }
}
