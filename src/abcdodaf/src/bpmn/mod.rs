//! BPMN 2.0 process modeling and execution
//!
//! This module wraps the bpxe engine to provide a simplified interface
//! for defining and executing BPMN processes within the workforce context.

pub mod process;
pub mod executor;

pub use process::{Process, ProcessBuilder};
pub use executor::ProcessExecutor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Represents a BPMN process instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInstance {
    /// Unique instance ID
    pub id: Uuid,
    /// Process definition ID
    pub process_id: String,
    /// Current state
    pub state: ProcessState,
    /// Process variables
    pub variables: HashMap<String, serde_json::Value>,
    /// Start time
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// End time (if completed)
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Process execution state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessState {
    /// Process is waiting to start
    Pending,
    /// Process is currently executing
    Running,
    /// Process is paused/suspended
    Suspended,
    /// Process completed successfully
    Completed,
    /// Process failed with error
    Failed,
    /// Process was cancelled
    Cancelled,
}

/// BPMN element types supported
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BpmnElement {
    /// Start event
    StartEvent,
    /// End event
    EndEvent,
    /// User task (requires human interaction)
    UserTask,
    /// Service task (automated)
    ServiceTask,
    /// Script task (executable code)
    ScriptTask,
    /// Manual task (non-system work)
    ManualTask,
    /// Exclusive gateway (XOR)
    ExclusiveGateway,
    /// Parallel gateway (AND)
    ParallelGateway,
    /// Inclusive gateway (OR)
    InclusiveGateway,
    /// Subprocess
    SubProcess,
}

impl ProcessInstance {
    /// Create a new process instance
    pub fn new(process_id: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            process_id: process_id.into(),
            state: ProcessState::Pending,
            variables: HashMap::new(),
            started_at: chrono::Utc::now(),
            completed_at: None,
        }
    }

    /// Set a process variable
    pub fn set_variable(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.variables.insert(key.into(), value);
    }

    /// Get a process variable
    pub fn get_variable(&self, key: &str) -> Option<&serde_json::Value> {
        self.variables.get(key)
    }

    /// Mark the process as completed
    pub fn complete(&mut self) {
        self.state = ProcessState::Completed;
        self.completed_at = Some(chrono::Utc::now());
    }

    /// Mark the process as failed
    pub fn fail(&mut self) {
        self.state = ProcessState::Failed;
        self.completed_at = Some(chrono::Utc::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_instance_creation() {
        let instance = ProcessInstance::new("test_process");
        assert_eq!(instance.process_id, "test_process");
        assert_eq!(instance.state, ProcessState::Pending);
        assert!(instance.variables.is_empty());
    }

    #[test]
    fn test_process_variables() {
        let mut instance = ProcessInstance::new("test");
        instance.set_variable("key", serde_json::json!("value"));
        assert_eq!(
            instance.get_variable("key"),
            Some(&serde_json::json!("value"))
        );
    }

    #[test]
    fn test_process_completion() {
        let mut instance = ProcessInstance::new("test");
        instance.complete();
        assert_eq!(instance.state, ProcessState::Completed);
        assert!(instance.completed_at.is_some());
    }
}
