//! DMN XML import/export support
//!
//! This module handles serialization and deserialization of DMN models
//! to/from XML format according to the DMN 1.3 specification.
//!
//! Note: Full XML support requires a dedicated XML library (xml-rs, quick-xml, etc.)
//! For now, we provide the infrastructure and basic serialization.

use crate::dmn::decision_graph::DecisionGraph;
use crate::dmn::decision_table::DecisionTable;
use crate::dmn::errors::{DmnError, DmnResult};
use serde_json::json;

/// DMN XML serializer/deserializer
pub struct DmnXmlManager;

impl DmnXmlManager {
    /// Export a decision table to JSON representation (XML serialization)
    /// In production, this would use a proper XML library
    pub fn export_table_json(table: &DecisionTable) -> DmnResult<String> {
        let json_obj = json!({
            "id": table.id,
            "hitPolicy": table.hit_policy.name(),
            "inputs": table.inputs.iter().map(|i| {
                json!({
                    "id": i.id,
                    "label": i.label,
                    "type": i.input_type
                })
            }).collect::<Vec<_>>(),
            "outputs": table.outputs.iter().map(|o| {
                json!({
                    "id": o.id,
                    "label": o.label,
                    "type": o.output_type
                })
            }).collect::<Vec<_>>(),
            "rules": table.rules.iter().map(|r| {
                json!({
                    "inputs": r.input_entries,
                    "outputs": r.output_entries,
                    "annotation": r.annotation
                })
            }).collect::<Vec<_>>()
        });

        Ok(json_obj.to_string())
    }

    /// Export a decision graph to JSON representation
    pub fn export_graph_json(graph: &DecisionGraph) -> DmnResult<String> {
        let nodes: Vec<_> = graph
            .nodes()
            .map(|node| {
                json!({
                    "id": node.id(),
                    "name": node.name(),
                    "type": node.node_type()
                })
            })
            .collect();

        let requirements: Vec<_> = graph
            .requirements()
            .iter()
            .map(|req| {
                json!({
                    "source": req.source_id,
                    "target": req.target_id
                })
            })
            .collect();

        let json_obj = json!({
            "id": graph.id,
            "nodes": nodes,
            "requirements": requirements
        });

        Ok(json_obj.to_string())
    }

    /// Generate a DMN XML string (simplified version)
    pub fn export_table_xml(table: &DecisionTable) -> DmnResult<String> {
        let mut xml = String::new();

        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<definitions xmlns=\"http://www.omg.org/spec/DMN/20151101/dmn.xsd\">\n");
        xml.push_str(&format!("  <decision id=\"{}\">\n", table.id));
        xml.push_str(&format!("    <decisionTable hitPolicy=\"{}\">\n", table.hit_policy.name()));

        // Input columns
        for input in &table.inputs {
            xml.push_str(&format!("      <input id=\"{}\">\n", input.id));
            xml.push_str(&format!("        <inputExpression>\n"));
            xml.push_str(&format!("          <text>{}</text>\n", input.label));
            xml.push_str(&format!("        </inputExpression>\n"));
            xml.push_str(&format!("      </input>\n"));
        }

        // Output columns
        for output in &table.outputs {
            xml.push_str(&format!(
                "      <output id=\"{}\" label=\"{}\" />\n",
                output.id, output.label
            ));
        }

        // Rules
        for rule in &table.rules {
            xml.push_str(&format!("      <rule>\n"));
            for entry in &rule.input_entries {
                if let Some(e) = entry {
                    xml.push_str(&format!("        <inputEntry>\n"));
                    xml.push_str(&format!("          <text>{}</text>\n", e));
                    xml.push_str(&format!("        </inputEntry>\n"));
                } else {
                    xml.push_str(&format!("        <inputEntry>\n"));
                    xml.push_str(&format!("          <text>-</text>\n"));
                    xml.push_str(&format!("        </inputEntry>\n"));
                }
            }
            for entry in &rule.output_entries {
                xml.push_str(&format!("        <outputEntry>\n"));
                xml.push_str(&format!("          <text>{}</text>\n", entry));
                xml.push_str(&format!("        </outputEntry>\n"));
            }
            xml.push_str(&format!("      </rule>\n"));
        }

        xml.push_str("    </decisionTable>\n");
        xml.push_str("  </decision>\n");
        xml.push_str("</definitions>\n");

        Ok(xml)
    }

    /// Parse a DMN model from XML (returns JSON representation)
    pub fn import_from_json(json_str: &str) -> DmnResult<serde_json::Value> {
        serde_json::from_str(json_str)
            .map_err(|e| DmnError::XmlParsingError(format!("JSON parse error: {}", e)))
    }

    /// Validate a DMN XML document
    pub fn validate_xml(xml: &str) -> DmnResult<()> {
        if !xml.contains("<?xml") {
            return Err(DmnError::XmlParsingError(
                "Invalid XML: missing XML declaration".to_string(),
            ));
        }

        if !xml.contains("<definitions") {
            return Err(DmnError::XmlParsingError(
                "Invalid DMN: missing definitions element".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dmn::decision_table::{DecisionTableInput, DecisionTableOutput, RuleEntry};
    use crate::dmn::expression::Expression;
    use crate::dmn::feel::FeelValue;

    fn create_test_table() -> DecisionTable {
        let mut table = DecisionTable::new("test_table");

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

        let mut rule = RuleEntry::new(1, 1);
        rule.set_input(0, Some(">= 18".to_string())).unwrap();
        rule.set_output(0, "\"Adult\"".to_string()).unwrap();
        table.add_rule(rule).unwrap();

        table
    }

    #[test]
    fn test_export_table_json() {
        let table = create_test_table();
        let json_str = DmnXmlManager::export_table_json(&table).unwrap();

        let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(json["id"], "test_table");
        assert_eq!(json["inputs"][0]["id"], "age");
        assert_eq!(json["outputs"][0]["id"], "category");
    }

    #[test]
    fn test_export_table_xml() {
        let table = create_test_table();
        let xml = DmnXmlManager::export_table_xml(&table).unwrap();

        assert!(xml.contains("<?xml"));
        assert!(xml.contains("test_table"));
        assert!(xml.contains("decisionTable"));
        assert!(xml.contains("Age"));
    }

    #[test]
    fn test_validate_xml_valid() {
        let xml = "<?xml version=\"1.0\"?>\n<definitions></definitions>";
        assert!(DmnXmlManager::validate_xml(xml).is_ok());
    }

    #[test]
    fn test_validate_xml_missing_declaration() {
        let xml = "<definitions></definitions>";
        assert!(DmnXmlManager::validate_xml(xml).is_err());
    }

    #[test]
    fn test_validate_xml_missing_definitions() {
        let xml = "<?xml version=\"1.0\"?>\n<root></root>";
        assert!(DmnXmlManager::validate_xml(xml).is_err());
    }

    #[test]
    fn test_import_from_json() {
        let json = r#"{"id": "test", "value": 42}"#;
        let result = DmnXmlManager::import_from_json(json).unwrap();
        assert_eq!(result["id"], "test");
        assert_eq!(result["value"], 42);
    }
}
