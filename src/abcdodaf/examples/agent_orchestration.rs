//! Agent orchestration example
//!
//! Demonstrates advanced AI agent orchestration with DoDAF architecture

use abcdodaf::dodaf::{
    BaseCapability as Capability, BaseCapabilityType as CapabilityType, CapabilityView,
    MissionArea, OperationalView, Service, ServiceType, ServiceView,
};
use abcdodaf::prelude::*;
// BPM+ triple threat is demonstrated in examples/bpm_plus_triple_threat.rs

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    println!("=== ABCDODAF Agent Orchestration Example ===\n");

    // Define DoDAF architecture for AI agent workforce
    let architecture = abcdodaf::dodaf::DodafArchitecture::new("AI Agent Workforce Architecture")
        .with_description("DoDAF 2.02 architecture for AI-powered task execution")
        .with_operational_view(
            OperationalView::new()
                .add_mission_area(MissionArea {
                    id: "ma-1".to_string(),
                    name: "Intelligent Task Processing".to_string(),
                    description: Some("Process complex tasks using AI agents".to_string()),
                    capabilities: vec!["cap-nlp".to_string(), "cap-reasoning".to_string()],
                })
                .add_activity(
                    abcdodaf::dodaf::OperationalActivity::new("oa-1", "Analyze Complex Problem")
                        .with_type(abcdodaf::dodaf::ActivityType::Automated)
                        .add_performer("Reasoning Agent")
                        .add_input("problem_description")
                        .add_output("analysis_result"),
                )
                .add_activity(
                    abcdodaf::dodaf::OperationalActivity::new("oa-2", "Generate Solution")
                        .with_type(abcdodaf::dodaf::ActivityType::Hybrid)
                        .add_performer("Code Generation Agent")
                        .add_performer("Human Expert")
                        .add_input("analysis_result")
                        .add_output("solution"),
                ),
        )
        .with_capability_view(
            CapabilityView::new()
                .add_capability(
                    Capability::new(
                        "cap-nlp",
                        "Natural Language Processing",
                        CapabilityType::Cognitive,
                    )
                    .with_description("Advanced NLP for understanding and generation")
                    .add_resource("LLM Model")
                    .add_metric("accuracy", serde_json::json!(0.95)),
                )
                .add_capability(
                    Capability::new(
                        "cap-reasoning",
                        "Logical Reasoning",
                        CapabilityType::Cognitive,
                    )
                    .with_description("Multi-step logical reasoning capability")
                    .add_resource("Reasoning Engine")
                    .add_metric("consistency", serde_json::json!(0.92)),
                )
                .add_capability(
                    Capability::new("cap-code", "Code Generation", CapabilityType::Cognitive)
                        .with_description("Generate and analyze code")
                        .add_resource("Code LLM")
                        .add_metric("correctness", serde_json::json!(0.88)),
                ),
        )
        .with_services_view(
            ServiceView::new()
                .add_service(
                    Service::new("svc-nlp", "NLP Service", ServiceType::AiInference)
                        .with_description("Natural language processing service")
                        .with_endpoint("http://localhost:8080/nlp")
                        .provides("cap-nlp"),
                )
                .add_service(
                    Service::new("svc-reasoning", "Reasoning Service", ServiceType::AiInference)
                        .with_description("Logical reasoning service")
                        .with_endpoint("http://localhost:8080/reasoning")
                        .provides("cap-reasoning"),
                ),
        );

    println!("DoDAF Architecture: {}", architecture.name);
    println!("Capabilities: {}", architecture.capability_view.capabilities.len());
    println!("Services: {}", architecture.services_view.services.len());

    // BPM+ triple threat (BPMN, CMMN, DMN) demonstrated in examples/bpm_plus_triple_threat.rs
    println!("\n=== BPM+ Triple Threat ===");
    println!("BPMN: Process flows and orchestration");
    println!("CMMN: Adaptive case management");
    println!("DMN: Business decision logic");
    println!("See examples/bpm_plus_triple_threat.rs for comprehensive example");

    // Create multi-agent workflow
    println!("\n=== Creating Multi-Agent Workflow ===");
    let workflow = WorkflowBuilder::new("multi_agent_analysis")
        .name("Multi-Agent Problem Analysis")
        .description("Coordinate multiple AI agents to analyze and solve complex problems")
        // Agent 1: Problem decomposition
        .add_agent_task(
            "decompose",
            AgentTask::new(
                "decompose",
                "Decompose Problem",
                AgentCapability::LogicalReasoning,
            )
            .with_agent_type(abcdodaf::workforce::AgentType::Reasoner)
            .with_autonomy(0.9)
            .add_parameter("max_depth", serde_json::json!(3)),
        )
        // Agent 2: Research and information gathering
        .add_agent_task(
            "research",
            AgentTask::new(
                "research",
                "Research Solutions",
                AgentCapability::InformationRetrieval,
            )
            .with_agent_type(abcdodaf::workforce::AgentType::ToolUser)
            .with_autonomy(0.8)
            .add_capability(AgentCapability::KnowledgeSynthesis),
        )
        // Agent 3: Code generation
        .add_agent_task(
            "code_gen",
            AgentTask::new(
                "code_gen",
                "Generate Implementation",
                AgentCapability::CodeGeneration,
            )
            .with_agent_type(abcdodaf::workforce::AgentType::CodeAgent)
            .with_autonomy(0.7)
            .add_capability(AgentCapability::PatternRecognition),
        )
        // Human expert review
        .add_human_task(
            "expert_review",
            HumanTask::new(
                "expert_review",
                "Expert Review",
                HumanRole::Expert,
            )
            .with_complexity(5)
            .with_duration(45),
        )
        .with_context(
            OperationalContext::new()
                .with_mission_area("Intelligent Task Processing")
                .with_capability("Multi-Agent Coordination")
                .with_resource("agent_pool_size", serde_json::json!(5))
                .with_constraint("max_parallel_agents", serde_json::json!(3)),
        )
        .build()?;

    println!("Workflow: {}", workflow.process.name);
    println!("Total tasks: {}", workflow.tasks.len());

    // Execute workflow
    println!("\n=== Executing Workflow ===");
    let result = workflow.execute().await?;

    println!("\n=== Results ===");
    println!("Status: {:?}", result.instance.state);
    println!("Agent tasks executed: {}", result.metrics.agent_tasks);
    println!("Human tasks executed: {}", result.metrics.human_tasks);
    println!("Total duration: {}ms", result.metrics.total_duration_ms);
    println!("Success rate: {:.1}%", result.metrics.success_rate * 100.0);

    // Export architecture to JSON
    println!("\n=== Exporting Architecture ===");
    let json = architecture.to_json()?;
    println!("Architecture exported ({} bytes)", json.len());

    Ok(())
}
