//! Workflow orchestration for AI workforce
//!
//! Provides high-level workflow building and execution capabilities
//! that combine BPMN processes with DoDAF architecture and workforce tasks.

use super::{
    AgentTask, ExecutionMetrics, HumanTask, SystemTask, TaskResult, WorkforceExecutionResult,
    WorkforceTask,
};
use crate::bpmn::executor::TaskHandler;
use crate::bpmn::{Process, ProcessBuilder, ProcessExecutor};
use crate::dodaf::OperationalContext;
use crate::error::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

/// Workflow builder for creating AI workforce workflows
pub struct WorkflowBuilder {
    process_id: String,
    process_name: String,
    process_description: Option<String>,
    tasks: Vec<WorkforceTask>,
    context: OperationalContext,
}

impl WorkflowBuilder {
    /// Create a new workflow builder
    pub fn new(id: impl Into<String>) -> Self {
        let id_str = id.into();
        Self {
            process_id: id_str.clone(),
            process_name: id_str,
            process_description: None,
            tasks: Vec::new(),
            context: OperationalContext::new(),
        }
    }

    /// Set workflow name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.process_name = name.into();
        self
    }

    /// Set workflow description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.process_description = Some(desc.into());
        self
    }

    /// Add an agent task
    pub fn add_agent_task(mut self, _id: impl Into<String>, task: AgentTask) -> Self {
        self.tasks.push(WorkforceTask::Agent(task));
        self
    }

    /// Add a human task
    pub fn add_human_task(mut self, _id: impl Into<String>, task: HumanTask) -> Self {
        self.tasks.push(WorkforceTask::Human(task));
        self
    }

    /// Add a system task
    pub fn add_system_task(mut self, _id: impl Into<String>, task: SystemTask) -> Self {
        self.tasks.push(WorkforceTask::System(task));
        self
    }

    /// Set operational context
    pub fn with_context(mut self, context: OperationalContext) -> Self {
        self.context = context;
        self
    }

    /// Build the workflow
    pub fn build(self) -> Result<Workflow> {
        // Build BPMN process from tasks
        let mut builder = ProcessBuilder::new(&self.process_id, &self.process_name);

        if let Some(desc) = self.process_description {
            builder = builder.description(desc);
        }

        // Add tasks to BPMN process
        for task in &self.tasks {
            match task {
                WorkforceTask::Agent(t) => {
                    builder = builder.add_service_task(&t.id, &t.name);
                },
                WorkforceTask::Human(t) => {
                    builder = builder.add_user_task(&t.id, &t.name);
                },
                WorkforceTask::System(t) => {
                    builder = builder.add_service_task(&t.id, &t.name);
                },
            }
        }

        let process = builder.build()?;
        Ok(Workflow { process, tasks: self.tasks, context: self.context })
    }
}

/// Executable workflow
#[derive(Debug, Clone)]
pub struct Workflow {
    /// BPMN process definition
    pub process: Process,
    /// Workforce tasks
    pub tasks: Vec<WorkforceTask>,
    /// Operational context
    pub context: OperationalContext,
}

impl Workflow {
    /// Execute the workflow
    pub async fn execute(self) -> Result<WorkforceExecutionResult> {
        let executor = WorkflowExecution::new(self);
        executor.run().await
    }

    /// Execute with custom handlers
    pub async fn execute_with_handlers(
        self,
        handlers: HashMap<String, Arc<dyn TaskHandler>>,
    ) -> Result<WorkforceExecutionResult> {
        let executor = WorkflowExecution::new(self).with_handlers(handlers);
        executor.run().await
    }
}

/// Workflow execution engine
pub struct WorkflowExecution {
    workflow: Workflow,
    executor: ProcessExecutor,
    custom_handlers: HashMap<String, Arc<dyn TaskHandler>>,
}

impl WorkflowExecution {
    /// Create a new workflow execution
    pub fn new(workflow: Workflow) -> Self {
        Self { workflow, executor: ProcessExecutor::new(), custom_handlers: HashMap::new() }
    }

    /// Add custom task handlers
    pub fn with_handlers(mut self, handlers: HashMap<String, Arc<dyn TaskHandler>>) -> Self {
        self.custom_handlers = handlers;
        self
    }

