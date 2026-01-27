//! Test data generation and fixtures

use crate::bpmn::{Process, ProcessBuilder};
use crate::dodaf::OperationalContext;
use crate::workforce::{
    AgentCapability, AgentTask, HumanRole, HumanTask, SystemOperation, SystemTask,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Test fixture for reusable test data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFixture {
    /// Fixture ID
    pub id: String,
    /// Name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Initial variables
    pub initial_variables: HashMap<String, serde_json::Value>,
    /// Tags for categorization
    pub tags: Vec<String>,
}

impl TestFixture {
    /// Create a new test fixture
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            initial_variables: HashMap::new(),
            tags: Vec::new(),
        }
    }

    /// Add initial variable
    pub fn with_variable(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.initial_variables.insert(key.into(), value);
        self
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Get variables as reference
    pub fn variables(&self) -> &HashMap<String, serde_json::Value> {
        &self.initial_variables
    }
}

/// Builder for test fixtures
pub struct FixtureBuilder {
    fixture: TestFixture,
}

impl FixtureBuilder {
    /// Create new builder
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self { fixture: TestFixture::new(id, name) }
    }

    /// Add variable
    pub fn with_variable(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.fixture = self.fixture.with_variable(key, value);
        self
    }

    /// Add multiple variables
    pub fn with_variables(mut self, vars: HashMap<String, serde_json::Value>) -> Self {
        for (key, value) in vars {
            self.fixture.initial_variables.insert(key, value);
        }
        self
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.fixture = self.fixture.with_description(desc);
        self
    }

    /// Add tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.fixture = self.fixture.with_tag(tag);
        self
    }

    /// Build fixture
    pub fn build(self) -> TestFixture {
        self.fixture
    }
}

/// Sample data generator for test data
pub struct SampleDataGenerator;

impl SampleDataGenerator {
    /// Generate sample agent task
    pub fn sample_agent_task() -> AgentTask {
        AgentTask::new(
            &format!("agent_{}", Uuid::new_v4()),
            "Sample Agent Task",
            AgentCapability::NaturalLanguageProcessing,
        )
        .with_autonomy(0.8)
    }

    /// Generate sample human task
    pub fn sample_human_task() -> HumanTask {
        HumanTask::new(
            &format!("human_{}", Uuid::new_v4()),
            "Sample Human Task",
            HumanRole::QualityAssurance,
        )
        .with_complexity(2)
    }

    /// Generate sample system task
    pub fn sample_system_task() -> SystemTask {
        SystemTask::new(
            &format!("system_{}", Uuid::new_v4()),
            "Sample System Task",
            SystemOperation::DatabaseWrite,
        )
        .with_timeout(5)
    }

    /// Generate sample JSON data
    pub fn sample_json_object() -> serde_json::Value {
        serde_json::json!({
            "id": Uuid::new_v4().to_string(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "status": "active",
            "data": {
                "value1": "test",
                "value2": 42
            }
        })
    }

    /// Generate sample JSON array
    pub fn sample_json_array(count: usize) -> serde_json::Value {
        let items: Vec<_> = (0..count)
            .map(|i| serde_json::json!({ "id": i, "value": format!("item_{}", i) }))
            .collect();
        serde_json::Value::Array(items)
    }

    /// Generate random string
    pub fn random_string(length: usize) -> String {
        use uuid::Uuid;
        let uuid = Uuid::new_v4().to_string();
        uuid.chars().take(length).collect()
    }

    /// Generate sample operational context
    pub fn sample_operational_context() -> OperationalContext {
        OperationalContext::new()
            .with_mission_area("Test Mission")
            .with_capability("Capability 1")
            .with_capability("Capability 2")
            .with_resource("budget", serde_json::json!(1000))
            .with_constraint("max_time_ms", serde_json::json!(300000))
    }

    /// Generate sample process with multiple tasks
    pub fn sample_process(num_tasks: usize) -> Result<Process, crate::error::AbcdodafError> {
        let mut builder = ProcessBuilder::new("sample", "Sample Process");

        for i in 0..num_tasks {
            if i % 2 == 0 {
                builder =
                    builder.add_user_task(&format!("task_{}", i), &format!("User Task {}", i));
            } else {
                builder = builder
                    .add_service_task(&format!("task_{}", i), &format!("Service Task {}", i));
            }
        }

        // Add flows between tasks
        if num_tasks > 1 {
            for i in 0..(num_tasks - 1) {
                let from = format!("task_{}", i);
                let to = format!("task_{}", i + 1);
                builder = builder.add_flow(&format!("flow_{}", i), &from, &to);
            }
        }

        builder.build()
    }

    /// Generate workflow variables fixture
    pub fn workflow_variables_fixture() -> TestFixture {
        FixtureBuilder::new("workflow_vars", "Workflow Variables")
            .with_description("Standard workflow variables for testing")
            .with_variable("workflow_id", serde_json::json!(Uuid::new_v4().to_string()))
            .with_variable("start_time", serde_json::json!(chrono::Utc::now().to_rfc3339()))
            .with_variable("context", serde_json::json!({ "priority": "high" }))
            .with_variable("timeout", serde_json::json!(30000))
            .with_tag("variables")
            .with_tag("workflow")
            .build()
    }

    /// Generate user input fixture
    pub fn user_input_fixture() -> TestFixture {
        FixtureBuilder::new("user_input", "User Input")
            .with_description("Sample user input for testing")
            .with_variable("user_id", serde_json::json!(Uuid::new_v4().to_string()))
            .with_variable("input_text", serde_json::json!("Test input from user"))
            .with_variable(
                "metadata",
                serde_json::json!({
                    "source": "web",
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }),
            )
            .with_tag("input")
            .with_tag("user")
            .build()
    }

    /// Generate task output fixture
    pub fn task_output_fixture() -> TestFixture {
        FixtureBuilder::new("task_output", "Task Output")
            .with_description("Sample task output for validation")
            .with_variable("status", serde_json::json!("success"))
            .with_variable("result", serde_json::json!("Task completed"))
            .with_variable("duration_ms", serde_json::json!(1234))
            .with_variable(
                "output_data",
                serde_json::json!({
                    "processed": true,
                    "items_count": 42
                }),
            )
            .with_tag("output")
            .with_tag("result")
            .build()
    }

    /// Generate error scenario fixture
    pub fn error_scenario_fixture() -> TestFixture {
        FixtureBuilder::new("error_scenario", "Error Scenario")
            .with_description("Variables for testing error handling")
            .with_variable("error_code", serde_json::json!("TASK_FAILED"))
            .with_variable("error_message", serde_json::json!("Task execution failed"))
            .with_variable("retry_count", serde_json::json!(0))
            .with_variable("should_retry", serde_json::json!(true))
            .with_tag("error")
            .with_tag("failure")
            .build()
    }

    /// Generate batch data fixture
    pub fn batch_data_fixture(batch_size: usize) -> TestFixture {
        let mut batch = Vec::new();
        for i in 0..batch_size {
            batch.push(serde_json::json!({
                "id": i,
                "value": format!("item_{}", i),
                "timestamp": chrono::Utc::now().to_rfc3339()
            }));
        }

        FixtureBuilder::new("batch_data", "Batch Data")
            .with_description(&format!("Batch of {} items for testing", batch_size))
            .with_variable("batch_id", serde_json::json!(Uuid::new_v4().to_string()))
            .with_variable("items", serde_json::Value::Array(batch))
            .with_variable("count", serde_json::json!(batch_size))
            .with_tag("batch")
            .with_tag("data")
            .build()
    }
}

/// Fixture pool for managing multiple fixtures
pub struct FixturePool {
    fixtures: HashMap<String, TestFixture>,
}

impl FixturePool {
    /// Create new fixture pool
    pub fn new() -> Self {
        Self { fixtures: HashMap::new() }
    }

