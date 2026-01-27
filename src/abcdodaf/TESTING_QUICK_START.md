# ABCDODAF Testing Framework - Quick Start Guide

## 5-Minute Setup

### 1. Run the Example

```bash
cargo run --example testing_framework_example
```

This demonstrates:
- Unit testing with mock handlers
- Test fixtures and sample data
- Test scenario DSL
- Performance benchmarking
- Coverage tracking
- Report generation

### 2. View Generated Reports

After running the example, check:
- HTML report: Can be saved with `ReportGenerator::save_html_report()`
- Coverage report: Shows path coverage statistics
- Performance metrics: Throughput and latency data

### 3. Run the Tests

```bash
cargo test testing_framework_tests
```

40+ tests covering all framework components.

---

## Common Patterns

### Pattern 1: Unit Test a Task

```rust
use abcdodaf::testing::*;

#[tokio::test]
async fn test_my_task() {
    let executor = ProcessExecutor::new();
    let harness = TestHarness::new(executor);

    let test = TaskTestCase::new("test1", "My Test", "task_id")
        .with_input("user_id", json!("user_123"))
        .with_expected_output("status", json!("success"))
        .with_timeout(5000);

    let result = harness.run_task_test(test).await.unwrap();
    assert!(result.passed);
}
```

### Pattern 2: Mock a Task Handler

```rust
use abcdodaf::testing::*;

let handler = MockHandlerBuilder::new()
    .fixed_response_value("status", json!("ok"))
    .build();

// Or for error simulation
let handler = MockHandlerBuilder::new()
    .failure("Task failed: database error")
    .build();

// Or record calls
let handler = RecordingMockHandler::new(MockBehavior::PassThrough);
handler.execute("task1", &input).await?;
assert!(handler.assert_called("task1").await);
```

### Pattern 3: Create Test Data

```rust
use abcdodaf::testing::*;

// Pre-built fixtures
let workflow_vars = SampleDataGenerator::workflow_variables_fixture();
let user_input = SampleDataGenerator::user_input_fixture();
let batch_data = SampleDataGenerator::batch_data_fixture(10);

// Custom fixture
let fixture = FixtureBuilder::new("my_data", "My Test Data")
    .with_variable("user_id", json!("user_001"))
    .with_variable("action", json!("process"))
    .with_tag("integration")
    .build();
```

### Pattern 4: Write Assertions

```rust
use abcdodaf::testing::*;

let assertions = WorkflowAssertions::new()
    .assert_total_duration_less_than(&metrics, 5000)
    .assert_success_rate(&metrics, 0.95)
    .assert_task_counts(&metrics, Some(3), Some(1), None);

assert!(assertions.all_passed());

// Task-level assertions
let task_asserts = TaskAssertions::new()
    .assert_success(&result)
    .assert_output_contains_key(&result, "user_id")
    .assert_duration_less_than(&result, 1000);
```

### Pattern 5: Define Test Scenarios

```rust
use abcdodaf::testing::*;

// Simple workflow
let scenario = ScenarioDsl::simple_workflow(
    "workflow_test",
    vec![
        ("analyze", "Analyze Input"),
        ("process", "Process Data"),
        ("store", "Store Results"),
    ],
);

// Retry workflow
let scenario = ScenarioDsl::workflow_with_retries(
    "retry_test",
    "critical_task",
    3,  // max retries
);

// Parallel workflow
let scenario = ScenarioDsl::parallel_workflow("parallel_test", 4);

// Error handling
let scenario = ScenarioDsl::error_handling_workflow("error_test");
```

### Pattern 6: Benchmark Performance

```rust
use abcdodaf::testing::*;

#[tokio::test]
async fn test_performance() {
    let suite = BenchmarkSuite::new("my_benchmarks");

    let result = suite
        .run_async_benchmark("workflow_exec", 100, || async {
            // Code to benchmark
            tokio::time::sleep(Duration::from_millis(5)).await;
        })
        .await;

    println!("Throughput: {:.2} ops/sec", result.ops_per_second);
    println!("P95: {}ms", result.metrics.p95_duration().unwrap_or(0));

    assert!(result.ops_per_second > 100.0);
}
```

### Pattern 7: Track Coverage

```rust
use abcdodaf::testing::*;

#[tokio::test]
async fn test_coverage() {
    let tracker = CoverageTracker::new();

    tracker.register_path("validate", "Input Validation").await;
    tracker.register_path("process", "Processing").await;
    tracker.register_path("error", "Error Handling").await;

    tracker.mark_critical("validate").await;
    tracker.mark_critical("process").await;

    // Execute tests...
    tracker.execute_path("validate").await;
    tracker.execute_path("process").await;

    let report = tracker.generate_report().await;
    println!("Coverage: {:.1}%", report.coverage_percent * 100.0);

    assert!(report.coverage_percent >= 0.8);
    assert!(report.missed_critical_paths.is_empty());
}
```

