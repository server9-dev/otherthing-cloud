//! Integration tests for BPMN JSON loader
//!
//! Tests the conversion from BPMN JSON format to Snarl format.

#[cfg(feature = "ui")]
mod tests {
    use abcdodaf::ui::bpmn_json_loader::{convert_bpmn_json, BpmnJsonConverter, LayoutConfig};

    #[test]
    fn test_basic_workflow_conversion() {
        let json = r#"{
            "bpmn_process": {
                "id": "test_process",
                "name": "Test Process",
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
                    "id": "task1",
                    "name": "Task 1",
                    "type": "serviceTask",
                    "taskType": "research"
                },
                {
                    "id": "end_event",
                    "name": "End",
                    "type": "endEvent"
                }
            ],
            "sequence_flows": [
                {
                    "id": "flow1",
                    "sourceRef": "start_event",
                    "targetRef": "task1"
                },
                {
                    "id": "flow2",
                    "sourceRef": "task1",
                    "targetRef": "end_event"
                }
            ]
        }"#;

        let result = convert_bpmn_json(json);
        assert!(result.is_ok(), "Failed to convert: {:?}", result.err());

        let workflow_file = result.unwrap();
        assert_eq!(workflow_file.name, "Test Process");
        assert_eq!(workflow_file.version, "1.0");

        // Verify we have 3 nodes
        let node_count = workflow_file.snarl.node_ids().count();
        assert_eq!(node_count, 3, "Expected 3 nodes, got {}", node_count);

        // Verify we have 2 connections
        let wire_count = workflow_file.snarl.wires().count();
        assert_eq!(wire_count, 2, "Expected 2 wires, got {}", wire_count);
    }

    #[test]
    fn test_gateway_conversion() {
        let json = r#"{
            "bpmn_process": {
                "id": "gateway_test",
                "name": "Gateway Test",
                "version": "1.0.0",
                "isExecutable": true
            },
            "workflow_steps": [
                {
                    "id": "start",
                    "name": "Start",
                    "type": "startEvent"
                },
                {
                    "id": "exclusive_gw",
                    "name": "Decision",
                    "type": "exclusiveGateway"
                },
                {
                    "id": "parallel_gw",
                    "name": "Parallel Split",
                    "type": "parallelGateway"
                },
                {
                    "id": "inclusive_gw",
                    "name": "Inclusive Join",
                    "type": "inclusiveGateway"
                }
            ],
            "sequence_flows": []
        }"#;

        let result = convert_bpmn_json(json);
        assert!(result.is_ok());

        let workflow_file = result.unwrap();
        assert_eq!(workflow_file.snarl.node_ids().count(), 4);
    }

    #[test]
    fn test_all_task_types() {
        let json = r#"{
            "bpmn_process": {
                "id": "task_types",
                "name": "Task Types Test",
                "version": "1.0.0",
                "isExecutable": true
            },
            "workflow_steps": [
                {"id": "user_task", "name": "User Task", "type": "userTask"},
                {"id": "service_task", "name": "Service Task", "type": "serviceTask"},
                {"id": "script_task", "name": "Script Task", "type": "scriptTask"},
                {"id": "manual_task", "name": "Manual Task", "type": "manualTask"},
                {"id": "business_rule", "name": "Business Rule", "type": "businessRuleTask"},
                {"id": "generic_task", "name": "Generic Task", "type": "task"}
            ],
            "sequence_flows": []
        }"#;

        let result = convert_bpmn_json(json);
        assert!(result.is_ok());

        let workflow_file = result.unwrap();
        let nodes: Vec<_> = workflow_file.snarl.node_ids().collect();
        assert_eq!(nodes.len(), 6);

        // Verify node types
        let node_types: Vec<_> = nodes.iter().map(|(_, node)| node.type_name()).collect();

        assert!(node_types.contains(&"User Task"));
        assert!(node_types.contains(&"Service Task"));
        assert!(node_types.contains(&"Script Task"));
        assert!(node_types.contains(&"Manual Task"));
        assert!(node_types.contains(&"Business Rule Task"));
        assert!(node_types.contains(&"Task"));
    }

    #[test]
    fn test_custom_layout() {
        let layout = LayoutConfig {
            start_x: 50.0,
            start_y: 50.0,
            horizontal_spacing: 300.0,
            vertical_spacing: 200.0,
            gateway_offset_x: 100.0,
        };

        let converter = BpmnJsonConverter::with_layout(layout);

        let json = r#"{
            "bpmn_process": {
                "id": "layout_test",
                "name": "Layout Test",
                "version": "1.0.0",
                "isExecutable": true
            },
            "workflow_steps": [
                {"id": "start", "name": "Start", "type": "startEvent"},
                {"id": "task1", "name": "Task 1", "type": "serviceTask"},
                {"id": "end", "name": "End", "type": "endEvent"}
            ],
            "sequence_flows": [
                {"id": "f1", "sourceRef": "start", "targetRef": "task1"},
                {"id": "f2", "sourceRef": "task1", "targetRef": "end"}
            ]
        }"#;

        let result = converter.load_from_string(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_missing_source_node() {
        let json = r#"{
            "bpmn_process": {
                "id": "error_test",
                "name": "Error Test",
                "version": "1.0.0",
                "isExecutable": true
            },
            "workflow_steps": [
                {"id": "start", "name": "Start", "type": "startEvent"}
            ],
            "sequence_flows": [
                {"id": "f1", "sourceRef": "nonexistent", "targetRef": "start"}
            ]
        }"#;

        let result = convert_bpmn_json(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Source node not found"));
    }

    #[test]
    fn test_missing_target_node() {
        let json = r#"{
            "bpmn_process": {
                "id": "error_test",
                "name": "Error Test",
                "version": "1.0.0",
                "isExecutable": true
            },
            "workflow_steps": [
                {"id": "start", "name": "Start", "type": "startEvent"}
            ],
            "sequence_flows": [
                {"id": "f1", "sourceRef": "start", "targetRef": "nonexistent"}
            ]
        }"#;

        let result = convert_bpmn_json(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Target node not found"));
    }

    #[test]
    fn test_complex_workflow() {
        let json = r#"{
            "bpmn_process": {
                "id": "complex_process",
                "name": "Complex Process",
                "version": "1.0.0",
                "isExecutable": true
            },
            "workflow_steps": [
                {"id": "start", "name": "Start", "type": "startEvent"},
                {"id": "task1", "name": "Initial Task", "type": "serviceTask"},
                {"id": "gateway", "name": "Decision", "type": "exclusiveGateway"},
                {"id": "task2a", "name": "Path A", "type": "userTask"},
                {"id": "task2b", "name": "Path B", "type": "userTask"},
                {"id": "join", "name": "Join", "type": "exclusiveGateway"},
                {"id": "task3", "name": "Final Task", "type": "serviceTask"},
                {"id": "end", "name": "End", "type": "endEvent"}
            ],
            "sequence_flows": [
                {"id": "f1", "sourceRef": "start", "targetRef": "task1"},
                {"id": "f2", "sourceRef": "task1", "targetRef": "gateway"},
                {"id": "f3", "sourceRef": "gateway", "targetRef": "task2a", "condition": "a"},
                {"id": "f4", "sourceRef": "gateway", "targetRef": "task2b", "condition": "b"},
                {"id": "f5", "sourceRef": "task2a", "targetRef": "join"},
                {"id": "f6", "sourceRef": "task2b", "targetRef": "join"},
                {"id": "f7", "sourceRef": "join", "targetRef": "task3"},
                {"id": "f8", "sourceRef": "task3", "targetRef": "end"}
            ]
        }"#;

        let result = convert_bpmn_json(json);
        assert!(result.is_ok());

        let workflow_file = result.unwrap();
        assert_eq!(workflow_file.snarl.node_ids().count(), 8);
        assert_eq!(workflow_file.snarl.wires().count(), 8);
    }

    #[test]
    fn test_minimal_process_info() {
        let json = r#"{
            "bpmn_process": {
                "id": "minimal",
                "name": "Minimal Process"
            },
            "workflow_steps": [
                {"id": "start", "name": "Start", "type": "startEvent"}
            ],
            "sequence_flows": []
        }"#;

        let result = convert_bpmn_json(json);
        assert!(result.is_ok());

        let workflow_file = result.unwrap();
        assert_eq!(workflow_file.name, "Minimal Process");
    }

    #[test]
    fn test_node_properties() {
        let json = r#"{
            "bpmn_process": {
                "id": "props_test",
                "name": "Properties Test",
                "version": "1.0.0",
                "isExecutable": true
            },
            "workflow_steps": [
                {
                    "id": "task1",
                    "name": "Task with Properties",
                    "type": "serviceTask",
                    "taskType": "research",
                    "implementation": "research_method",
                    "inputs": ["input1", "input2"],
                    "outputs": ["output1"],
                    "duration_minutes": 30
                }
            ],
            "sequence_flows": []
        }"#;

        let result = convert_bpmn_json(json);
        assert!(result.is_ok());

        let workflow_file = result.unwrap();
        let nodes: Vec<_> = workflow_file.snarl.node_ids().collect();
        assert_eq!(nodes.len(), 1);

        let (_, node) = nodes[0];
        assert_eq!(node.name(), "Task with Properties");
        assert_eq!(node.type_name(), "Service Task");
    }
}
