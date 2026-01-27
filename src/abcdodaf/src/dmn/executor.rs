//! Decision executor for running decision tables and decision graphs

use crate::dmn::decision_table::{DecisionTable, HitPolicy};
use crate::dmn::errors::{DmnError, DmnResult};
use crate::dmn::expression::ExpressionEvaluator;
use crate::dmn::feel::FeelValue;
use std::collections::HashMap;

/// Result of executing a decision rule
#[derive(Debug, Clone)]
pub struct DecisionRuleResult {
    /// Index of the matching rule
    pub rule_index: usize,
    /// Output values from the rule
    pub outputs: HashMap<String, FeelValue>,
}

/// Result of executing a decision table
#[derive(Debug, Clone)]
pub struct DecisionTableResult {
    /// All matching rule results
    pub results: Vec<DecisionRuleResult>,
    /// Final consolidated output based on hit policy
    pub final_output: HashMap<String, FeelValue>,
}

/// Executor for running DMN decision tables and graphs
pub struct DecisionExecutor {
    context: HashMap<String, FeelValue>,
}

impl DecisionExecutor {
    /// Create a new executor
    pub fn new() -> Self {
        Self { context: HashMap::new() }
    }

    /// Create with initial context
    pub fn with_context(context: HashMap<String, FeelValue>) -> Self {
        Self { context }
    }

    /// Set an input variable
    pub fn set_input(&mut self, name: String, value: FeelValue) {
        self.context.insert(name, value);
    }

    /// Execute a decision table
    pub fn execute_table(&self, table: &DecisionTable) -> DmnResult<DecisionTableResult> {
        // Validate the table
        table.validate()?;

        // Find matching rules
        let matching_rules = table.find_matching_rules(&self.context)?;

        // Apply hit policy
        let results = match table.hit_policy {
            HitPolicy::Unique => self.apply_unique_policy(&matching_rules, table)?,
            HitPolicy::First => self.apply_first_policy(&matching_rules, table)?,
            HitPolicy::Priority => self.apply_priority_policy(&matching_rules, table)?,
            HitPolicy::Any => self.apply_any_policy(&matching_rules, table)?,
            HitPolicy::Collect => self.apply_collect_policy(&matching_rules, table)?,
            HitPolicy::RuleOrder => self.apply_rule_order_policy(&matching_rules, table)?,
            HitPolicy::OutputOrder => self.apply_output_order_policy(&matching_rules, table)?,
        };

        if results.is_empty() {
            // Check for default values
            let mut final_output = HashMap::new();
            for output in &table.outputs {
                if let Some(default) = &output.default_value {
                    final_output.insert(output.id.clone(), default.clone());
                }
            }

            if final_output.is_empty() {
                return Err(DmnError::NoMatchingRules);
            }

            return Ok(DecisionTableResult { results: Vec::new(), final_output });
        }

        // Consolidate results based on hit policy
        let final_output = self.consolidate_outputs(&results, table)?;

        Ok(DecisionTableResult { results, final_output })
    }

    /// Apply UNIQUE hit policy - exactly one rule must match
    fn apply_unique_policy(
        &self,
        matching_rules: &[usize],
        table: &DecisionTable,
    ) -> DmnResult<Vec<DecisionRuleResult>> {
        if matching_rules.len() != 1 {
            return Err(DmnError::AmbiguousRules(format!(
                "UNIQUE policy requires exactly one match, got {}",
                matching_rules.len()
            )));
        }

        let rule_idx = matching_rules[0];
        let rule = &table.rules[rule_idx];

        let mut outputs = HashMap::new();
        for (output_idx, output) in table.outputs.iter().enumerate() {
            let output_expr = &rule.output_entries[output_idx];
            let value = self.evaluate_output_entry(output_expr)?;
            outputs.insert(output.id.clone(), value);
        }

        Ok(vec![DecisionRuleResult { rule_index: rule_idx, outputs }])
    }

    /// Apply FIRST hit policy - first matching rule wins
    fn apply_first_policy(
        &self,
        matching_rules: &[usize],
        table: &DecisionTable,
    ) -> DmnResult<Vec<DecisionRuleResult>> {
        if matching_rules.is_empty() {
            return Ok(Vec::new());
        }

        let rule_idx = matching_rules[0];
        let rule = &table.rules[rule_idx];

        let mut outputs = HashMap::new();
        for (output_idx, output) in table.outputs.iter().enumerate() {
            let output_expr = &rule.output_entries[output_idx];
            let value = self.evaluate_output_entry(output_expr)?;
            outputs.insert(output.id.clone(), value);
        }

        Ok(vec![DecisionRuleResult { rule_index: rule_idx, outputs }])
    }

    /// Apply PRIORITY hit policy - highest priority output wins
    fn apply_priority_policy(
        &self,
        matching_rules: &[usize],
        table: &DecisionTable,
    ) -> DmnResult<Vec<DecisionRuleResult>> {
        // For now, use first rule (full priority would use output ordering)
        self.apply_first_policy(matching_rules, table)
    }

