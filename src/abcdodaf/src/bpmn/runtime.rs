//! Enhanced execution runtime with debugging and visualization support

use super::{Process, ProcessInstance, ProcessState};
use super::process::Task;
use crate::error::{AbcdodafError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{info, warn};
use uuid::Uuid;

/// Token representing execution flow in BPMN
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionToken {
    /// Token ID
    pub id: Uuid,
    /// Current element ID the token is at
    pub current_element: String,
    /// Token state
    pub state: TokenState,
    /// Token creation time
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Token variables (local scope)
    pub variables: HashMap<String, serde_json::Value>,
}

/// Token state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenState {
    /// Token is active and moving
    Active,
    /// Token is waiting at a node
    Waiting,
    /// Token has been consumed
    Consumed,
    /// Token is blocked (breakpoint)
    Blocked,
}

/// Execution event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionEvent {
    /// Process started
    ProcessStarted {
        instance_id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Token created at element
    TokenCreated {
        token_id: Uuid,
        element_id: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Token moved to new element
    TokenMoved {
        token_id: Uuid,
        from_element: String,
        to_element: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Task started execution
    TaskStarted {
        task_id: String,
        token_id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Task completed successfully
    TaskCompleted {
        task_id: String,
        token_id: Uuid,
        duration_ms: u64,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Task failed with error
    TaskFailed {
        task_id: String,
        token_id: Uuid,
        error: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Variable changed
    VariableChanged {
        variable_name: String,
        old_value: Option<serde_json::Value>,
        new_value: serde_json::Value,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Gateway evaluated
    GatewayEvaluated {
        gateway_id: String,
        outgoing_flows: Vec<String>,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Breakpoint hit
    BreakpointHit {
        element_id: String,
        token_id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Process completed
    ProcessCompleted {
        instance_id: Uuid,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Process failed
    ProcessFailed {
        instance_id: Uuid,
        error: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// Execution context for a running process
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Instance being executed
    pub instance: ProcessInstance,
    /// Active tokens
    pub tokens: Vec<ExecutionToken>,
    /// Current element being executed (for highlighting)
    pub current_element: Option<String>,
    /// Execution mode
    pub mode: ExecutionMode,
    /// Task performance data
    pub performance: HashMap<String, TaskPerformance>,
}

/// Execution mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMode {
    /// Normal continuous execution
    Continuous,
    /// Step-by-step execution (pauses after each step)
    StepByStep,
    /// Paused (waiting for resume)
    Paused,
}

/// Task performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPerformance {
    /// Task ID
    pub task_id: String,
    /// Number of executions
    pub execution_count: u64,
    /// Total duration in milliseconds
    pub total_duration_ms: u64,
    /// Average duration
    pub avg_duration_ms: u64,
    /// Min duration
    pub min_duration_ms: u64,
    /// Max duration
    pub max_duration_ms: u64,
    /// Last execution timestamp
    pub last_execution: chrono::DateTime<chrono::Utc>,
}

impl TaskPerformance {
    pub fn new(task_id: String) -> Self {
        Self {
            task_id,
            execution_count: 0,
            total_duration_ms: 0,
            avg_duration_ms: 0,
            min_duration_ms: u64::MAX,
            max_duration_ms: 0,
            last_execution: chrono::Utc::now(),
        }
    }

    pub fn record_execution(&mut self, duration_ms: u64) {
        self.execution_count += 1;
        self.total_duration_ms += duration_ms;
        self.avg_duration_ms = self.total_duration_ms / self.execution_count;
        self.min_duration_ms = self.min_duration_ms.min(duration_ms);
        self.max_duration_ms = self.max_duration_ms.max(duration_ms);
        self.last_execution = chrono::Utc::now();
    }
}

/// Breakpoint for debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    /// Element ID where breakpoint is set
    pub element_id: String,
    /// Whether breakpoint is enabled
    pub enabled: bool,
    /// Condition (optional - if None, always breaks)
    pub condition: Option<String>,
    /// Hit count
    pub hit_count: u64,
}

impl Breakpoint {
    pub fn new(element_id: String) -> Self {
        Self {
            element_id,
            enabled: true,
            condition: None,
            hit_count: 0,
        }
    }

    pub fn with_condition(element_id: String, condition: String) -> Self {
        Self {
            element_id,
            enabled: true,
            condition: Some(condition),
            hit_count: 0,
        }
    }
}

/// Enhanced runtime executor with debugging support
pub struct EnhancedRuntime {
    /// Task handlers
    handlers: HashMap<String, Arc<dyn TaskHandler>>,
    /// Active execution contexts
    contexts: Arc<RwLock<HashMap<Uuid, ExecutionContext>>>,
    /// Event broadcast channel
    event_tx: broadcast::Sender<ExecutionEvent>,
    /// Audit trail (execution history)
    audit_trail: Arc<RwLock<VecDeque<ExecutionEvent>>>,
    /// Breakpoints
    breakpoints: Arc<RwLock<HashMap<String, Breakpoint>>>,
    /// Maximum audit trail size
    max_audit_size: usize,
}

/// Task handler trait for execution
#[async_trait]
pub trait TaskHandler: Send + Sync {
    /// Execute a task with given context
    async fn execute(
        &self,
        task: &Task,
        variables: &HashMap<String, serde_json::Value>,
    ) -> Result<HashMap<String, serde_json::Value>>;

    /// Get handler name for logging
    fn name(&self) -> &str {
        "unnamed_handler"
    }
}

impl EnhancedRuntime {
    /// Create a new enhanced runtime
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(1000);
        Self {
            handlers: HashMap::new(),
            contexts: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            audit_trail: Arc::new(RwLock::new(VecDeque::new())),
            breakpoints: Arc::new(RwLock::new(HashMap::new())),
            max_audit_size: 10000,
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

    /// Subscribe to execution events
    pub fn subscribe_events(&self) -> broadcast::Receiver<ExecutionEvent> {
        self.event_tx.subscribe()
    }

    /// Emit an execution event
    async fn emit_event(&self, event: ExecutionEvent) {
        // Store in audit trail
        let mut audit = self.audit_trail.write().await;
        audit.push_back(event.clone());

        // Trim if exceeds max size
        while audit.len() > self.max_audit_size {
            audit.pop_front();
        }
        drop(audit);

        // Broadcast to subscribers
        let _ = self.event_tx.send(event);
    }

    /// Get audit trail (execution history)
    pub async fn get_audit_trail(&self) -> Vec<ExecutionEvent> {
        let audit = self.audit_trail.read().await;
        audit.iter().cloned().collect()
    }

    /// Get recent audit events
    pub async fn get_recent_audit(&self, count: usize) -> Vec<ExecutionEvent> {
        let audit = self.audit_trail.read().await;
        audit.iter().rev().take(count).cloned().collect()
    }

    /// Add a breakpoint
    pub async fn add_breakpoint(&self, element_id: String) {
        let mut breakpoints = self.breakpoints.write().await;
        breakpoints.insert(element_id.clone(), Breakpoint::new(element_id));
    }

    /// Add a conditional breakpoint
    pub async fn add_conditional_breakpoint(&self, element_id: String, condition: String) {
        let mut breakpoints = self.breakpoints.write().await;
        breakpoints.insert(
            element_id.clone(),
            Breakpoint::with_condition(element_id, condition),
        );
    }

    /// Remove a breakpoint
    pub async fn remove_breakpoint(&self, element_id: &str) {
        let mut breakpoints = self.breakpoints.write().await;
        breakpoints.remove(element_id);
    }

    /// Toggle breakpoint enabled state
    pub async fn toggle_breakpoint(&self, element_id: &str) {
        let mut breakpoints = self.breakpoints.write().await;
        if let Some(bp) = breakpoints.get_mut(element_id) {
            bp.enabled = !bp.enabled;
        }
    }

    /// Get all breakpoints
    pub async fn get_breakpoints(&self) -> Vec<Breakpoint> {
        let breakpoints = self.breakpoints.read().await;
        breakpoints.values().cloned().collect()
    }

    /// Check if should break at element
    async fn should_break(&self, element_id: &str, _variables: &HashMap<String, serde_json::Value>) -> bool {
        let mut breakpoints = self.breakpoints.write().await;
        if let Some(bp) = breakpoints.get_mut(element_id) {
            if bp.enabled {
                bp.hit_count += 1;
                // TODO: Evaluate condition if present
                return true;
            }
        }
        false
    }

    /// Start a process instance
    pub async fn start_process(
        &self,
        process: &Process,
        mode: ExecutionMode,
    ) -> Result<Uuid> {
        let mut instance = ProcessInstance::new(&process.id);
        instance.state = ProcessState::Running;
        let instance_id = instance.id;

        // Create initial token at start event
        let token = ExecutionToken {
            id: Uuid::new_v4(),
            current_element: "start".to_string(), // Simplified - should find actual start event
            state: TokenState::Active,
            created_at: chrono::Utc::now(),
            variables: HashMap::new(),
        };

        let context = ExecutionContext {
            instance: instance.clone(),
            tokens: vec![token.clone()],
            current_element: Some("start".to_string()),
            mode,
            performance: HashMap::new(),
        };

        let mut contexts = self.contexts.write().await;
        contexts.insert(instance_id, context);

        // Emit events
        self.emit_event(ExecutionEvent::ProcessStarted {
            instance_id,
            timestamp: chrono::Utc::now(),
        }).await;

        self.emit_event(ExecutionEvent::TokenCreated {
            token_id: token.id,
            element_id: "start".to_string(),
            timestamp: chrono::Utc::now(),
        }).await;

        info!("Started process instance: {} in mode {:?}", instance_id, mode);
        Ok(instance_id)
    }

    /// Get execution context
    pub async fn get_context(&self, instance_id: Uuid) -> Option<ExecutionContext> {
        let contexts = self.contexts.read().await;
        contexts.get(&instance_id).cloned()
    }

    /// Get performance data for an instance
    pub async fn get_performance(&self, instance_id: Uuid) -> Option<HashMap<String, TaskPerformance>> {
        let contexts = self.contexts.read().await;
        contexts.get(&instance_id).map(|ctx| ctx.performance.clone())
    }

    /// Pause execution
    pub async fn pause(&self, instance_id: Uuid) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        if let Some(context) = contexts.get_mut(&instance_id) {
            context.mode = ExecutionMode::Paused;
            Ok(())
        } else {
            Err(AbcdodafError::WorkflowError("Instance not found".to_string()))
        }
    }

    /// Resume execution
    pub async fn resume(&self, instance_id: Uuid) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        if let Some(context) = contexts.get_mut(&instance_id) {
            context.mode = ExecutionMode::Continuous;
            Ok(())
        } else {
            Err(AbcdodafError::WorkflowError("Instance not found".to_string()))
        }
    }

    /// Step to next element (for step-by-step debugging)
    pub async fn step(&self, instance_id: Uuid) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        if let Some(context) = contexts.get_mut(&instance_id) {
            context.mode = ExecutionMode::StepByStep;
            // The actual stepping logic would be in execute_process
            Ok(())
        } else {
            Err(AbcdodafError::WorkflowError("Instance not found".to_string()))
        }
    }

    /// Cancel execution
    pub async fn cancel(&self, instance_id: Uuid) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        if let Some(context) = contexts.get_mut(&instance_id) {
            context.instance.state = ProcessState::Cancelled;
            context.instance.completed_at = Some(chrono::Utc::now());
            Ok(())
        } else {
            Err(AbcdodafError::WorkflowError("Instance not found".to_string()))
        }
    }

    /// Execute a task within the runtime
    async fn execute_task(
        &self,
        task: &Task,
        token: &ExecutionToken,
        instance_id: Uuid,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let task_type = format!("{:?}", task.task_type).to_lowercase();

        // Check for breakpoint
        if self.should_break(&task.id, &token.variables).await {
            self.emit_event(ExecutionEvent::BreakpointHit {
                element_id: task.id.clone(),
                token_id: token.id,
                timestamp: chrono::Utc::now(),
            }).await;

            // Pause execution
            self.pause(instance_id).await?;
        }

        // Emit task started event
        self.emit_event(ExecutionEvent::TaskStarted {
            task_id: task.id.clone(),
            token_id: token.id,
            timestamp: chrono::Utc::now(),
        }).await;

        let start_time = std::time::Instant::now();

        // Execute handler
        let result = if let Some(handler) = self.handlers.get(&task_type) {
            handler.execute(task, &token.variables).await
        } else {
            warn!("No handler found for task type: {}", task_type);
            Ok(HashMap::new())
        };

        let duration_ms = start_time.elapsed().as_millis() as u64;

        // Update performance metrics
        {
            let mut contexts = self.contexts.write().await;
            if let Some(context) = contexts.get_mut(&instance_id) {
                let perf = context.performance
                    .entry(task.id.clone())
                    .or_insert_with(|| TaskPerformance::new(task.id.clone()));
                perf.record_execution(duration_ms);
            }
        }

        // Emit completion or failure event
        match &result {
            Ok(_) => {
                self.emit_event(ExecutionEvent::TaskCompleted {
                    task_id: task.id.clone(),
                    token_id: token.id,
                    duration_ms,
                    timestamp: chrono::Utc::now(),
                }).await;
            }
            Err(e) => {
                self.emit_event(ExecutionEvent::TaskFailed {
                    task_id: task.id.clone(),
                    token_id: token.id,
                    error: e.to_string(),
                    timestamp: chrono::Utc::now(),
                }).await;
            }
        }

        result
    }

    /// Execute a process (simplified sequential execution)
    pub async fn execute_process(
        &self,
        process: &Process,
        initial_variables: HashMap<String, serde_json::Value>,
        mode: ExecutionMode,
    ) -> Result<ProcessInstance> {
        let instance_id = self.start_process(process, mode).await?;

        // Set initial variables
        {
            let mut contexts = self.contexts.write().await;
            if let Some(context) = contexts.get_mut(&instance_id) {
                context.instance.variables = initial_variables.clone();
            }
        }

        // Execute tasks sequentially (simplified - production would follow BPMN flow)
        for task in &process.tasks {
            // Check execution mode and state
            loop {
                let (mode, state) = {
                    let contexts = self.contexts.read().await;
                    contexts.get(&instance_id).map(|c| (c.mode, c.instance.state)).unwrap_or((ExecutionMode::Continuous, ProcessState::Running))
                };

                // Check if cancelled
                if state == ProcessState::Cancelled {
                    return Err(AbcdodafError::WorkflowError("Execution cancelled".to_string()));
                }

                // Check if paused
                if mode == ExecutionMode::Paused {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    continue;
                }

                break;
            }

            // Get current token
            let token = {
                let contexts = self.contexts.read().await;
                contexts.get(&instance_id)
                    .and_then(|c| c.tokens.first().cloned())
            };

            if let Some(token) = token {
                // Update current element
                {
                    let mut contexts = self.contexts.write().await;
                    if let Some(context) = contexts.get_mut(&instance_id) {
                        context.current_element = Some(task.id.clone());
                    }
                }

                // Execute task
                match self.execute_task(task, &token, instance_id).await {
                    Ok(output_vars) => {
                        // Update variables
                        let mut contexts = self.contexts.write().await;
                        if let Some(context) = contexts.get_mut(&instance_id) {
                            for (key, value) in output_vars {
                                context.instance.set_variable(key, value);
                            }
                        }
                    }
                    Err(e) => {
                        let mut contexts = self.contexts.write().await;
                        if let Some(context) = contexts.get_mut(&instance_id) {
                            context.instance.fail();
                        }

                        self.emit_event(ExecutionEvent::ProcessFailed {
                            instance_id,
                            error: e.to_string(),
                            timestamp: chrono::Utc::now(),
                        }).await;

                        return Err(e);
                    }
                }
            }
        }

        // Complete process
        let mut contexts = self.contexts.write().await;
        if let Some(context) = contexts.get_mut(&instance_id) {
            context.instance.complete();
            context.current_element = None;

            self.emit_event(ExecutionEvent::ProcessCompleted {
                instance_id,
                timestamp: chrono::Utc::now(),
            }).await;

            Ok(context.instance.clone())
        } else {
            Err(AbcdodafError::WorkflowError("Instance not found".to_string()))
        }
    }

    /// Get all active instances
    pub async fn get_active_instances(&self) -> Vec<(Uuid, ExecutionContext)> {
        let contexts = self.contexts.read().await;
        contexts.iter()
            .filter(|(_, ctx)| {
                matches!(ctx.instance.state, ProcessState::Running | ProcessState::Suspended)
            })
            .map(|(id, ctx)| (*id, ctx.clone()))
            .collect()
    }

    /// Get bottleneck analysis
    pub async fn analyze_bottlenecks(&self, instance_id: Uuid) -> Vec<(String, TaskPerformance)> {
        if let Some(context) = self.get_context(instance_id).await {
            let mut tasks: Vec<_> = context.performance.iter()
                .map(|(id, perf)| (id.clone(), perf.clone()))
                .collect();

            // Sort by average duration (descending)
            tasks.sort_by(|a, b| b.1.avg_duration_ms.cmp(&a.1.avg_duration_ms));
            tasks
        } else {
            Vec::new()
        }
    }
}

impl Default for EnhancedRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bpmn::{ProcessBuilder, TaskType};
    use crate::bpmn::process::Task;

    struct MockHandler;

    #[async_trait]
    impl TaskHandler for MockHandler {
        async fn execute(
            &self,
            _task: &Task,
            variables: &HashMap<String, serde_json::Value>,
        ) -> Result<HashMap<String, serde_json::Value>> {
            let mut output = variables.clone();
            output.insert("executed".to_string(), serde_json::json!(true));
            Ok(output)
        }

        fn name(&self) -> &str {
            "mock_handler"
        }
    }

    #[tokio::test]
    async fn test_enhanced_runtime_execution() {
        let process = ProcessBuilder::new("test", "Test")
            .add_user_task("t1", "Task 1")
            .build()
            .unwrap();

        let runtime = EnhancedRuntime::new()
            .register_handler("user", Arc::new(MockHandler));

        let mut event_rx = runtime.subscribe_events();

        // Spawn task to collect events
        tokio::spawn(async move {
            while let Ok(event) = event_rx.recv().await {
                println!("Event: {:?}", event);
            }
        });

        let result = runtime
            .execute_process(&process, HashMap::new(), ExecutionMode::Continuous)
            .await;

        assert!(result.is_ok());
        let instance = result.unwrap();
        assert_eq!(instance.state, ProcessState::Completed);
    }

    #[tokio::test]
    async fn test_breakpoints() {
        let runtime = EnhancedRuntime::new();

        runtime.add_breakpoint("task1".to_string()).await;
        assert_eq!(runtime.get_breakpoints().await.len(), 1);

        runtime.toggle_breakpoint("task1").await;
        let breakpoints = runtime.get_breakpoints().await;
        assert!(!breakpoints[0].enabled);

        runtime.remove_breakpoint("task1").await;
        assert_eq!(runtime.get_breakpoints().await.len(), 0);
    }

    #[tokio::test]
    async fn test_execution_control() {
        let process = ProcessBuilder::new("test", "Test")
            .add_user_task("t1", "Task 1")
            .build()
            .unwrap();

        let runtime = EnhancedRuntime::new()
            .register_handler("user", Arc::new(MockHandler));

        let instance_id = runtime
            .start_process(&process, ExecutionMode::Continuous)
            .await
            .unwrap();

        // Test pause/resume
        runtime.pause(instance_id).await.unwrap();
        let context = runtime.get_context(instance_id).await.unwrap();
        assert_eq!(context.mode, ExecutionMode::Paused);

        runtime.resume(instance_id).await.unwrap();
        let context = runtime.get_context(instance_id).await.unwrap();
        assert_eq!(context.mode, ExecutionMode::Continuous);
    }
}
