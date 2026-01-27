//! BPMN JSON Validation Module
//!
//! Provides comprehensive validation for BPMN JSON workflow files.
//! This module is UI-independent and can be used standalone in CLI tools.
//!
//! # Architecture
//!
//! The validation module is organized into focused submodules:
//!
//! - `types` - Error types and validation result structures
//! - `constants` - BPMN 2.0 specification constants
//! - `process` - Process metadata validation
//! - `workflow_steps` - Workflow step/node validation
//! - `sequence_flows` - Sequence flow/connection validation
//! - `structure` - Overall workflow structure validation
//!
//! # Example
//!
//! ```
//! use abcdodaf::bpmn::json_validation::validate_bpmn_json;
//!
//! let json = r#"{
//!     "bpmn_process": {
//!         "id": "proc1",
//!         "name": "Test Process",
//!         "version": "1.0.0",
//!         "isExecutable": true,
//!         "processType": "agent_workflow"
//!     },
//!     "workflow_steps": [
//!         {
//!             "id": "start",
//!             "name": "Start",
//!             "type": "startEvent"
//!         }
//!     ],
//!     "sequence_flows": []
//! }"#;
//!
//! match validate_bpmn_json(json) {
//!     Ok(()) => println!("Valid!"),
//!     Err(errors) => {
//!         for error in errors {
//!             println!("{:?}: {}", error.severity, error.message);
//!         }
//!     }
//! }
//! ```

use crate::bpmn::json_format::BpmnJsonWorkflow;

// Public submodules
pub mod constants;
pub mod process;
pub mod sequence_flows;
pub mod structure;
pub mod types;
pub mod workflow_steps;

// Re-export commonly used types
pub use types::{
    ErrorCategory, ErrorSeverity, ValidationError, ValidationResult, ValidationSummary,
};

/// Validate a BPMN JSON string
///
/// # Arguments
/// * `json_str` - JSON string containing the BPMN workflow
///
/// # Returns
/// * `Ok(())` if validation passes
/// * `Err(Vec<ValidationError>)` if validation fails with list of errors
///
/// # Example
/// ```
/// use abcdodaf::bpmn::json_validation::validate_bpmn_json;
///
/// let json = r#"{
///     "bpmn_process": {
///         "id": "proc1",
///         "name": "Test Process",
///         "version": "1.0.0",
///         "isExecutable": true,
///         "processType": "agent_workflow"
///     },
///     "workflow_steps": [],
///     "sequence_flows": []
/// }"#;
///
/// match validate_bpmn_json(json) {
///     Ok(()) => println!("Valid!"),
///     Err(errors) => {
///         for error in errors {
///             println!("{:?}: {}", error.severity, error.message);
///         }
///     }
/// }
/// ```
pub fn validate_bpmn_json(json_str: &str) -> ValidationResult {
    // First, try to parse the JSON
    let workflow: BpmnJsonWorkflow = match serde_json::from_str(json_str) {
        Ok(w) => w,
        Err(e) => {
            return Err(vec![ValidationError::error(
                ErrorCategory::InvalidValue,
                format!("Failed to parse JSON: {}", e),
            )
            .with_suggestion("Ensure the JSON is well-formed and matches the expected schema")]);
        },
    };

    validate_bpmn_workflow(&workflow)
}

