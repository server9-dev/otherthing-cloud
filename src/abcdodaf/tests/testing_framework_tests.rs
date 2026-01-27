//! Comprehensive tests for the testing framework itself

use abcdodaf::bpmn::ProcessExecutor;
use abcdodaf::prelude::TaskHandler;
use abcdodaf::testing::*;
use std::collections::HashMap;

#[tokio::test]
async fn test_harness_basic_workflow() {
    let executor = ProcessExecutor::new();
    let harness = TestHarness::new(executor);

    let test = TaskTestCase::new("test_001", "Basic Test", "task_1")
        .with_description("Tests basic task execution")
        .with_input("input", serde_json::json!("test"))
        .with_expected_output("executed", serde_json::json!(true))
        .with_timeout(5000)
        .with_tag("basic");

    let result = harness.run_task_test(test).await.unwrap();
    assert!(result.passed);
}

#[tokio::test]
async fn test_harness_multiple_tests() {
    let executor = ProcessExecutor::new();
    let harness = TestHarness::new(executor);

    let test1 =
        TaskTestCase::new("t1", "Test 1", "task_1").with_input("value", serde_json::json!(1));
    let test2 =
        TaskTestCase::new("t2", "Test 2", "task_2").with_input("value", serde_json::json!(2));
    let test3 =
        TaskTestCase::new("t3", "Test 3", "task_3").with_input("value", serde_json::json!(3));

    let results = harness.run_task_tests(vec![test1, test2, test3]).await.unwrap();

    assert_eq!(results.len(), 3);
    let summary = harness.get_summary().await;
    assert_eq!(summary.total_tests, 3);
}

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
    let handler = MockHandlerBuilder::new()
        .fixed_response_value("status", serde_json::json!("success"))
        .build();

    let output = handler.execute("task1", &HashMap::new()).await.unwrap();
    assert_eq!(output.get("status"), Some(&serde_json::json!("success")));
}

