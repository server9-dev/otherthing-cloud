//! DMN 1.3 (Decision Model and Notation) Implementation
//!
//! This module implements DMN 1.3 elements for business decision modeling.
//! DMN provides reusable decision logic that can be invoked from BPMN processes or CMMN cases.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// DMN Decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmnDecision {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub decision_logic: DecisionLogic,
    pub input_data: Vec<InputData>,
    pub output_data: OutputData,
    pub knowledge_sources: Vec<String>,
}

/// Decision Logic types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DecisionLogic {
    /// Decision Table - the most common DMN element
    DecisionTable { table: DecisionTable },
    /// Literal Expression - simple expression
    LiteralExpression {
        expression: String,
        expression_language: String, // e.g., "FEEL", "JavaScript"
    },
    /// Invocation - calling another decision
    Invocation { invoked_decision: String, bindings: HashMap<String, String> },
    /// Decision Service
    DecisionService { output_decisions: Vec<String>, encapsulated_decisions: Vec<String> },
}

/// Decision Table - core DMN decision logic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionTable {
    pub hit_policy: HitPolicy,
    pub inputs: Vec<InputClause>,
    pub outputs: Vec<OutputClause>,
    pub rules: Vec<DecisionRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HitPolicy {
    /// Return first matching rule
    First,
    /// All matching rules must have same output
    Unique,
    /// Return all matching rules
    RuleOrder,
    /// Return outputs in descending order of output priority
    OutputOrder,
    /// Return any matching rule
    Any,
    /// Collect all matching outputs
    Collect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputClause {
    pub id: String,
    pub label: String,
    pub input_expression: String,
    pub input_values: Option<Vec<String>>, // Allowed values
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputClause {
    pub id: String,
    pub label: String,
    pub name: String,
    pub output_values: Option<Vec<String>>, // Allowed values
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRule {
    pub id: String,
    pub input_entries: Vec<String>,  // One per input clause
    pub output_entries: Vec<String>, // One per output clause
    pub annotation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputData {
    pub id: String,
    pub name: String,
    pub data_type: DataType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputData {
    pub id: String,
    pub name: String,
    pub data_type: DataType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    String,
    Number,
    Boolean,
    Date,
    Time,
    DateTime,
    Duration,
    Custom(String),
}

impl DmnDecision {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            decision_logic: DecisionLogic::LiteralExpression {
                expression: "true".to_string(),
                expression_language: "FEEL".to_string(),
            },
            input_data: Vec::new(),
            output_data: OutputData {
                id: "output1".to_string(),
                name: "result".to_string(),
                data_type: DataType::Boolean,
            },
            knowledge_sources: Vec::new(),
        }
    }

    pub fn with_decision_table(
        id: impl Into<String>,
        name: impl Into<String>,
        table: DecisionTable,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            decision_logic: DecisionLogic::DecisionTable { table },
            input_data: Vec::new(),
            output_data: OutputData {
                id: "output1".to_string(),
                name: "result".to_string(),
                data_type: DataType::String,
            },
            knowledge_sources: Vec::new(),
        }
    }

    pub fn add_input(&mut self, input: InputData) -> &mut Self {
        self.input_data.push(input);
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        // Validate based on decision logic type
        match &self.decision_logic {
            DecisionLogic::DecisionTable { table } => {
                // Check that rules match input/output clauses
                for rule in &table.rules {
                    if rule.input_entries.len() != table.inputs.len() {
                        return Err(format!(
                            "Rule '{}' has {} input entries but table has {} input clauses",
                            rule.id,
                            rule.input_entries.len(),
                            table.inputs.len()
                        ));
                    }
                    if rule.output_entries.len() != table.outputs.len() {
                        return Err(format!(
                            "Rule '{}' has {} output entries but table has {} output clauses",
                            rule.id,
                            rule.output_entries.len(),
                            table.outputs.len()
                        ));
                    }
                }

                // Check for at least one rule
                if table.rules.is_empty() {
                    return Err(format!("Decision table '{}' has no rules", self.name));
                }
            },
            DecisionLogic::LiteralExpression { expression, .. } => {
                if expression.is_empty() {
                    return Err(format!("Decision '{}' has empty expression", self.name));
                }
            },
            _ => {},
        }

        Ok(())
    }

    /// Execute the decision with given inputs
    pub fn execute(
        &self,
        inputs: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value, String> {
        match &self.decision_logic {
            DecisionLogic::DecisionTable { table } => self.execute_decision_table(table, inputs),
            DecisionLogic::LiteralExpression { expression, expression_language } => {
                // For now, just return a placeholder
                // In a real implementation, you'd evaluate the expression
                Ok(serde_json::json!({
                    "result": format!("Evaluated: {} ({})", expression, expression_language)
                }))
            },
            _ => Err("Decision logic type not yet implemented".to_string()),
        }
    }

    fn execute_decision_table(
        &self,
        table: &DecisionTable,
        inputs: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value, String> {
        let mut matching_rules = Vec::new();

        // Find matching rules
        for rule in &table.rules {
            if self.rule_matches(table, rule, inputs)? {
                matching_rules.push(rule);
            }
        }

        // Apply hit policy
        match table.hit_policy {
            HitPolicy::First => {
                if let Some(rule) = matching_rules.first() {
                    Ok(self.rule_to_output(table, rule))
                } else {
                    Err("No matching rules found".to_string())
                }
            },
            HitPolicy::Unique => {
                if matching_rules.is_empty() {
                    Err("No matching rules found".to_string())
                } else if matching_rules.len() > 1 {
                    Err("Multiple rules matched - violates UNIQUE hit policy".to_string())
                } else {
                    Ok(self.rule_to_output(table, matching_rules[0]))
                }
            },
            HitPolicy::RuleOrder | HitPolicy::Collect => {
                let outputs: Vec<_> =
                    matching_rules.iter().map(|rule| self.rule_to_output(table, rule)).collect();
                Ok(serde_json::json!(outputs))
            },
            _ => Err("Hit policy not yet implemented".to_string()),
        }
    }

    fn rule_matches(
        &self,
        table: &DecisionTable,
        rule: &DecisionRule,
        inputs: &HashMap<String, serde_json::Value>,
    ) -> Result<bool, String> {
        for (i, input_clause) in table.inputs.iter().enumerate() {
            let entry = &rule.input_entries[i];

            // "-" means any value matches
            if entry == "-" {
                continue;
            }

            // Simple string comparison for now
            // In a real implementation, you'd evaluate FEEL expressions
            if let Some(input_value) = inputs.get(&input_clause.label) {
                let input_str = input_value.to_string().trim_matches('"').to_string();
                if entry != &input_str {
                    return Ok(false);
                }
            } else {
                return Err(format!("Input '{}' not provided", input_clause.label));
            }
        }

        Ok(true)
    }

    fn rule_to_output(&self, table: &DecisionTable, rule: &DecisionRule) -> serde_json::Value {
        let mut output = serde_json::Map::new();
        for (i, output_clause) in table.outputs.iter().enumerate() {
            output.insert(
                output_clause.name.clone(),
                serde_json::Value::String(rule.output_entries[i].clone()),
            );
        }
        serde_json::Value::Object(output)
    }

    /// Export to DMN 1.3 XML (simplified)
    pub fn to_dmn_xml(&self) -> String {
        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<definitions xmlns=\"https://www.omg.org/spec/DMN/20191111/MODEL/\">\n");
        xml.push_str(&format!("  <decision id=\"{}\" name=\"{}\">\n", self.id, self.name));

        match &self.decision_logic {
            DecisionLogic::DecisionTable { table } => {
                xml.push_str("    <decisionTable>\n");
                for input in &table.inputs {
                    xml.push_str(&format!(
                        "      <input id=\"{}\" label=\"{}\">\n",
                        input.id, input.label
                    ));
                    xml.push_str(&format!(
                        "        <inputExpression>{}</inputExpression>\n",
                        input.input_expression
                    ));
                    xml.push_str("      </input>\n");
                }
                for output in &table.outputs {
                    xml.push_str(&format!(
                        "      <output id=\"{}\" label=\"{}\" name=\"{}\" />\n",
                        output.id, output.label, output.name
                    ));
                }
                for rule in &table.rules {
                    xml.push_str(&format!("      <rule id=\"{}\">\n", rule.id));
                    for entry in &rule.input_entries {
                        xml.push_str(&format!(
                            "        <inputEntry><text>{}</text></inputEntry>\n",
                            entry
                        ));
                    }
                    for entry in &rule.output_entries {
                        xml.push_str(&format!(
                            "        <outputEntry><text>{}</text></outputEntry>\n",
                            entry
                        ));
                    }
                    xml.push_str("      </rule>\n");
                }
                xml.push_str("    </decisionTable>\n");
            },
            _ => {
                xml.push_str(
                    "    <!-- Decision logic type not yet implemented in XML export -->\n",
                );
            },
        }

        xml.push_str("  </decision>\n");
        xml.push_str("</definitions>");
        xml
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dmn_decision_creation() {
        let decision = DmnDecision::new("decision1", "Approve Request");
        assert_eq!(decision.id, "decision1");
    }

    #[test]
    fn test_decision_table_validation() {
        let table = DecisionTable {
            hit_policy: HitPolicy::First,
            inputs: vec![InputClause {
                id: "input1".to_string(),
                label: "Amount".to_string(),
                input_expression: "amount".to_string(),
                input_values: None,
            }],
            outputs: vec![OutputClause {
                id: "output1".to_string(),
                label: "Decision".to_string(),
                name: "approved".to_string(),
                output_values: None,
            }],
            rules: vec![DecisionRule {
                id: "rule1".to_string(),
                input_entries: vec!["< 1000".to_string()],
                output_entries: vec!["true".to_string()],
                annotation: Some("Auto-approve small amounts".to_string()),
            }],
        };

        let decision = DmnDecision::with_decision_table("d1", "Test", table);
        assert!(decision.validate().is_ok());
    }

    #[test]
    fn test_decision_table_execution() {
        let table = DecisionTable {
            hit_policy: HitPolicy::First,
            inputs: vec![InputClause {
                id: "input1".to_string(),
                label: "customer_type".to_string(),
                input_expression: "customer.type".to_string(),
                input_values: None,
            }],
            outputs: vec![OutputClause {
                id: "output1".to_string(),
                label: "Discount".to_string(),
                name: "discount".to_string(),
                output_values: None,
            }],
            rules: vec![
                DecisionRule {
                    id: "rule1".to_string(),
                    input_entries: vec!["VIP".to_string()],
                    output_entries: vec!["20%".to_string()],
                    annotation: None,
                },
                DecisionRule {
                    id: "rule2".to_string(),
                    input_entries: vec!["Regular".to_string()],
                    output_entries: vec!["10%".to_string()],
                    annotation: None,
                },
            ],
        };

        let decision = DmnDecision::with_decision_table("d1", "Discount Decision", table);

        let mut inputs = HashMap::new();
        inputs.insert("customer_type".to_string(), serde_json::json!("VIP"));

        let result = decision.execute(&inputs).unwrap();
        assert!(result.to_string().contains("20%"));
    }
}
