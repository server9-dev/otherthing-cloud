//! BPMN JSON Validation Module
//!
//! Provides comprehensive validation for BPMN JSON workflow files.
//! This module is UI-independent and can be used standalone in CLI tools.

use crate::bpmn::json_format::{BpmnJsonWorkflow, SequenceFlow, WorkflowStep};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Validation error with detailed context
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error severity level
    pub severity: ErrorSeverity,
    /// Error category
    pub category: ErrorCategory,
    /// Human-readable error message
    pub message: String,
    /// Context information (node ID, field name, etc.)
    pub context: Option<String>,
    /// Suggested fix or additional information
    pub suggestion: Option<String>,
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Critical error - workflow cannot be loaded or executed
    Error,
    /// Warning - workflow may have issues but can be loaded
    Warning,
    /// Informational - best practice suggestions
    Info,
}

/// Error categories for easier filtering and handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// Missing required field
    MissingField,
    /// Invalid field value
    InvalidValue,
    /// Reference to non-existent element
    BrokenReference,
    /// Duplicate identifier
    DuplicateId,
    /// Structural issue (disconnected nodes, no start/end, etc.)
    StructuralIssue,
    /// Type-specific validation
    TypeValidation,
}

impl ValidationError {
    /// Create a new error
    pub fn error(category: ErrorCategory, message: impl Into<String>) -> Self {
        Self {
            severity: ErrorSeverity::Error,
            category,
            message: message.into(),
            context: None,
            suggestion: None,
        }
    }

    /// Create a new warning
    pub fn warning(category: ErrorCategory, message: impl Into<String>) -> Self {
        Self {
            severity: ErrorSeverity::Warning,
            category,
            message: message.into(),
            context: None,
            suggestion: None,
        }
    }

    /// Create a new info message
    pub fn info(category: ErrorCategory, message: impl Into<String>) -> Self {
        Self {
            severity: ErrorSeverity::Info,
            category,
            message: message.into(),
            context: None,
            suggestion: None,
        }
    }

    /// Add context to the error
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Add a suggestion for fixing the error
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

/// Validation result type
pub type ValidationResult = Result<(), Vec<ValidationError>>;

/// Valid BPMN node types as per BPMN 2.0 specification
const VALID_NODE_TYPES: &[&str] = &[
    // Events
    "startEvent",
    "endEvent",
    "intermediateCatchEvent",
    "intermediateThrowEvent",
    "boundaryEvent",
    // Tasks
    "task",
    "serviceTask",
    "userTask",
    "manualTask",
    "scriptTask",
    "businessRuleTask",
    "sendTask",
    "receiveTask",
    // Gateways
    "exclusiveGateway",
    "parallelGateway",
    "inclusiveGateway",
    "eventBasedGateway",
    "complexGateway",
    // Subprocesses
    "subProcess",
    "callActivity",
    "transaction",
    "adHocSubProcess",
];

/// Valid event types for events
const VALID_EVENT_TYPES: &[&str] = &[
    "none",
    "message",
    "timer",
    "error",
    "escalation",
    "cancel",
    "compensation",
    "conditional",
    "link",
    "signal",
    "terminate",
    "multiple",
    "parallelMultiple",
];

/// Valid task types for service tasks
const VALID_TASK_TYPES: &[&str] = &[
    "research",
    "design",
    "code_generation",
    "testing",
    "documentation",
    "integration",
    "debugging",
    "optimization",
    "deployment",
    "monitoring",
];

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
///         "name": "Test Process"
///     },
///     "workflow_steps": [],
///     "sequence_flows": []
/// }"#;
///
/// match validate_bpmn_json(json) {
///     Ok(()) => println!("Valid!"),
///     Err(errors) => {
///         for error in errors {
///             println!("{}: {}", error.severity, error.message);
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
        }
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
    errors.extend(validate_process_info(workflow));

    // Validate workflow steps
    errors.extend(validate_workflow_steps(&workflow.workflow_steps));

    // Validate sequence flows
    errors.extend(validate_sequence_flows(
        &workflow.sequence_flows,
        &workflow.workflow_steps,
    ));

    // Validate overall structure
    errors.extend(validate_workflow_structure(workflow));

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validate process information
fn validate_process_info(workflow: &BpmnJsonWorkflow) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    let process = &workflow.bpmn_process;

    // Validate process ID
    if process.id.trim().is_empty() {
        errors.push(
            ValidationError::error(
                ErrorCategory::MissingField,
                "Process ID cannot be empty",
            )
            .with_context("bpmn_process.id")
            .with_suggestion("Provide a unique identifier for the process"),
        );
    }

    // Validate process name
    if process.name.trim().is_empty() {
        errors.push(
            ValidationError::warning(
                ErrorCategory::MissingField,
                "Process name is empty",
            )
            .with_context("bpmn_process.name")
            .with_suggestion("Provide a descriptive name for the process"),
        );
    }

    // Validate version format (should be semver-like)
    if !is_valid_version(&process.version) {
        errors.push(
            ValidationError::warning(
                ErrorCategory::InvalidValue,
                format!("Invalid version format: '{}'", process.version),
            )
            .with_context("bpmn_process.version")
            .with_suggestion("Use semantic versioning format (e.g., '1.0.0')"),
        );
    }

    errors
}

