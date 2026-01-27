//! Example demonstrating the comprehensive testing framework for ABCDODAF workflows
//!
//! This example shows:
//! 1. Unit testing individual tasks
//! 2. Integration testing complete workflows
//! 3. Performance benchmarking
//! 4. Test scenario DSL usage
//! 5. Coverage tracking
//! 6. Report generation

use abcdodaf::prelude::*;
use abcdodaf::testing::*;
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== ABCDODAF Testing Framework Demonstration ===\n");

    // Example 1: Unit Testing with Mock Handlers
    unit_testing_example().await?;

    // Example 2: Test Fixtures and Sample Data
    fixtures_example()?;

    // Example 3: Test Scenario DSL
    dsl_example()?;

    // Example 4: Performance Benchmarking
    performance_example().await?;

    // Example 5: Coverage Tracking
    coverage_example().await?;

    // Example 6: Complete Test Report Generation
    report_generation_example().await?;

    println!("\n=== All Examples Completed Successfully ===\n");

    Ok(())
}

/// Demonstrates unit testing with mock handlers
async fn unit_testing_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("1. UNIT TESTING WITH MOCK HANDLERS\n");

    let executor = ProcessExecutor::new();

    // Create mock handlers with different behaviors
    let handler_success = MockHandlerBuilder::new()
        .fixed_response_value("status", serde_json::json!("success"))
        .build();

    let handler_with_delay = MockHandlerBuilder::new()
        .slow_response(
            100,
            {
                let mut map = HashMap::new();
                map.insert("result".to_string(), serde_json::json!("processed"));
                map
            },
        )
        .build();

    // Register handlers
    let _executor = executor
        .register_handler("success_task", Arc::new(handler_success))
        .register_handler("slow_task", Arc::new(handler_with_delay));

    // Create test harness
    let harness = TestHarness::new(_executor);

    // Create test cases
    let test_case1 = TaskTestCase::new("unit_test_1", "Basic Task Execution", "task_1")
        .with_description("Tests basic task execution with success")
        .with_input("user_id", serde_json::json!("user_123"))
        .with_expected_output("status", serde_json::json!("success"))
        .with_timeout(5000)
        .with_tag("unit")
        .with_tag("basic");

    let test_case2 = TaskTestCase::new("unit_test_2", "Task with Input Processing", "task_2")
        .with_description("Tests task that processes input data")
        .with_input("data", serde_json::json!({"value": 42}))
        .with_expected_output("processed", serde_json::json!(true))
        .with_timeout(3000)
        .with_tag("unit")
        .with_tag("processing");

    // Run tests
    let results = harness.run_task_tests(vec![test_case1, test_case2]).await?;

    // Display results
    for result in &results {
        println!("  Test: {}", result.test_case.name);
        println!("    Status: {}", if result.passed { "PASS" } else { "FAIL" });
        println!("    Duration: {}ms", result.duration_ms);
    }

    // Get summary
    let summary = harness.get_summary().await;
    println!("\n  Summary: {} passed, {} failed out of {}",
        summary.passed_tests,
        summary.failed_tests,
        summary.total_tests
    );

    println!();
    Ok(())
}

/// Demonstrates test fixtures and sample data generation
fn fixtures_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("2. TEST FIXTURES AND SAMPLE DATA\n");

    // Create custom fixture
    let custom_fixture = FixtureBuilder::new("workflow_test_data", "Workflow Test Data")
        .with_description("Data for workflow testing")
        .with_variable("user_id", serde_json::json!("user_001"))
        .with_variable("action", serde_json::json!("process"))
        .with_variable("timestamp", serde_json::json!(chrono::Utc::now().to_rfc3339()))
        .with_tag("workflow")
        .with_tag("data")
        .build();

    println!("  Custom Fixture: {}", custom_fixture.name);
    println!("    Variables: {}", custom_fixture.variables().len());

    // Use sample data generator
    let workflow_vars = SampleDataGenerator::workflow_variables_fixture();
    println!("\n  Sample Workflow Variables Fixture: {}", workflow_vars.name);
    println!("    Variables: {}", workflow_vars.variables().len());

    let user_input = SampleDataGenerator::user_input_fixture();
    println!("\n  Sample User Input Fixture: {}", user_input.name);
    for (key, _) in user_input.variables() {
        println!("    - {}", key);
    }

    let batch_data = SampleDataGenerator::batch_data_fixture(5);
    println!("\n  Sample Batch Data (5 items): {}", batch_data.name);

    // Create fixture pool
    let pool = FixturePool::new()
        .add(custom_fixture)
        .add(workflow_vars)
        .add(user_input)
        .add(batch_data);

    println!("\n  Fixture Pool:");
    println!("    Total fixtures: {}", pool.count());
    println!("    Fixture IDs: {:?}", pool.list_ids());

    println!();
    Ok(())
}