    /// Apply ANY hit policy - all must produce same output
    fn apply_any_policy(
        &self,
        matching_rules: &[usize],
        table: &DecisionTable,
    ) -> DmnResult<Vec<DecisionRuleResult>> {
        if matching_rules.is_empty() {
            return Ok(Vec::new());
        }

        let mut all_results = Vec::new();
        let mut first_output = None;

        for rule_idx in matching_rules {
            let rule = &table.rules[*rule_idx];
            let mut outputs = HashMap::new();

            for (output_idx, output) in table.outputs.iter().enumerate() {
                let output_expr = &rule.output_entries[output_idx];
                let value = self.evaluate_output_entry(output_expr)?;
                outputs.insert(output.id.clone(), value);
            }

            // Check if all outputs are the same
            if let Some(ref first) = first_output {
                if &outputs != first {
                    return Err(DmnError::AmbiguousRules(
                        "ANY policy requires all matching rules to produce same output".to_string(),
                    ));
                }
            } else {
                first_output = Some(outputs.clone());
            }

            all_results.push(DecisionRuleResult { rule_index: *rule_idx, outputs });
        }

        Ok(all_results)
    }

    /// Apply COLLECT hit policy - collect all outputs
    fn apply_collect_policy(
        &self,
        matching_rules: &[usize],
        table: &DecisionTable,
    ) -> DmnResult<Vec<DecisionRuleResult>> {
        let mut all_results = Vec::new();

        for rule_idx in matching_rules {
            let rule = &table.rules[*rule_idx];
            let mut outputs = HashMap::new();

            for (output_idx, output) in table.outputs.iter().enumerate() {
                let output_expr = &rule.output_entries[output_idx];
                let value = self.evaluate_output_entry(output_expr)?;
                outputs.insert(output.id.clone(), value);
            }

            all_results.push(DecisionRuleResult { rule_index: *rule_idx, outputs });
        }

        Ok(all_results)
    }

    /// Apply RULE ORDER hit policy - all in definition order
    fn apply_rule_order_policy(
        &self,
        matching_rules: &[usize],
        table: &DecisionTable,
    ) -> DmnResult<Vec<DecisionRuleResult>> {
        self.apply_collect_policy(matching_rules, table)
    }

    /// Apply OUTPUT ORDER hit policy - all ordered by output priority
    fn apply_output_order_policy(
        &self,
        matching_rules: &[usize],
        table: &DecisionTable,
    ) -> DmnResult<Vec<DecisionRuleResult>> {
        // For now, same as COLLECT (full implementation would sort by priority)
        self.apply_collect_policy(matching_rules, table)
    }

    /// Evaluate an output entry expression
    fn evaluate_output_entry(&self, entry: &str) -> DmnResult<FeelValue> {
        let evaluator = ExpressionEvaluator::with_context(self.context.clone());
        let expr = crate::dmn::expression::Expression::parse(entry)?;
        evaluator.evaluate(&expr)
    }

    /// Consolidate multiple results into final output
    fn consolidate_outputs(
        &self,
        results: &[DecisionRuleResult],
        table: &DecisionTable,
    ) -> DmnResult<HashMap<String, FeelValue>> {
        if results.is_empty() {
            return Ok(HashMap::new());
        }

        // For single result, just return it
        if results.len() == 1 {
            return Ok(results[0].outputs.clone());
        }

        // For multiple results, create a list of outputs for each key
        let mut consolidated = HashMap::new();

        for output in &table.outputs {
            let mut values = Vec::new();
            for result in results {
                if let Some(val) = result.outputs.get(&output.id) {
                    values.push(val.clone());
                }
            }

            if !values.is_empty() {
                consolidated.insert(output.id.clone(), FeelValue::List(values));
            }
        }

        Ok(consolidated)
    }
}

impl Default for DecisionExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dmn::decision_table::{DecisionTableInput, DecisionTableOutput, RuleEntry};
    use crate::dmn::expression::Expression;

    fn create_test_table() -> DecisionTable {
        let mut table = DecisionTable::new("test_table");
        table.hit_policy = HitPolicy::First;

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

        let mut rule1 = RuleEntry::new(1, 1);
        rule1.set_input(0, Some("< 18".to_string())).unwrap();
        rule1.set_output(0, "\"Minor\"".to_string()).unwrap();
        table.add_rule(rule1).unwrap();

        let mut rule2 = RuleEntry::new(1, 1);
        rule2.set_input(0, Some(">= 18".to_string())).unwrap();
        rule2.set_output(0, "\"Adult\"".to_string()).unwrap();
        table.add_rule(rule2).unwrap();

        table
    }

    #[test]
    fn test_executor_creation() {
        let executor = DecisionExecutor::new();
        assert!(executor.context.is_empty());
    }

    #[test]
    fn test_set_input() {
        let mut executor = DecisionExecutor::new();
        executor.set_input("age".to_string(), FeelValue::Number(25.0));
        assert_eq!(executor.context.get("age"), Some(&FeelValue::Number(25.0)));
    }

    #[test]
    fn test_execute_first_policy() {
        let table = create_test_table();
        let mut executor = DecisionExecutor::new();
        executor.set_input("age".to_string(), FeelValue::Number(25.0));

        let result = executor.execute_table(&table).unwrap();
        assert!(!result.results.is_empty());
    }
}
