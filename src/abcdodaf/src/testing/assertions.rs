//! Assertion framework for workflow and task testing

use crate::bpmn::ProcessInstance;
use crate::workforce::{ExecutionMetrics, TaskResult};
use serde::{Deserialize, Serialize};

/// Result of an assertion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionResult {
    /// Description of what was asserted
    pub description: String,
    /// Whether the assertion passed
    pub passed: bool,
    /// Detailed message
    pub message: String,
}

impl AssertionResult {
    /// Create a passing assertion
    pub fn passed(description: impl Into<String>, message: impl Into<String>) -> Self {
        Self { description: description.into(), passed: true, message: message.into() }
    }

    /// Create a failing assertion
    pub fn failed(description: impl Into<String>, message: impl Into<String>) -> Self {
        Self { description: description.into(), passed: false, message: message.into() }
    }
}

/// Assertions for workflow execution
pub struct WorkflowAssertions {
    assertions: Vec<AssertionResult>,
}

impl WorkflowAssertions {
    /// Create new assertions
    pub fn new() -> Self {
        Self { assertions: Vec::new() }
    }

    /// Assert total execution time
    pub fn assert_total_duration_less_than(
        mut self,
        metrics: &ExecutionMetrics,
        max_ms: u64,
    ) -> Self {
        // Note: ExecutionMetrics::total_duration_ms
        let passed = metrics.total_duration_ms < max_ms;
        let assertion = AssertionResult {
            description: format!("Total duration < {}ms", max_ms),
            passed,
            message: if passed {
                format!("Total duration {}ms is within limit", metrics.total_duration_ms)
            } else {
                format!("Total duration {}ms exceeds limit {}ms", metrics.total_duration_ms, max_ms)
            },
        };
        self.assertions.push(assertion);
        self
    }

    /// Assert minimum success rate
    pub fn assert_success_rate(mut self, metrics: &ExecutionMetrics, min_rate: f64) -> Self {
        let passed = metrics.success_rate >= min_rate;
        let assertion = AssertionResult {
            description: format!("Success rate >= {:.2}", min_rate),
            passed,
            message: if passed {
                format!(
                    "Success rate {:.2}% meets minimum {:.2}%",
                    metrics.success_rate * 100.0,
                    min_rate * 100.0
                )
            } else {
                format!(
                    "Success rate {:.2}% below minimum {:.2}%",
                    metrics.success_rate * 100.0,
                    min_rate * 100.0
                )
            },
        };
        self.assertions.push(assertion);
        self
    }

    /// Assert expected task counts
    pub fn assert_task_counts(
        mut self,
        metrics: &ExecutionMetrics,
        expected_agents: Option<usize>,
        expected_humans: Option<usize>,
        expected_systems: Option<usize>,
    ) -> Self {
        if let Some(expected) = expected_agents {
            let passed = metrics.agent_tasks == expected;
            self.assertions.push(AssertionResult {
                description: format!("Agent tasks == {}", expected),
                passed,
                message: if passed {
                    format!("Agent tasks count matches: {}", expected)
                } else {
                    format!("Agent tasks expected {}, got {}", expected, metrics.agent_tasks)
                },
            });
        }

        if let Some(expected) = expected_humans {
            let passed = metrics.human_tasks == expected;
            self.assertions.push(AssertionResult {
                description: format!("Human tasks == {}", expected),
                passed,
                message: if passed {
                    format!("Human tasks count matches: {}", expected)
                } else {
                    format!("Human tasks expected {}, got {}", expected, metrics.human_tasks)
                },
            });
        }

        if let Some(expected) = expected_systems {
            let passed = metrics.system_tasks == expected;
            self.assertions.push(AssertionResult {
                description: format!("System tasks == {}", expected),
                passed,
                message: if passed {
                    format!("System tasks count matches: {}", expected)
                } else {
                    format!("System tasks expected {}, got {}", expected, metrics.system_tasks)
                },
            });
        }

        self
    }

