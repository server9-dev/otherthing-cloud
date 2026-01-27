//! BpmnDiagram to Snarl Converter Example
//!
//! Demonstrates bidirectional conversion between BpmnDiagram (library format)
//! and Snarl<EnhancedBpmnNode> (visual editor format).

use abcdodaf::bpmn::elements::*;
use abcdodaf::ui::BpmnDiagramConverter;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== BpmnDiagram to Snarl Converter Example ===\n");

    // Create a sample BPMN diagram
    let diagram = create_sample_diagram();
    println!("Created sample BPMN diagram:");
    println!("  - ID: {}", diagram.id);
    println!("  - Processes: {}", diagram.processes.len());
    println!("  - Process elements: {} start, {} tasks, {} gateways, {} end",
        diagram.processes[0].start_events.len(),
        diagram.processes[0].tasks.len(),
        diagram.processes[0].gateways.len(),
        diagram.processes[0].end_events.len(),
    );
    println!("  - Sequence flows: {}\n", diagram.processes[0].sequence_flows.len());

    // Convert to Snarl for visual editing
    println!("Converting BpmnDiagram to Snarl...");
    let snarl = BpmnDiagramConverter::to_snarl(&diagram)?;

    println!("Snarl created successfully:");
    let node_count = snarl.node_ids().count();
    println!("  - Nodes: {}", node_count);
    println!();

    // Convert back to BpmnDiagram
    println!("Converting Snarl back to BpmnDiagram...");
    let result_diagram = BpmnDiagramConverter::from_snarl(
        &snarl,
        "result_diagram",
        "Converted Diagram",
    )?;

    println!("BpmnDiagram created from Snarl:");
    println!("  - ID: {}", result_diagram.id);
    println!("  - Processes: {}", result_diagram.processes.len());
    println!("  - Process elements: {} start, {} tasks, {} gateways, {} end",
        result_diagram.processes[0].start_events.len(),
        result_diagram.processes[0].tasks.len(),
        result_diagram.processes[0].gateways.len(),
        result_diagram.processes[0].end_events.len(),
    );
    println!("  - Sequence flows: {}\n", result_diagram.processes[0].sequence_flows.len());

    // Note: generate_diagram_info is available but not demonstrated here
    // It can be used to extract visual layout information from the Snarl
    println!("Visual layout information is preserved in the Snarl structure");

    // Verify round-trip conversion
    println!("\n=== Round-trip Verification ===");
    verify_roundtrip(&diagram, &result_diagram);

    println!("\n✓ Conversion example completed successfully!");

    Ok(())
}

