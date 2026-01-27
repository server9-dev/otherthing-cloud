//! Test loading a real BPMN JSON workflow file

#[cfg(feature = "ui")]
#[test]
fn test_load_real_bpmn_json_workflow() {
    use abcdodaf::ui::workspace::Workspace;
    use std::path::PathBuf;

    // Path to the real test BPMN JSON file
    let test_file_path = PathBuf::from("test_bpmn_json_workflow.json");

    // Check if file exists
    if !test_file_path.exists() {
        eprintln!("Test file not found: {:?}", test_file_path);
        eprintln!("This test requires test_bpmn_json_workflow.json to exist");
        return;
    }

    // Create workspace and open the file
    let mut workspace = Workspace::new();
    let result = workspace.open_workflow(test_file_path.clone());

    assert!(result.is_ok(), "Failed to open real BPMN JSON workflow: {:?}", result.err());

    let workflow_id = result.unwrap();

    // Verify the workflow was loaded correctly
    let workflow = workspace.get_workflow(workflow_id).expect("Workflow not found");

    println!("Loaded workflow: {}", workflow.name);
    println!("  - Processes: {}", workflow.diagram.processes.len());

    if !workflow.diagram.processes.is_empty() {
        let process = &workflow.diagram.processes[0];
        println!("  - Start events: {}", process.start_events.len());
        println!("  - End events: {}", process.end_events.len());
        println!("  - Tasks: {}", process.tasks.len());
        println!("  - Gateways: {}", process.gateways.len());
        println!("  - Sequence flows: {}", process.sequence_flows.len());
    }

    println!("  - Snarl nodes: {}", workflow.snarl.nodes().count());

    // Verify expected structure
    assert_eq!(workflow.name, "Test BPMN JSON Workflow");
    assert_eq!(workflow.diagram.processes.len(), 1);

    let process = &workflow.diagram.processes[0];
    assert_eq!(process.start_events.len(), 1, "Should have 1 start event");
    assert_eq!(process.end_events.len(), 1, "Should have 1 end event");
    assert_eq!(process.tasks.len(), 3, "Should have 3 tasks (1 service + 2 user)");
    assert_eq!(process.gateways.len(), 1, "Should have 1 gateway");
    assert_eq!(process.sequence_flows.len(), 6, "Should have 6 sequence flows");

    // Verify node count matches (1 start + 3 tasks + 1 gateway + 1 end = 6)
    assert_eq!(workflow.snarl.nodes().count(), 6, "Should have 6 total nodes in Snarl");

    println!("\n✓ All validations passed!");
}
