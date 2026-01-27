//! BPMN JSON Validation Example
//!
//! Demonstrates how to use the standalone JSON validation functionality.
//! This example shows validation of both valid and invalid workflows.

use abcdodaf::bpmn::{validate_bpmn_json, ErrorSeverity, ValidationSummary};

fn main() {
    println!("=== BPMN JSON Validation Examples ===\n");

    // Example 1: Valid workflow
    example_valid_workflow();
    println!("\n{}\n", "=".repeat(80));

    // Example 2: Missing start event
    example_missing_start_event();
    println!("\n{}\n", "=".repeat(80));

    // Example 3: Broken references
    example_broken_references();
    println!("\n{}\n", "=".repeat(80));

    // Example 4: Duplicate IDs
    example_duplicate_ids();
    println!("\n{}\n", "=".repeat(80));

    // Example 5: Invalid node types
    example_invalid_node_types();
    println!("\n{}\n", "=".repeat(80));

    // Example 6: Complex workflow with warnings
    example_complex_workflow();
}

fn example_valid_workflow() {
    println!(">>> Example 1: Valid Workflow");
    println!("Testing a properly structured BPMN workflow.\n");

    let json = r#"{
        "bpmn_process": {
            "id": "order_processing",
            "name": "Order Processing Workflow",
            "version": "1.0.0",
            "isExecutable": true,
            "processType": "business_process"
        },
        "workflow_steps": [
            {
                "id": "start_order",
                "name": "Receive Order",
                "type": "startEvent",
                "eventType": "message"
            },
            {
                "id": "validate_order",
                "name": "Validate Order",
                "type": "serviceTask",
                "taskType": "testing"
            },
            {
                "id": "check_inventory",
                "name": "Check Inventory",
                "type": "serviceTask",
                "taskType": "research"
            },
            {
                "id": "end_order",
                "name": "Order Complete",
                "type": "endEvent"
            }
        ],
        "sequence_flows": [
            {
                "id": "flow1",
                "sourceRef": "start_order",
                "targetRef": "validate_order"
            },
            {
                "id": "flow2",
                "sourceRef": "validate_order",
                "targetRef": "check_inventory"
            },
            {
                "id": "flow3",
                "sourceRef": "check_inventory",
                "targetRef": "end_order"
            }
        ]
    }"#;

    match validate_bpmn_json(json) {
        Ok(()) => {
            println!("✓ Validation passed!");
        }
        Err(errors) => {
            print_validation_errors(&errors);
        }
    }
}

fn example_missing_start_event() {
    println!(">>> Example 2: Missing Start Event");
    println!("Testing a workflow without a start event.\n");

    let json = r#"{
        "bpmn_process": {
            "id": "incomplete_workflow",
            "name": "Incomplete Workflow",
            "version": "1.0.0"
        },
        "workflow_steps": [
            {
                "id": "task1",
                "name": "Some Task",
                "type": "serviceTask",
                "taskType": "code_generation"
            },
            {
                "id": "end",
                "name": "End",
                "type": "endEvent"
            }
        ],
        "sequence_flows": [
            {
                "id": "flow1",
                "sourceRef": "task1",
                "targetRef": "end"
            }
        ]
    }"#;

    match validate_bpmn_json(json) {
        Ok(()) => {
            println!("✓ Validation passed!");
        }
        Err(errors) => {
            print_validation_errors(&errors);
        }
    }
}

fn example_broken_references() {
    println!(">>> Example 3: Broken References");
    println!("Testing a workflow with invalid sequence flow references.\n");

    let json = r#"{
        "bpmn_process": {
            "id": "broken_refs",
            "name": "Broken References",
            "version": "1.0.0"
        },
        "workflow_steps": [
            {
                "id": "start",
                "name": "Start",
                "type": "startEvent"
            },
            {
                "id": "task1",
                "name": "Task 1",
                "type": "serviceTask",
                "taskType": "design"
            }
        ],
        "sequence_flows": [
            {
                "id": "flow1",
                "sourceRef": "start",
                "targetRef": "nonexistent_task"
            },
            {
                "id": "flow2",
                "sourceRef": "task1",
                "targetRef": "missing_end"
            }
        ]
    }"#;

    match validate_bpmn_json(json) {
        Ok(()) => {
            println!("✓ Validation passed!");
        }
        Err(errors) => {
            print_validation_errors(&errors);
        }
    }
}

