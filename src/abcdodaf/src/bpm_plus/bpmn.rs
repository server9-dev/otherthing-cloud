//! BPMN 2.0 (Business Process Model and Notation) Implementation
//!
//! This module implements BPMN 2.0 elements for process modeling.
//! BPMN is used for modeling structured, repeatable processes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// BPMN 2.0 Process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BpmnProcess {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_executable: bool,
    pub flow_elements: Vec<FlowElement>,
    pub data_objects: HashMap<String, DataObject>,
}

/// BPMN Flow Elements
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FlowElement {
    /// Start Event
    StartEvent { id: String, name: String, outgoing: Vec<String> },
    /// End Event
    EndEvent { id: String, name: String, incoming: Vec<String> },
    /// User Task
    UserTask {
        id: String,
        name: String,
        assignee: Option<String>,
        incoming: Vec<String>,
        outgoing: Vec<String>,
        documentation: Option<String>,
    },
    /// Service Task (automated)
    ServiceTask {
        id: String,
        name: String,
        implementation: ServiceImplementation,
        incoming: Vec<String>,
        outgoing: Vec<String>,
    },
    /// Business Rule Task (DMN)
    BusinessRuleTask {
        id: String,
        name: String,
        decision_ref: Option<String>, // Reference to DMN decision
        incoming: Vec<String>,
        outgoing: Vec<String>,
    },
    /// Script Task
    ScriptTask {
        id: String,
        name: String,
        script_format: String,
        script: String,
        incoming: Vec<String>,
        outgoing: Vec<String>,
    },
    /// Exclusive Gateway (XOR)
    ExclusiveGateway {
        id: String,
        name: String,
        incoming: Vec<String>,
        outgoing: Vec<String>,
        default_flow: Option<String>,
    },
    /// Parallel Gateway (AND)
    ParallelGateway { id: String, name: String, incoming: Vec<String>, outgoing: Vec<String> },
    /// Inclusive Gateway (OR)
    InclusiveGateway { id: String, name: String, incoming: Vec<String>, outgoing: Vec<String> },
    /// Sequence Flow
    SequenceFlow {
        id: String,
        name: Option<String>,
        source_ref: String,
        target_ref: String,
        condition: Option<String>,
    },
    /// Sub-Process
    SubProcess {
        id: String,
        name: String,
        flow_elements: Vec<FlowElement>,
        incoming: Vec<String>,
        outgoing: Vec<String>,
    },
    /// Call Activity (invoke another process or case)
    CallActivity {
        id: String,
        name: String,
        called_element: String, // Process ID or Case ID
        incoming: Vec<String>,
        outgoing: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServiceImplementation {
    /// External service (e.g., REST API)
    External { endpoint: String },
    /// Agent-based service
    Agent { agent_type: String },
    /// Expression
    Expression { expression: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataObject {
    pub id: String,
    pub name: String,
    pub structure_ref: Option<String>,
}

impl BpmnProcess {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            is_executable: true,
            flow_elements: Vec::new(),
            data_objects: HashMap::new(),
        }
    }

    pub fn add_flow_element(&mut self, element: FlowElement) -> &mut Self {
        self.flow_elements.push(element);
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        // Check for at least one start event
        let start_events: Vec<_> = self
            .flow_elements
            .iter()
            .filter(|e| matches!(e, FlowElement::StartEvent { .. }))
            .collect();

        if start_events.is_empty() {
            return Err(format!("Process '{}' must have at least one start event", self.name));
        }

        // Check for at least one end event
        let end_events: Vec<_> = self
            .flow_elements
            .iter()
            .filter(|e| matches!(e, FlowElement::EndEvent { .. }))
            .collect();

        if end_events.is_empty() {
            return Err(format!("Process '{}' must have at least one end event", self.name));
        }

        // Validate all flow connections
        for element in &self.flow_elements {
            match element {
                FlowElement::SequenceFlow { source_ref, target_ref, .. } => {
                    if !self.has_element(source_ref) {
                        return Err(format!("Source element '{}' not found", source_ref));
                    }
                    if !self.has_element(target_ref) {
                        return Err(format!("Target element '{}' not found", target_ref));
                    }
                },
                _ => {},
            }
        }

        Ok(())
    }

    pub(crate) fn has_element(&self, element_id: &str) -> bool {
        self.flow_elements.iter().any(|e| match e {
            FlowElement::StartEvent { id, .. } => id == element_id,
            FlowElement::EndEvent { id, .. } => id == element_id,
            FlowElement::UserTask { id, .. } => id == element_id,
            FlowElement::ServiceTask { id, .. } => id == element_id,
            FlowElement::BusinessRuleTask { id, .. } => id == element_id,
            FlowElement::ScriptTask { id, .. } => id == element_id,
            FlowElement::ExclusiveGateway { id, .. } => id == element_id,
            FlowElement::ParallelGateway { id, .. } => id == element_id,
            FlowElement::InclusiveGateway { id, .. } => id == element_id,
            FlowElement::SubProcess { id, .. } => id == element_id,
            FlowElement::CallActivity { id, .. } => id == element_id,
            FlowElement::SequenceFlow { id, .. } => id == element_id,
        })
    }

    /// Export to BPMN 2.0 XML (simplified)
    pub fn to_bpmn_xml(&self) -> String {
        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<definitions xmlns=\"http://www.omg.org/spec/BPMN/20100524/MODEL\"\n");
        xml.push_str("             xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\">\n");
        xml.push_str(&format!(
            "  <process id=\"{}\" name=\"{}\" isExecutable=\"{}\">\n",
            self.id, self.name, self.is_executable
        ));

        for element in &self.flow_elements {
            xml.push_str(&self.element_to_xml(element, 4));
        }

        xml.push_str("  </process>\n");
        xml.push_str("</definitions>");
        xml
    }

    fn element_to_xml(&self, element: &FlowElement, indent: usize) -> String {
        let spaces = " ".repeat(indent);
        match element {
            FlowElement::StartEvent { id, name, .. } => {
                format!("{}<startEvent id=\"{}\" name=\"{}\" />\n", spaces, id, name)
            },
            FlowElement::EndEvent { id, name, .. } => {
                format!("{}<endEvent id=\"{}\" name=\"{}\" />\n", spaces, id, name)
            },
            FlowElement::UserTask { id, name, .. } => {
                format!("{}<userTask id=\"{}\" name=\"{}\" />\n", spaces, id, name)
            },
            FlowElement::ServiceTask { id, name, .. } => {
                format!("{}<serviceTask id=\"{}\" name=\"{}\" />\n", spaces, id, name)
            },
            FlowElement::BusinessRuleTask { id, name, decision_ref, .. } => {
                if let Some(ref_id) = decision_ref {
                    format!(
                        "{}<businessRuleTask id=\"{}\" name=\"{}\" calledDecision=\"{}\" />\n",
                        spaces, id, name, ref_id
                    )
                } else {
                    format!("{}<businessRuleTask id=\"{}\" name=\"{}\" />\n", spaces, id, name)
                }
            },
            FlowElement::SequenceFlow { id, source_ref, target_ref, .. } => {
                format!(
                    "{}<sequenceFlow id=\"{}\" sourceRef=\"{}\" targetRef=\"{}\" />\n",
                    spaces, id, source_ref, target_ref
                )
            },
            FlowElement::ExclusiveGateway { id, name, .. } => {
                format!("{}<exclusiveGateway id=\"{}\" name=\"{}\" />\n", spaces, id, name)
            },
            FlowElement::ParallelGateway { id, name, .. } => {
                format!("{}<parallelGateway id=\"{}\" name=\"{}\" />\n", spaces, id, name)
            },
            _ => format!(
                "{}<!-- Element {} not yet implemented in XML export -->\n",
                spaces,
                self.get_element_id(element)
            ),
        }
    }

    fn get_element_id<'a>(&self, element: &'a FlowElement) -> &'a str {
        match element {
            FlowElement::StartEvent { id, .. } => id,
            FlowElement::EndEvent { id, .. } => id,
            FlowElement::UserTask { id, .. } => id,
            FlowElement::ServiceTask { id, .. } => id,
            FlowElement::BusinessRuleTask { id, .. } => id,
            FlowElement::ScriptTask { id, .. } => id,
            FlowElement::ExclusiveGateway { id, .. } => id,
            FlowElement::ParallelGateway { id, .. } => id,
            FlowElement::InclusiveGateway { id, .. } => id,
            FlowElement::SubProcess { id, .. } => id,
            FlowElement::CallActivity { id, .. } => id,
            FlowElement::SequenceFlow { id, .. } => id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bpmn_process_creation() {
        let mut process = BpmnProcess::new("proc1", "Test Process");

        process.add_flow_element(FlowElement::StartEvent {
            id: "start1".to_string(),
            name: "Start".to_string(),
            outgoing: vec!["flow1".to_string()],
        });

        process.add_flow_element(FlowElement::UserTask {
            id: "task1".to_string(),
            name: "Review".to_string(),
            assignee: None,
            incoming: vec!["flow1".to_string()],
            outgoing: vec!["flow2".to_string()],
            documentation: None,
        });

        process.add_flow_element(FlowElement::EndEvent {
            id: "end1".to_string(),
            name: "End".to_string(),
            incoming: vec!["flow2".to_string()],
        });

        process.add_flow_element(FlowElement::SequenceFlow {
            id: "flow1".to_string(),
            name: None,
            source_ref: "start1".to_string(),
            target_ref: "task1".to_string(),
            condition: None,
        });

        process.add_flow_element(FlowElement::SequenceFlow {
            id: "flow2".to_string(),
            name: None,
            source_ref: "task1".to_string(),
            target_ref: "end1".to_string(),
            condition: None,
        });

        assert!(process.validate().is_ok());
    }

    #[test]
    fn test_bpmn_xml_export() {
        let mut process = BpmnProcess::new("proc1", "Test");
        process.add_flow_element(FlowElement::StartEvent {
            id: "start".to_string(),
            name: "Start".to_string(),
            outgoing: vec![],
        });

        let xml = process.to_bpmn_xml();
        assert!(xml.contains("<process"));
        assert!(xml.contains("<startEvent"));
    }
}
