//! Overall workflow structure validation
//!
//! Validates high-level workflow structure including start/end events,
//! disconnected nodes, and gateway connection rules.

use crate::bpmn::json_format::BpmnJsonWorkflow;
use std::collections::{HashMap, HashSet};

use super::types::{ErrorCategory, ValidationError};

/// Validate overall workflow structure
pub fn validate_workflow_structure(workflow: &BpmnJsonWorkflow) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Check for start events
    let start_events: Vec<_> =
        workflow.workflow_steps.iter().filter(|s| s.step_type == "startEvent").collect();

    if start_events.is_empty() {
        errors.push(
            ValidationError::error(ErrorCategory::StructuralIssue, "Workflow has no start event")
                .with_suggestion("Add at least one startEvent node"),
        );
    }

    // Check for end events
    let end_events: Vec<_> =
        workflow.workflow_steps.iter().filter(|s| s.step_type == "endEvent").collect();

    if end_events.is_empty() {
        errors.push(
            ValidationError::error(ErrorCategory::StructuralIssue, "Workflow has no end event")
                .with_suggestion("Add at least one endEvent node"),
        );
    }

    // Check for disconnected nodes
    errors.extend(check_disconnected_nodes(workflow));

    // Check for gateway validation issues
    errors.extend(validate_gateway_connections(workflow));

    errors
}

/// Check for nodes that are not connected to any flows
fn check_disconnected_nodes(workflow: &BpmnJsonWorkflow) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Build sets of connected nodes
    let mut connected_nodes = HashSet::new();
    for flow in &workflow.sequence_flows {
        connected_nodes.insert(flow.source_ref.clone());
        connected_nodes.insert(flow.target_ref.clone());
    }

    // Check each node
    for step in &workflow.workflow_steps {
        if !connected_nodes.contains(&step.id) {
            // Exception: Single start or end event workflows
            if workflow.workflow_steps.len() == 1 {
                continue;
            }

            errors.push(
                ValidationError::warning(
                    ErrorCategory::StructuralIssue,
                    format!("Node '{}' ({}) is not connected to any flows", step.id, step.name),
                )
                .with_context(&step.id)
                .with_suggestion("Connect this node with sequence flows or remove it"),
            );
        }
    }

    errors
}