    /// Get all assertions
    pub fn get_assertions(&self) -> &[AssertionResult] {
        &self.assertions
    }

    /// Check if all assertions passed
    pub fn all_passed(&self) -> bool {
        self.assertions.iter().all(|a| a.passed)
    }

    /// Get pass count
    pub fn passed_count(&self) -> usize {
        self.assertions.iter().filter(|a| a.passed).count()
    }

    /// Get fail count
    pub fn failed_count(&self) -> usize {
        self.assertions.iter().filter(|a| !a.passed).count()
    }
}

impl Default for WorkflowAssertions {
    fn default() -> Self {
        Self::new()
    }
}

/// Assertions for individual task execution
pub struct TaskAssertions {
    assertions: Vec<AssertionResult>,
}

impl TaskAssertions {
    /// Create new task assertions
    pub fn new() -> Self {
        Self { assertions: Vec::new() }
    }

    /// Assert task succeeded
    pub fn assert_success(mut self, result: &TaskResult) -> Self {
        let assertion = AssertionResult {
            description: "Task execution succeeded".to_string(),
            passed: result.success,
            message: if result.success {
                "Task completed successfully".to_string()
            } else {
                format!("Task failed: {}", result.error.as_deref().unwrap_or("unknown error"))
            },
        };
        self.assertions.push(assertion);
        self
    }

    /// Assert task output contains key
    pub fn assert_output_contains_key(mut self, result: &TaskResult, key: &str) -> Self {
        let passed = result.output.get(key).is_some();
        let assertion = AssertionResult {
            description: format!("Output contains key '{}'", key),
            passed,
            message: if passed {
                format!("Output contains expected key '{}'", key)
            } else {
                format!("Output missing expected key '{}'", key)
            },
        };
        self.assertions.push(assertion);
        self
    }

    /// Assert task output value
    pub fn assert_output_value(
        mut self,
        result: &TaskResult,
        key: &str,
        expected: serde_json::Value,
    ) -> Self {
        let passed = result.output.get(key) == Some(&expected);
        let actual = result.output.get(key);
        let assertion = AssertionResult {
            description: format!("Output[{}] == {:?}", key, expected),
            passed,
            message: if passed {
                format!("Output[{}] has expected value", key)
            } else {
                format!("Output[{}] expected {:?}, got {:?}", key, expected, actual)
            },
        };
        self.assertions.push(assertion);
        self
    }

    /// Assert execution time
    pub fn assert_duration_less_than(mut self, result: &TaskResult, max_ms: u64) -> Self {
        let passed = result.duration_ms < max_ms;
        let assertion = AssertionResult {
            description: format!("Execution time < {}ms", max_ms),
            passed,
            message: if passed {
                format!("Execution completed in {}ms", result.duration_ms)
            } else {
                format!("Execution took {}ms, exceeds limit {}ms", result.duration_ms, max_ms)
            },
        };
        self.assertions.push(assertion);
        self
    }

    /// Assert output matches predicate
    pub fn assert_output_matches<F>(
        mut self,
        result: &TaskResult,
        description: impl Into<String>,
        predicate: F,
    ) -> Self
    where
        F: Fn(&serde_json::Value) -> bool,
    {
        let passed = predicate(&result.output);
        let assertion = AssertionResult {
            description: description.into(),
            passed,
            message: if passed {
                "Output matches condition".to_string()
            } else {
                "Output does not match condition".to_string()
            },
        };
        self.assertions.push(assertion);
        self
    }

    /// Get all assertions
    pub fn get_assertions(&self) -> &[AssertionResult] {
        &self.assertions
    }

    /// Check if all assertions passed
    pub fn all_passed(&self) -> bool {
        self.assertions.iter().all(|a| a.passed)
    }

    /// Get pass/fail counts
    pub fn stats(&self) -> (usize, usize) {
        let passed = self.assertions.iter().filter(|a| a.passed).count();
        let failed = self.assertions.len() - passed;
        (passed, failed)
    }
}

