//! Mock task handlers for testing

use crate::bpmn::executor::TaskHandler;
use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Mock task handler for testing
pub struct MockTaskHandler {
    /// Handler behavior configuration
    pub behavior: MockBehavior,
    /// Recorded invocations
    executions: Arc<RwLock<Vec<ExecutionRecord>>>,
}

/// Behavior configuration for mock handlers
#[derive(Debug, Clone)]
pub enum MockBehavior {
    /// Always succeeds and returns the input as output
    PassThrough,
    /// Always succeeds with specific output
    FixedResponse(HashMap<String, serde_json::Value>),
    /// Simulates a delay (in milliseconds)
    SlowResponse {
        delay_ms: u64,
        output: HashMap<String, serde_json::Value>,
    },
    /// Simulates failure
    Failure(String),
    /// Returns different outputs based on input
    Conditional(Vec<(String, serde_json::Value, MockBehavior)>),
}

/// Record of a task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    /// Task ID
    pub task_id: String,
    /// Input variables
    pub input: HashMap<String, serde_json::Value>,
    /// Output variables
    pub output: Option<HashMap<String, serde_json::Value>>,
    /// Error if any
    pub error: Option<String>,
    /// Execution time in milliseconds
    pub duration_ms: u64,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl MockTaskHandler {
    /// Create a new mock handler with pass-through behavior
    pub fn new() -> Self {
        Self {
            behavior: MockBehavior::PassThrough,
            executions: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create with specific behavior
    pub fn with_behavior(behavior: MockBehavior) -> Self {
        Self {
            behavior,
            executions: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get all recorded executions
    pub async fn get_executions(&self) -> Vec<ExecutionRecord> {
        self.executions.read().await.clone()
    }

    /// Clear execution history
    pub async fn clear_executions(&self) {
        self.executions.write().await.clear();
    }

    /// Get execution count
    pub async fn execution_count(&self) -> usize {
        self.executions.read().await.len()
    }
}

impl Default for MockTaskHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TaskHandler for MockTaskHandler {
    async fn execute(
        &self,
        task_id: &str,
        variables: &HashMap<String, serde_json::Value>,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let start = std::time::Instant::now();
        let input = variables.clone();

        let result = match &self.behavior {
            MockBehavior::PassThrough => Ok(variables.clone()),

            MockBehavior::FixedResponse(output) => Ok(output.clone()),

            MockBehavior::SlowResponse { delay_ms, output } => {
                tokio::time::sleep(std::time::Duration::from_millis(*delay_ms)).await;
                Ok(output.clone())
            }

            MockBehavior::Failure(msg) => {
                Err(crate::error::AbcdodafError::WorkflowError(msg.clone()))
            }

            MockBehavior::Conditional(conditions) => {
                let mut matched = None;
                for (key, trigger_value, behavior) in conditions {
                    if variables.get(key) == Some(trigger_value) {
                        matched = Some(behavior);
                        break;
                    }
                }

                match matched {
                    Some(MockBehavior::FixedResponse(output)) => Ok(output.clone()),
                    Some(MockBehavior::Failure(msg)) => {
                        Err(crate::error::AbcdodafError::WorkflowError(msg.clone()))
                    }
                    _ => Ok(variables.clone()),
                }
            }
        };

        let duration_ms = start.elapsed().as_millis() as u64;
        let output = result.as_ref().ok().cloned();
        let error = result.as_ref().err().map(|e| e.to_string());

        let record = ExecutionRecord {
            task_id: task_id.to_string(),
            input,
            output,
            error,
            duration_ms,
            timestamp: chrono::Utc::now(),
        };

        self.executions.write().await.push(record);

        result
    }
}

/// Builder for mock handlers with fluent API
pub struct MockHandlerBuilder {
    behavior: MockBehavior,
}

impl MockHandlerBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            behavior: MockBehavior::PassThrough,
        }
    }

    /// Set pass-through behavior
    pub fn pass_through(mut self) -> Self {
        self.behavior = MockBehavior::PassThrough;
        self
    }

    /// Set fixed response
    pub fn fixed_response(mut self, response: HashMap<String, serde_json::Value>) -> Self {
        self.behavior = MockBehavior::FixedResponse(response);
        self
    }

    /// Set response with a single key-value
    pub fn fixed_response_value(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        let mut response = HashMap::new();
        response.insert(key.into(), value);
        self.behavior = MockBehavior::FixedResponse(response);
        self
    }

    /// Set slow response
    pub fn slow_response(
        mut self,
        delay_ms: u64,
        output: HashMap<String, serde_json::Value>,
    ) -> Self {
        self.behavior = MockBehavior::SlowResponse { delay_ms, output };
        self
    }

    /// Set failure behavior
    pub fn failure(mut self, message: impl Into<String>) -> Self {
        self.behavior = MockBehavior::Failure(message.into());
        self
    }

    /// Build the mock handler
    pub fn build(self) -> MockTaskHandler {
        MockTaskHandler::with_behavior(self.behavior)
    }

    /// Build as Arc for use in executors
    pub fn build_arc(self) -> Arc<dyn TaskHandler> {
        Arc::new(self.build())
    }
}

impl Default for MockHandlerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Recording mock handler that logs all invocations
pub struct RecordingMockHandler {
    /// Base behavior
    base: Arc<MockTaskHandler>,
    /// Call history
    history: Arc<RwLock<Vec<HandlerCall>>>,
}

/// A single handler call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandlerCall {
    /// Task ID
    pub task_id: String,
    /// Call order (sequential)
    pub sequence: usize,
    /// Input variables
    pub input: HashMap<String, serde_json::Value>,
    /// Output or error
    pub result: CallResult,
    /// Execution time
    pub duration_ms: u64,
}

