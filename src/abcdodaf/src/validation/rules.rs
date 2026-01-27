//! Rule definitions and management
//!
//! Declarative system for defining validation rules with conditions, actions, and metadata.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Rule severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleSeverity {
    /// Critical - must be fixed
    Critical,
    /// Error - should be fixed
    Error,
    /// Warning - should review
    Warning,
    /// Info - informational only
    Info,
}

impl RuleSeverity {
    pub fn score(&self) -> u32 {
        match self {
            Self::Critical => 10,
            Self::Error => 5,
            Self::Warning => 2,
            Self::Info => 0,
        }
    }
}

/// Rule category
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuleCategory {
    /// DODAF compliance
    DODAF,
    /// BPMN compliance
    BPMN,
    /// Security requirements
    Security,
    /// Data quality
    DataQuality,
    /// Business logic
    Business,
    /// Performance
    Performance,
    /// Custom category
    Custom(String),
}

/// Rule condition type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCondition {
    /// Element must exist
    MustExist { element_type: String },
    /// Element must not exist
    MustNotExist { element_type: String },
    /// Field must have value
    FieldRequired { field_path: String },
    /// Field must match pattern
    FieldMatches { field_path: String, pattern: String },
    /// Count constraint
    CountConstraint { element_type: String, min: Option<usize>, max: Option<usize> },
    /// Custom expression (JSON path or similar)
    CustomExpression { expression: String },
    /// All conditions must be true
    And(Vec<RuleCondition>),
    /// Any condition must be true
    Or(Vec<RuleCondition>),
    /// Condition must be false
    Not(Box<RuleCondition>),
}

/// Rule definition with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleDefinition {
    /// Unique rule identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Detailed description
    pub description: String,
    /// Rule severity
    pub severity: RuleSeverity,
    /// Rule category
    pub category: RuleCategory,
    /// Standards this rule enforces
    pub standards: Vec<String>,
    /// Rule condition
    pub condition: RuleCondition,
    /// Suggestion for fixing
    pub fix_suggestion: Option<String>,
    /// Documentation link
    pub documentation_url: Option<String>,
    /// Tags for filtering
    pub tags: Vec<String>,
    /// Whether rule is enabled
    pub enabled: bool,
}

impl RuleDefinition {
    /// Create a new rule definition
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        severity: RuleSeverity,
        category: RuleCategory,
        condition: RuleCondition,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            severity,
            category,
            standards: Vec::new(),
            condition,
            fix_suggestion: None,
            documentation_url: None,
            tags: Vec::new(),
            enabled: true,
        }
    }

    /// Add standard reference
    pub fn with_standard(mut self, standard: impl Into<String>) -> Self {
        self.standards.push(standard.into());
        self
    }

    /// Add fix suggestion
    pub fn with_fix(mut self, suggestion: impl Into<String>) -> Self {
        self.fix_suggestion = Some(suggestion.into());
        self
    }

    /// Add documentation URL
    pub fn with_docs(mut self, url: impl Into<String>) -> Self {
        self.documentation_url = Some(url.into());
        self
    }

    /// Add tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}

/// Collection of rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSet {
    /// Rule set identifier
    pub id: String,
    /// Rule set name
    pub name: String,
    /// Description
    pub description: String,
    /// Rules in this set
    pub rules: Vec<RuleDefinition>,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

impl RuleSet {
    /// Create a new rule set
    pub fn new(id: impl Into<String>, name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            rules: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a rule to the set
    pub fn add_rule(mut self, rule: RuleDefinition) -> Self {
        self.rules.push(rule);
        self
    }

    /// Get rules by category
    pub fn rules_by_category(&self, category: &RuleCategory) -> Vec<&RuleDefinition> {
        self.rules.iter().filter(|r| &r.category == category).collect()
    }

    /// Get rules by severity
    pub fn rules_by_severity(&self, severity: RuleSeverity) -> Vec<&RuleDefinition> {
        self.rules.iter().filter(|r| r.severity == severity).collect()
    }

    /// Get enabled rules only
    pub fn enabled_rules(&self) -> Vec<&RuleDefinition> {
        self.rules.iter().filter(|r| r.enabled).collect()
    }
}

/// Rule implementation trait
pub trait Rule: Send + Sync {
    /// Get rule definition
    fn definition(&self) -> &RuleDefinition;

    /// Validate against this rule
    fn validate(&self, context: &dyn std::any::Any) -> Result<bool, String>;

    /// Get violation message
    fn violation_message(&self, context: &dyn std::any::Any) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_definition_creation() {
        let rule = RuleDefinition::new(
            "DODAF_001",
            "Process must have operational activity",
            "Every BPMN process must map to at least one DODAF operational activity",
            RuleSeverity::Error,
            RuleCategory::DODAF,
            RuleCondition::FieldRequired {
                field_path: "metadata.dodaf.operational_activity".to_string(),
            },
        )
        .with_standard("DODAF 2.02")
        .with_fix("Add operational activity metadata to the process")
        .with_tag("operational");

        assert_eq!(rule.id, "DODAF_001");
        assert_eq!(rule.severity, RuleSeverity::Error);
        assert_eq!(rule.standards, vec!["DODAF 2.02"]);
        assert!(rule.fix_suggestion.is_some());
    }

    #[test]
    fn test_rule_set_filtering() {
        let mut ruleset = RuleSet::new(
            "test_set",
            "Test Rules",
            "Test rule set",
        );

        ruleset = ruleset.add_rule(RuleDefinition::new(
            "R1",
            "Rule 1",
            "Description",
            RuleSeverity::Error,
            RuleCategory::DODAF,
            RuleCondition::MustExist { element_type: "activity".to_string() },
        ));

        ruleset = ruleset.add_rule(RuleDefinition::new(
            "R2",
            "Rule 2",
            "Description",
            RuleSeverity::Warning,
            RuleCategory::BPMN,
            RuleCondition::MustExist { element_type: "start".to_string() },
        ));

        let dodaf_rules = ruleset.rules_by_category(&RuleCategory::DODAF);
        assert_eq!(dodaf_rules.len(), 1);

        let error_rules = ruleset.rules_by_severity(RuleSeverity::Error);
        assert_eq!(error_rules.len(), 1);
    }

    #[test]
    fn test_rule_severity_scoring() {
        assert_eq!(RuleSeverity::Critical.score(), 10);
        assert_eq!(RuleSeverity::Error.score(), 5);
        assert_eq!(RuleSeverity::Warning.score(), 2);
        assert_eq!(RuleSeverity::Info.score(), 0);
    }
}
