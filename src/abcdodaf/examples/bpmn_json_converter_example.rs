//! BPMN JSON Converter Example
//!
//! Demonstrates how to load BPMN workflow files and convert them to Snarl format.

use abcdodaf::ui::bpmn_json_loader::{load_workflow_file, BpmnJsonConverter};
use std::path::PathBuf;

fn main() -> Result<(), String> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("=== BPMN JSON to Snarl Converter Example ===\n");

    // Example 1: Load from a JSON string
    println!("Example 1: Converting from JSON string");
    let json = r#"{
        "bpmn_process": {
            "id": "example_process",
            "name": "Example Workflow",
            "version": "1.0.0",
            "isExecutable": true,
            "processType": "agent_workflow"
        },
        "workflow_steps": [
            {
                "id": "start_event",
                "name": "Start",
                "type": "startEvent"
            },
            {
                "id": "research_step",
                "name": "Research Phase",
                "type": "serviceTask",
                "taskType": "research",
                "implementation": "research_method",
                "inputs": ["requirements"],
                "outputs": ["findings"]
            },
            {
                "id": "design_step",
                "name": "Design Phase",
                "type": "serviceTask",
                "taskType": "design",
                "implementation": "architectural_design",
                "inputs": ["findings"],
                "outputs": ["architecture"]
            },
            {
                "id": "decision_point",
                "name": "Ready to Implement?",
                "type": "exclusiveGateway"
            },
            {
                "id": "implementation_step",
                "name": "Implementation Phase",
                "type": "serviceTask",
                "taskType": "code_generation",
                "inputs": ["architecture"],
                "outputs": ["code"]
            },
            {
                "id": "end_event",
                "name": "Complete",
                "type": "endEvent"
            }
        ],
        "sequence_flows": [
            {"id": "flow1", "sourceRef": "start_event", "targetRef": "research_step"},
            {"id": "flow2", "sourceRef": "research_step", "targetRef": "design_step"},
            {"id": "flow3", "sourceRef": "design_step", "targetRef": "decision_point"},
            {"id": "flow4", "sourceRef": "decision_point", "targetRef": "implementation_step", "condition": "approved"},
            {"id": "flow5", "sourceRef": "implementation_step", "targetRef": "end_event"}
        ]
    }"#;

    let converter = BpmnJsonConverter::new();
    let workflow = converter.load_from_string(json)?;

    println!("  Workflow Name: {}", workflow.name);
    println!("  Version: {}", workflow.version);
    println!("  Nodes: {}", workflow.snarl.node_ids().count());
    println!("  Wires: {}", workflow.snarl.wires().count());

    // Example 2: Try to load a workflow file if one exists
    println!("\nExample 2: Loading from workflow file");
    let workflow_path = PathBuf::from("workflows/task_01_bpmn_xml.json");

    if workflow_path.exists() {
        match load_workflow_file(&workflow_path) {
            Ok(workflow) => {
                println!("  Successfully loaded: {}", workflow.name);
                println!("  Nodes: {}", workflow.snarl.node_ids().count());
                println!("  Wires: {}", workflow.snarl.wires().count());

                // Print node details
                println!("\n  Nodes:");
                for (_node_id, node) in workflow.snarl.node_ids() {
                    println!("    - {} ({})", node.name(), node.type_name());
                }
            },
            Err(e) => {
                println!("  Error loading workflow: {}", e);
            },
        }
    } else {
        println!("  Workflow file not found: {:?}", workflow_path);
    }

    // Example 3: Demonstrate supported node types
    println!("\nExample 3: Supported BPMN node types");
    let types_example = r#"{
        "bpmn_process": {
            "id": "types_showcase",
            "name": "BPMN Node Types Showcase",
            "version": "1.0.0",
            "isExecutable": true
        },
        "workflow_steps": [
            {"id": "start", "name": "Start", "type": "startEvent"},
            {"id": "user_task", "name": "User Task", "type": "userTask"},
            {"id": "service_task", "name": "Service Task", "type": "serviceTask"},
            {"id": "script_task", "name": "Script Task", "type": "scriptTask"},
            {"id": "manual_task", "name": "Manual Task", "type": "manualTask"},
            {"id": "business_rule", "name": "Business Rule", "type": "businessRuleTask"},
            {"id": "exclusive_gw", "name": "Exclusive Gateway", "type": "exclusiveGateway"},
            {"id": "parallel_gw", "name": "Parallel Gateway", "type": "parallelGateway"},
            {"id": "inclusive_gw", "name": "Inclusive Gateway", "type": "inclusiveGateway"},
            {"id": "end", "name": "End", "type": "endEvent"}
        ],
        "sequence_flows": []
    }"#;

    let types_workflow = converter.load_from_string(types_example)?;
    println!(
        "  Created workflow with {} different node types",
        types_workflow.snarl.node_ids().count()
    );

    for (_node_id, node) in types_workflow.snarl.node_ids() {
        println!("    - {}: {}", node.type_name(), node.name());
    }

    println!("\n=== Conversion complete! ===");
    println!("\nThe WorkflowFile can now be:");
    println!("  - Saved using workspace.save_workflow()");
    println!("  - Loaded into the UI editor");
    println!("  - Edited visually with Snarl");
    println!("  - Validated with the validation system");
    println!("  - Executed with the runtime engine");

    Ok(())
}
