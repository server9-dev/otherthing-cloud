# ABCDODAF Testing Framework

Comprehensive testing framework for BPMN + DoDAF 2.02 workflow validation and quality assurance.

## Overview

The testing framework provides a complete solution for testing ABCDODAF workflows with features including:

- **Unit Testing API** for individual task validation
- **Integration Testing** for complete workflow execution
- **Mock Task Handlers** for test isolation
- **Test Fixtures** and sample data generation
- **Assertion Framework** for outcome validation
- **Test Scenario DSL** for fluent test writing
- **Performance Benchmarking** tools
- **Code Coverage Tracking** for workflow paths
- **Test Report Generation** (HTML/JSON)
- **CI/CD Integration** support

## Architecture

```
┌─────────────────────────────────────────┐
│      Test Harness                       │
│  ┌─────────────────────────────────┐   │
│  │ Task Test Runner                │   │
│  │ Process Test Validator          │   │
│  │ Test Lifecycle Management       │   │
│  └─────────────────────────────────┘   │
└─────────────────────────────────────────┘
         │
         ├─────────────────────────────────┐
         │                                 │
    ┌────▼──────┐              ┌──────────▼────┐
    │  Mock      │              │  Assertions   │
    │  Handlers  │              │  Framework    │
    └────────────┘              └───────────────┘
         │                             │
         ├─────────────────────────────┤
         │                             │
    ┌────▼──────┐              ┌──────────▼────┐
    │  Fixtures  │              │  Test DSL     │
    │  Generator │              │               │
    └────────────┘              └───────────────┘
         │                             │
         └─────────────┬───────────────┘
                       │
              ┌────────▼────────┐
              │  Performance    │
              │  Benchmarking   │
              └────────┬────────┘
                       │
              ┌────────▼────────┐
              │  Coverage       │
              │  Tracking       │
              └────────┬────────┘
                       │
              ┌────────▼────────┐
              │  Report         │
              │  Generation     │
              └─────────────────┘
```

## Core Components

### 1. Test Harness (`harness.rs`)

The foundation for all test execution.

```rust
use abcdodaf::testing::*;

#[tokio::main]
async fn main() -> Result<()> {
    let executor = ProcessExecutor::new();
    let harness = TestHarness::new(executor);

    let test = TaskTestCase::new("t1", "My Test", "task_id")
        .with_input("user_id", json!("user_123"))
        .with_expected_output("status", json!("success"))
        .with_timeout(5000);

    let result = harness.run_task_test(test).await?;
    println!("Test passed: {}", result.passed);

    let summary = harness.get_summary().await;
    println!("Pass rate: {:.1}%", summary.pass_rate * 100.0);

    Ok(())
}
```

**Key Features:**
- Configurable timeout and parallel execution
- Automatic result collection and summary
- Fail-fast mode for rapid iteration
- Verbose logging support

**Configuration:**
```rust
let config = TestHarnessConfig {
    default_timeout_ms: 5000,
    fail_fast: true,
    verbose: true,
    capture_output: true,
    max_parallelism: 4,
};
let harness = TestHarness::with_config(executor, config);
```

### 2. Mock Handlers (`mock_handlers.rs`)

Simulate task behavior for isolated testing.

```rust
use abcdodaf::testing::*;

// Pass-through handler
let handler = MockTaskHandler::new();

// Fixed response
let handler = MockHandlerBuilder::new()
    .fixed_response_value("status", json!("success"))
    .build();

// Slow response simulation
let handler = MockHandlerBuilder::new()
    .slow_response(100, {
        let mut map = HashMap::new();
        map.insert("result".to_string(), json!("processed"));
        map
    })
    .build();

// Failure simulation
let handler = MockHandlerBuilder::new()
    .failure("Task failed: database error")
    .build();

// Recording handler
let handler = RecordingMockHandler::new(MockBehavior::PassThrough);
handler.execute("task1", &input).await?;
let history = handler.get_history().await;
assert!(handler.assert_called("task1").await);
```

**Behavior Modes:**
- **PassThrough**: Returns input as output
- **FixedResponse**: Always returns same output
- **SlowResponse**: Simulates delay
- **Failure**: Simulates error conditions
- **Conditional**: Routes based on input

### 3. Assertions Framework (`assertions.rs`)

Comprehensive outcome validation.