    /// Run the workflow
    pub async fn run(mut self) -> Result<WorkforceExecutionResult> {
        let start_time = Instant::now();

        // Register default handlers
        self.register_default_handlers();

        // Execute the process
        let mut variables = HashMap::new();
        variables.insert("context".to_string(), serde_json::to_value(&self.workflow.context)?);

        let instance = self.executor.execute_process(&self.workflow.process, variables).await?;

        // Collect task results (mock for now - in production, track actual executions)
        let task_results: Vec<TaskResult> = self
            .workflow
            .tasks
            .iter()
            .map(|task| TaskResult {
                task_id: task.id().to_string(),
                success: true,
                error: None,
                output: serde_json::json!({}),
                duration_ms: 100,
            })
            .collect();

        // Calculate metrics
        let total_duration = start_time.elapsed().as_millis() as u64;
        let agent_tasks = self
            .workflow
            .tasks
            .iter()
            .filter(|t| matches!(t, WorkforceTask::Agent(_)))
            .count();
        let human_tasks = self
            .workflow
            .tasks
            .iter()
            .filter(|t| matches!(t, WorkforceTask::Human(_)))
            .count();
        let system_tasks = self
            .workflow
            .tasks
            .iter()
            .filter(|t| matches!(t, WorkforceTask::System(_)))
            .count();

        let successful = task_results.iter().filter(|r| r.success).count();
        let success_rate = if task_results.is_empty() {
            1.0
        } else {
            successful as f64 / task_results.len() as f64
        };

        Ok(WorkforceExecutionResult {
            instance,
            task_results,
            metrics: ExecutionMetrics {
                total_duration_ms: total_duration,
                agent_tasks,
                human_tasks,
                system_tasks,
                success_rate,
            },
        })
    }

    fn register_default_handlers(&mut self) {
        // Register custom handlers from the map
        for (task_type, handler) in &self.custom_handlers {
            self.executor =
                std::mem::take(&mut self.executor).register_handler(task_type, handler.clone());
        }

        // Add default handlers if not provided
        if !self.custom_handlers.contains_key("service") {
            self.executor = std::mem::take(&mut self.executor)
                .register_handler("service", Arc::new(DefaultServiceHandler));
        }
        if !self.custom_handlers.contains_key("user") {
            self.executor = std::mem::take(&mut self.executor)
                .register_handler("user", Arc::new(DefaultUserHandler));
        }
    }
}

/// Default handler for service tasks
struct DefaultServiceHandler;

#[async_trait]
impl TaskHandler for DefaultServiceHandler {
    async fn execute(
        &self,
        _task_id: &str,
        variables: &HashMap<String, serde_json::Value>,
    ) -> Result<HashMap<String, serde_json::Value>> {
        // Default implementation - pass through variables
        Ok(variables.clone())
    }
}

/// Default handler for user tasks
struct DefaultUserHandler;

#[async_trait]
impl TaskHandler for DefaultUserHandler {
    async fn execute(
        &self,
        _task_id: &str,
        variables: &HashMap<String, serde_json::Value>,
    ) -> Result<HashMap<String, serde_json::Value>> {
        // Default implementation - simulate user approval
        let mut result = variables.clone();
        result.insert("user_approved".to_string(), serde_json::json!(true));
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workforce::{AgentCapability, HumanRole, SystemOperation};

    #[tokio::test]
    async fn test_workflow_builder() {
        let workflow = WorkflowBuilder::new("test_workflow")
            .name("Test Workflow")
            .description("A test workflow")
            .add_agent_task(
                "agent1",
                AgentTask::new("agent1", "Process", AgentCapability::NaturalLanguageProcessing),
            )
            .add_human_task(
                "human1",
                HumanTask::new("human1", "Review", HumanRole::QualityAssurance),
            )
            .add_system_task(
                "system1",
                SystemTask::new("system1", "Save", SystemOperation::DatabaseWrite),
            )
            .build()
            .unwrap();

        assert_eq!(workflow.tasks.len(), 3);
        assert_eq!(workflow.process.tasks.len(), 3);
    }

    #[tokio::test]
    async fn test_workflow_execution() {
        let workflow = WorkflowBuilder::new("exec_test")
            .name("Execution Test")
            .add_agent_task("a1", AgentTask::new("a1", "Agent", AgentCapability::CodeGeneration))
            .build()
            .unwrap();

        let result = workflow.execute().await.unwrap();
        assert_eq!(result.metrics.agent_tasks, 1);
        assert!(result.metrics.success_rate > 0.0);
    }
}
