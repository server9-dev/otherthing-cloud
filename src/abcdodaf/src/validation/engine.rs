//! Validation engine for executing rules
//!
//! Core engine that evaluates rules against BPMN diagrams and DODAF metadata.

use super::rules::{RuleCondition, RuleDefinition, RuleSeverity};
use super::violations::{Violation, ViolationType};
use crate::bpmn::elements::BpmnDiagram;
use std::collections::HashMap;

/// Validation context
pub struct ValidationContext<'a> {
    /// The diagram being validated
    pub diagram: &'a BpmnDiagram,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Context path (for nested validation)
    pub path: Vec<String>,
}

impl<'a> ValidationContext<'a> {
    /// Create a new validation context
    pub fn new(diagram: &'a BpmnDiagram) -> Self {
        Self {
            diagram,
            metadata: HashMap::new(),
            path: Vec::new(),
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Push path segment
    pub fn push_path(&mut self, segment: impl Into<String>) {
        self.path.push(segment.into());
    }

    /// Pop path segment
    pub fn pop_path(&mut self) {
        self.path.pop();
    }

    /// Get current path as string
    pub fn current_path(&self) -> String {
        self.path.join(".")
    }
}

/// Validation result for a single rule
#[derive(Debug, Clone)]
pub struct RuleValidationResult {
    /// Rule that was validated
    pub rule_id: String,
    /// Whether rule passed
    pub passed: bool,
    /// Violation if rule failed
    pub violation: Option<Violation>,
}

/// Overall validation result
#[derive(Debug)]
pub struct ValidationResult {
    /// Individual rule results
    pub results: Vec<RuleValidationResult>,
    /// Total rules checked
    pub total_rules: usize,
    /// Rules passed
    pub passed: usize,
    /// Rules failed
    pub failed: usize,
    /// Critical violations
    pub critical_count: usize,
    /// Error violations
    pub error_count: usize,
    /// Warning violations
    pub warning_count: usize,
}

impl ValidationResult {
    /// Create new validation result
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            total_rules: 0,
            passed: 0,
            failed: 0,
            critical_count: 0,
            error_count: 0,
            warning_count: 0,
        }
    }

    /// Add a rule result
    pub fn add_result(&mut self, result: RuleValidationResult) {
        self.total_rules += 1;

        if result.passed {
            self.passed += 1;
        } else {
            self.failed += 1;

            if let Some(ref violation) = result.violation {
                match violation.severity {
                    RuleSeverity::Critical => self.critical_count += 1,
                    RuleSeverity::Error => self.error_count += 1,
                    RuleSeverity::Warning => self.warning_count += 1,
                    RuleSeverity::Info => {}
                }
            }
        }

        self.results.push(result);
    }

    /// Check if all rules passed
    pub fn is_valid(&self) -> bool {
        self.failed == 0
    }

    /// Get pass rate percentage
    pub fn pass_rate(&self) -> f64 {
        if self.total_rules == 0 {
            100.0
        } else {
            (self.passed as f64 / self.total_rules as f64) * 100.0
        }
    }

    /// Get all violations
    pub fn violations(&self) -> Vec<&Violation> {
        self.results
            .iter()
            .filter_map(|r| r.violation.as_ref())
            .collect()
    }

    /// Get violations by severity
    pub fn violations_by_severity(&self, severity: RuleSeverity) -> Vec<&Violation> {
        self.violations()
            .into_iter()
            .filter(|v| v.severity == severity)
            .collect()
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation engine
pub struct ValidationEngine {
    /// Registered rules
    rules: Vec<RuleDefinition>,
}

impl ValidationEngine {
    /// Create a new validation engine
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
        }
    }

    /// Register a rule
    pub fn register_rule(&mut self, rule: RuleDefinition) {
        self.rules.push(rule);
    }

    /// Register multiple rules
    pub fn register_rules(&mut self, rules: Vec<RuleDefinition>) {
        self.rules.extend(rules);
    }

    /// Validate a diagram against all registered rules
    pub fn validate(&self, context: &ValidationContext) -> ValidationResult {
        let mut result = ValidationResult::new();

        for rule in &self.rules {
            if !rule.enabled {
                continue;
            }

            let rule_result = self.validate_rule(rule, context);
            result.add_result(rule_result);
        }

        result
    }

    /// Validate a diagram against specific rules
    pub fn validate_with_rules(
        &self,
        context: &ValidationContext,
        rule_ids: &[String],
    ) -> ValidationResult {
        let mut result = ValidationResult::new();

        for rule in &self.rules {
            if !rule.enabled || !rule_ids.contains(&rule.id) {
                continue;
            }

            let rule_result = self.validate_rule(rule, context);
            result.add_result(rule_result);
        }

        result
    }

    /// Validate a single rule
    fn validate_rule(&self, rule: &RuleDefinition, context: &ValidationContext) -> RuleValidationResult {
        let passed = self.evaluate_condition(&rule.condition, context);

        let violation = if !passed {
            Some(Violation {
                rule_id: rule.id.clone(),
                rule_name: rule.name.clone(),
                severity: rule.severity,
                violation_type: ViolationType::RuleViolation,
                element_id: None,
                element_type: None,
                message: rule.description.clone(),
                location: context.current_path(),
                fix_suggestion: rule.fix_suggestion.clone(),
            })
        } else {
            None
        };

        RuleValidationResult {
            rule_id: rule.id.clone(),
            passed,
            violation,
        }
    }