```rust
use abcdodaf::testing::*;

// Workflow assertions
let assertions = WorkflowAssertions::new()
    .assert_total_duration_less_than(&metrics, 5000)
    .assert_success_rate(&metrics, 0.95)
    .assert_task_counts(&metrics, Some(5), Some(2), Some(1));

assert!(assertions.all_passed());

// Task assertions
let task_assertions = TaskAssertions::new()
    .assert_success(&result)
    .assert_output_contains_key(&result, "user_id")
    .assert_output_value(&result, "status", json!("success"))
    .assert_duration_less_than(&result, 1000);

let (passed, failed) = task_assertions.stats();
println!("Assertions: {} passed, {} failed", passed, failed);

// State assertions
let completed = StateAssertions::assert_completed(&instance);
let not_failed = StateAssertions::assert_not_failed(&instance);
```

### 4. Test Fixtures (`fixtures.rs`)

Reusable test data and configuration.

```rust
use abcdodaf::testing::*;

// Create custom fixture
let fixture = FixtureBuilder::new("workflow_data", "Workflow Test Data")
    .with_variable("user_id", json!("user_001"))
    .with_variable("action", json!("process"))
    .with_tag("workflow")
    .build();

// Sample data generation
let workflow_vars = SampleDataGenerator::workflow_variables_fixture();
let user_input = SampleDataGenerator::user_input_fixture();
let batch_data = SampleDataGenerator::batch_data_fixture(10);

// Fixture pool
let pool = FixturePool::new()
    .add(fixture)
    .add(workflow_vars);

let fixture = pool.get("workflow_data");
let workflows = pool.get_by_tag("workflow");
```

**Sample Fixtures:**
- `workflow_variables_fixture()` - Standard workflow variables
- `user_input_fixture()` - User input data
- `task_output_fixture()` - Expected task output
- `error_scenario_fixture()` - Error handling scenarios
- `batch_data_fixture(n)` - Batch processing data

### 5. Test Scenario DSL (`dsl.rs`)

Fluent API for writing test scenarios.

```rust
use abcdodaf::testing::*;

// Simple sequential workflow
let scenario = ScenarioDsl::simple_workflow(
    "workflow_test",
    vec![
        ("analyze", "Analyze Input"),
        ("process", "Process Data"),
        ("store", "Store Results"),
    ],
);

// Workflow with retries
let scenario = ScenarioDsl::workflow_with_retries(
    "retry_test",
    "critical_task",
    3,  // max retries
);

// Parallel execution
let scenario = ScenarioDsl::parallel_workflow(
    "parallel_test",
    4,  // number of parallel tasks
);

// Error handling
let scenario = ScenarioDsl::error_handling_workflow("error_test");

// Custom scenario
let scenario = ScenarioBuilder::new("custom", "Custom Workflow")
    .with_description("Custom test scenario")
    .add_setup(
        StepBuilder::new("init", "Initialize")
            .initialize()
            .with_parameter("db", json!(true))
            .build()
    )
    .add_execution(
        StepBuilder::new("task1", "Main Task")
            .execute_task()
            .with_timeout(5000)
            .build()
    )
    .add_validation(
        ValidationBuilder::new("check_result", "output")
            .equals(json!("success"))
            .with_error_message("Output should be success")
            .build()
    )
    .build();
```

### 6. Performance Benchmarking (`performance.rs`)

Measure and analyze performance.

```rust
use abcdodaf::testing::*;

let suite = BenchmarkSuite::new("workflow_performance");

// Async benchmark
let result = suite
    .run_async_benchmark("workflow_exec", 100, || async {
        // Task execution simulation
        tokio::time::sleep(Duration::from_millis(5)).await;
    })
    .await;

println!("Average: {:.2}ms", result.metrics.average_duration());
println!("Min: {}ms", result.metrics.min_duration().unwrap_or(0));
println!("Max: {}ms", result.metrics.max_duration().unwrap_or(0));
println!("P95: {}ms", result.metrics.p95_duration().unwrap_or(0));
println!("P99: {}ms", result.metrics.p99_duration().unwrap_or(0));
println!("Throughput: {:.2} ops/sec", result.ops_per_second);

// Sync benchmark
let result = suite
    .run_sync_benchmark("cpu_task", 50, || {
        // CPU-intensive work
        let _ = (0..1000).sum::<i32>();
    })
    .await;

// Manual timing
let timer = PerfTimer::start("operation");
// ... do work ...
let elapsed = timer.stop();
println!("Operation took {}ms", elapsed);
```