/// Validate gateway-specific connection rules
fn validate_gateway_connections(workflow: &BpmnJsonWorkflow) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Count incoming and outgoing flows for each node
    let mut incoming_counts: HashMap<String, usize> = HashMap::new();
    let mut outgoing_counts: HashMap<String, usize> = HashMap::new();

    for flow in &workflow.sequence_flows {
        *outgoing_counts.entry(flow.source_ref.clone()).or_insert(0) += 1;
        *incoming_counts.entry(flow.target_ref.clone()).or_insert(0) += 1;
    }

    for step in &workflow.workflow_steps {
        let incoming = incoming_counts.get(&step.id).copied().unwrap_or(0);
        let outgoing = outgoing_counts.get(&step.id).copied().unwrap_or(0);

        match step.step_type.as_str() {
            "startEvent" => {
                if incoming > 0 {
                    errors.push(
                        ValidationError::error(
                            ErrorCategory::StructuralIssue,
                            format!("Start event '{}' has incoming flows", step.id),
                        )
                        .with_context(&step.id)
                        .with_suggestion("Start events should not have incoming sequence flows"),
                    );
                }
                if outgoing == 0 {
                    errors.push(
                        ValidationError::warning(
                            ErrorCategory::StructuralIssue,
                            format!("Start event '{}' has no outgoing flows", step.id),
                        )
                        .with_context(&step.id)
                        .with_suggestion("Start events should have at least one outgoing flow"),
                    );
                }
            },
            "endEvent" => {
                if outgoing > 0 {
                    errors.push(
                        ValidationError::error(
                            ErrorCategory::StructuralIssue,
                            format!("End event '{}' has outgoing flows", step.id),
                        )
                        .with_context(&step.id)
                        .with_suggestion("End events should not have outgoing sequence flows"),
                    );
                }
                if incoming == 0 {
                    errors.push(
                        ValidationError::warning(
                            ErrorCategory::StructuralIssue,
                            format!("End event '{}' has no incoming flows", step.id),
                        )
                        .with_context(&step.id)
                        .with_suggestion("End events should have at least one incoming flow"),
                    );
                }
            },
            "exclusiveGateway" | "parallelGateway" | "inclusiveGateway" => {
                if incoming == 0 && outgoing == 0 {
                    errors.push(
                        ValidationError::warning(
                            ErrorCategory::StructuralIssue,
                            format!("Gateway '{}' has no connections", step.id),
                        )
                        .with_context(&step.id)
                        .with_suggestion("Gateways should have both incoming and outgoing flows"),
                    );
                } else if outgoing == 1 && incoming > 1 {
                    // Converging gateway - OK
                } else if incoming == 1 && outgoing > 1 {
                    // Diverging gateway - OK
                } else if incoming > 1 && outgoing > 1 {
                    // Mixed gateway - warning
                    errors.push(
                        ValidationError::info(
                            ErrorCategory::StructuralIssue,
                            format!("Gateway '{}' is used as both merge and split", step.id),
                        )
                        .with_context(&step.id)
                        .with_suggestion(
                            "Consider using separate gateways for merge and split operations",
                        ),
                    );
                }
            },
            _ => {
                // Regular tasks
                if incoming == 0 && outgoing == 0 && workflow.workflow_steps.len() > 1 {
                    // Already handled in disconnected nodes check
                }
            },
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bpmn::validation::types::ErrorSeverity;
    use crate::bpmn::json_format::{BpmnProcessInfo, SequenceFlow, WorkflowStep};

    // Helper function to create a basic WorkflowStep
    fn create_step(id: &str, name: &str, step_type: &str) -> WorkflowStep {
        WorkflowStep {
            id: id.to_string(),
            name: name.to_string(),
            step_type: step_type.to_string(),
            task_type: None,
            event_type: None,
            implementation: None,
            inputs: vec![],
            outputs: vec![],
            duration_minutes: None,
        }
    }

    // Helper function to create a basic SequenceFlow
    fn create_flow(id: &str, source: &str, target: &str) -> SequenceFlow {
        SequenceFlow {
            id: id.to_string(),
            source_ref: source.to_string(),
            target_ref: target.to_string(),
            name: None,
            condition: None,
        }
    }

    #[test]
    fn test_missing_start_event() {
        let workflow = BpmnJsonWorkflow {
            bpmn_process: BpmnProcessInfo {
                id: "test".to_string(),
                name: "Test".to_string(),
                version: "1.0.0".to_string(),
                is_executable: true,
                process_type: "test".to_string(),
            },
            workflow_steps: vec![create_step("end", "End", "endEvent")],
            sequence_flows: vec![],
            dodaf_metadata: None,
            task_metadata: None,
        };

        let errors = validate_workflow_structure(&workflow);
        assert!(errors.iter().any(|e| e.message.contains("no start event")));
    }

    #[test]
    fn test_disconnected_node() {
        let workflow = BpmnJsonWorkflow {
            bpmn_process: BpmnProcessInfo {
                id: "test".to_string(),
                name: "Test".to_string(),
                version: "1.0.0".to_string(),
                is_executable: true,
                process_type: "test".to_string(),
            },
            workflow_steps: vec![
                create_step("start", "Start", "startEvent"),
                create_step("orphan", "Orphan", "task"),
            ],
            sequence_flows: vec![],
            dodaf_metadata: None,
            task_metadata: None,
        };

        let errors = check_disconnected_nodes(&workflow);
        assert!(errors.iter().any(|e| e.message.contains("not connected")));
    }

    #[test]
    fn test_valid_gateway_structure() {
        let workflow = BpmnJsonWorkflow {
            bpmn_process: BpmnProcessInfo {
                id: "test".to_string(),
                name: "Test".to_string(),
                version: "1.0.0".to_string(),
                is_executable: true,
                process_type: "test".to_string(),
            },
            workflow_steps: vec![
                create_step("start", "Start", "startEvent"),
                create_step("gateway", "Gateway", "exclusiveGateway"),
                create_step("task1", "Task 1", "task"),
                create_step("end", "End", "endEvent"),
            ],
            sequence_flows: vec![
                create_flow("flow1", "start", "gateway"),
                create_flow("flow2", "gateway", "task1"),
                create_flow("flow3", "task1", "end"),
            ],
            dodaf_metadata: None,
            task_metadata: None,
        };

        let errors = validate_gateway_connections(&workflow);
        // Should have no critical errors for this valid structure
        assert!(!errors.iter().any(|e| matches!(e.severity, ErrorSeverity::Error)));
    }
}
