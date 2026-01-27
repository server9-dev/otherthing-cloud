//! Decision Table implementation
//!
//! DMN decision tables are the primary way to model decision logic.
//! This module implements the complete decision table structure with
//! support for all DMN 1.3 hit policies.
//!
//! ## Hit Policies
//!
//! - **UNIQUE**: Exactly one rule matches, all must be disjoint
//! - **FIRST**: First matching rule wins
//! - **PRIORITY**: Multiple rules can match, highest priority wins
//! - **ANY**: All matching rules must produce same output
//! - **COLLECT**: All matching rules, results collected
//! - **RULE ORDER**: All matching rules in definition order
//! - **OUTPUT ORDER**: All matching rules, ordered by output priority

use crate::dmn::errors::{DmnError, DmnResult};
use crate::dmn::expression::Expression;
use crate::dmn::feel::FeelValue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Decision table hit policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HitPolicy {
    /// Exactly one rule should match
    Unique,
    /// First matching rule wins
    First,
    /// Rules can overlap, highest priority wins
    Priority,
    /// All rules must produce same output
    Any,
    /// Collect all outputs
    Collect,
    /// Rules in definition order
    RuleOrder,
    /// Rules ordered by output priority
    OutputOrder,
}

impl Default for HitPolicy {
    fn default() -> Self {
        HitPolicy::Unique
    }
}

impl HitPolicy {
    /// Get hit policy name
    pub fn name(&self) -> &str {
        match self {
            HitPolicy::Unique => "U",
            HitPolicy::First => "F",
            HitPolicy::Priority => "P",
            HitPolicy::Any => "A",
            HitPolicy::Collect => "C",
            HitPolicy::RuleOrder => "R",
            HitPolicy::OutputOrder => "O",
        }
    }

    /// Parse hit policy from string
    pub fn from_str(s: &str) -> DmnResult<Self> {
        match s.to_uppercase().as_str() {
            "UNIQUE" | "U" => Ok(HitPolicy::Unique),
            "FIRST" | "F" => Ok(HitPolicy::First),
            "PRIORITY" | "P" => Ok(HitPolicy::Priority),
            "ANY" | "A" => Ok(HitPolicy::Any),
            "COLLECT" | "C" => Ok(HitPolicy::Collect),
            "RULE ORDER" | "RULEORDER" | "R" => Ok(HitPolicy::RuleOrder),
            "OUTPUT ORDER" | "OUTPUTORDER" | "O" => Ok(HitPolicy::OutputOrder),
            _ => Err(DmnError::InvalidDecisionTable(format!("Unknown hit policy: {}", s))),
        }
    }
}

/// Decision table input column definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionTableInput {
    /// Input identifier
    pub id: String,
    /// Input label/name
    pub label: String,
    /// Expression to evaluate (usually a variable reference)
    pub expression: Expression,
    /// Optional input type
    pub input_type: Option<String>,
}

/// Decision table output column definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionTableOutput {
    /// Output identifier
    pub id: String,
    /// Output label/name
    pub label: String,
    /// Output type
    pub output_type: Option<String>,
    /// Default value if no rules match
    pub default_value: Option<FeelValue>,
}

/// Entry in a decision table rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEntry {
    /// Input entries (conditions) - one per input column
    pub input_entries: Vec<Option<String>>,
    /// Output entries (conclusions) - one per output column
    pub output_entries: Vec<String>,
    /// Rule annotation/description
    pub annotation: Option<String>,
}

impl RuleEntry {
    /// Create a new rule entry
    pub fn new(num_inputs: usize, num_outputs: usize) -> Self {
        Self {
            input_entries: vec![None; num_inputs],
            output_entries: vec![String::new(); num_outputs],
            annotation: None,
        }
    }

    /// Set an input entry
    pub fn set_input(&mut self, index: usize, value: Option<String>) -> DmnResult<()> {
        if index >= self.input_entries.len() {
            return Err(DmnError::InvalidDecisionTable(format!(
                "Input index {} out of range",
                index
            )));
        }
        self.input_entries[index] = value;
        Ok(())
    }