### 7. Coverage Tracking (`coverage.rs`)

Monitor test coverage of workflow paths.

```rust
use abcdodaf::testing::*;

let tracker = CoverageTracker::new();

// Register workflow paths
tracker.register_path("validate_input", "Input Validation").await;
tracker.register_path("process_success", "Success Path").await;
tracker.register_path("handle_error", "Error Handler").await;

// Mark critical paths
tracker.mark_critical("validate_input").await;
tracker.mark_critical("process_success").await;

// Execute paths during testing
tracker.execute_path("validate_input").await;
tracker.execute_path("process_success").await;

// Generate coverage report
let report = tracker.generate_report().await;
println!("Coverage: {:.1}%", report.coverage_percent * 100.0);
println!("Covered: {}/{}", report.covered_paths, report.total_paths);

// Analyze coverage
let analysis = CoverageAnalyzer::analyze_coverage(&report);
println!("Confidence: {}", analysis.confidence);
for rec in analysis.recommendations {
    println!("Recommendation: {}", rec);
}

// Branch coverage
let branch_cov = BranchCoverage::new();
branch_cov.register_branch("if_success", "Success Branch").await;
branch_cov.register_branch("if_error", "Error Branch").await;
branch_cov.take_branch("if_success").await;
println!("Branch coverage: {:.1}%", branch_cov.coverage_percent().await * 100.0);
```

### 8. Report Generation (`reporting.rs`)

Generate comprehensive test reports.

```rust
use abcdodaf::testing::*;

let summary = TestSummary {
    total_tests: 100,
    passed_tests: 95,
    failed_tests: 5,
    total_duration_ms: 5000,
    pass_rate: 0.95,
};

let mut report = TestReport::new("my_test_suite", summary);

// Add test results
report = report.with_test_result(TestResultDetail {
    test_id: "t1".to_string(),
    test_name: "Workflow Init".to_string(),
    status: TestStatus::Passed,
    duration_ms: 250,
    error: None,
    assertions: vec![
        AssertionResult::passed("Created", "Workflow created"),
    ],
    tags: vec!["setup".to_string()],
});

// Add performance metrics
let perf = PerformanceSection {
    benchmarks: vec![],
    average_throughput: 50.0,
    notes: vec!["Good performance".to_string()],
};
report = report.with_performance(perf);

// Add coverage
let coverage = CoverageSection {
    path_coverage: PathCoverage {
        total_paths: 10,
        covered_paths: 9,
        coverage_percent: 0.9,
        paths: vec![],
        missed_critical_paths: vec![],
    },
    coverage_percent: 0.9,
    status: CoverageStatus::Good,
};
report = report.with_coverage(coverage);

// Generate reports
let html = ReportGenerator::generate_html(&report);
ReportGenerator::save_html_report(&report, "test_report.html")?;

let json = ReportGenerator::generate_json(&report)?;
ReportGenerator::save_json_report(&report, "test_report.json")?;
```

## Usage Patterns

### Pattern 1: Unit Testing Tasks

```rust
#[tokio::test]
async fn test_data_processing_task() {
    let executor = ProcessExecutor::new();
    let harness = TestHarness::new(executor);

    let test = TaskTestCase::new("unit_1", "Process Data", "process_task")
        .with_input("data", json!({"id": 1, "value": "test"}))
        .with_expected_output("processed", json!(true))
        .with_timeout(2000);

    let result = harness.run_task_test(test).await.unwrap();
    assert!(result.passed);
    assert!(result.duration_ms < 2000);
}
```

### Pattern 2: Integration Testing

```rust
#[tokio::test]
async fn test_complete_workflow() {
    let scenario = ScenarioDsl::simple_workflow(
        "integration_test",
        vec![
            ("init", "Initialize"),
            ("process", "Process"),
            ("verify", "Verify"),
        ],
    );

    // Execute workflow
    for step in scenario.execution_steps {
        println!("Executing: {}", step.name);
    }

    // Validate results
    for validation in scenario.validation_steps {
        println!("Validating: {}", validation.subject);
    }
}
```

### Pattern 3: Performance Testing

```rust
#[tokio::test]
async fn test_workflow_performance() {
    let suite = BenchmarkSuite::new("perf_test");

    let result = suite
        .run_async_benchmark("workflow", 100, || async {
            // Execute workflow
        })
        .await;

    assert!(result.ops_per_second > 100.0);
    assert!(result.metrics.p95_duration().unwrap_or(0) < 100);
}
```

