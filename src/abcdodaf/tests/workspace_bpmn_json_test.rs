//! Test workspace integration with BPMN JSON loader
//!
//! This test verifies that the workspace can properly load BPMN JSON files
//! with workflow_steps and convert them to the internal BpmnDiagram format.

#[cfg(feature = "ui")]
mod workspace_bpmn_json_tests {
    use abcdodaf::ui::workspace::Workspace;

    #[test]
    fn test_open_bpmn_json_workflow() {
        // Create a test BPMN JSON file
        let test_file_content = r#"{
  "bpmn_process": {
    "id": "test_process",
    "name": "Test Workflow",
    "version": "1.0.0",
    "isExecutable": true,
    "processType": "standard"
  },
  "workflow_steps": [
    {
      "id": "start_event",
      "name": "Start",
      "type": "startEvent"
    },
    {
      "id": "task_1",
      "name": "Task 1",
      "type": "serviceTask",
      "taskType": "service"
    },
    {
      "id": "end_event",
      "name": "End",
      "type": "endEvent"
    }
  ],
  "sequence_flows": [
    {
      "id": "flow_1",
      "sourceRef": "start_event",
      "targetRef": "task_1"
    },
    {
      "id": "flow_2",
      "sourceRef": "task_1",
      "targetRef": "end_event"
    }
  ]
}"#;

        // Write to temp file
        let temp_dir = std::env::temp_dir();
        let test_file_path = temp_dir.join("test_bpmn_json_workflow.json");
        std::fs::write(&test_file_path, test_file_content).expect("Failed to write test file");

        // Create workspace and open the file
        let mut workspace = Workspace::new();
        let result = workspace.open_workflow(test_file_path.clone());

        assert!(result.is_ok(), "Failed to open BPMN JSON workflow: {:?}", result.err());

        let workflow_id = result.unwrap();

        // Verify the workflow was loaded correctly
        let workflow = workspace.get_workflow(workflow_id).expect("Workflow not found");

        assert_eq!(workflow.name, "Test Workflow");
        assert!(workflow.file_path.is_some());
        assert_eq!(workflow.file_path.as_ref().unwrap(), &test_file_path);
        assert!(!workflow.is_modified);

        // Verify the diagram was created
        assert_eq!(workflow.diagram.name, Some("Test Workflow".to_string()));
        assert_eq!(workflow.diagram.processes.len(), 1);

        let process = &workflow.diagram.processes[0];
        assert_eq!(process.start_events.len(), 1);
        assert_eq!(process.end_events.len(), 1);
        assert_eq!(process.tasks.len(), 1);
        assert_eq!(process.sequence_flows.len(), 2);

        // Verify the snarl was created
        assert_eq!(workflow.snarl.nodes().count(), 3); // start, task, end