fn example_duplicate_ids() {
    println!(">>> Example 4: Duplicate IDs");
    println!("Testing a workflow with duplicate node IDs.\n");

    let json = r#"{
        "bpmn_process": {
            "id": "duplicate_test",
            "name": "Duplicate IDs Test",
            "version": "1.0.0"
        },
        "workflow_steps": [
            {
                "id": "task1",
                "name": "First Task",
                "type": "startEvent"
            },
            {
                "id": "task1",
                "name": "Duplicate Task",
                "type": "endEvent"
            }
        ],
        "sequence_flows": []
    }"#;

    match validate_bpmn_json(json) {
        Ok(()) => {
            println!("✓ Validation passed!");
        }
        Err(errors) => {
            print_validation_errors(&errors);
        }
    }
}

fn example_invalid_node_types() {
    println!(">>> Example 5: Invalid Node Types");
    println!("Testing a workflow with invalid BPMN node types.\n");

    let json = r#"{
        "bpmn_process": {
            "id": "invalid_types",
            "name": "Invalid Types Test",
            "version": "1.0.0"
        },
        "workflow_steps": [
            {
                "id": "start",
                "name": "Start",
                "type": "customStartEvent"
            },
            {
                "id": "task1",
                "name": "Task",
                "type": "magicTask"
            }
        ],
        "sequence_flows": []
    }"#;

    match validate_bpmn_json(json) {
        Ok(()) => {
            println!("✓ Validation passed!");
        }
        Err(errors) => {
            print_validation_errors(&errors);
        }
    }
}

fn example_complex_workflow() {
    println!(">>> Example 6: Complex Workflow with Warnings");
    println!("Testing a complex workflow that has warnings but is still valid.\n");

    let json = r#"{
        "bpmn_process": {
            "id": "agent_research_workflow",
            "name": "AI Agent Research Workflow",
            "version": "2.1.3",
            "isExecutable": true,
            "processType": "agent_workflow"
        },
        "workflow_steps": [
            {
                "id": "start",
                "name": "Start Research",
                "type": "startEvent"
            },
            {
                "id": "gateway_split",
                "name": "Split Research Tasks",
                "type": "parallelGateway"
            },
            {
                "id": "research_sources",
                "name": "Research Sources",
                "type": "serviceTask",
                "taskType": "research",
                "duration_minutes": 30
            },
            {
                "id": "analyze_data",
                "name": "Analyze Data",
                "type": "serviceTask",
                "taskType": "code_generation",
                "duration_minutes": 45
            },
            {
                "id": "gateway_merge",
                "name": "Merge Results",
                "type": "parallelGateway"
            },
            {
                "id": "generate_report",
                "name": "Generate Report",
                "type": "serviceTask",
                "taskType": "documentation",
                "duration_minutes": 20
            },
            {
                "id": "end",
                "name": "Research Complete",
                "type": "endEvent"
            }
        ],
        "sequence_flows": [
            {
                "id": "flow1",
                "sourceRef": "start",
                "targetRef": "gateway_split"
            },
            {
                "id": "flow2",
                "sourceRef": "gateway_split",
                "targetRef": "research_sources"
            },
            {
                "id": "flow3",
                "sourceRef": "gateway_split",
                "targetRef": "analyze_data"
            },
            {
                "id": "flow4",
                "sourceRef": "research_sources",
                "targetRef": "gateway_merge"
            },
            {
                "id": "flow5",
                "sourceRef": "analyze_data",
                "targetRef": "gateway_merge"
            },
            {
                "id": "flow6",
                "sourceRef": "gateway_merge",
                "targetRef": "generate_report"
            },
            {
                "id": "flow7",
                "sourceRef": "generate_report",
                "targetRef": "end"
            }
        ]
    }"#;

    match validate_bpmn_json(json) {
        Ok(()) => {
            println!("✓ Validation passed!");
        }
        Err(errors) => {
            print_validation_errors(&errors);
        }
    }
}

fn print_validation_errors(errors: &[abcdodaf::bpmn::ValidationError]) {
    let summary = ValidationSummary::from_errors(errors);

    println!("{}", summary.format());
    println!();

    for (i, error) in errors.iter().enumerate() {
        let severity_str = match error.severity {
            ErrorSeverity::Error => "ERROR",
            ErrorSeverity::Warning => "WARNING",
            ErrorSeverity::Info => "INFO",
        };

        let category_str = format!("[{:?}]", error.category);

        println!("{}. {} {} {}", i + 1, severity_str, category_str, error.message);

        if let Some(ref context) = error.context {
            println!("   Context: {}", context);
        }

        if let Some(ref suggestion) = error.suggestion {
            println!("   Suggestion: {}", suggestion);
        }

        println!();
    }
}
