//! BPMN JSON Validation Module (Re-export)
//!
//! This module re-exports validation functionality from the refactored
//! validation submodule for backward compatibility.
//!
//! The actual implementation has been split into focused submodules under
//! `bpmn/validation/` for better maintainability.

// Re-export all public items from the validation module
pub use crate::bpmn::validation::{
    constants, process, sequence_flows, structure, types, workflow_steps, validate_bpmn_json,
    validate_bpmn_workflow, ErrorCategory, ErrorSeverity, ValidationError, ValidationResult,
    ValidationSummary,
};
