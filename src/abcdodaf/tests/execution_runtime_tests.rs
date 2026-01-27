//! Integration tests for enhanced execution runtime

use abcdodaf::bpmn::{
    EnhancedRuntime, ExecutionEvent, ExecutionMode, ProcessBuilder, TaskHandler,
};
use abcdodaf::error::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

struct TestHandler;

#[async_trait]
impl TaskHandler for TestHandler {
    async fn execute(
        &self,
        _task: &abcdodaf::bpmn::process::Task,
        variables: &HashMap<String, serde_json::Value>,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let mut output = variables.clone();
        output.insert("test_executed".to_string(), serde_json::json!(true));
        Ok(output)
    }

    fn name(&self) -> &str {
        "test_handler"
    }
}

#[tokio::test]
async fn test_enhanced_runtime_basic_execution() {
    let process = ProcessBuilder::new("test", "Test Process")
        .add_user_task("task1", "Task 1")
        .build()
        .unwrap();

    let runtime = EnhancedRuntime::new().register_handler("user", Arc::new(TestHandler));

    let instance = runtime
        .execute_process(&process, HashMap::new(), ExecutionMode::Continuous)
        .await
        .unwrap();

    assert_eq!(instance.state, abcdodaf::bpmn::ProcessState::Completed);
}

#[tokio::test]
async fn test_breakpoint_management() {
    let runtime = EnhancedRuntime::new();

    // Add breakpoint
    runtime.add_breakpoint("task1".to_string()).await;
    let breakpoints = runtime.get_breakpoints().await;
    assert_eq!(breakpoints.len(), 1);
    assert_eq!(breakpoints[0].element_id, "task1");
    assert!(breakpoints[0].enabled);

    // Toggle breakpoint
    runtime.toggle_breakpoint("task1").await;
    let breakpoints = runtime.get_breakpoints().await;
    assert!(!breakpoints[0].enabled);

    // Remove breakpoint
    runtime.remove_breakpoint("task1").await;
    let breakpoints = runtime.get_breakpoints().await;
    assert_eq!(breakpoints.len(), 0);
}

#[tokio::test]
async fn test_execution_events() {
    let process = ProcessBuilder::new("test", "Test Process")
        .add_user_task("task1", "Task 1")
        .build()
        .unwrap();

    let runtime = EnhancedRuntime::new().register_handler("user", Arc::new(TestHandler));

    let mut event_rx = runtime.subscribe_events();
    let mut events = Vec::new();

    // Spawn event collector
    let collector = tokio::spawn(async move {
        let mut collected = Vec::new();
        while let Ok(event) = event_rx.recv().await {
            collected.push(event);
            if collected.len() >= 4 {
                break;
            }
        }
        collected
    });

    // Execute process
    let _ = runtime
        .execute_process(&process, HashMap::new(), ExecutionMode::Continuous)
        .await;

    // Wait for events
    events = collector.await.unwrap();

    // Check events
    assert!(!events.is_empty());
    assert!(events
        .iter()
        .any(|e| matches!(e, ExecutionEvent::ProcessStarted { .. })));
}

#[tokio::test]
async fn test_pause_resume() {
    let process = ProcessBuilder::new("test", "Test Process")
        .add_user_task("task1", "Task 1")
        .build()
        .unwrap();

    let runtime = EnhancedRuntime::new().register_handler("user", Arc::new(TestHandler));

    let instance_id = runtime
        .start_process(&process, ExecutionMode::Continuous)
        .await
        .unwrap();

    // Pause
    runtime.pause(instance_id).await.unwrap();
    let context = runtime.get_context(instance_id).await.unwrap();
    assert_eq!(context.mode, ExecutionMode::Paused);

    // Resume
    runtime.resume(instance_id).await.unwrap();
    let context = runtime.get_context(instance_id).await.unwrap();
    assert_eq!(context.mode, ExecutionMode::Continuous);
}

#[tokio::test]
async fn test_performance_tracking() {
    let process = ProcessBuilder::new("test", "Test Process")
        .add_user_task("task1", "Task 1")
        .add_service_task("task2", "Task 2")
        .build()
        .unwrap();

    let runtime = EnhancedRuntime::new()
        .register_handler("user", Arc::new(TestHandler))
        .register_handler("service", Arc::new(TestHandler));

    let instance = runtime
        .execute_process(&process, HashMap::new(), ExecutionMode::Continuous)
        .await
        .unwrap();

    // Get performance data
    let performance = runtime.get_performance(instance.id).await;
    assert!(performance.is_some());

    let perf = performance.unwrap();
    assert!(perf.len() >= 1);

    // Check metrics exist
    for (task_id, metrics) in perf {
        assert!(metrics.execution_count > 0);
        assert!(metrics.avg_duration_ms >= 0);
    }
}

#[tokio::test]
async fn test_audit_trail() {
    let process = ProcessBuilder::new("test", "Test Process")
        .add_user_task("task1", "Task 1")
        .build()
        .unwrap();

    let runtime = EnhancedRuntime::new().register_handler("user", Arc::new(TestHandler));

    let _ = runtime
        .execute_process(&process, HashMap::new(), ExecutionMode::Continuous)
        .await;

    // Get audit trail
    let audit = runtime.get_audit_trail().await;
    assert!(!audit.is_empty());

    // Get recent events
    let recent = runtime.get_recent_audit(5).await;
    assert!(!recent.is_empty());
    assert!(recent.len() <= 5);
}

#[tokio::test]
async fn test_conditional_breakpoint() {
    let runtime = EnhancedRuntime::new();

    // Add conditional breakpoint
    runtime
        .add_conditional_breakpoint("task1".to_string(), "x > 10".to_string())
        .await;

    let breakpoints = runtime.get_breakpoints().await;
    assert_eq!(breakpoints.len(), 1);
    assert!(breakpoints[0].condition.is_some());
    assert_eq!(breakpoints[0].condition.as_ref().unwrap(), "x > 10");
}

#[tokio::test]
async fn test_bottleneck_analysis() {
    let process = ProcessBuilder::new("test", "Test Process")
        .add_user_task("fast_task", "Fast Task")
        .add_user_task("slow_task", "Slow Task")
        .build()
        .unwrap();

    let runtime = EnhancedRuntime::new().register_handler("user", Arc::new(TestHandler));

    let instance = runtime
        .execute_process(&process, HashMap::new(), ExecutionMode::Continuous)
        .await
        .unwrap();

    // Analyze bottlenecks
    let bottlenecks = runtime.analyze_bottlenecks(instance.id).await;

    // Should return tasks sorted by duration
    assert!(!bottlenecks.is_empty());

    // Check ordering (descending by avg duration)
    for i in 1..bottlenecks.len() {
        assert!(
            bottlenecks[i - 1].1.avg_duration_ms >= bottlenecks[i].1.avg_duration_ms
        );
    }
}
