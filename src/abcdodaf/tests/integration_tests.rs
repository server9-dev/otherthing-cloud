//! Integration tests for ABCDODAF library

use abcdodaf::prelude::*;
use abcdodaf::dodaf::{
    BaseCapability as Capability, BaseCapabilityType as CapabilityType,
    CapabilityView, Service, ServiceType, ServiceView, OperationalView,
    DodafArchitecture, MissionArea, OperationalActivity, ActivityType,
};
// BPM+ is tested in bpm_plus module tests
use abcdodaf::workforce::*;

#[tokio::test]
async fn test_complete_workflow_execution() {
    // Create a complete workflow with all task types
    let workflow = WorkflowBuilder::new("integration_test")
        .name("Integration Test Workflow")
        .description("Test all workflow components")
        .add_agent_task(
            "agent1",
            AgentTask::new(
                "agent1",
                "AI Analysis",
                AgentCapability::NaturalLanguageProcessing,
            )
            .with_autonomy(0.9),
        )
        .add_human_task(
            "human1",
            HumanTask::new("human1", "Human Review", HumanRole::QualityAssurance)
                .with_complexity(3),
        )
        .add_system_task(
            "system1",
            SystemTask::new("system1", "System Process", SystemOperation::DatabaseWrite)
                .with_timeout(10),
        )
        .build()
        .unwrap();

    let result = workflow.execute().await.unwrap();

    assert_eq!(result.metrics.agent_tasks, 1);
    assert_eq!(result.metrics.human_tasks, 1);
    assert_eq!(result.metrics.system_tasks, 1);
    assert!(result.metrics.success_rate > 0.0);
}

#[test]
fn test_dodaf_architecture_complete() {
    // Create complete DoDAF architecture
    let arch = DodafArchitecture::new("Test Architecture")
        .with_description("Complete DoDAF test")
        .with_operational_view(
            OperationalView::new()
                .add_mission_area(MissionArea {
                    id: "m1".to_string(),
                    name: "Mission 1".to_string(),
                    description: None,
                    capabilities: vec!["c1".to_string()],
                })
                .add_activity(
                    OperationalActivity::new("a1", "Activity 1")
                        .with_type(ActivityType::Automated),
                ),
        )
        .with_capability_view(
            CapabilityView::new()
                .add_capability(Capability::new("c1", "Capability 1", CapabilityType::Cognitive)),
        )
        .with_services_view(
            ServiceView::new().add_service(Service::new(
                "s1",
                "Service 1",
                ServiceType::AiInference,
            )),
        );

    assert_eq!(arch.operational_view.mission_areas.len(), 1);
    assert_eq!(arch.capability_view.capabilities.len(), 1);
    assert_eq!(arch.services_view.services.len(), 1);

    // Test JSON export
    let json = arch.to_json().unwrap();
    assert!(!json.is_empty());
}

#[test]
fn test_bpm_plus_model_creation() {
    use abcdodaf::bpm_plus::BpmPlusModel;

    // Test BPM+ triple threat model creation
    let model = BpmPlusModel::new("Test Model", "Test Description");
    assert_eq!(model.metadata.name, "Test Model");
    assert_eq!(model.processes.len(), 0);
    assert_eq!(model.cases.len(), 0);
    assert_eq!(model.decisions.len(), 0);
}

#[test]
fn test_operational_context_building() {
    let context = OperationalContext::new()
        .with_mission_area("Test Mission")
        .with_capability("Capability 1")
        .with_capability("Capability 2")
        .with_resource("resource1", serde_json::json!({"value": 100}))
        .with_constraint("constraint1", serde_json::json!({"max": 50}));

    assert_eq!(context.mission_area, Some("Test Mission".to_string()));
    assert_eq!(context.required_capabilities.len(), 2);
    assert_eq!(context.resources.len(), 1);
    assert_eq!(context.constraints.len(), 1);
}

#[test]
fn test_agent_capabilities() {
    let agent = AgentTask::new("a1", "Multi-capability Agent", AgentCapability::CodeGeneration)
        .add_capability(AgentCapability::LogicalReasoning)
        .add_capability(AgentCapability::PatternRecognition);

    assert!(agent.has_capability(&AgentCapability::CodeGeneration));
    assert!(agent.has_capability(&AgentCapability::LogicalReasoning));
    assert!(agent.has_capability(&AgentCapability::PatternRecognition));
    assert!(!agent.has_capability(&AgentCapability::ComputerVision));
}