    /// Set an output entry
    pub fn set_output(&mut self, index: usize, value: String) -> DmnResult<()> {
        if index >= self.output_entries.len() {
            return Err(DmnError::InvalidDecisionTable(format!(
                "Output index {} out of range",
                index
            )));
        }
        self.output_entries[index] = value;
        Ok(())
    }
}

/// Decision table definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionTable {
    /// Table identifier
    pub id: String,
    /// Hit policy
    pub hit_policy: HitPolicy,
    /// Aggregation function for COLLECT policy (e.g., SUM, COUNT)
    pub aggregation_function: Option<String>,
    /// Input columns
    pub inputs: Vec<DecisionTableInput>,
    /// Output columns
    pub outputs: Vec<DecisionTableOutput>,
    /// Decision rules
    pub rules: Vec<RuleEntry>,
}

impl DecisionTable {
    /// Create a new decision table
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            hit_policy: HitPolicy::default(),
            aggregation_function: None,
            inputs: Vec::new(),
            outputs: Vec::new(),
            rules: Vec::new(),
        }
    }

    /// Add an input column
    pub fn add_input(&mut self, input: DecisionTableInput) {
        self.inputs.push(input);
    }

    /// Add an output column
    pub fn add_output(&mut self, output: DecisionTableOutput) {
        self.outputs.push(output);
    }

    /// Add a rule
    pub fn add_rule(&mut self, rule: RuleEntry) -> DmnResult<()> {
        if rule.input_entries.len() != self.inputs.len() {
            return Err(DmnError::InvalidDecisionTable(format!(
                "Rule has {} inputs, expected {}",
                rule.input_entries.len(),
                self.inputs.len()
            )));
        }
        if rule.output_entries.len() != self.outputs.len() {
            return Err(DmnError::InvalidDecisionTable(format!(
                "Rule has {} outputs, expected {}",
                rule.output_entries.len(),
                self.outputs.len()
            )));
        }
        self.rules.push(rule);
        Ok(())
    }

    /// Validate the decision table structure
    pub fn validate(&self) -> DmnResult<()> {
        if self.inputs.is_empty() {
            return Err(DmnError::InvalidDecisionTable(
                "Decision table must have at least one input".to_string(),
            ));
        }

        if self.outputs.is_empty() {
            return Err(DmnError::InvalidDecisionTable(
                "Decision table must have at least one output".to_string(),
            ));
        }

        for rule in &self.rules {
            if rule.input_entries.len() != self.inputs.len() {
                return Err(DmnError::InvalidDecisionTable(format!(
                    "Rule has incorrect number of inputs: {} vs {}",
                    rule.input_entries.len(),
                    self.inputs.len()
                )));
            }
            if rule.output_entries.len() != self.outputs.len() {
                return Err(DmnError::InvalidDecisionTable(format!(
                    "Rule has incorrect number of outputs: {} vs {}",
                    rule.output_entries.len(),
                    self.outputs.len()
                )));
            }
        }

        Ok(())
    }

    /// Check which rules match the given inputs
    pub fn find_matching_rules(
        &self,
        inputs: &HashMap<String, FeelValue>,
    ) -> DmnResult<Vec<usize>> {
        let mut matching = Vec::new();

        for (rule_idx, rule) in self.rules.iter().enumerate() {
            if self.rule_matches(rule, inputs)? {
                matching.push(rule_idx);
            }
        }

        Ok(matching)
    }

    /// Check if a rule matches the inputs
    fn rule_matches(
        &self,
        rule: &RuleEntry,
        inputs: &HashMap<String, FeelValue>,
    ) -> DmnResult<bool> {
        for (col_idx, input_entry) in rule.input_entries.iter().enumerate() {
            let input = &self.inputs[col_idx];

            // Get the input value from context
            let input_id = input.id.clone();
            let input_value =
                inputs.get(&input_id).ok_or_else(|| DmnError::MissingInput(input_id.clone()))?;

            // If no condition, it's a wildcard (always matches)
            let Some(condition) = input_entry else {
                continue;
            };

            // Check if condition matches
            if !self.condition_matches(condition, input_value)? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Check if a condition matches an input value
    fn condition_matches(&self, condition: &str, value: &FeelValue) -> DmnResult<bool> {
        let condition = condition.trim();

        // Empty condition is a wildcard
        if condition.is_empty() || condition == "-" {
            return Ok(true);
        }

        // Exact match
        if let Ok(cond_val) = Expression::parse(condition) {
            if let crate::dmn::expression::Expression::Literal(lit) = cond_val {
                return value.equals(&lit);
            }
        }

        // Comparison operators
        if condition.starts_with("<=") {
            let val_str = condition[2..].trim();
            let val = Expression::parse(val_str)?;
            if let crate::dmn::expression::Expression::Literal(lit) = val {
                let num_val = value.to_number()?;
                let num_cond = lit.to_number()?;
                return Ok(num_val <= num_cond);
            }
        }

        if condition.starts_with(">=") {
            let val_str = condition[2..].trim();
            let val = Expression::parse(val_str)?;
            if let crate::dmn::expression::Expression::Literal(lit) = val {
                let num_val = value.to_number()?;
                let num_cond = lit.to_number()?;
                return Ok(num_val >= num_cond);
            }
        }

        if condition.starts_with("<") {
            let val_str = condition[1..].trim();
            let val = Expression::parse(val_str)?;
            if let crate::dmn::expression::Expression::Literal(lit) = val {
                let num_val = value.to_number()?;
                let num_cond = lit.to_number()?;
                return Ok(num_val < num_cond);
            }
        }

        if condition.starts_with(">") {
            let val_str = condition[1..].trim();
            let val = Expression::parse(val_str)?;
            if let crate::dmn::expression::Expression::Literal(lit) = val {
                let num_val = value.to_number()?;
                let num_cond = lit.to_number()?;
                return Ok(num_val > num_cond);
            }
        }

        // Default: treat as exact match
        let cond_val = Expression::parse(condition)?;
        if let crate::dmn::expression::Expression::Literal(lit) = cond_val {
            return value.equals(&lit);
        }

        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hit_policy_names() {
        assert_eq!(HitPolicy::Unique.name(), "U");
        assert_eq!(HitPolicy::First.name(), "F");
        assert_eq!(HitPolicy::Priority.name(), "P");
    }

    #[test]
    fn test_hit_policy_parsing() {
        assert_eq!(HitPolicy::from_str("UNIQUE").unwrap(), HitPolicy::Unique);
        assert_eq!(HitPolicy::from_str("F").unwrap(), HitPolicy::First);
        assert!(HitPolicy::from_str("INVALID").is_err());
    }

    #[test]
    fn test_decision_table_creation() {
        let table = DecisionTable::new("test_table");
        assert_eq!(table.id, "test_table");
        assert_eq!(table.hit_policy, HitPolicy::Unique);
        assert!(table.inputs.is_empty());
        assert!(table.outputs.is_empty());
    }

    #[test]
    fn test_rule_entry_creation() {
        let rule = RuleEntry::new(2, 1);
        assert_eq!(rule.input_entries.len(), 2);
        assert_eq!(rule.output_entries.len(), 1);
    }

    #[test]
    fn test_add_input_output() {
        let mut table = DecisionTable::new("test");

        let input = DecisionTableInput {
            id: "age".to_string(),
            label: "Age".to_string(),
            expression: Expression::Literal(FeelValue::Null),
            input_type: Some("number".to_string()),
        };
        table.add_input(input);

        let output = DecisionTableOutput {
            id: "category".to_string(),
            label: "Category".to_string(),
            output_type: Some("string".to_string()),
            default_value: None,
        };
        table.add_output(output);

        assert_eq!(table.inputs.len(), 1);
        assert_eq!(table.outputs.len(), 1);
    }

    #[test]
    fn test_validation_fails_no_inputs() {
        let table = DecisionTable::new("test");
        assert!(table.validate().is_err());
    }
}
