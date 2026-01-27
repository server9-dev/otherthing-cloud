#[cfg(feature = "ui")]
#[test]
fn test_workspace_creation() {
    use abcdodaf::ui::Workspace;

    let mut workspace = Workspace::new();
    assert_eq!(workspace.workflow_count(), 0);

    // Create a new workflow
    let id = workspace.create_new_workflow();
    assert_eq!(workspace.workflow_count(), 1);
    assert_eq!(workspace.active_workflow_id(), Some(id));

    // Get the workflow
    let workflow = workspace.get_workflow(id);
    assert!(workflow.is_some());

    if let Some(doc) = workflow {
        assert_eq!(doc.id, id);
        assert!(!doc.is_modified);
        assert!(doc.file_path.is_none());
    }
}

#[cfg(feature = "ui")]
#[test]
fn test_validation() {
    use abcdodaf::ui::{Validator, EnhancedBpmnNode, BpmnNodeType};
    use egui_snarl::Snarl;

    let mut snarl = Snarl::new();

    // Empty workflow should be valid
    let result = Validator::validate_workflow(&snarl);
    assert!(result.is_valid());

    // Add a task without start/end
    snarl.insert_node(
        egui::pos2(100.0, 100.0),
        EnhancedBpmnNode::user_task("task1", "Test Task")
    );

    let result = Validator::validate_workflow(&snarl);
    assert!(!result.is_valid());
    assert!(result.error_count() > 0);
}
