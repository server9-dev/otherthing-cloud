//! Workflow step (node) validation
//!
//! Validates individual BPMN workflow steps including type validation,
//! ID uniqueness, and type-specific requirements.

use crate::bpmn::json_format::WorkflowStep;
use std::collections::HashSet;

use super::constants::{VALID_EVENT_TYPES, VALID_NODE_TYPES, VALID_TASK_TYPES};
use super::types::{ErrorCategory, ValidationError};

/// Validate workflow steps (nodes)
pub fn validate_workflow_steps(steps: &[WorkflowStep]) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    let mut seen_ids = HashSet::new();

    if steps.is_empty() {
        errors.push(
            ValidationError::error(ErrorCategory::StructuralIssue, "Workflow has no steps/nodes")
                .with_suggestion("Add at least a start event and end event"),
        );
        return errors;
    }

    for step in steps {
        // Validate step ID uniqueness
        if !seen_ids.insert(step.id.clone()) {
            errors.push(
                ValidationError::error(
                    ErrorCategory::DuplicateId,
                    format!("Duplicate node ID: '{}'", step.id),
                )
                .with_context(&step.id)
                .with_suggestion("Ensure each node has a unique ID"),
            );
        }

        // Validate step ID is not empty
        if step.id.trim().is_empty() {
            errors.push(ValidationError::error(
                ErrorCategory::MissingField,
                "Node ID cannot be empty",
            ));
        }

        // Validate step name
        if step.name.trim().is_empty() {
            errors.push(
                ValidationError::warning(
                    ErrorCategory::MissingField,
                    format!("Node '{}' has no name", step.id),
                )
                .with_context(&step.id)
                .with_suggestion("Provide a descriptive name for the node"),
            );
        }

        // Validate node type
        if !VALID_NODE_TYPES.contains(&step.step_type.as_str()) {
            errors.push(
                ValidationError::error(
                    ErrorCategory::InvalidValue,
                    format!("Invalid node type: '{}'", step.step_type),
                )
                .with_context(&step.id)
                .with_suggestion(format!(
                    "Use one of the valid BPMN node types: {}",
                    VALID_NODE_TYPES.join(", ")
                )),
            );
        }

        // Type-specific validation
        errors.extend(validate_node_type_specifics(step));
    }

    errors
}

/// Validate node-type-specific requirements
fn validate_node_type_specifics(step: &WorkflowStep) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    match step.step_type.as_str() {
        "serviceTask" => {
            // Service tasks should have a task type
            if step.task_type.is_none() {
                errors.push(
                    ValidationError::warning(
                        ErrorCategory::MissingField,
                        format!("Service task '{}' has no taskType specified", step.id),
                    )
                    .with_context(&step.id)
                    .with_suggestion(format!(
                        "Consider specifying a taskType: {}",
                        VALID_TASK_TYPES.join(", ")
                    )),
                );
            } else if let Some(ref task_type) = step.task_type {
                if !VALID_TASK_TYPES.contains(&task_type.as_str()) {
                    errors.push(
                        ValidationError::warning(
                            ErrorCategory::InvalidValue,
                            format!("Unknown task type: '{}'", task_type),
                        )
                        .with_context(&step.id)
                        .with_suggestion(format!(
                            "Consider using one of: {}",
                            VALID_TASK_TYPES.join(", ")
                        )),
                    );
                }
            }
        },
        "scriptTask" => {
            // Script tasks should have implementation
            if step.implementation.is_none() {
                errors.push(
                    ValidationError::warning(
                        ErrorCategory::MissingField,
                        format!("Script task '{}' has no implementation/script specified", step.id),
                    )
                    .with_context(&step.id)
                    .with_suggestion("Specify the script or implementation to execute"),
                );
            }
        },
        "startEvent" | "endEvent" | "intermediateCatchEvent" | "intermediateThrowEvent" => {
            // Events may have event types
            if let Some(ref event_type) = step.event_type {
                if !VALID_EVENT_TYPES.contains(&event_type.as_str()) {
                    errors.push(
                        ValidationError::warning(
                            ErrorCategory::InvalidValue,
                            format!("Unknown event type: '{}'", event_type),
                        )
                        .with_context(&step.id)
                        .with_suggestion(format!(
                            "Consider using one of: {}",
                            VALID_EVENT_TYPES.join(", ")
                        )),
                    );
                }
            }
        },
        _ => {},
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_workflow_steps() {
        let errors = validate_workflow_steps(&[]);
        assert!(!errors.is_empty());
        assert!(errors[0].message.contains("no steps"));
    }

    #[test]
    fn test_duplicate_step_ids() {
        let steps = vec![
            WorkflowStep {
                id: "step1".to_string(),
                name: "Step 1".to_string(),
                step_type: "startEvent".to_string(),
                task_type: None,
                event_type: None,
                implementation: None,
                inputs: vec![],
                outputs: vec![],
                duration_minutes: None,
            },
            WorkflowStep {
                id: "step1".to_string(),
                name: "Step 1 Duplicate".to_string(),
                step_type: "endEvent".to_string(),
                task_type: None,
                event_type: None,
                implementation: None,
                inputs: vec![],
                outputs: vec![],
                duration_minutes: None,
            },
        ];

        let errors = validate_workflow_steps(&steps);
        assert!(errors.iter().any(|e| e.category == ErrorCategory::DuplicateId));
    }

    #[test]
    fn test_invalid_node_type() {
        let steps = vec![WorkflowStep {
            id: "step1".to_string(),
            name: "Test".to_string(),
            step_type: "invalidType".to_string(),
            task_type: None,
            event_type: None,
            implementation: None,
            inputs: vec![],
            outputs: vec![],
            duration_minutes: None,
        }];

        let errors = validate_workflow_steps(&steps);
        assert!(errors.iter().any(|e| e.message.contains("Invalid node type")));
    }
}