#[tokio::test]
async fn test_mock_handler_failure() {
    let handler = MockHandlerBuilder::new().failure("Test error").build();

    let result = handler.execute("task1", &HashMap::new()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_recording_handler() {
    let handler = RecordingMockHandler::new(MockBehavior::PassThrough);

    let input1 = HashMap::new();
    handler.execute("task1", &input1).await.unwrap();
    handler.execute("task2", &input1).await.unwrap();

    assert!(handler.assert_called("task1").await);
    assert!(handler.assert_called("task2").await);
    assert!(handler.assert_call_order(&["task1", "task2"]).await);
    assert_eq!(handler.call_count().await, 2);
}

#[test]
fn test_task_test_case_builder() {
    let test = TaskTestCase::new("t1", "Test 1", "task_id")
        .with_description("A test case")
        .with_input("input1", serde_json::json!("value1"))
        .with_expected_output("output1", serde_json::json!("expected"))
        .with_timeout(3000)
        .with_tag("unit")
        .with_tag("basic");

    assert_eq!(test.id, "t1");
    assert_eq!(test.name, "Test 1");
    assert_eq!(test.timeout_ms, 3000);
    assert_eq!(test.tags.len(), 2);
    assert!(test.tags.contains(&"unit".to_string()));
}

#[test]
fn test_fixture_creation() {
    let fixture = FixtureBuilder::new("f1", "Fixture 1")
        .with_variable("v1", serde_json::json!(1))
        .with_variable("v2", serde_json::json!(2))
        .with_description("Test fixture")
        .with_tag("test")
        .build();

    assert_eq!(fixture.id, "f1");
    assert_eq!(fixture.name, "Fixture 1");
    assert_eq!(fixture.variables().len(), 2);
    assert!(fixture.tags.contains(&"test".to_string()));
}

#[test]
fn test_fixture_pool() {
    let pool = FixturePool::new()
        .add(TestFixture::new("f1", "Fixture 1").with_tag("type1"))
        .add(TestFixture::new("f2", "Fixture 2").with_tag("type2"))
        .add(TestFixture::new("f3", "Fixture 3").with_tag("type1"));

    assert_eq!(pool.count(), 3);
    assert!(pool.get("f1").is_some());
    assert_eq!(pool.get_by_tag("type1").len(), 2);
}

#[test]
fn test_sample_data_generation() {
    let agent = SampleDataGenerator::sample_agent_task();
    assert!(!agent.id.is_empty());

    let json = SampleDataGenerator::sample_json_object();
    assert!(json.is_object());

    let array = SampleDataGenerator::sample_json_array(5);
    assert!(array.is_array());
    assert_eq!(array.as_array().unwrap().len(), 5);
}

#[test]
fn test_workflow_assertions() {
    let metrics = abcdodaf::workforce::ExecutionMetrics {
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
    assert_eq!(assertions.passed_count(), 4);
}

#[test]
fn test_task_assertions() {
    let output = serde_json::json!({
        "result": "success"
    });

    let result = abcdodaf::workforce::TaskResult {
        task_id: "t1".to_string(),
        success: true,
        error: None,
        output,
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

#[test]
fn test_scenario_builder() {
    let scenario = ScenarioBuilder::new("s1", "Test Scenario")
        .with_description("A test scenario")
        .with_variable("var1", serde_json::json!("value1"))
        .with_tag("unit")
        .build();

    assert_eq!(scenario.id, "s1");
    assert_eq!(scenario.variables.len(), 1);
    assert_eq!(scenario.tags.len(), 1);
}

#[test]
fn test_simple_workflow_dsl() {
    let scenario = ScenarioDsl::simple_workflow(
        "workflow",
        vec![("t1", "Task 1"), ("t2", "Task 2"), ("t3", "Task 3")],
    );

    assert_eq!(scenario.execution_steps.len(), 3);
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

#[tokio::test]
async fn test_performance_metrics() {
    let mut metrics = PerformanceMetrics::new("test");
    metrics.add_duration(100);
    metrics.add_duration(200);
    metrics.add_duration(300);

    assert_eq!(metrics.count(), 3);
    assert_eq!(metrics.average_duration(), 200.0);
    assert_eq!(metrics.min_duration(), Some(100));
    assert_eq!(metrics.max_duration(), Some(300));
}

#[tokio::test]
async fn test_async_benchmark() {
    let suite = BenchmarkSuite::new("test_suite");

    let result = suite
        .run_async_benchmark("async_test", 10, || async {
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        })
        .await;

    assert_eq!(result.metrics.count(), 10);
    assert!(result.ops_per_second > 0.0);
}

#[test]
fn test_perf_timer() {
    let timer = PerfTimer::start("test");
    std::thread::sleep(std::time::Duration::from_millis(10));
    let elapsed = timer.elapsed_ms();
    assert!(elapsed >= 10);
}

#[tokio::test]
async fn test_coverage_tracker() {
    let tracker = CoverageTracker::new();

    tracker.register_path("path1", "Path 1").await;
    tracker.register_path("path2", "Path 2").await;
    tracker.mark_critical("path1").await;

    tracker.execute_path("path1").await;

    let coverage = tracker.coverage_percent().await;
    assert_eq!(coverage, 0.5);

    let report = tracker.generate_report().await;
    assert_eq!(report.total_paths, 2);
    assert_eq!(report.covered_paths, 1);
}

#[tokio::test]
async fn test_branch_coverage() {
    let coverage = BranchCoverage::new();

    coverage.register_branch("b1", "Branch 1").await;
    coverage.register_branch("b2", "Branch 2").await;

    coverage.take_branch("b1").await;

    let percent = coverage.coverage_percent().await;
    assert_eq!(percent, 0.5);
}

#[test]
fn test_coverage_analysis() {
    let report = PathCoverage {
        total_paths: 10,
        covered_paths: 8,
        coverage_percent: 0.8,
        paths: Vec::new(),
        missed_critical_paths: Vec::new(),
    };

    let analysis = CoverageAnalyzer::analyze_coverage(&report);
    assert_eq!(analysis.coverage_percent, 0.8);
    assert_eq!(analysis.confidence, "Medium");
}

#[test]
fn test_coverage_comparison() {
    let before = PathCoverage {
        total_paths: 10,
        covered_paths: 5,
        coverage_percent: 0.5,
        paths: Vec::new(),
        missed_critical_paths: Vec::new(),
    };

    let after = PathCoverage {
        total_paths: 10,
        covered_paths: 8,
        coverage_percent: 0.8,
        paths: Vec::new(),
        missed_critical_paths: Vec::new(),
    };

    let comparison = CoverageAnalyzer::compare_coverage(&before, &after);
    assert!(comparison.improvement > 0.0);
    assert_eq!(comparison.status, "Improved");
}

#[test]
fn test_test_report_generation() {
    let summary = TestSummary {
        total_tests: 10,
        passed_tests: 9,
        failed_tests: 1,
        total_duration_ms: 1000,
        pass_rate: 0.9,
    };

    let report = TestReport::new("test_suite", summary);
    assert_eq!(report.suite_name, "test_suite");

    let html = report.to_html();
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("Test Report"));

    let json = report.to_json();
    assert!(json.is_ok());
}

#[test]
fn test_coverage_status() {
    assert_eq!(CoverageStatus::from_percent(0.95), CoverageStatus::Excellent);
    assert_eq!(CoverageStatus::from_percent(0.8), CoverageStatus::Good);
    assert_eq!(CoverageStatus::from_percent(0.6), CoverageStatus::Acceptable);
    assert_eq!(CoverageStatus::from_percent(0.5), CoverageStatus::Poor);
}

#[test]
fn test_assertion_result() {
    let passed = AssertionResult::passed("Check", "OK");
    assert!(passed.passed);

    let failed = AssertionResult::failed("Check", "Failed");
    assert!(!failed.passed);
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
        .with_error_message("Should be success")
        .build();

    assert_eq!(validation.id, "v1");
    assert!(validation.error_message.is_some());
}