        // Clean up
        std::fs::remove_file(test_file_path).ok();
    }

    #[test]
    fn test_open_bpmn_diagram_format() {
        // Create a test BpmnDiagram format file
        let test_file_content = r#"{
  "version": "2.0",
  "diagram": {
    "id": "test_diagram",
    "name": "Test Diagram",
    "documentation": null,
    "processes": [
      {
        "id": "process_1",
        "name": "Test Process",
        "documentation": null,
        "is_executable": true,
        "process_type": "None",
        "start_events": [
          {
            "id": "start_1",
            "name": "Start",
            "documentation": null,
            "event_definition": null,
            "is_interrupting": true
          }
        ],
        "end_events": [
          {
            "id": "end_1",
            "name": "End",
            "documentation": null,
            "event_definition": null
          }
        ],
        "intermediate_events": [],
        "tasks": [],
        "subprocesses": [],
        "gateways": [],
        "sequence_flows": [
          {
            "id": "flow_1",
            "name": null,
            "source_ref": "start_1",
            "target_ref": "end_1",
            "condition_expression": null,
            "is_immediate": true
          }
        ],
        "data_objects": [],
        "data_associations": [],
        "text_annotations": [],
        "groups": [],
        "lanes": [],
        "metadata": {}
      }
    ],
    "collaborations": [],
    "data_stores": [],
    "messages": [],
    "signals": []
  }
}"#;

        // Write to temp file
        let temp_dir = std::env::temp_dir();
        let test_file_path = temp_dir.join("test_bpmn_diagram_format.json");
        std::fs::write(&test_file_path, test_file_content).expect("Failed to write test file");

        // Create workspace and open the file
        let mut workspace = Workspace::new();
        let result = workspace.open_workflow(test_file_path.clone());

        assert!(result.is_ok(), "Failed to open BpmnDiagram format: {:?}", result.err());

        let workflow_id = result.unwrap();

        // Verify the workflow was loaded correctly
        let workflow = workspace.get_workflow(workflow_id).expect("Workflow not found");

        assert_eq!(workflow.name, "Test Diagram");
        assert_eq!(workflow.diagram.name, Some("Test Diagram".to_string()));

        // Clean up
        std::fs::remove_file(test_file_path).ok();
    }

    #[test]
    fn test_open_snarl_format_legacy() {
        // Create a test Snarl format file (legacy)
        // Note: Snarl serialization uses a map format, not arrays
        let test_file_content = r#"{
  "version": "1.0",
  "name": "Legacy Snarl Workflow",
  "snarl": {
    "nodes": {},
    "wires": []
  }
}"#;

        // Write to temp file
        let temp_dir = std::env::temp_dir();
        let test_file_path = temp_dir.join("test_snarl_legacy.json");
        std::fs::write(&test_file_path, test_file_content).expect("Failed to write test file");

        // Create workspace and open the file
        let mut workspace = Workspace::new();
        let result = workspace.open_workflow(test_file_path.clone());

        assert!(result.is_ok(), "Failed to open Snarl format: {:?}", result.err());

        let workflow_id = result.unwrap();

        // Verify the workflow was loaded correctly
        let workflow = workspace.get_workflow(workflow_id).expect("Workflow not found");

        assert_eq!(workflow.name, "Legacy Snarl Workflow");

        // Clean up
        std::fs::remove_file(test_file_path).ok();
    }

    #[test]
    fn test_save_and_reload_workflow() {
        // Create a test BPMN JSON file
        let test_file_content = r#"{
  "bpmn_process": {
    "id": "roundtrip_process",
    "name": "Roundtrip Test",
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
      "id": "task",
      "name": "Task",
      "type": "userTask"
    },
    {
      "id": "end",
      "name": "End",
      "type": "endEvent"
    }
  ],
  "sequence_flows": [
    {
      "id": "f1",
      "sourceRef": "start",
      "targetRef": "task"
    },
    {
      "id": "f2",
      "sourceRef": "task",
      "targetRef": "end"
    }
  ]
}"#;

        // Write to temp file
        let temp_dir = std::env::temp_dir();
        let test_file_path = temp_dir.join("test_roundtrip.json");
        std::fs::write(&test_file_path, test_file_content).expect("Failed to write test file");

        // Create workspace and open the file
        let mut workspace = Workspace::new();
        let workflow_id = workspace
            .open_workflow(test_file_path.clone())
            .expect("Failed to open workflow");

        // Save the workflow (which should convert to BpmnDiagram format)
        let save_path = temp_dir.join("test_roundtrip_saved.json");
        workspace
            .save_workflow_as(workflow_id, save_path.clone())
            .expect("Failed to save workflow");

        // Reload the saved workflow
        let mut workspace2 = Workspace::new();
        let workflow_id2 = workspace2
            .open_workflow(save_path.clone())
            .expect("Failed to reload saved workflow");

        let workflow = workspace2.get_workflow(workflow_id2).expect("Workflow not found");

        assert_eq!(workflow.name, "Roundtrip Test");
        assert_eq!(workflow.diagram.processes.len(), 1);

        let process = &workflow.diagram.processes[0];
        assert_eq!(process.start_events.len(), 1);
        assert_eq!(process.end_events.len(), 1);
        assert_eq!(process.tasks.len(), 1);

        // Clean up
        std::fs::remove_file(test_file_path).ok();
        std::fs::remove_file(save_path).ok();
    }

    #[test]
    fn test_invalid_format_detection() {
        // Create an invalid JSON file
        let test_file_content = r#"{
  "unknown_field": "unknown_value",
  "not_a_workflow": true
}"#;

        // Write to temp file
        let temp_dir = std::env::temp_dir();
        let test_file_path = temp_dir.join("test_invalid.json");
        std::fs::write(&test_file_path, test_file_content).expect("Failed to write test file");

        // Create workspace and try to open the file
        let mut workspace = Workspace::new();
        let result = workspace.open_workflow(test_file_path.clone());

        assert!(result.is_err(), "Should have failed to open invalid format");

        let error_message = result.unwrap_err();
        assert!(error_message.contains("Unknown file format"));

        // Clean up
        std::fs::remove_file(test_file_path).ok();
    }

    #[test]
    fn test_complex_bpmn_json_workflow() {
        // Test with gateways and multiple paths
        let test_file_content = r#"{
  "bpmn_process": {
    "id": "complex_process",
    "name": "Complex Test Workflow",
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
      "id": "task1",
      "name": "Initial Task",
      "type": "serviceTask"
    },
    {
      "id": "gateway1",
      "name": "Decision",
      "type": "exclusiveGateway"
    },
    {
      "id": "task2a",
      "name": "Path A",
      "type": "userTask"
    },
    {
      "id": "task2b",
      "name": "Path B",
      "type": "userTask"
    },
    {
      "id": "gateway2",
      "name": "Merge",
      "type": "exclusiveGateway"
    },
    {
      "id": "end",
      "name": "End",
      "type": "endEvent"
    }
  ],
  "sequence_flows": [
    {"id": "f1", "sourceRef": "start", "targetRef": "task1"},
    {"id": "f2", "sourceRef": "task1", "targetRef": "gateway1"},
    {"id": "f3", "sourceRef": "gateway1", "targetRef": "task2a", "name": "A"},
    {"id": "f4", "sourceRef": "gateway1", "targetRef": "task2b", "name": "B"},
    {"id": "f5", "sourceRef": "task2a", "targetRef": "gateway2"},
    {"id": "f6", "sourceRef": "task2b", "targetRef": "gateway2"},
    {"id": "f7", "sourceRef": "gateway2", "targetRef": "end"}
  ]
}"#;

        // Write to temp file
        let temp_dir = std::env::temp_dir();
        let test_file_path = temp_dir.join("test_complex_bpmn.json");
        std::fs::write(&test_file_path, test_file_content).expect("Failed to write test file");

        // Create workspace and open the file
        let mut workspace = Workspace::new();
        let result = workspace.open_workflow(test_file_path.clone());

        assert!(result.is_ok(), "Failed to open complex BPMN JSON: {:?}", result.err());

        let workflow_id = result.unwrap();
        let workflow = workspace.get_workflow(workflow_id).expect("Workflow not found");

        // Verify structure
        assert_eq!(workflow.snarl.nodes().count(), 7);

        let process = &workflow.diagram.processes[0];
        assert_eq!(process.gateways.len(), 2);
        assert_eq!(process.tasks.len(), 3);
        assert_eq!(process.sequence_flows.len(), 7);

        // Clean up
        std::fs::remove_file(test_file_path).ok();
    }
}
