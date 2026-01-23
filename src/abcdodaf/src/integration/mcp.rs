//! Model Context Protocol (MCP) integration
//!
//! Integrates with RhizOS MCP-based infrastructure for hardware-agnostic execution

use crate::error::Result;
use crate::workforce::{AgentTask, SystemTask};
use serde::{Deserialize, Serialize};

/// MCP integration for executing tasks on RhizOS network
pub struct McpIntegration {
    /// MCP server endpoint
    endpoint: String,
    /// Client configuration
    config: McpConfig,
}

/// MCP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// Timeout in seconds
    pub timeout_secs: u64,
    /// Retry attempts
    pub retry_attempts: u32,
    /// Enable GPU acceleration
    pub gpu_enabled: bool,
}

/// MCP task submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTaskSubmission {
    /// Task ID
    pub task_id: String,
    /// Task type
    pub task_type: String,
    /// Task parameters
    pub parameters: serde_json::Value,
    /// Resource requirements
    pub resources: McpResourceRequirements,
}

/// Resource requirements for MCP task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResourceRequirements {
    /// CPU cores
    pub cpu_cores: u32,
    /// Memory in MB
    pub memory_mb: u32,
    /// GPU required
    pub gpu_required: bool,
    /// GPU memory in MB (if GPU required)
    pub gpu_memory_mb: Option<u32>,
}

impl McpIntegration {
    /// Create a new MCP integration
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            config: McpConfig::default(),
        }
    }

    /// Configure the integration
    pub fn with_config(mut self, config: McpConfig) -> Self {
        self.config = config;
        self
    }

    /// Submit an agent task to MCP network
    pub async fn submit_agent_task(&self, task: &AgentTask) -> Result<String> {
        let submission = McpTaskSubmission {
            task_id: task.id.clone(),
            task_type: "agent_task".to_string(),
            parameters: serde_json::to_value(task)?,
            resources: McpResourceRequirements {
                cpu_cores: 2,
                memory_mb: 4096,
                gpu_required: self.config.gpu_enabled,
                gpu_memory_mb: if self.config.gpu_enabled { Some(8192) } else { None },
            },
        };

        self.submit_task(submission).await
    }

    /// Submit a system task to MCP network
    pub async fn submit_system_task(&self, task: &SystemTask) -> Result<String> {
        let submission = McpTaskSubmission {
            task_id: task.id.clone(),
            task_type: "system_task".to_string(),
            parameters: serde_json::to_value(task)?,
            resources: McpResourceRequirements {
                cpu_cores: 1,
                memory_mb: 2048,
                gpu_required: false,
                gpu_memory_mb: None,
            },
        };

        self.submit_task(submission).await
    }

    /// Internal task submission
    async fn submit_task(&self, submission: McpTaskSubmission) -> Result<String> {
        // In production, this would make an actual HTTP request to the MCP orchestrator
        // For now, return a mock job ID
        tracing::info!(
            "Submitting task {} to MCP endpoint: {}",
            submission.task_id,
            self.endpoint
        );

        // Mock implementation
        Ok(format!("job-{}", uuid::Uuid::new_v4()))
    }

    /// Query task status
    pub async fn get_task_status(&self, job_id: &str) -> Result<TaskStatus> {
        // Mock implementation
        tracing::debug!("Querying status for job: {}", job_id);
        Ok(TaskStatus::Running)
    }

    /// Cancel a running task
    pub async fn cancel_task(&self, job_id: &str) -> Result<()> {
        tracing::info!("Cancelling job: {}", job_id);
        Ok(())
    }
}

/// Task execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task is queued
    Queued,
    /// Task is running
    Running,
    /// Task completed successfully
    Completed,
    /// Task failed
    Failed,
    /// Task was cancelled
    Cancelled,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 300,
            retry_attempts: 3,
            gpu_enabled: false,
        }
    }
}

impl Default for McpResourceRequirements {
    fn default() -> Self {
        Self {
            cpu_cores: 1,
            memory_mb: 1024,
            gpu_required: false,
            gpu_memory_mb: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workforce::AgentCapability;

    #[tokio::test]
    async fn test_mcp_integration() {
        let mcp = McpIntegration::new("http://localhost:3000");
        let task = AgentTask::new("test", "Test Task", AgentCapability::NaturalLanguageProcessing);

        let job_id = mcp.submit_agent_task(&task).await.unwrap();
        assert!(job_id.starts_with("job-"));
    }

    #[tokio::test]
    async fn test_task_status() {
        let mcp = McpIntegration::new("http://localhost:3000");
        let status = mcp.get_task_status("job-123").await.unwrap();
        assert!(matches!(status, TaskStatus::Running));
    }
}
