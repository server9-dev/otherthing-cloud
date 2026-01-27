//! Sequence flow (connection) validation
//!
//! Validates BPMN sequence flows including reference validation,
//! ID uniqueness, and preventing self-loops.

use crate::bpmn::json_format::{SequenceFlow, WorkflowStep};
use std::collections::HashSet;

use super::types::{ErrorCategory, ValidationError};

/// Validate sequence flows (connections)
pub fn validate_sequence_flows(
    flows: &[SequenceFlow],
    steps: &[WorkflowStep],
) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    let mut seen_flow_ids = HashSet::new();

    // Build a map of valid node IDs
    let valid_node_ids: HashSet<String> = steps.iter().map(|s| s.id.clone()).collect();

    for flow in flows {
        // Validate flow ID uniqueness
        if !seen_flow_ids.insert(flow.id.clone()) {
            errors.push(
                ValidationError::error(
                    ErrorCategory::DuplicateId,
                    format!("Duplicate flow ID: '{}'", flow.id),
                )
                .with_context(&flow.id),
            );
        }

        // Validate flow ID is not empty
        if flow.id.trim().is_empty() {
            errors.push(ValidationError::error(
                ErrorCategory::MissingField,
                "Sequence flow ID cannot be empty",
            ));
        }

        // Validate sourceRef exists
        if !valid_node_ids.contains(&flow.source_ref) {
            errors.push(
                ValidationError::error(
                    ErrorCategory::BrokenReference,
                    format!(
                        "Sequence flow '{}' references non-existent source node: '{}'",
                        flow.id, flow.source_ref
                    ),
                )
                .with_context(&flow.id)
                .with_suggestion("Ensure the sourceRef matches an existing node ID"),
            );
        }

        // Validate targetRef exists
        if !valid_node_ids.contains(&flow.target_ref) {
            errors.push(
                ValidationError::error(
                    ErrorCategory::BrokenReference,
                    format!(
                        "Sequence flow '{}' references non-existent target node: '{}'",
                        flow.id, flow.target_ref
                    ),
                )
                .with_context(&flow.id)
                .with_suggestion("Ensure the targetRef matches an existing node ID"),
            );
        }

        // Validate no self-loops
        if flow.source_ref == flow.target_ref {
            errors.push(
                ValidationError::warning(
                    ErrorCategory::StructuralIssue,
                    format!(
                        "Sequence flow '{}' creates a self-loop on node '{}'",
                        flow.id, flow.source_ref
                    ),
                )
                .with_context(&flow.id)
                .with_suggestion("Self-loops are generally not recommended in BPMN workflows"),
            );
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_broken_reference() {
        let steps = vec![WorkflowStep {
            id: "start".to_string(),
            name: "Start".to_string(),
            step_type: "startEvent".to_string(),
            task_type: None,
            event_type: None,
            implementation: None,
            inputs: vec![],
            outputs: vec![],
            duration_minutes: None,
        }];

        let flows = vec![SequenceFlow {
            id: "flow1".to_string(),
            source_ref: "start".to_string(),
            target_ref: "nonexistent".to_string(),
            name: None,
            condition: None,
        }];

        let errors = validate_sequence_flows(&flows, &steps);
        assert!(errors
            .iter()
            .any(|e| e.category == ErrorCategory::BrokenReference));
    }

    #[test]
    fn test_self_loop_warning() {
        let steps = vec![WorkflowStep {
            id: "task1".to_string(),
            name: "Task".to_string(),
            step_type: "task".to_string(),
            task_type: None,
            event_type: None,
            implementation: None,
            inputs: vec![],
            outputs: vec![],
            duration_minutes: None,
        }];

        let flows = vec![SequenceFlow {
            id: "flow1".to_string(),
            source_ref: "task1".to_string(),
            target_ref: "task1".to_string(),
            name: None,
            condition: None,
        }];

        let errors = validate_sequence_flows(&flows, &steps);
        assert!(errors.iter().any(|e| e.message.contains("self-loop")));
    }

    #[test]
    fn test_duplicate_flow_ids() {
        let steps = vec![
            WorkflowStep {
                id: "start".to_string(),
                name: "Start".to_string(),
                step_type: "startEvent".to_string(),
                task_type: None,
                event_type: None,
                implementation: None,
                inputs: vec![],
                outputs: vec![],
                duration_minutes: None,
            },
            WorkflowStep {
                id: "end".to_string(),
                name: "End".to_string(),
                step_type: "endEvent".to_string(),
                task_type: None,
                event_type: None,
                implementation: None,
                inputs: vec![],
                outputs: vec![],
                duration_minutes: None,
            },
        ];

        let flows = vec![
            SequenceFlow {
                id: "flow1".to_string(),
                source_ref: "start".to_string(),
                target_ref: "end".to_string(),
                name: None,
                condition: None,
            },
            SequenceFlow {
                id: "flow1".to_string(),
                source_ref: "start".to_string(),
                target_ref: "end".to_string(),
                name: None,
                condition: None,
            },
        ];

        let errors = validate_sequence_flows(&flows, &steps);
        assert!(errors.iter().any(|e| e.category == ErrorCategory::DuplicateId));
    }
}