impl Default for TaskAssertions {
    fn default() -> Self {
        Self::new()
    }
}

/// State assertions for process instances
pub struct StateAssertions;

impl StateAssertions {
    /// Assert process is completed
    pub fn assert_completed(instance: &ProcessInstance) -> AssertionResult {
        use crate::bpmn::ProcessState;
        let passed = instance.state == ProcessState::Completed;
        AssertionResult {
            description: "Process state is Completed".to_string(),
            passed,
            message: if passed {
                "Process completed successfully".to_string()
            } else {
                format!("Process in state {:?}, expected Completed", instance.state)
            },
        }
    }

    /// Assert process is running
    pub fn assert_running(instance: &ProcessInstance) -> AssertionResult {
        use crate::bpmn::ProcessState;
        let passed = instance.state == ProcessState::Running;
        AssertionResult {
            description: "Process state is Running".to_string(),
            passed,
            message: if passed {
                "Process is running".to_string()
            } else {
                format!("Process in state {:?}, expected Running", instance.state)
            },
        }
    }

    /// Assert process is not failed
    pub fn assert_not_failed(instance: &ProcessInstance) -> AssertionResult {
        use crate::bpmn::ProcessState;
        let passed = instance.state != ProcessState::Failed;
        AssertionResult {
            description: "Process state is not Failed".to_string(),
            passed,
            message: if passed {
                "Process did not fail".to_string()
            } else {
                "Process failed".to_string()
            },
        }
    }

    /// Assert process has variable
    pub fn assert_has_variable(instance: &ProcessInstance, key: &str) -> AssertionResult {
        let passed = instance.get_variable(key).is_some();
        AssertionResult {
            description: format!("Process has variable '{}'", key),
            passed,
            message: if passed {
                format!("Process contains variable '{}'", key)
            } else {
                format!("Process missing variable '{}'", key)
            },
        }
    }

    /// Assert process variable value
    pub fn assert_variable_value(
        instance: &ProcessInstance,
        key: &str,
        expected: serde_json::Value,
    ) -> AssertionResult {
        let passed = instance.get_variable(key) == Some(&expected);
        AssertionResult {
            description: format!("Variable[{}] == {:?}", key, expected),
            passed,
            message: if passed {
                format!("Variable[{}] has expected value", key)
            } else {
                format!(
                    "Variable[{}] expected {:?}, got {:?}",
                    key,
                    expected,
                    instance.get_variable(key)
                )
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_assertion_result_creation() {
        let passed = AssertionResult::passed("Test", "Passed");
        assert!(passed.passed);

        let failed = AssertionResult::failed("Test", "Failed");
        assert!(!failed.passed);
    }

    #[test]
    fn test_workflow_assertions() {
        let metrics = ExecutionMetrics {
            total_duration_ms: 100,
            agent_tasks: 1,
            human_tasks: 1,
            system_tasks: 1,
            success_rate: 0.95,
        };

        let assertions = WorkflowAssertions::new()
            .assert_total_duration_less_than(&metrics, 200)
            .assert_success_rate(&metrics, 0.9)
            .assert_task_counts(&metrics, Some(1), Some(1), Some(1));

        assert!(assertions.all_passed());
        assert_eq!(assertions.passed_count(), 5);
    }

    #[test]
    fn test_task_assertions() {
        let mut output = HashMap::new();
        output.insert("result".to_string(), serde_json::json!("success"));

        let result = TaskResult {
            task_id: "t1".to_string(),
            success: true,
            error: None,
            output: serde_json::Value::Object(output.into_iter().collect()),
            duration_ms: 100,
        };

        let assertions = TaskAssertions::new()
            .assert_success(&result)
            .assert_output_contains_key(&result, "result")
            .assert_duration_less_than(&result, 200);

        assert!(assertions.all_passed());
        let (passed, failed) = assertions.stats();
        assert_eq!(passed, 3);
        assert_eq!(failed, 0);
    }
}