/// Result of a handler call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CallResult {
    /// Successful execution
    Success(HashMap<String, serde_json::Value>),
    /// Failed execution
    Error(String),
}

impl RecordingMockHandler {
    /// Create a new recording handler
    pub fn new(base_behavior: MockBehavior) -> Self {
        Self {
            base: Arc::new(MockTaskHandler::with_behavior(base_behavior)),
            history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get call history
    pub async fn get_history(&self) -> Vec<HandlerCall> {
        self.history.read().await.clone()
    }

    /// Get calls for a specific task
    pub async fn get_task_calls(&self, task_id: &str) -> Vec<HandlerCall> {
        self.history
            .read()
            .await
            .iter()
            .filter(|c| c.task_id == task_id)
            .cloned()
            .collect()
    }

    /// Get call count
    pub async fn call_count(&self) -> usize {
        self.history.read().await.len()
    }

    /// Assert that a task was called
    pub async fn assert_called(&self, task_id: &str) -> bool {
        self.history
            .read()
            .await
            .iter()
            .any(|c| c.task_id == task_id)
    }

    /// Assert call order
    pub async fn assert_call_order(&self, expected_tasks: &[&str]) -> bool {
        let history = self.history.read().await;
        if history.len() != expected_tasks.len() {
            return false;
        }
        history
            .iter()
            .zip(expected_tasks)
            .all(|(call, expected_id)| call.task_id == *expected_id)
    }

    /// Clear history
    pub async fn clear_history(&self) {
        self.history.write().await.clear();
    }
}

#[async_trait]
impl TaskHandler for RecordingMockHandler {
    async fn execute(
        &self,
        task_id: &str,
        variables: &HashMap<String, serde_json::Value>,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let start = std::time::Instant::now();
        let sequence = self.history.read().await.len() + 1;

        let result = self.base.execute(task_id, variables).await;

        let duration_ms = start.elapsed().as_millis() as u64;
        let call_result = match &result {
            Ok(output) => CallResult::Success(output.clone()),
            Err(e) => CallResult::Error(e.to_string()),
        };

        let call = HandlerCall {
            task_id: task_id.to_string(),
            sequence,
            input: variables.clone(),
            result: call_result,
            duration_ms,
        };

        self.history.write().await.push(call);

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_handler_pass_through() {
        let handler = MockTaskHandler::new();
        let mut input = HashMap::new();
        input.insert("key".to_string(), serde_json::json!("value"));

        let output = handler.execute("task1", &input).await.unwrap();
        assert_eq!(output.get("key"), Some(&serde_json::json!("value")));
    }

    #[tokio::test]
    async fn test_mock_handler_fixed_response() {
        let mut response = HashMap::new();
        response.insert("result".to_string(), serde_json::json!("fixed"));

        let handler = MockTaskHandler::with_behavior(MockBehavior::FixedResponse(response));
        let output = handler.execute("task1", &HashMap::new()).await.unwrap();
        assert_eq!(output.get("result"), Some(&serde_json::json!("fixed")));
    }

    #[tokio::test]
    async fn test_mock_handler_builder() {
        let handler = MockHandlerBuilder::new()
            .fixed_response_value("status", serde_json::json!("ok"))
            .build();

        let output = handler.execute("task1", &HashMap::new()).await.unwrap();
        assert_eq!(output.get("status"), Some(&serde_json::json!("ok")));
    }

    #[tokio::test]
    async fn test_execution_recording() {
        let handler = MockTaskHandler::new();
        let mut input = HashMap::new();
        input.insert("test".to_string(), serde_json::json!("data"));

        handler.execute("task1", &input).await.unwrap();
        handler.execute("task2", &input).await.unwrap();

        assert_eq!(handler.execution_count().await, 2);
        let executions = handler.get_executions().await;
        assert_eq!(executions[0].task_id, "task1");
        assert_eq!(executions[1].task_id, "task2");
    }

    #[tokio::test]
    async fn test_recording_mock_handler() {
        let handler = RecordingMockHandler::new(MockBehavior::PassThrough);
        let input = HashMap::new();

        handler.execute("task1", &input).await.unwrap();
        handler.execute("task2", &input).await.unwrap();

        assert!(handler.assert_called("task1").await);
        assert!(handler.assert_called("task2").await);
        assert!(handler.assert_call_order(&["task1", "task2"]).await);
    }
}
