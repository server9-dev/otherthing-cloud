//! Domain-Specific Language for test scenarios

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A test scenario definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestScenario {
    /// Scenario ID
    pub id: String,
    /// Scenario name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Setup steps
    pub setup_steps: Vec<Step>,
    /// Execution steps
    pub execution_steps: Vec<Step>,
    /// Validation steps
    pub validation_steps: Vec<ValidationStep>,
    /// Cleanup steps
    pub cleanup_steps: Vec<Step>,
    /// Scenario variables
    pub variables: HashMap<String, serde_json::Value>,
    /// Tags
    pub tags: Vec<String>,
}

/// A single step in a scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    /// Step ID
    pub id: String,
    /// Step name
    pub name: String,
    /// Step type
    pub step_type: StepType,
    /// Parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Optional condition to execute step
    pub condition: Option<String>,
    /// Timeout in milliseconds
    pub timeout_ms: Option<u64>,
}

/// Types of steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepType {
    /// Execute a task
    ExecuteTask,
    /// Verify output
    VerifyOutput,
    /// Wait for condition
    Wait,
    /// Initialize data
    Initialize,
    /// Check state
    CheckState,
    /// Custom step
    Custom(String),
}

/// Validation step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationStep {
    /// Validation ID
    pub id: String,
    /// What is being validated
    pub subject: String,
    /// Validation type
    pub validation_type: ValidationType,
    /// Expected value
    pub expected: serde_json::Value,
    /// Error message if fails
    pub error_message: Option<String>,
}

/// Types of validations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationType {
    /// Value equals
    Equals,
    /// Value contains
    Contains,
    /// Value matches pattern
    Matches,
    /// Value is in range
    InRange,
    /// Custom validation
    Custom(String),
}

impl TestScenario {
    /// Create new scenario
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            setup_steps: Vec::new(),
            execution_steps: Vec::new(),
            validation_steps: Vec::new(),
            cleanup_steps: Vec::new(),
            variables: HashMap::new(),
            tags: Vec::new(),
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add setup step
    pub fn add_setup_step(mut self, step: Step) -> Self {
        self.setup_steps.push(step);
        self
    }

    /// Add execution step
    pub fn add_execution_step(mut self, step: Step) -> Self {
        self.execution_steps.push(step);
        self
    }

    /// Add validation step
    pub fn add_validation(mut self, validation: ValidationStep) -> Self {
        self.validation_steps.push(validation);
        self
    }

    /// Add cleanup step
    pub fn add_cleanup_step(mut self, step: Step) -> Self {
        self.cleanup_steps.push(step);
        self
    }

    /// Add scenario variable
    pub fn with_variable(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.variables.insert(key.into(), value);
        self
    }

    /// Add tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}

/// Builder for test scenarios
pub struct ScenarioBuilder {
    scenario: TestScenario,
}

impl ScenarioBuilder {
    /// Create new builder
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self { scenario: TestScenario::new(id, name) }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.scenario = self.scenario.with_description(desc);
        self
    }

    /// Add setup step
    pub fn add_setup(mut self, step: Step) -> Self {
        self.scenario = self.scenario.add_setup_step(step);
        self
    }

    /// Add multiple setup steps
    pub fn add_setups(mut self, steps: Vec<Step>) -> Self {
        for step in steps {
            self.scenario = self.scenario.add_setup_step(step);
        }
        self
    }

    /// Add execution step
    pub fn add_execution(mut self, step: Step) -> Self {
        self.scenario = self.scenario.add_execution_step(step);
        self
    }

    /// Add multiple execution steps
    pub fn add_executions(mut self, steps: Vec<Step>) -> Self {
        for step in steps {
            self.scenario = self.scenario.add_execution_step(step);
        }
        self
    }

    /// Add validation
    pub fn add_validation(mut self, validation: ValidationStep) -> Self {
        self.scenario = self.scenario.add_validation(validation);
        self
    }

    /// Add multiple validations
    pub fn add_validations(mut self, validations: Vec<ValidationStep>) -> Self {
        for validation in validations {
            self.scenario = self.scenario.add_validation(validation);
        }
        self
    }

