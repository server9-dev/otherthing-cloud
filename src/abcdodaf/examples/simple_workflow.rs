//! Simple workflow example
//!
//! Demonstrates how to create and execute a basic workflow using ABCDODAF

use abcdodaf::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("=== ABCDODAF Simple Workflow Example ===\n");

    // Create a simple workflow with agent, human, and system tasks
    let workflow = WorkflowBuilder::new("customer_support_workflow")
        .name("Customer Support Workflow")
        .description("Process customer inquiry with AI assistance and human review")
        // Agent task: Analyze customer inquiry
        .add_agent_task(
            "analyze_inquiry",
            AgentTask::new(
                "analyze_inquiry",
                "Analyze Customer Inquiry",
                AgentCapability::NaturalLanguageProcessing,
            )
            .with_description("Use NLP to understand customer intent and extract key information")
            .with_autonomy(0.8)
            .add_parameter("model", serde_json::json!("gpt-4"))
            .add_parameter("temperature", serde_json::json!(0.7)),
        )
        // Agent task: Generate response
        .add_agent_task(
            "generate_response",
            AgentTask::new(
                "generate_response",
                "Generate Response",
                AgentCapability::NaturalLanguageProcessing,
            )
            .with_description("Generate a helpful response to the customer")
            .with_autonomy(0.7),
        )
        // Human task: Review and approve
        .add_human_task(
            "review_response",
            HumanTask::new(
                "review_response",
                "Review AI Response",
                HumanRole::QualityAssurance,
            )
            .with_description("Review AI-generated response for quality and accuracy")
            .with_complexity(3)
            .with_duration(10),
        )
        // System task: Send response
        .add_system_task(
            "send_response",
            SystemTask::new(
                "send_response",
                "Send Response to Customer",
                SystemOperation::ApiCall,
            )
            .with_description("Send the approved response via email API")
            .add_config("api_endpoint", serde_json::json!("https://api.email.com/send"))
            .with_timeout(30),
        )
        // System task: Log interaction
        .add_system_task(
            "log_interaction",
            SystemTask::new(
                "log_interaction",
                "Log Interaction",
                SystemOperation::DatabaseWrite,
            )
            .with_description("Store interaction in customer database")
            .with_retry_policy(
                abcdodaf::workforce::task::RetryPolicy::new(3)
                    .with_initial_delay(1000)
                    .with_backoff(2.0),
            ),
        )
        .with_context(
            OperationalContext::new()
                .with_mission_area("Customer Support Operations")
                .with_capability("AI-Assisted Support")
                .with_resource("max_response_time_secs", serde_json::json!(300)),
        )
        .build()?;

    println!("Workflow created: {}", workflow.process.name);
    println!("Tasks: {}", workflow.tasks.len());
    println!("\nTask breakdown:");
    for task in &workflow.tasks {
        println!("  - {}: {}", task.id(), task.name());
    }

    // Execute the workflow
    println!("\nExecuting workflow...\n");
    let result = workflow.execute().await?;

    // Display results
    println!("=== Execution Results ===");
    println!("Status: {:?}", result.instance.state);
    println!("\nMetrics:");
    println!("  Total Duration: {}ms", result.metrics.total_duration_ms);
    println!("  Agent Tasks: {}", result.metrics.agent_tasks);
    println!("  Human Tasks: {}", result.metrics.human_tasks);
    println!("  System Tasks: {}", result.metrics.system_tasks);
    println!("  Success Rate: {:.1}%", result.metrics.success_rate * 100.0);

    println!("\nTask Results:");
    for task_result in result.task_results {
        println!(
            "  {} - {} ({}ms)",
            task_result.task_id,
            if task_result.success { "✓" } else { "✗" },
            task_result.duration_ms
        );
    }

    Ok(())
}
