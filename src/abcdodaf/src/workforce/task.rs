//! Non-agentic operations modeling
//!
//! Models human and system tasks within the workforce

use crate::dodaf::{ActivityType, OperationalActivity};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Human task requiring human interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanTask {
    /// Task ID
    pub id: String,
    /// Task name
    pub name: String,
    /// Task description
    pub description: Option<String>,
    /// Required human role
    pub role: HumanRole,
    /// Task complexity (1-5)
    pub complexity: u8,
    /// Estimated duration in minutes
    pub estimated_duration_mins: Option<u32>,
    /// Task parameters/instructions
    pub parameters: HashMap<String, serde_json::Value>,
    /// Whether task can be delegated
    pub delegatable: bool,
}

/// System/automated task (non-AI)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemTask {
    /// Task ID
    pub id: String,
    /// Task name
    pub name: String,
    /// Task description
    pub description: Option<String>,
    /// System operation type
    pub operation: SystemOperation,
    /// Task configuration
    pub configuration: HashMap<String, serde_json::Value>,
    /// Retry configuration
    pub retry_policy: Option<RetryPolicy>,
    /// Timeout in seconds
    pub timeout_secs: Option<u64>,
}

/// Human role categories
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HumanRole {
    /// Subject matter expert
    Expert,
    /// Quality assurance/reviewer
    QualityAssurance,
    /// Manager/decision maker
    Manager,
    /// Operator/executor
    Operator,
    /// Customer/end user
    Customer,
    /// Administrator
    Administrator,
    /// Analyst
    Analyst,
    /// Custom role
    Custom(String),
}

/// System operation types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemOperation {
    /// Database read operation
    DatabaseRead,
    /// Database write operation
    DatabaseWrite,
    /// API call
    ApiCall,
    /// File I/O operation
    FileOperation,
    /// Message queue operation
    MessageQueue,
    /// Cache operation
    CacheOperation,
    /// Computation/calculation
    Computation,
    /// Data transformation
    DataTransformation,
    /// Notification/alert
    Notification,
    /// Custom operation
    Custom(String),
}

/// Retry policy for system tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum retry attempts
    pub max_attempts: u32,
    /// Initial delay in milliseconds
    pub initial_delay_ms: u64,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
    /// Maximum delay in milliseconds
    pub max_delay_ms: u64,
}

impl HumanTask {
    /// Create a new human task
    pub fn new(id: impl Into<String>, name: impl Into<String>, role: HumanRole) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            role,
            complexity: 3, // Medium complexity by default
            estimated_duration_mins: None,
            parameters: HashMap::new(),
            delegatable: false,
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set complexity (1-5)
    pub fn with_complexity(mut self, complexity: u8) -> Self {
        self.complexity = complexity.clamp(1, 5);
        self
    }

    /// Set estimated duration
    pub fn with_duration(mut self, mins: u32) -> Self {
        self.estimated_duration_mins = Some(mins);
        self
    }

    /// Mark as delegatable
    pub fn delegatable(mut self) -> Self {
        self.delegatable = true;
        self
    }

    /// Add parameter
    pub fn add_parameter(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.parameters.insert(key.into(), value);
        self
    }

    /// Convert to DoDAF operational activity
    pub fn to_operational_activity(&self) -> OperationalActivity {
        OperationalActivity::new(&self.id, &self.name)
            .with_type(ActivityType::Manual)
            .with_description(self.description.clone().unwrap_or_default())
            .add_performer(format!("{:?}", self.role))
    }
}

impl SystemTask {
    /// Create a new system task
    pub fn new(id: impl Into<String>, name: impl Into<String>, operation: SystemOperation) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            operation,
            configuration: HashMap::new(),
            retry_policy: None,
            timeout_secs: None,
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set retry policy
    pub fn with_retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = Some(policy);
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = Some(secs);
        self
    }

    /// Add configuration
    pub fn add_config(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.configuration.insert(key.into(), value);
        self
    }

    /// Convert to DoDAF operational activity
    pub fn to_operational_activity(&self) -> OperationalActivity {
        OperationalActivity::new(&self.id, &self.name)
            .with_type(ActivityType::Automated)
            .with_description(self.description.clone().unwrap_or_default())
            .add_performer("System")
    }
}

impl RetryPolicy {
    /// Create a new retry policy
    pub fn new(max_attempts: u32) -> Self {
        Self { max_attempts, initial_delay_ms: 1000, backoff_multiplier: 2.0, max_delay_ms: 30000 }
    }

    /// Set initial delay
    pub fn with_initial_delay(mut self, delay_ms: u64) -> Self {
        self.initial_delay_ms = delay_ms;
        self
    }

    /// Set backoff multiplier
    pub fn with_backoff(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }

    /// Set maximum delay
    pub fn with_max_delay(mut self, delay_ms: u64) -> Self {
        self.max_delay_ms = delay_ms;
        self
    }

    /// Calculate delay for a given attempt
    pub fn calculate_delay(&self, attempt: u32) -> u64 {
        let delay =
            (self.initial_delay_ms as f64) * self.backoff_multiplier.powi(attempt as i32 - 1);
        delay.min(self.max_delay_ms as f64) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_task_creation() {
        let task = HumanTask::new("h1", "Review Output", HumanRole::QualityAssurance)
            .with_complexity(4)
            .with_duration(30)
            .delegatable();

        assert_eq!(task.id, "h1");
        assert_eq!(task.role, HumanRole::QualityAssurance);
        assert_eq!(task.complexity, 4);
        assert!(task.delegatable);
    }

    #[test]
    fn test_system_task_creation() {
        let task = SystemTask::new("s1", "Save to DB", SystemOperation::DatabaseWrite)
            .with_timeout(10)
            .with_retry_policy(RetryPolicy::new(3))
            .add_config("table", serde_json::json!("users"));

        assert_eq!(task.id, "s1");
        assert_eq!(task.operation, SystemOperation::DatabaseWrite);
        assert!(task.retry_policy.is_some());
    }

    #[test]
    fn test_retry_policy() {
        let policy = RetryPolicy::new(3).with_initial_delay(1000).with_backoff(2.0);

        assert_eq!(policy.calculate_delay(1), 1000);
        assert_eq!(policy.calculate_delay(2), 2000);
        assert_eq!(policy.calculate_delay(3), 4000);
    }
}