    /// Add cleanup step
    pub fn add_cleanup(mut self, step: Step) -> Self {
        self.scenario = self.scenario.add_cleanup_step(step);
        self
    }

    /// Add variable
    pub fn with_variable(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.scenario = self.scenario.with_variable(key, value);
        self
    }

    /// Add tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.scenario = self.scenario.with_tag(tag);
        self
    }

    /// Build scenario
    pub fn build(self) -> TestScenario {
        self.scenario
    }
}

/// High-level DSL for writing test scenarios fluently
pub struct ScenarioDsl;

impl ScenarioDsl {
    /// Create new scenario from scratch
    pub fn scenario(id: impl Into<String>, name: impl Into<String>) -> ScenarioBuilder {
        ScenarioBuilder::new(id, name)
    }

    /// Create step builder
    pub fn step(id: impl Into<String>, name: impl Into<String>) -> StepBuilder {
        StepBuilder::new(id, name)
    }

    /// Create validation builder
    pub fn validation(id: impl Into<String>, subject: impl Into<String>) -> ValidationBuilder {
        ValidationBuilder::new(id, subject)
    }

    /// Build a simple sequential workflow test
    pub fn simple_workflow(
        scenario_id: impl Into<String>,
        tasks: Vec<(&str, &str)>,
    ) -> TestScenario {
        let mut scenario = TestScenario::new(scenario_id, "Simple Workflow Test");

        for (task_id, task_name) in tasks {
            let step = StepBuilder::new(task_id, task_name)
                .execute_task()
                .with_parameter("task_id", serde_json::json!(task_id))
                .build();

            scenario = scenario.add_execution_step(step);
        }

        scenario
    }

    /// Build a workflow with retries
    pub fn workflow_with_retries(
        scenario_id: impl Into<String>,
        task_id: impl Into<String>,
        max_retries: usize,
    ) -> TestScenario {
        let task_id = task_id.into();
        let mut scenario = TestScenario::new(scenario_id, "Workflow with Retries");

        for attempt in 1..=max_retries {
            let step =
                StepBuilder::new(&format!("retry_{}", attempt), &format!("Attempt {}", attempt))
                    .execute_task()
                    .with_parameter("task_id", serde_json::json!(&task_id))
                    .with_parameter("attempt", serde_json::json!(attempt))
                    .build();

            scenario = scenario.add_execution_step(step);
        }

        scenario
    }

    /// Build error handling workflow
    pub fn error_handling_workflow(scenario_id: impl Into<String>) -> TestScenario {
        ScenarioBuilder::new(scenario_id, "Error Handling Workflow")
            .add_execution(StepBuilder::new("task1", "Main Task").execute_task().build())
            .add_validation(
                ValidationBuilder::new("check_error", "error_status")
                    .equals(serde_json::json!("handled"))
                    .build(),
            )
            .build()
    }

    /// Build parallel execution workflow
    pub fn parallel_workflow(scenario_id: impl Into<String>, task_count: usize) -> TestScenario {
        let mut scenario = TestScenario::new(scenario_id, "Parallel Workflow");

        for i in 0..task_count {
            let step =
                StepBuilder::new(&format!("parallel_task_{}", i), &format!("Parallel Task {}", i))
                    .execute_task()
                    .with_parameter("parallel", serde_json::json!(true))
                    .build();

            scenario = scenario.add_execution_step(step);
        }

        scenario
    }
}

/// Builder for steps
pub struct StepBuilder {
    step: Step,
}

