//! Process information validation
//!
//! Validates BPMN process metadata including ID, name, and version.

use crate::bpmn::json_format::BpmnJsonWorkflow;

use super::types::{ErrorCategory, ValidationError};

/// Validate process information
pub fn validate_process_info(workflow: &BpmnJsonWorkflow) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    let process = &workflow.bpmn_process;

    // Validate process ID
    if process.id.trim().is_empty() {
        errors.push(
            ValidationError::error(ErrorCategory::MissingField, "Process ID cannot be empty")
                .with_context("bpmn_process.id")
                .with_suggestion("Provide a unique identifier for the process"),
        );
    }

    // Validate process name
    if process.name.trim().is_empty() {
        errors.push(
            ValidationError::warning(ErrorCategory::MissingField, "Process name is empty")
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bpmn::json_format::BpmnProcessInfo;

    #[test]
    fn test_is_valid_version() {
        assert!(is_valid_version("1.0.0"));
        assert!(is_valid_version("2.1.5"));
        assert!(!is_valid_version("1.0"));
        assert!(!is_valid_version("invalid"));
        assert!(!is_valid_version("1.0.x"));
    }

    #[test]
    fn test_validate_empty_process_id() {
        let workflow = BpmnJsonWorkflow {
            bpmn_process: BpmnProcessInfo {
                id: "".to_string(),
                name: "Test".to_string(),
                version: "1.0.0".to_string(),
                is_executable: true,
                process_type: "test".to_string(),
            },
            workflow_steps: vec![],
            sequence_flows: vec![],
            dodaf_metadata: None,
            task_metadata: None,
        };

        let errors = validate_process_info(&workflow);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.message.contains("Process ID")));
    }
}