### Pattern 8: Generate Reports

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

// Add results, performance, coverage...
report = report.with_test_result(/* ... */);

// Generate reports
let html = ReportGenerator::generate_html(&report);
ReportGenerator::save_html_report(&report, "report.html")?;

let json = ReportGenerator::generate_json(&report)?;
ReportGenerator::save_json_report(&report, "report.json")?;
```

---

## Configuration

### Test Harness Configuration

```rust
let config = TestHarnessConfig {
    default_timeout_ms: 5000,      // Default timeout
    fail_fast: false,              // Stop on first failure
    verbose: true,                 // Detailed logging
    capture_output: true,          // Capture task output
    max_parallelism: 4,            // Parallel tests
};

let harness = TestHarness::with_config(executor, config);
```

---

## What's Available

### Core Components
- **Harness**: `TestHarness` - Test execution engine
- **Mocks**: `MockTaskHandler`, `RecordingMockHandler` - Task simulation
- **Assertions**: `WorkflowAssertions`, `TaskAssertions`, `StateAssertions`
- **Fixtures**: `TestFixture`, `FixtureBuilder`, `SampleDataGenerator`
- **DSL**: `ScenarioDsl`, `ScenarioBuilder` - Scenario definition
- **Performance**: `BenchmarkSuite`, `PerfTimer` - Benchmarking
- **Coverage**: `CoverageTracker`, `BranchCoverage` - Coverage tracking
- **Reporting**: `TestReport`, `ReportGenerator` - Report generation

### Import Everything

```rust
use abcdodaf::testing::*;
```

This imports all public types and traits.

---

## Module Organization

```
testing/
├── harness.rs       - Test execution
├── mock_handlers.rs - Mock simulation
├── assertions.rs    - Outcome validation
├── fixtures.rs      - Test data
├── dsl.rs          - Scenario definition
├── performance.rs  - Benchmarking
├── coverage.rs     - Coverage tracking
└── reporting.rs    - Report generation
```

---

## Key Features at a Glance

| Feature | Use Case |
|---------|----------|
| `TaskTestCase` | Test individual task logic |
| `MockTaskHandler` | Isolate tasks for testing |
| `WorkflowAssertions` | Validate workflow metrics |
| `TestFixture` | Reuse test data |
| `ScenarioDsl` | Define complex test scenarios |
| `BenchmarkSuite` | Measure performance |
| `CoverageTracker` | Monitor test completeness |
| `TestReport` | Generate test reports |

---

## Next Steps

1. **Run example**: `cargo run --example testing_framework_example`
2. **Read guide**: Open `TESTING_FRAMEWORK.md`
3. **Review tests**: Check `tests/testing_framework_tests.rs`
4. **Write tests**: Use patterns above in your code
5. **Generate reports**: Create automated test reports

---

## Documentation

- **Complete Guide**: `TESTING_FRAMEWORK.md` (676 lines)
- **Task Summary**: `TESTING_TASK_7_SUMMARY.md` (519 lines)
- **API Docs**: `cargo doc --open` → `abcdodaf::testing`
- **Examples**: `examples/testing_framework_example.rs`
- **Tests**: `tests/testing_framework_tests.rs` (40+ examples)

---

## Support

### Issues?
1. Check `TESTING_FRAMEWORK.md` troubleshooting section
2. Review example code in `examples/testing_framework_example.rs`
3. Look at test patterns in `tests/testing_framework_tests.rs`
4. Check inline API documentation

### Want to Contribute?
The framework is ready for extension:
- Add custom assertion types
- Create domain-specific DSL extensions
- Add integration with external systems
- Create custom report generators

---

## Quick Reference

### Creating a Test
```rust
TaskTestCase::new(id, name, task_id)
    .with_input(key, value)
    .with_expected_output(key, value)
    .with_timeout(ms)
    .with_tag(tag)
```

### Creating a Mock
```rust
MockHandlerBuilder::new()
    .fixed_response_value(key, value)  // or .slow_response() or .failure()
    .build()
```

### Creating Fixtures
```rust
FixtureBuilder::new(id, name)
    .with_variable(key, value)
    .with_tag(tag)
    .build()
```

### Defining Scenarios
```rust
ScenarioDsl::simple_workflow(id, tasks)
// or
ScenarioDsl::workflow_with_retries(id, task, max)
// or
ScenarioDsl::parallel_workflow(id, count)
```

### Benchmarking
```rust
suite.run_async_benchmark(name, iterations, || async {
    // code to benchmark
}).await
```

### Tracking Coverage
```rust
tracker.register_path(id, name).await;
tracker.execute_path(id).await;
let report = tracker.generate_report().await;
```

---

**Happy Testing!** 🚀

For more information, see `TESTING_FRAMEWORK.md`.
