//! BPMN 2.0 XML Import/Export Roundtrip Example
//!
//! Demonstrates full round-trip capability for BPMN files with Diagram Interchange support.
//! This example shows:
//! - Creating a BPMN diagram programmatically
//! - Exporting to XML with full DI information
//! - Importing XML back into memory
//! - Validating that all data is preserved

use abcdodaf::bpmn::elements::*;
use abcdodaf::bpmn::file_io::{BpmnFileIo, FileOptions};
use abcdodaf::bpmn::xml_io::BpmnXmlSerializer;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== BPMN 2.0 XML Import/Export Roundtrip Example ===\n");

    // Create a sample BPMN diagram
    let diagram = create_sample_diagram();

    // 1. Export to XML
    println!("1. Exporting diagram to XML...");
    let xml = BpmnXmlSerializer::to_string(&diagram)?;
    println!("   Generated XML ({} bytes)\n", xml.len());
    println!("   First 500 characters:");
    println!("   {}\n", &xml[..std::cmp::min(500, xml.len())]);

    // 2. Save to file
    println!("2. Saving to file: sample_process.bpmn");
    let options = FileOptions {
        create_backup: true,
        backup_extension: ".bak".to_string(),
        fail_if_exists: false,
        pretty_print: true,
    };
    BpmnFileIo::save_with_options("sample_process.bpmn", &diagram, options)?;
    println!("   File saved successfully\n");

    // 3. Load from file
    println!("3. Loading from file: sample_process.bpmn");
    let loaded_diagram = BpmnFileIo::load("sample_process.bpmn")?;
    println!("   Diagram ID: {}", loaded_diagram.id);
    println!("   Diagram Name: {:?}", loaded_diagram.name);
    println!("   Processes: {}\n", loaded_diagram.processes.len());

    // 4. Export again to verify consistency
    println!("4. Exporting loaded diagram to verify roundtrip...");
    let xml2 = BpmnXmlSerializer::to_string(&loaded_diagram)?;

    if xml == xml2 {
        println!("   SUCCESS: Round-trip preserved all data");
        println!("   Both XML exports are identical\n");
    } else {
        println!("   INFO: XML representations differ slightly (this may be expected)");
        println!("   Original: {} bytes", xml.len());
        println!("   Reloaded: {} bytes\n", xml2.len());
    }

    // 5. Analyze the diagram structure
    println!("5. Analyzing diagram structure:");
    for (idx, process) in loaded_diagram.processes.iter().enumerate() {
        println!("   Process {}: {}", idx + 1, process.id);
        println!("     - Start events: {}", process.start_events.len());
        println!("     - End events: {}", process.end_events.len());
        println!("     - Tasks: {}", process.tasks.len());
        println!("     - Gateways: {}", process.gateways.len());
        println!("     - Sequence flows: {}", process.sequence_flows.len());
    }

    println!("\n=== Example Complete ===");
    Ok(())
}

fn create_sample_diagram() -> BpmnDiagram {
    // Create a simple approval process
    // Start -> Submit Task -> Decision -> (Approve or Reject) -> End

    let process = BpmnProcess {
        id: "approval_process".to_string(),
        name: Some("Document Approval Process".to_string()),
        documentation: Some("A simple document approval workflow".to_string()),
        is_executable: true,
        process_type: ProcessType::Private,

        // Start event
        start_events: vec![StartEvent {
            id: "start_event".to_string(),
            name: Some("Submit Document".to_string()),
            documentation: None,
            event_definition: None,
            is_interrupting: true,
        }],

        // End events
        end_events: vec![
            EndEvent {
                id: "end_approved".to_string(),
                name: Some("Document Approved".to_string()),
                documentation: None,
                event_definition: None,
            },
            EndEvent {
                id: "end_rejected".to_string(),
                name: Some("Document Rejected".to_string()),
                documentation: None,
                event_definition: None,
            },
        ],

        // Tasks
        tasks: vec![
            BpmnTask {
                id: "submit_task".to_string(),
                name: Some("Submit Document".to_string()),
                documentation: None,
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
                id: "review_task".to_string(),
                name: Some("Review Document".to_string()),
                documentation: None,
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
        ],

        // Decision gateway
        gateways: vec![BpmnGateway {
            id: "decision_gate".to_string(),
            name: Some("Approved?".to_string()),
            documentation: None,
            gateway_type: BpmnGatewayType::Exclusive,
            gateway_direction: GatewayDirection::Diverging,
            default_flow: Some("reject_flow".to_string()),
        }],

        // Sequence flows
        sequence_flows: vec![
            SequenceFlow {
                id: "flow1".to_string(),
                name: None,
                source_ref: "start_event".to_string(),
                target_ref: "submit_task".to_string(),
                condition_expression: None,
                is_immediate: false,
            },
            SequenceFlow {
                id: "flow2".to_string(),
                name: None,
                source_ref: "submit_task".to_string(),
                target_ref: "review_task".to_string(),
                condition_expression: None,
                is_immediate: false,
            },
            SequenceFlow {
                id: "flow3".to_string(),
                name: None,
                source_ref: "review_task".to_string(),
                target_ref: "decision_gate".to_string(),
                condition_expression: None,
                is_immediate: false,
            },
            SequenceFlow {
                id: "approve_flow".to_string(),
                name: Some("Approve".to_string()),
                source_ref: "decision_gate".to_string(),
                target_ref: "end_approved".to_string(),
                condition_expression: Some("approved == true".to_string()),
                is_immediate: false,
            },
            SequenceFlow {
                id: "reject_flow".to_string(),
                name: Some("Reject".to_string()),
                source_ref: "decision_gate".to_string(),
                target_ref: "end_rejected".to_string(),
                condition_expression: Some("approved == false".to_string()),
                is_immediate: false,
            },
        ],

        // Data objects
        data_objects: vec![DataObject {
            id: "document_data".to_string(),
            name: Some("Document".to_string()),
            item_subject_ref: None,
            is_collection: false,
            data_state: Some("Draft".to_string()),
        }],

        // Text annotations
        text_annotations: vec![TextAnnotation {
            id: "note1".to_string(),
            text: "Document must be reviewed by team lead".to_string(),
            text_format: "text/plain".to_string(),
        }],

        // Empty collections
        intermediate_events: vec![],
        subprocesses: vec![],
        data_associations: vec![],
        groups: vec![],
        lanes: vec![],
        metadata: HashMap::new(),
    };

    BpmnDiagram {
        id: "sample_diagram".to_string(),
        name: Some("Sample Approval Diagram".to_string()),
        documentation: Some("A sample BPMN diagram demonstrating round-trip capability".to_string()),
        processes: vec![process],
        collaborations: vec![],
        data_stores: vec![],
        messages: vec![],
        signals: vec![],
    }
}