impl StepBuilder {
    /// Create new step builder
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            step: Step {
                id: id.into(),
                name: name.into(),
                step_type: StepType::Custom("unknown".to_string()),
                parameters: HashMap::new(),
                condition: None,
                timeout_ms: None,
            },
        }
    }

    /// Set as task execution step
    pub fn execute_task(mut self) -> Self {
        self.step.step_type = StepType::ExecuteTask;
        self
    }

    /// Set as verification step
    pub fn verify_output(mut self) -> Self {
        self.step.step_type = StepType::VerifyOutput;
        self
    }

    /// Set as wait step
    pub fn wait(mut self) -> Self {
        self.step.step_type = StepType::Wait;
        self
    }

    /// Set as initialization step
    pub fn initialize(mut self) -> Self {
        self.step.step_type = StepType::Initialize;
        self
    }

    /// Add parameter
    pub fn with_parameter(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.step.parameters.insert(key.into(), value);
        self
    }

    /// Set condition
    pub fn with_condition(mut self, condition: impl Into<String>) -> Self {
        self.step.condition = Some(condition.into());
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, ms: u64) -> Self {
        self.step.timeout_ms = Some(ms);
        self
    }

    /// Build step
    pub fn build(self) -> Step {
        self.step
    }
}

/// Builder for validations
pub struct ValidationBuilder {
    validation: ValidationStep,
}

impl ValidationBuilder {
    /// Create new validation builder
    pub fn new(id: impl Into<String>, subject: impl Into<String>) -> Self {
        Self {
            validation: ValidationStep {
                id: id.into(),
                subject: subject.into(),
                validation_type: ValidationType::Equals,
                expected: serde_json::json!(null),
                error_message: None,
            },
        }
    }

    /// Set equality validation
    pub fn equals(mut self, expected: serde_json::Value) -> Self {
        self.validation.validation_type = ValidationType::Equals;
        self.validation.expected = expected;
        self
    }

    /// Set contains validation
    pub fn contains(mut self, expected: serde_json::Value) -> Self {
        self.validation.validation_type = ValidationType::Contains;
        self.validation.expected = expected;
        self
    }

    /// Set pattern matching validation
    pub fn matches(mut self, pattern: impl Into<String>) -> Self {
        self.validation.validation_type = ValidationType::Matches;
        self.validation.expected = serde_json::json!(pattern.into());
        self
    }

    /// Set range validation
    pub fn in_range(mut self, min: i64, max: i64) -> Self {
        self.validation.validation_type = ValidationType::InRange;
        self.validation.expected = serde_json::json!({ "min": min, "max": max });
        self
    }

    /// Set error message
    pub fn with_error_message(mut self, msg: impl Into<String>) -> Self {
        self.validation.error_message = Some(msg.into());
        self
    }

    /// Build validation
    pub fn build(self) -> ValidationStep {
        self.validation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_builder() {
        let scenario = ScenarioBuilder::new("s1", "Test Scenario")
            .with_description("A test scenario")
            .with_variable("var1", serde_json::json!("value1"))
            .with_tag("unit")
            .build();

        assert_eq!(scenario.id, "s1");
        assert_eq!(scenario.name, "Test Scenario");
        assert_eq!(scenario.variables.len(), 1);
    }

    #[test]
    fn test_step_builder() {
        let step = StepBuilder::new("s1", "Step 1")
            .execute_task()
            .with_parameter("task_id", serde_json::json!("t1"))
            .with_timeout(5000)
            .build();

        assert_eq!(step.id, "s1");
        assert!(step.timeout_ms.is_some());
    }

    #[test]
    fn test_validation_builder() {
        let validation = ValidationBuilder::new("v1", "result")
            .equals(serde_json::json!("success"))
            .with_error_message("Result should be success")
            .build();

        assert_eq!(validation.id, "v1");
        assert!(validation.error_message.is_some());
    }

    #[test]
    fn test_scenario_dsl() {
        let scenario =
            ScenarioDsl::simple_workflow("workflow", vec![("t1", "Task 1"), ("t2", "Task 2")]);

        assert_eq!(scenario.execution_steps.len(), 2);
    }

    #[test]
    fn test_retry_workflow_dsl() {
        let scenario = ScenarioDsl::workflow_with_retries("retry_test", "task1", 3);
        assert_eq!(scenario.execution_steps.len(), 3);
    }

    #[test]
    fn test_parallel_workflow_dsl() {
        let scenario = ScenarioDsl::parallel_workflow("parallel", 4);
        assert_eq!(scenario.execution_steps.len(), 4);
    }
}