/// Check if a version string is valid (basic semver check)
fn is_valid_version(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return false;
    }
    parts.iter().all(|p| p.parse::<u32>().is_ok())
}

/// Validate workflow steps (nodes)
fn validate_workflow_steps(steps: &[WorkflowStep]) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    let mut seen_ids = HashSet::new();

    if steps.is_empty() {
        errors.push(
            ValidationError::error(
                ErrorCategory::StructuralIssue,
                "Workflow has no steps/nodes",
            )
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
        }
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
        }
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
        }
        _ => {}
    }

    errors
}

/// Validate sequence flows (connections)
fn validate_sequence_flows(
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
                    format!("Sequence flow '{}' creates a self-loop on node '{}'", flow.id, flow.source_ref),
                )
                .with_context(&flow.id)
                .with_suggestion("Self-loops are generally not recommended in BPMN workflows"),
            );
        }
    }

    errors
}

/// Validate overall workflow structure
fn validate_workflow_structure(workflow: &BpmnJsonWorkflow) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Check for start events
    let start_events: Vec<_> = workflow
        .workflow_steps
        .iter()
        .filter(|s| s.step_type == "startEvent")
        .collect();

    if start_events.is_empty() {
        errors.push(
            ValidationError::error(
                ErrorCategory::StructuralIssue,
                "Workflow has no start event",
            )
            .with_suggestion("Add at least one startEvent node"),
        );
    }

    // Check for end events
    let end_events: Vec<_> = workflow
        .workflow_steps
        .iter()
        .filter(|s| s.step_type == "endEvent")
        .collect();

    if end_events.is_empty() {
        errors.push(
            ValidationError::error(
                ErrorCategory::StructuralIssue,
                "Workflow has no end event",
            )
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
            }
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
            }
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
                        .with_suggestion("Consider using separate gateways for merge and split operations"),
                    );
                }
            }
            _ => {
                // Regular tasks
                if incoming == 0 && outgoing == 0 && workflow.workflow_steps.len() > 1 {
                    // Already handled in disconnected nodes check
                }
            }
        }
    }

    errors
}

/// Validation summary helper
pub struct ValidationSummary {
    pub errors: usize,
    pub warnings: usize,
    pub info: usize,
}

impl ValidationSummary {
    /// Create a summary from a list of validation errors
    pub fn from_errors(errors: &[ValidationError]) -> Self {
        let mut summary = Self {
            errors: 0,
            warnings: 0,
            info: 0,
        };

        for error in errors {
            match error.severity {
                ErrorSeverity::Error => summary.errors += 1,
                ErrorSeverity::Warning => summary.warnings += 1,
                ErrorSeverity::Info => summary.info += 1,
            }
        }

        summary
    }

    /// Check if there are any errors (not warnings or info)
    pub fn has_errors(&self) -> bool {
        self.errors > 0
    }

    /// Format the summary as a string
    pub fn format(&self) -> String {
        format!(
            "Validation summary: {} error(s), {} warning(s), {} info message(s)",
            self.errors, self.warnings, self.info
        )
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
            let has_start_error = errors
                .iter()
                .any(|e| e.message.contains("no start event"));
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
            let has_duplicate_error = errors
                .iter()
                .any(|e| e.category == ErrorCategory::DuplicateId);
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
            let has_broken_ref = errors
                .iter()
                .any(|e| e.category == ErrorCategory::BrokenReference);
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
            let has_version_warning = errors
                .iter()
                .any(|e| e.message.contains("Invalid version format"));
            assert!(has_version_warning, "Expected version format warning");
        }
    }
}