### Pattern 4: Coverage-Driven Testing

```rust
#[tokio::test]
async fn test_with_coverage() {
    let tracker = CoverageTracker::new();

    tracker.register_path("path1", "Path 1").await;
    tracker.register_path("path2", "Path 2").await;
    tracker.mark_critical("path1").await;

    // Execute tests, tracking coverage
    tracker.execute_path("path1").await;
    tracker.execute_path("path2").await;

    let report = tracker.generate_report().await;
    assert_eq!(report.covered_paths, 2);
    assert!(report.missed_critical_paths.is_empty());
}
```

## Best Practices

### 1. Test Organization

```rust
// Group related tests
mod workflow_tests {
    use super::*;

    #[tokio::test]
    async fn test_workflow_initialization() { }

    #[tokio::test]
    async fn test_workflow_execution() { }

    #[tokio::test]
    async fn test_workflow_completion() { }
}
```

### 2. Mock Handler Strategy

- Use `PassThrough` for basic testing
- Use `FixedResponse` for predictable outcomes
- Use `SlowResponse` for timeout testing
- Use `Failure` for error scenarios
- Use `RecordingMockHandler` for call verification

### 3. Fixture Reuse

```rust
// Create fixture pool at test suite level
lazy_static::lazy_static! {
    static ref TEST_FIXTURES: FixturePool = {
        FixturePool::new()
            .add(SampleDataGenerator::workflow_variables_fixture())
            .add(SampleDataGenerator::user_input_fixture())
    };
}

// Reuse in tests
#[tokio::test]
async fn test_with_fixtures() {
    let workflow_vars = TEST_FIXTURES.get("workflow_vars").unwrap();
    // Use fixture...
}
```

### 4. Assertion Patterns

```rust
// Chain assertions
let assertions = WorkflowAssertions::new()
    .assert_total_duration_less_than(&metrics, 5000)
    .assert_success_rate(&metrics, 0.95)
    .assert_task_counts(&metrics, Some(3), None, None);

assert!(assertions.all_passed(),
    "Failed assertions: {:?}",
    assertions.get_assertions());
```

### 5. Coverage Requirements

Set minimum coverage thresholds in CI/CD:

```rust
let report = tracker.generate_report().await;
assert!(report.coverage_percent >= 0.80,
    "Coverage below 80%: {:.1}%",
    report.coverage_percent * 100.0);
assert!(report.missed_critical_paths.is_empty(),
    "Missed critical paths: {:?}",
    report.missed_critical_paths);
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Test Suite

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: dtolnay/rust-toolchain@stable

      - name: Run tests
        run: cargo test --all-features

      - name: Generate coverage
        run: cargo tarpaulin --out Html --output-dir coverage

      - name: Upload report
        uses: actions/upload-artifact@v2
        with:
          name: test-reports
          path: |
            test_report.html
            coverage/
```

## Performance Benchmarks

Expected performance metrics for common workflows:

- **Simple task execution**: <100ms
- **Multi-task workflow**: 100-500ms
- **Parallel task execution**: Variable (depends on task count)
- **Error handling path**: <200ms additional overhead

## Troubleshooting

### Test Timeout Issues

```rust
// Increase timeout for slow operations
let test = TaskTestCase::new("t1", "Slow Task", "task")
    .with_timeout(10000);  // 10 seconds
```

### Mock Handler Not Matching

```rust
// Debug handler calls
let handler = RecordingMockHandler::new(MockBehavior::PassThrough);
let history = handler.get_history().await;
for call in history {
    println!("Task: {}, Input: {:?}", call.task_id, call.input);
}
```

### Coverage Not Complete

```rust
// Identify missing paths
let report = tracker.generate_report().await;
for path in &report.paths {
    if path.execution_count == 0 {
        println!("Uncovered path: {}", path.name);
    }
}
```

## Examples

Run the testing framework example:

```bash
cargo run --example testing_framework_example
```

This demonstrates:
1. Unit testing with mock handlers
2. Test fixtures and sample data
3. Test scenario DSL
4. Performance benchmarking
5. Coverage tracking
6. Report generation

## API Reference

See inline documentation:

```bash
cargo doc --open
```

Navigate to `abcdodaf::testing` for complete API documentation.
