//! BPMN process executor

use super::{Process, ProcessInstance, ProcessState};
use crate::error::{AbcdodafError, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Trait for task execution handlers
#[async_trait]
pub trait TaskHandler: Send + Sync {
    /// Execute a task with given context
    async fn execute(
        &self,
        task_id: &str,
        variables: &HashMap<String, serde_json::Value>,
    ) -> Result<HashMap<String, serde_json::Value>>;
}

/// Process executor
pub struct ProcessExecutor {
    /// Task handlers mapped by task type
    handlers: HashMap<String, Arc<dyn TaskHandler>>,
    /// Active process instances
    instances: Arc<RwLock<HashMap<uuid::Uuid, ProcessInstance>>>,
}

impl ProcessExecutor {
    /// Create a new process executor
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            instances: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a task handler
    pub fn register_handler(
        mut self,
        task_type: impl Into<String>,
        handler: Arc<dyn TaskHandler>,
    ) -> Self {
        self.handlers.insert(task_type.into(), handler);
        self
    }

    /// Start a process instance
    pub async fn start_process(&self, process: &Process) -> Result<uuid::Uuid> {
        let mut instance = ProcessInstance::new(&process.id);
        instance.state = ProcessState::Running;

        let instance_id = instance.id;
        info!("Starting process instance: {}", instance_id);

        let mut instances = self.instances.write().await;
        instances.insert(instance_id, instance);

        Ok(instance_id)
    }

    /// Execute a process
    pub async fn execute_process(
        &self,
        process: &Process,
        initial_variables: HashMap<String, serde_json::Value>,
    ) -> Result<ProcessInstance> {
        let instance_id = self.start_process(process).await?;

        // Set initial variables
        {
            let mut instances = self.instances.write().await;
            if let Some(instance) = instances.get_mut(&instance_id) {
                for (key, value) in initial_variables {
                    instance.set_variable(key, value);
                }
            }
        }

        // Execute tasks sequentially (simplified - in production use BPMN flow)
        for task in &process.tasks {
            debug!("Executing task: {} ({})", task.name, task.id);

            // Get current variables
            let variables = {
                let instances = self.instances.read().await;
                instances
                    .get(&instance_id)
                    .map(|i| i.variables.clone())
                    .unwrap_or_default()
            };

            // Find appropriate handler
            let task_type = format!("{:?}", task.task_type).to_lowercase();
            if let Some(handler) = self.handlers.get(&task_type) {
                match handler.execute(&task.id, &variables).await {
                    Ok(output_vars) => {
                        // Update instance variables
                        let mut instances = self.instances.write().await;
                        if let Some(instance) = instances.get_mut(&instance_id) {
                            for (key, value) in output_vars {
                                instance.set_variable(key, value);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Task execution failed: {}", e);
                        let mut instances = self.instances.write().await;
                        if let Some(instance) = instances.get_mut(&instance_id) {
                            instance.fail();
                        }
                        return Err(AbcdodafError::WorkflowError(format!(
                            "Task {} failed: {}",
                            task.id, e
                        )));
                    }
                }
            } else {
                warn!("No handler found for task type: {}", task_type);
            }
        }

        // Mark as completed
        let mut instances = self.instances.write().await;
        if let Some(instance) = instances.get_mut(&instance_id) {
            instance.complete();
            info!("Process instance {} completed", instance_id);
            Ok(instance.clone())
        } else {
            Err(AbcdodafError::WorkflowError(
                "Instance not found".to_string(),
            ))
        }
    }

    /// Get a process instance
    pub async fn get_instance(&self, instance_id: uuid::Uuid) -> Option<ProcessInstance> {
        let instances = self.instances.read().await;
        instances.get(&instance_id).cloned()
    }

    /// Cancel a process instance
    pub async fn cancel_instance(&self, instance_id: uuid::Uuid) -> Result<()> {
        let mut instances = self.instances.write().await;
        if let Some(instance) = instances.get_mut(&instance_id) {
            instance.state = ProcessState::Cancelled;
            instance.completed_at = Some(chrono::Utc::now());
            Ok(())
        } else {
            Err(AbcdodafError::WorkflowError(
                "Instance not found".to_string(),
            ))
        }
    }
}

impl Default for ProcessExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bpmn::ProcessBuilder;

    struct MockTaskHandler;

    #[async_trait]
    impl TaskHandler for MockTaskHandler {
        async fn execute(
            &self,
            _task_id: &str,
            variables: &HashMap<String, serde_json::Value>,
        ) -> Result<HashMap<String, serde_json::Value>> {
            let mut output = variables.clone();
            output.insert("executed".to_string(), serde_json::json!(true));
            Ok(output)
        }
    }

    #[tokio::test]
    async fn test_process_execution() {
        let process = ProcessBuilder::new("test", "Test")
            .add_user_task("t1", "Task 1")
            .build()
            .unwrap();

        let executor = ProcessExecutor::new().register_handler(
            "user",
            Arc::new(MockTaskHandler),
        );

        let instance = executor
            .execute_process(&process, HashMap::new())
            .await
            .unwrap();

        assert_eq!(instance.state, ProcessState::Completed);
        assert!(instance.get_variable("executed").is_some());
    }
}
