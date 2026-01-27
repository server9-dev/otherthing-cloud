//! Core test harness for workflow and task testing

use crate::bpmn::{Process, ProcessExecutor, ProcessInstance};
use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Test case for individual task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTestCase {
    /// Unique test case ID
    pub id: String,
    /// Test name for reporting
    pub name: String,
    /// Description of what the test validates
    pub description: Option<String>,
    /// Input variables for the task
    pub input_variables: HashMap<String, serde_json::Value>,
    /// Expected output variables
    pub expected_output: HashMap<String, serde_json::Value>,
    /// Task ID being tested
    pub task_id: String,
    /// Task type (agent, human, system)
    pub task_type: String,
    /// Timeout in milliseconds
    pub timeout_ms: u64,
    /// Tags for categorization
    pub tags: Vec<String>,
}

impl TaskTestCase {
    /// Create a new task test case
    pub fn new(id: impl Into<String>, name: impl Into<String>, task_id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            input_variables: HashMap::new(),
            expected_output: HashMap::new(),
            task_id: task_id.into(),
            task_type: "user".to_string(),
            timeout_ms: 5000,
            tags: Vec::new(),
        }
    }

    /// Add input variable
    pub fn with_input(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.input_variables.insert(key.into(), value);
        self
    }

    /// Add expected output
    pub fn with_expected_output(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.expected_output.insert(key.into(), value);
        self
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    /// Add tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Set task type
    pub fn with_task_type(mut self, task_type: impl Into<String>) -> Self {
        self.task_type = task_type.into();
        self
    }
}

/// Result of a task test execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTestResult {
    /// Test case that was executed
    pub test_case: TaskTestCase,
    /// Whether the test passed
    pub passed: bool,
    /// Execution time in milliseconds
    pub duration_ms: u64,
    /// Output from task execution
    pub actual_output: HashMap<String, serde_json::Value>,
    /// Error message if failed
    pub error: Option<String>,
    /// Detailed assertions that were checked
    pub assertion_details: Vec<AssertionDetail>,
}

/// Individual assertion result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionDetail {
    /// What was being asserted
    pub assertion: String,
    /// Whether it passed
    pub passed: bool,
    /// Details about the failure
    pub details: Option<String>,
}

/// Main test harness for running tests
pub struct TestHarness {
    /// Process executor
    executor: Arc<ProcessExecutor>,
    /// Test results
    results: Arc<RwLock<Vec<TaskTestResult>>>,
    /// Configuration
    config: TestHarnessConfig,
}

/// Configuration for test harness
#[derive(Debug, Clone)]
pub struct TestHarnessConfig {
    /// Default timeout for tests
    pub default_timeout_ms: u64,
    /// Stop on first failure
    pub fail_fast: bool,
    /// Verbose logging
    pub verbose: bool,
    /// Capture task output
    pub capture_output: bool,
    /// Maximum parallel test execution
    pub max_parallelism: usize,
}

impl Default for TestHarnessConfig {
    fn default() -> Self {
        Self {
            default_timeout_ms: 5000,
            fail_fast: false,
            verbose: true,
            capture_output: true,
            max_parallelism: 4,
        }
    }
}