    /// Add fixture to pool
    pub fn add(mut self, fixture: TestFixture) -> Self {
        self.fixtures.insert(fixture.id.clone(), fixture);
        self
    }

    /// Get fixture by ID
    pub fn get(&self, id: &str) -> Option<&TestFixture> {
        self.fixtures.get(id)
    }

    /// Get fixtures by tag
    pub fn get_by_tag(&self, tag: &str) -> Vec<&TestFixture> {
        self.fixtures.values().filter(|f| f.tags.contains(&tag.to_string())).collect()
    }

    /// List all fixture IDs
    pub fn list_ids(&self) -> Vec<&str> {
        self.fixtures.keys().map(|k| k.as_str()).collect()
    }

    /// Get fixture count
    pub fn count(&self) -> usize {
        self.fixtures.len()
    }
}

impl Default for FixturePool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_creation() {
        let fixture = TestFixture::new("f1", "Fixture 1")
            .with_variable("key", serde_json::json!("value"))
            .with_description("Test fixture")
            .with_tag("test");

        assert_eq!(fixture.id, "f1");
        assert_eq!(fixture.name, "Fixture 1");
        assert_eq!(fixture.variables().len(), 1);
        assert!(fixture.tags.contains(&"test".to_string()));
    }

    #[test]
    fn test_fixture_builder() {
        let fixture = FixtureBuilder::new("f1", "Fixture")
            .with_variable("v1", serde_json::json!(1))
            .with_variable("v2", serde_json::json!(2))
            .with_tag("unit")
            .build();

        assert_eq!(fixture.variables().len(), 2);
    }

    #[test]
    fn test_sample_data_generation() {
        let agent = SampleDataGenerator::sample_agent_task();
        assert!(!agent.id.is_empty());

        let json = SampleDataGenerator::sample_json_object();
        assert!(json.is_object());

        let array = SampleDataGenerator::sample_json_array(5);
        assert!(array.is_array());
    }

    #[test]
    fn test_fixture_pool() {
        let pool = FixturePool::new()
            .add(TestFixture::new("f1", "Fixture 1").with_tag("type1"))
            .add(TestFixture::new("f2", "Fixture 2").with_tag("type2"))
            .add(TestFixture::new("f3", "Fixture 3").with_tag("type1"));

        assert_eq!(pool.count(), 3);
        assert_eq!(pool.get_by_tag("type1").len(), 2);
        assert!(pool.get("f1").is_some());
    }

    #[test]
    fn test_sample_fixtures() {
        let workflow_vars = SampleDataGenerator::workflow_variables_fixture();
        assert!(workflow_vars.tags.contains(&"workflow".to_string()));

        let user_input = SampleDataGenerator::user_input_fixture();
        assert!(user_input.tags.contains(&"input".to_string()));

        let batch = SampleDataGenerator::batch_data_fixture(10);
        assert!(batch.tags.contains(&"batch".to_string()));
    }
}