/// Demonstrates test scenario DSL
fn dsl_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("3. TEST SCENARIO DSL\n");

    // Example 1: Simple workflow scenario
    let simple = ScenarioDsl::simple_workflow(
        "simple_workflow_test",
        vec![
            ("task_analyze", "Analyze Input"),
            ("task_process", "Process Data"),
            ("task_store", "Store Results"),
        ],
    );

    println!("  Simple Workflow Scenario: {}", simple.name);
    println!("    Execution steps: {}", simple.execution_steps.len());

    // Example 2: Workflow with retries
    let retry_workflow = ScenarioDsl::workflow_with_retries(
        "retry_workflow_test",
        "critical_task",
        3,
    );

    println!("\n  Retry Workflow Scenario: {}", retry_workflow.name);
    println!("    Retry attempts: {}", retry_workflow.execution_steps.len());

    // Example 3: Parallel execution
    let parallel = ScenarioDsl::parallel_workflow(
        "parallel_workflow_test",
        4,
    );

    println!("\n  Parallel Workflow Scenario: {}", parallel.name);
    println!("    Parallel tasks: {}", parallel.execution_steps.len());

    // Example 4: Error handling workflow
    let error_handling = ScenarioDsl::error_handling_workflow("error_workflow_test");

    println!("\n  Error Handling Workflow: {}", error_handling.name);
    println!("    Validations: {}", error_handling.validation_steps.len());

    println!();
    Ok(())
}

/// Demonstrates performance benchmarking
async fn performance_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("4. PERFORMANCE BENCHMARKING\n");

    let suite = BenchmarkSuite::new("workflow_performance");

    // Run async benchmark
    let result = suite
        .run_async_benchmark("workflow_execution", 10, || async {
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        })
        .await;

    println!("  Benchmark: {}", result.name);
    println!("    Average: {:.2}ms", result.metrics.average_duration());
    println!("    Min: {}ms", result.metrics.min_duration().unwrap_or(0));
    println!("    Max: {}ms", result.metrics.max_duration().unwrap_or(0));
    println!("    P95: {}ms", result.metrics.p95_duration().unwrap_or(0));
    println!("    P99: {}ms", result.metrics.p99_duration().unwrap_or(0));
    println!("    Throughput: {:.2} ops/sec", result.ops_per_second);

    // Performance timer example
    let timer = PerfTimer::start("heavy_operation");
    std::thread::sleep(std::time::Duration::from_millis(20));
    let elapsed = timer.stop();

    println!("\n  Heavy Operation Timing: {}ms", elapsed);

    println!();
    Ok(())
}

/// Demonstrates coverage tracking
async fn coverage_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("5. COVERAGE TRACKING\n");

    let tracker = CoverageTracker::new();

    // Register workflow paths
    tracker.register_path("validate_input", "Input Validation").await;
    tracker.register_path("process_success", "Success Path").await;
    tracker.register_path("process_error", "Error Handling").await;
    tracker.register_path("cleanup", "Cleanup Operations").await;

    // Mark critical paths
    tracker.mark_critical("validate_input").await;
    tracker.mark_critical("process_success").await;

    // Simulate execution
    tracker.execute_path("validate_input").await;
    tracker.execute_path("process_success").await;

    // Generate report
    let report = tracker.generate_report().await;

    println!("  Coverage Report:");
    println!("    Total paths: {}", report.total_paths);
    println!("    Covered paths: {}", report.covered_paths);
    println!("    Coverage: {:.1}%", report.coverage_percent * 100.0);
    println!("    Missed critical paths: {}", report.missed_critical_paths.len());

    // Analyze coverage
    let analysis = CoverageAnalyzer::analyze_coverage(&report);
    println!("\n  Coverage Analysis:");
    println!("    Confidence: {}", analysis.confidence);
    println!("    Has critical gaps: {}", analysis.has_critical_gaps);

    if !analysis.recommendations.is_empty() {
        println!("    Recommendations:");
        for rec in &analysis.recommendations {
            println!("      - {}", rec);
        }
    }

    println!();
    Ok(())
}

/// Demonstrates report generation
async fn report_generation_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("6. TEST REPORT GENERATION\n");

    // Create test summary
    let summary = TestSummary {
        total_tests: 20,
        passed_tests: 18,
        failed_tests: 2,
        total_duration_ms: 5000,
        pass_rate: 0.9,
    };

    // Create report
    let mut report = TestReport::new("integration_test_suite", summary);

    // Add sample test results
    let result1 = TestResultDetail {
        test_id: "test_001".to_string(),
        test_name: "Workflow Initialization".to_string(),
        status: abcdodaf::testing::reporting::TestStatus::Passed,
        duration_ms: 250,
        error: None,
        assertions: vec![
            AssertionResult::passed("Workflow created", "Workflow initialized successfully"),
        ],
        tags: vec!["setup".to_string()],
    };

    let result2 = TestResultDetail {
        test_id: "test_002".to_string(),
        test_name: "Task Execution".to_string(),
        status: abcdodaf::testing::reporting::TestStatus::Passed,
        duration_ms: 500,
        error: None,
        assertions: vec![
            AssertionResult::passed("Task completed", "Task finished with expected output"),
        ],
        tags: vec!["execution".to_string()],
    };

    report = report
        .with_test_result(result1)
        .with_test_result(result2);

    // Generate HTML report
    let html = ReportGenerator::generate_html(&report);
    println!("  Generated HTML Report: {} bytes", html.len());
    println!("    Contains: <!DOCTYPE html>, style tags, and test results");

    // Generate JSON report
    if let Ok(json) = ReportGenerator::generate_json(&report) {
        println!("  Generated JSON Report: {} bytes", json.len());
    }

    // Display summary
    println!("\n  Report Summary:");
    println!("    Suite: {}", report.suite_name);
    println!("    Total tests: {}", report.summary.total_tests);
    println!("    Passed: {}", report.summary.passed_tests);
    println!("    Failed: {}", report.summary.failed_tests);
    println!("    Pass rate: {:.1}%", report.summary.pass_rate * 100.0);
    println!("    Duration: {}ms", report.summary.total_duration_ms);

    println!();
    Ok(())
}