#[test]
fn test_retry_policy_calculation() {
    let policy = task::RetryPolicy::new(5)
        .with_initial_delay(1000)
        .with_backoff(2.0)
        .with_max_delay(10000);

    assert_eq!(policy.calculate_delay(1), 1000);
    assert_eq!(policy.calculate_delay(2), 2000);
    assert_eq!(policy.calculate_delay(3), 4000);
    assert_eq!(policy.calculate_delay(4), 8000);
    assert_eq!(policy.calculate_delay(5), 10000); // Capped at max
    assert_eq!(policy.calculate_delay(6), 10000); // Still capped
}

#[test]
fn test_process_builder_validation() {
    // Empty process should fail
    let result = ProcessBuilder::new("empty", "Empty").build();
    assert!(result.is_err());

    // Valid process should succeed
    let result = ProcessBuilder::new("valid", "Valid")
        .add_user_task("t1", "Task 1")
        .build();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_workflow_with_context() {
    let context = OperationalContext::new()
        .with_mission_area("Test Mission")
        .with_capability("AI Processing")
        .with_resource("max_duration", serde_json::json!(300));

    let workflow = WorkflowBuilder::new("context_test")
        .add_agent_task(
            "a1",
            AgentTask::new("a1", "Process", AgentCapability::NaturalLanguageProcessing),
        )
        .with_context(context.clone())
        .build()
        .unwrap();

    assert_eq!(
        workflow.context.mission_area,
        Some("Test Mission".to_string())
    );

    let result = workflow.execute().await.unwrap();
    assert!(result.instance.completed_at.is_some());
}

#[test]
fn test_task_type_conversions() {
    let agent_task = AgentTask::new("a1", "Agent", AgentCapability::CodeGeneration);
    let workforce_task = WorkforceTask::Agent(agent_task);

    assert_eq!(workforce_task.id(), "a1");
    assert_eq!(workforce_task.name(), "Agent");

    let operational = workforce_task.to_operational_activity();
    assert_eq!(operational.activity_type, ActivityType::Automated);
}

#[test]
fn test_capability_view_lookup() {
    let view = CapabilityView::new()
        .add_capability(Capability::new("c1", "Cap1", CapabilityType::Cognitive))
        .add_capability(Capability::new("c2", "Cap2", CapabilityType::Physical));

    assert!(view.get_capability("c1").is_some());
    assert!(view.get_capability("c2").is_some());
    assert!(view.get_capability("c3").is_none());

    let cap = view.get_capability("c1").unwrap();
    assert_eq!(cap.name, "Cap1");
}

#[test]
fn test_process_xml_export() {
    let process = ProcessBuilder::new("test", "Test Process")
        .add_user_task("t1", "Task 1")
        .add_service_task("t2", "Task 2")
        .add_flow("f1", "t1", "t2")
        .build()
        .unwrap();

    let xml = process.to_xml().unwrap();

    assert!(xml.contains("<?xml"));
    assert!(xml.contains("userTask"));
    assert!(xml.contains("serviceTask"));
    assert!(xml.contains("sequenceFlow"));
}

#[tokio::test]
async fn test_multi_agent_coordination() {
    let workflow = WorkflowBuilder::new("multi_agent")
        .add_agent_task(
            "reasoner",
            AgentTask::new("reasoner", "Reasoning", AgentCapability::LogicalReasoning)
                .with_agent_type(AgentType::Reasoner),
        )
        .add_agent_task(
            "coder",
            AgentTask::new("coder", "Coding", AgentCapability::CodeGeneration)
                .with_agent_type(AgentType::CodeAgent),
        )
        .add_agent_task(
            "reviewer",
            AgentTask::new("reviewer", "Review", AgentCapability::PatternRecognition)
                .with_agent_type(AgentType::ToolUser),
        )
        .build()
        .unwrap();

    let result = workflow.execute().await.unwrap();
    assert_eq!(result.metrics.agent_tasks, 3);
    assert_eq!(result.task_results.len(), 3);
}