/// Create a sample BPMN diagram for demonstration
fn create_sample_diagram() -> BpmnDiagram {
    let process = BpmnProcess {
        id: "sample_process".to_string(),
        name: Some("Sample Process".to_string()),
        documentation: Some("A sample process demonstrating converter capabilities".to_string()),
        is_executable: true,
        process_type: ProcessType::Private,

        start_events: vec![
            StartEvent {
                id: "start_1".to_string(),
                name: Some("Start".to_string()),
                documentation: None,
                event_definition: None,
                is_interrupting: true,
            },
        ],

        end_events: vec![
            EndEvent {
                id: "end_1".to_string(),
                name: Some("Success".to_string()),
                documentation: None,
                event_definition: None,
            },
            EndEvent {
                id: "end_2".to_string(),
                name: Some("Failure".to_string()),
                documentation: None,
                event_definition: Some(EventDefinition::Error {
                    error_ref: Some("error_1".to_string()),
                }),
            },
        ],

        intermediate_events: vec![
            IntermediateEvent {
                id: "timer_1".to_string(),
                name: Some("Wait Timer".to_string()),
                documentation: None,
                event_definition: Some(EventDefinition::Timer {
                    time_expression: "PT5M".to_string(),
                }),
                is_catching: true,
                is_interrupting: false,
                attached_to_ref: None,
            },
        ],

        tasks: vec![
            BpmnTask {
                id: "task_1".to_string(),
                name: Some("Validate Input".to_string()),
                documentation: Some("Validate incoming request data".to_string()),
                task_type: BpmnTaskType::User {
                    implementation: None,
                    rendering: None,
                },
                default_flow: None,
                io_specification: None,
                properties: HashMap::new(),
                loop_characteristics: None,
                is_for_compensation: false,
            },
            BpmnTask {
                id: "task_2".to_string(),
                name: Some("Process Data".to_string()),
                documentation: Some("Process the validated data".to_string()),
                task_type: BpmnTaskType::Service {
                    implementation: Some("rest".to_string()),
                    operation_ref: Some("processOperation".to_string()),
                },
                default_flow: None,
                io_specification: None,
                properties: {
                    let mut props = HashMap::new();
                    props.insert(
                        "endpoint".to_string(),
                        serde_json::json!("https://api.example.com/process"),
                    );
                    props
                },
                loop_characteristics: None,
                is_for_compensation: false,
            },
            BpmnTask {
                id: "task_3".to_string(),
                name: Some("Send Notification".to_string()),
                documentation: None,
                task_type: BpmnTaskType::Send {
                    message_ref: Some("notification_msg".to_string()),
                    operation_ref: None,
                },
                default_flow: None,
                io_specification: None,
                properties: HashMap::new(),
                loop_characteristics: None,
                is_for_compensation: false,
            },
        ],

        gateways: vec![
            BpmnGateway {
                id: "gateway_1".to_string(),
                name: Some("Valid?".to_string()),
                documentation: None,
                gateway_type: BpmnGatewayType::Exclusive,
                gateway_direction: GatewayDirection::Diverging,
                default_flow: None,
            },
        ],

        sequence_flows: vec![
            SequenceFlow {
                id: "flow_1".to_string(),
                name: None,
                source_ref: "start_1".to_string(),
                target_ref: "task_1".to_string(),
                condition_expression: None,
                is_immediate: true,
            },
            SequenceFlow {
                id: "flow_2".to_string(),
                name: None,
                source_ref: "task_1".to_string(),
                target_ref: "gateway_1".to_string(),
                condition_expression: None,
                is_immediate: true,
            },
            SequenceFlow {
                id: "flow_3".to_string(),
                name: Some("Valid".to_string()),
                source_ref: "gateway_1".to_string(),
                target_ref: "task_2".to_string(),
                condition_expression: Some("${valid == true}".to_string()),
                is_immediate: true,
            },
            SequenceFlow {
                id: "flow_4".to_string(),
                name: Some("Invalid".to_string()),
                source_ref: "gateway_1".to_string(),
                target_ref: "end_2".to_string(),
                condition_expression: Some("${valid == false}".to_string()),
                is_immediate: true,
            },
            SequenceFlow {
                id: "flow_5".to_string(),
                name: None,
                source_ref: "task_2".to_string(),
                target_ref: "timer_1".to_string(),
                condition_expression: None,
                is_immediate: true,
            },
            SequenceFlow {
                id: "flow_6".to_string(),
                name: None,
                source_ref: "timer_1".to_string(),
                target_ref: "task_3".to_string(),
                condition_expression: None,
                is_immediate: true,
            },
            SequenceFlow {
                id: "flow_7".to_string(),
                name: None,
                source_ref: "task_3".to_string(),
                target_ref: "end_1".to_string(),
                condition_expression: None,
                is_immediate: true,
            },
        ],

        subprocesses: Vec::new(),
        data_objects: Vec::new(),
        data_associations: Vec::new(),
        text_annotations: Vec::new(),
        groups: Vec::new(),
        lanes: Vec::new(),
        metadata: HashMap::new(),
    };

    BpmnDiagram {
        id: "sample_diagram".to_string(),
        name: Some("Sample Diagram".to_string()),
        documentation: Some("Demonstrates BPMN to Snarl conversion".to_string()),
        processes: vec![process],
        collaborations: Vec::new(),
        data_stores: Vec::new(),
        messages: vec![
            Message {
                id: "notification_msg".to_string(),
                name: Some("Notification".to_string()),
                item_ref: None,
            },
        ],
        signals: Vec::new(),
    }
}

/// Verify that round-trip conversion preserves data
fn verify_roundtrip(original: &BpmnDiagram, converted: &BpmnDiagram) {
    let orig_proc = &original.processes[0];
    let conv_proc = &converted.processes[0];

    println!("Comparing element counts:");

    let start_match = orig_proc.start_events.len() == conv_proc.start_events.len();
    println!("  Start events: {} -> {} {}",
        orig_proc.start_events.len(),
        conv_proc.start_events.len(),
        if start_match { "✓" } else { "✗" }
    );

    let task_match = orig_proc.tasks.len() == conv_proc.tasks.len();
    println!("  Tasks: {} -> {} {}",
        orig_proc.tasks.len(),
        conv_proc.tasks.len(),
        if task_match { "✓" } else { "✗" }
    );

    let gateway_match = orig_proc.gateways.len() == conv_proc.gateways.len();
    println!("  Gateways: {} -> {} {}",
        orig_proc.gateways.len(),
        conv_proc.gateways.len(),
        if gateway_match { "✓" } else { "✗" }
    );

    let intermediate_match = orig_proc.intermediate_events.len() == conv_proc.intermediate_events.len();
    println!("  Intermediate events: {} -> {} {}",
        orig_proc.intermediate_events.len(),
        conv_proc.intermediate_events.len(),
        if intermediate_match { "✓" } else { "✗" }
    );

    let end_match = orig_proc.end_events.len() == conv_proc.end_events.len();
    println!("  End events: {} -> {} {}",
        orig_proc.end_events.len(),
        conv_proc.end_events.len(),
        if end_match { "✓" } else { "✗" }
    );

    let flow_match = orig_proc.sequence_flows.len() == conv_proc.sequence_flows.len();
    println!("  Sequence flows: {} -> {} {}",
        orig_proc.sequence_flows.len(),
        conv_proc.sequence_flows.len(),
        if flow_match { "✓" } else { "✗" }
    );

    if start_match && task_match && gateway_match && intermediate_match && end_match && flow_match {
        println!("\n✓ All element counts match!");
    } else {
        println!("\n✗ Some element counts don't match (this may be expected for complex conversions)");
    }
}