/// Validate a BPMN workflow structure
///
/// # Arguments
/// * `workflow` - The BPMN workflow to validate
///
/// # Returns
/// * `Ok(())` if validation passes
/// * `Err(Vec<ValidationError>)` if validation fails with list of errors
pub fn validate_bpmn_workflow(workflow: &BpmnJsonWorkflow) -> ValidationResult {
    let mut errors = Vec::new();

    // Validate process info
    errors.extend(process::validate_process_info(workflow));

    // Validate workflow steps
    errors.extend(workflow_steps::validate_workflow_steps(&workflow.workflow_steps));

    // Validate sequence flows
    errors.extend(sequence_flows::validate_sequence_flows(
        &workflow.sequence_flows,
        &workflow.workflow_steps,
    ));

    // Validate overall structure
    errors.extend(structure::validate_workflow_structure(workflow));

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_workflow() {
        let json = r#"{
            "bpmn_process": {
                "id": "test_process",
                "name": "Test Process",
                "version": "1.0.0",
                "isExecutable": true,
                "processType": "agent_workflow"
            },
            "workflow_steps": [
                {
                    "id": "start",
                    "name": "Start",
                    "type": "startEvent"
                },
                {
                    "id": "task1",
                    "name": "Task 1",
                    "type": "serviceTask",
                    "taskType": "research"
                },
                {
                    "id": "end",
                    "name": "End",
                    "type": "endEvent"
                }
            ],
            "sequence_flows": [
                {
                    "id": "flow1",
                    "sourceRef": "start",
                    "targetRef": "task1"
                },
                {
                    "id": "flow2",
                    "sourceRef": "task1",
                    "targetRef": "end"
                }
            ]
        }"#;

        let result = validate_bpmn_json(json);
        assert!(result.is_ok(), "Expected valid workflow, got: {:?}", result);
    }

    #[test]
    fn test_missing_start_event() {
        let json = r#"{
            "bpmn_process": {
                "id": "test",
                "name": "Test",
                "version": "1.0.0"
            },
            "workflow_steps": [
                {
                    "id": "end",
                    "name": "End",
                    "type": "endEvent"
                }
            ],
            "sequence_flows": []
        }"#;

        let result = validate_bpmn_json(json);
        assert!(result.is_err());

        if let Err(errors) = result {
            let has_start_error = errors.iter().any(|e| e.message.contains("no start event"));
            assert!(has_start_error, "Expected missing start event error");
        }
    }

    #[test]
    fn test_duplicate_node_ids() {
        let json = r#"{
            "bpmn_process": {
                "id": "test",
                "name": "Test",
                "version": "1.0.0"
            },
            "workflow_steps": [
                {
                    "id": "node1",
                    "name": "Node 1",
                    "type": "startEvent"
                },
                {
                    "id": "node1",
                    "name": "Node 1 Duplicate",
                    "type": "endEvent"
                }
            ],
            "sequence_flows": []
        }"#;

        let result = validate_bpmn_json(json);
        assert!(result.is_err());

        if let Err(errors) = result {
            let has_duplicate_error =
                errors.iter().any(|e| e.category == ErrorCategory::DuplicateId);
            assert!(has_duplicate_error, "Expected duplicate ID error");
        }
    }

    #[test]
    fn test_broken_reference() {
        let json = r#"{
            "bpmn_process": {
                "id": "test",
                "name": "Test",
                "version": "1.0.0"
            },
            "workflow_steps": [
                {
                    "id": "start",
                    "name": "Start",
                    "type": "startEvent"
                }
            ],
            "sequence_flows": [
                {
                    "id": "flow1",
                    "sourceRef": "start",
                    "targetRef": "nonexistent"
                }
            ]
        }"#;

        let result = validate_bpmn_json(json);
        assert!(result.is_err());

        if let Err(errors) = result {
            let has_broken_ref =
                errors.iter().any(|e| e.category == ErrorCategory::BrokenReference);
            assert!(has_broken_ref, "Expected broken reference error");
        }
    }

    #[test]
    fn test_invalid_node_type() {
        let json = r#"{
            "bpmn_process": {
                "id": "test",
                "name": "Test",
                "version": "1.0.0"
            },
            "workflow_steps": [
                {
                    "id": "start",
                    "name": "Start",
                    "type": "invalidType"
                }
            ],
            "sequence_flows": []
        }"#;

        let result = validate_bpmn_json(json);
        assert!(result.is_err());

        if let Err(errors) = result {
            let has_invalid_type = errors.iter().any(|e| {
                e.category == ErrorCategory::InvalidValue && e.message.contains("Invalid node type")
            });
            assert!(has_invalid_type, "Expected invalid node type error");
        }
    }

    #[test]
    fn test_validation_summary() {
        let errors = vec![
            ValidationError::error(ErrorCategory::MissingField, "Error 1"),
            ValidationError::warning(ErrorCategory::InvalidValue, "Warning 1"),
            ValidationError::info(ErrorCategory::StructuralIssue, "Info 1"),
        ];

        let summary = ValidationSummary::from_errors(&errors);
        assert_eq!(summary.errors, 1);
        assert_eq!(summary.warnings, 1);
        assert_eq!(summary.info, 1);
        assert!(summary.has_errors());
    }

    #[test]
    fn test_gateway_validation() {
        let json = r#"{
            "bpmn_process": {
                "id": "test",
                "name": "Test",
                "version": "1.0.0"
            },
            "workflow_steps": [
                {
                    "id": "start",
                    "name": "Start",
                    "type": "startEvent"
                },
                {
                    "id": "gateway",
                    "name": "Decision",
                    "type": "exclusiveGateway"
                },
                {
                    "id": "task1",
                    "name": "Task 1",
                    "type": "serviceTask",
                    "taskType": "research"
                },
                {
                    "id": "task2",
                    "name": "Task 2",
                    "type": "serviceTask",
                    "taskType": "testing"
                },
                {
                    "id": "end",
                    "name": "End",
                    "type": "endEvent"
                }
            ],
            "sequence_flows": [
                {
                    "id": "flow1",
                    "sourceRef": "start",
                    "targetRef": "gateway"
                },
                {
                    "id": "flow2",
                    "sourceRef": "gateway",
                    "targetRef": "task1"
                },
                {
                    "id": "flow3",
                    "sourceRef": "gateway",
                    "targetRef": "task2"
                },
                {
                    "id": "flow4",
                    "sourceRef": "task1",
                    "targetRef": "end"
                },
                {
                    "id": "flow5",
                    "sourceRef": "task2",
                    "targetRef": "end"
                }
            ]
        }"#;

        let result = validate_bpmn_json(json);
        // This should pass - valid gateway usage
        assert!(result.is_ok(), "Gateway workflow should be valid: {:?}", result);
    }

    #[test]
    fn test_invalid_version() {
        let json = r#"{
            "bpmn_process": {
                "id": "test",
                "name": "Test",
                "version": "invalid"
            },
            "workflow_steps": [
                {
                    "id": "start",
                    "name": "Start",
                    "type": "startEvent"
                },
                {
                    "id": "end",
                    "name": "End",
                    "type": "endEvent"
                }
            ],
            "sequence_flows": [
                {
                    "id": "flow1",
                    "sourceRef": "start",
                    "targetRef": "end"
                }
            ]
        }"#;

        let result = validate_bpmn_json(json);
        assert!(result.is_err());

        if let Err(errors) = result {
            let has_version_warning =
                errors.iter().any(|e| e.message.contains("Invalid version format"));
            assert!(has_version_warning, "Expected version format warning");
        }
    }
}