    /// Evaluate a rule condition
    fn evaluate_condition(&self, condition: &RuleCondition, context: &ValidationContext) -> bool {
        match condition {
            RuleCondition::MustExist { element_type } => {
                self.check_element_exists(element_type, context)
            }
            RuleCondition::MustNotExist { element_type } => {
                !self.check_element_exists(element_type, context)
            }
            RuleCondition::FieldRequired { field_path } => {
                self.check_field_required(field_path, context)
            }
            RuleCondition::FieldMatches { field_path, pattern } => {
                self.check_field_matches(field_path, pattern, context)
            }
            RuleCondition::CountConstraint { element_type, min, max } => {
                self.check_count_constraint(element_type, *min, *max, context)
            }
            RuleCondition::CustomExpression { expression: _ } => {
                // Custom expressions would be evaluated here
                // For now, return true
                true
            }
            RuleCondition::And(conditions) => {
                conditions.iter().all(|c| self.evaluate_condition(c, context))
            }
            RuleCondition::Or(conditions) => {
                conditions.iter().any(|c| self.evaluate_condition(c, context))
            }
            RuleCondition::Not(condition) => {
                !self.evaluate_condition(condition, context)
            }
        }
    }

    /// Check if element type exists
    fn check_element_exists(&self, element_type: &str, context: &ValidationContext) -> bool {
        match element_type {
            "process" => !context.diagram.processes.is_empty(),
            "start_event" => context.diagram.processes.iter().any(|p| !p.start_events.is_empty()),
            "end_event" => context.diagram.processes.iter().any(|p| !p.end_events.is_empty()),
            "task" => context.diagram.processes.iter().any(|p| !p.tasks.is_empty()),
            "gateway" => context.diagram.processes.iter().any(|p| !p.gateways.is_empty()),
            "sequence_flow" => context.diagram.processes.iter().any(|p| !p.sequence_flows.is_empty()),
            _ => false,
        }
    }

    /// Check if field is present
    fn check_field_required(&self, _field_path: &str, _context: &ValidationContext) -> bool {
        // Field path evaluation would be implemented here
        // For now, return true
        true
    }

    /// Check if field matches pattern
    fn check_field_matches(&self, _field_path: &str, _pattern: &str, _context: &ValidationContext) -> bool {
        // Pattern matching would be implemented here
        // For now, return true
        true
    }

    /// Check count constraint
    fn check_count_constraint(
        &self,
        element_type: &str,
        min: Option<usize>,
        max: Option<usize>,
        context: &ValidationContext,
    ) -> bool {
        let count = self.count_elements(element_type, context);

        if let Some(min_count) = min {
            if count < min_count {
                return false;
            }
        }

        if let Some(max_count) = max {
            if count > max_count {
                return false;
            }
        }

        true
    }

    /// Count elements of a type
    fn count_elements(&self, element_type: &str, context: &ValidationContext) -> usize {
        match element_type {
            "process" => context.diagram.processes.len(),
            "start_event" => context.diagram.processes.iter().map(|p| p.start_events.len()).sum(),
            "end_event" => context.diagram.processes.iter().map(|p| p.end_events.len()).sum(),
            "task" => context.diagram.processes.iter().map(|p| p.tasks.len()).sum(),
            "gateway" => context.diagram.processes.iter().map(|p| p.gateways.len()).sum(),
            _ => 0,
        }
    }
}

impl Default for ValidationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bpmn::elements::*;
    use crate::validation::rules::RuleCategory;

    #[test]
    fn test_validation_engine() {
        let mut engine = ValidationEngine::new();

        engine.register_rule(RuleDefinition::new(
            "BPMN_001",
            "Process must exist",
            "Diagram must contain at least one process",
            RuleSeverity::Critical,
            RuleCategory::BPMN,
            RuleCondition::MustExist { element_type: "process".to_string() },
        ));

        let diagram = BpmnDiagram {
            id: "test".to_string(),
            name: None,
            documentation: None,
            processes: vec![BpmnProcess {
                id: "p1".to_string(),
                name: None,
                documentation: None,
                is_executable: true,
                process_type: ProcessType::None,
                start_events: vec![],
                end_events: vec![],
                intermediate_events: vec![],
                tasks: vec![],
                subprocesses: vec![],
                gateways: vec![],
                sequence_flows: vec![],
                data_objects: vec![],
                data_associations: vec![],
                text_annotations: vec![],
                groups: vec![],
                lanes: vec![],
                metadata: std::collections::HashMap::new(),
            }],
            collaborations: vec![],
            data_stores: vec![],
            messages: vec![],
            signals: vec![],
            diagram_info: None,
        };

        let context = ValidationContext::new(&diagram);
        let result = engine.validate(&context);

        assert_eq!(result.total_rules, 1);
        assert_eq!(result.passed, 1);
        assert!(result.is_valid());
    }
}