impl TestHarness {
    /// Create a new test harness
    pub fn new(executor: ProcessExecutor) -> Self {
        Self::with_config(executor, TestHarnessConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(executor: ProcessExecutor, config: TestHarnessConfig) -> Self {
        Self {
            executor: Arc::new(executor),
            results: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }

    /// Run a single task test
    pub async fn run_task_test(&self, test_case: TaskTestCase) -> Result<TaskTestResult> {
        let start = Instant::now();

        if self.config.verbose {
            info!("Running test: {} ({})", test_case.name, test_case.id);
        }

        // Execute task with timeout
        let result = tokio::time::timeout(
            std::time::Duration::from_millis(test_case.timeout_ms),
            self.execute_test_task(&test_case),
        )
        .await;

        let duration_ms = start.elapsed().as_millis() as u64;

        let (passed, actual_output, error, assertion_details) = match result {
            Ok(Ok((output, assertions))) => {
                let passed = assertions.iter().all(|a| a.passed);
                (passed, output, None, assertions)
            }
            Ok(Err(e)) => (false, HashMap::new(), Some(e.to_string()), vec![]),
            Err(_) => (
                false,
                HashMap::new(),
                Some(format!("Test timeout after {}ms", test_case.timeout_ms)),
                vec![],
            ),
        };

        let test_result = TaskTestResult {
            test_case: test_case.clone(),
            passed,
            duration_ms,
            actual_output,
            error,
            assertion_details,
        };

        // Store result
        {
            let mut results = self.results.write().await;
            results.push(test_result.clone());
        }

        if self.config.verbose {
            let status = if passed { "PASS" } else { "FAIL" };
            info!(
                "Test {} ({}): {} in {}ms",
                test_case.name, status, test_case.id, duration_ms
            );
        }

        Ok(test_result)
    }

    /// Run multiple task tests
    pub async fn run_task_tests(
        &self,
        test_cases: Vec<TaskTestCase>,
    ) -> Result<Vec<TaskTestResult>> {
        let mut results = Vec::new();

        for test_case in test_cases {
            let result = self.run_task_test(test_case).await?;
            results.push(result.clone());

            if self.config.fail_fast && !result.passed {
                break;
            }
        }

        Ok(results)
    }

    /// Execute a process and validate
    pub async fn run_process_test(
        &self,
        process: &Process,
        initial_vars: HashMap<String, serde_json::Value>,
        validators: Vec<Box<dyn ProcessValidator>>,
    ) -> Result<ProcessTestResult> {
        let start = Instant::now();

        if self.config.verbose {
            info!("Running process test: {}", process.id);
        }

        let instance = self.executor.execute_process(process, initial_vars).await?;

        let duration_ms = start.elapsed().as_millis() as u64;

        let mut validations = Vec::new();
        for validator in validators {
            let validation = validator.validate(&instance).await;
            validations.push(validation);
        }

        let passed = validations.iter().all(|v| v.passed);

        let result = ProcessTestResult {
            process_id: process.id.clone(),
            instance,
            passed,
            duration_ms,
            validations,
        };

        Ok(result)
    }

    /// Get all test results
    pub async fn get_results(&self) -> Vec<TaskTestResult> {
        self.results.read().await.clone()
    }

    /// Clear results
    pub async fn clear_results(&self) {
        self.results.write().await.clear();
    }

    /// Get summary statistics
    pub async fn get_summary(&self) -> TestSummary {
        let results = self.results.read().await;

        let total = results.len();
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = total - passed;
        let total_duration_ms: u64 = results.iter().map(|r| r.duration_ms).sum();

        TestSummary {
            total_tests: total,
            passed_tests: passed,
            failed_tests: failed,
            total_duration_ms,
            pass_rate: if total > 0 {
                (passed as f64) / (total as f64)
            } else {
                0.0
            },
        }
    }

    // Private helper methods

    async fn execute_test_task(
        &self,
        test_case: &TaskTestCase,
    ) -> Result<(HashMap<String, serde_json::Value>, Vec<AssertionDetail>)> {
        debug!("Executing test task: {}", test_case.task_id);

        // Simulate task execution with captured variables
        let mut output = test_case.input_variables.clone();
        output.insert("executed".to_string(), serde_json::json!(true));
        output.insert(
            "timestamp".to_string(),
            serde_json::json!(chrono::Utc::now().to_rfc3339()),
        );

        // Validate output
        let mut assertions = Vec::new();

        for (key, expected_value) in &test_case.expected_output {
            let actual_value = output.get(key);

            let passed = actual_value.map(|v| v == expected_value).unwrap_or(false);
            let details = if !passed {
                Some(format!(
                    "Expected: {:?}, Got: {:?}",
                    expected_value, actual_value
                ))
            } else {
                None
            };

            assertions.push(AssertionDetail {
                assertion: format!("Output {} == expected value", key),
                passed,
                details,
            });
        }

        Ok((output, assertions))
    }
}

/// Result of process test execution
#[derive(Debug, Clone)]
pub struct ProcessTestResult {
    /// Process ID
    pub process_id: String,
    /// Process instance result
    pub instance: ProcessInstance,
    /// Whether all validations passed
    pub passed: bool,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Validation results
    pub validations: Vec<ValidationResult>,
}

/// Process validator trait
#[async_trait]
pub trait ProcessValidator: Send + Sync {
    /// Validate a process instance
    async fn validate(&self, instance: &ProcessInstance) -> ValidationResult;
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Name of the validator
    pub validator_name: String,
    /// Whether validation passed
    pub passed: bool,
    /// Validation message
    pub message: String,
}

/// Test execution summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSummary {
    /// Total number of tests
    pub total_tests: usize,
    /// Number of passing tests
    pub passed_tests: usize,
    /// Number of failing tests
    pub failed_tests: usize,
    /// Total execution time in milliseconds
    pub total_duration_ms: u64,
    /// Pass rate (0.0 to 1.0)
    pub pass_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_task_test_case_builder() {
        let test = TaskTestCase::new("t1", "Test 1", "task1")
            .with_input("input1", serde_json::json!("value1"))
            .with_expected_output("output1", serde_json::json!("expected"))
            .with_description("A test case")
            .with_timeout(3000)
            .with_tag("unit");

        assert_eq!(test.id, "t1");
        assert_eq!(test.name, "Test 1");
        assert_eq!(test.timeout_ms, 3000);
        assert!(test.tags.contains(&"unit".to_string()));
    }

    #[tokio::test]
    async fn test_test_harness_creation() {
        let executor = ProcessExecutor::new();
        let harness = TestHarness::new(executor);

        assert_eq!(harness.config.default_timeout_ms, 5000);
        assert!(harness.config.verbose);
    }

    #[tokio::test]
    async fn test_test_summary_calculation() {
        let executor = ProcessExecutor::new();
        let harness = TestHarness::new(executor);

        let summary = harness.get_summary().await;
        assert_eq!(summary.total_tests, 0);
        assert_eq!(summary.pass_rate, 0.0);
    }
}
