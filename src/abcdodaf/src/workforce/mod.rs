//! AI Workforce Modeling
//!
//! This module provides abstractions for modeling both agentic (AI-driven)
//! and non-agentic (human/system) operations within an AI assistant workforce.

pub mod agent;
pub mod task;
pub mod orchestration;

pub use agent::{AgentTask, AgentCapability, AgentType};
pub use task::{HumanTask, SystemTask, HumanRole, SystemOperation};
pub use orchestration::{WorkflowBuilder, WorkflowExecution};

use crate::bpmn::ProcessInstance;
use crate::dodaf::OperationalActivity;
use serde::{Deserialize, Serialize};

/// Unified task representation for workforce operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkforceTask {
    /// AI agent task
    Agent(AgentTask),
    /// Human task
    Human(HumanTask),
    /// System/automated task
    System(SystemTask),
}

impl WorkforceTask {
    /// Get the task ID
    pub fn id(&self) -> &str {
        match self {
            WorkforceTask::Agent(t) => &t.id,
            WorkforceTask::Human(t) => &t.id,
            WorkforceTask::System(t) => &t.id,
        }
    }

    /// Get the task name
    pub fn name(&self) -> &str {
        match self {
            WorkforceTask::Agent(t) => &t.name,
            WorkforceTask::Human(t) => &t.name,
            WorkforceTask::System(t) => &t.name,
        }
    }

    /// Convert to operational activity
    pub fn to_operational_activity(&self) -> OperationalActivity {
        match self {
            WorkforceTask::Agent(t) => t.to_operational_activity(),
            WorkforceTask::Human(t) => t.to_operational_activity(),
            WorkforceTask::System(t) => t.to_operational_activity(),
        }
    }
}

/// Workforce execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkforceExecutionResult {
    /// Process instance
    pub instance: ProcessInstance,
    /// Task results
    pub task_results: Vec<TaskResult>,
    /// Execution metrics
    pub metrics: ExecutionMetrics,
}

/// Individual task execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Task ID
    pub task_id: String,
    /// Success status
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Output data
    pub output: serde_json::Value,
    /// Execution time in milliseconds
    pub duration_ms: u64,
}

/// Execution metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    /// Total execution time in milliseconds
    pub total_duration_ms: u64,
    /// Number of agent tasks executed
    pub agent_tasks: usize,
    /// Number of human tasks executed
    pub human_tasks: usize,
    /// Number of system tasks executed
    pub system_tasks: usize,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workforce_task_creation() {
        let agent_task = AgentTask::new("a1", "Agent Task", AgentCapability::NaturalLanguageProcessing);
        let task = WorkforceTask::Agent(agent_task);

        assert_eq!(task.id(), "a1");
        assert_eq!(task.name(), "Agent Task");
    }
}
