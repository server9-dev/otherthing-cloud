//! Comprehensive tests for workspace file loading/saving functionality
//!
//! Tests cover:
//! - File loading (valid, missing, invalid JSON, wrong schema, already open)
//! - File saving (new, existing, save as, no path)
//! - Workspace management (create, open multiple, switch, close, mark modified)
//! - Error handling (invalid paths, corrupted JSON)

#[cfg(feature = "ui")]
mod workspace_tests {
    use abcdodaf::ui::Workspace;
    use serde_json::json;
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::{NamedTempFile, TempDir};

    // ============================================================================
    // Helper Functions
    // ============================================================================

    /// Create a valid workflow JSON for testing
    fn create_valid_workflow_json() -> serde_json::Value {
        json!({
            "version": "1.0",
            "name": "Test Workflow",
            "snarl": {
                "nodes": [],
                "wires": []
            }
        })
    }

    /// Create a workflow JSON with wrong schema
    fn create_invalid_schema_json() -> serde_json::Value {
        json!({
            "invalid_field": "test",
            "missing_required_fields": true
        })
    }

    /// Create a workflow file with valid content
    fn create_test_workflow_file() -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = serde_json::to_string_pretty(&create_valid_workflow_json()).unwrap();
        temp_file.write_all(content.as_bytes()).unwrap();
        temp_file.flush().unwrap();
        temp_file
    }

    /// Create a workflow file with invalid JSON
    fn create_invalid_json_file() -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"{ invalid json }").unwrap();
        temp_file.flush().unwrap();
        temp_file
    }

    /// Create a workflow file with wrong schema
    fn create_wrong_schema_file() -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = serde_json::to_string_pretty(&create_invalid_schema_json()).unwrap();
        temp_file.write_all(content.as_bytes()).unwrap();
        temp_file.flush().unwrap();
        temp_file
    }

    // ============================================================================
    // File Loading Tests
    // ============================================================================

    #[test]
    fn test_load_valid_workflow_file() {
        let mut workspace = Workspace::new();
        let temp_file = create_test_workflow_file();
        let path = temp_file.path().to_path_buf();

        let result = workspace.open_workflow(path);
        assert!(result.is_ok(), "Should successfully load valid workflow file");

        let id = result.unwrap();
        assert_eq!(workspace.workflow_count(), 1);
        assert_eq!(workspace.active_workflow_id(), Some(id));

        let doc = workspace.get_workflow(id).unwrap();
        assert_eq!(doc.name, "Test Workflow");
        assert!(!doc.is_modified);
        assert!(doc.file_path.is_some());
    }

    #[test]
    fn test_load_missing_file() {
        let mut workspace = Workspace::new();
        let non_existent_path = PathBuf::from("/tmp/non_existent_workflow_12345.json");

        let result = workspace.open_workflow(non_existent_path);
        assert!(result.is_err(), "Should fail to load missing file");

        let error_msg = result.unwrap_err();
        assert!(
            error_msg.contains("Failed to read file"),
            "Error message should mention file reading failure: {}",
            error_msg
        );

        assert_eq!(workspace.workflow_count(), 0);
    }

    #[test]
    fn test_load_invalid_json() {
        let mut workspace = Workspace::new();
        let temp_file = create_invalid_json_file();
        let path = temp_file.path().to_path_buf();

        let result = workspace.open_workflow(path);
        assert!(result.is_err(), "Should fail to load invalid JSON");

        let error_msg = result.unwrap_err();
        assert!(
            error_msg.contains("Failed to parse JSON"),
            "Error message should mention JSON parsing failure: {}",
            error_msg
        );

        assert_eq!(workspace.workflow_count(), 0);
    }

    #[test]
    fn test_load_wrong_schema() {
        let mut workspace = Workspace::new();
        let temp_file = create_wrong_schema_file();
        let path = temp_file.path().to_path_buf();

        let result = workspace.open_workflow(path);
        assert!(result.is_err(), "Should fail to load workflow with wrong schema");

        let error_msg = result.unwrap_err();
        assert!(
            error_msg.contains("Failed to parse JSON") || error_msg.contains("missing field"),
            "Error message should mention parsing or missing field: {}",
            error_msg
        );

        assert_eq!(workspace.workflow_count(), 0);
    }

    #[test]
    fn test_load_already_open_file() {
        let mut workspace = Workspace::new();
        let temp_file = create_test_workflow_file();
        let path = temp_file.path().to_path_buf();

        // Open the file first time
        let id1 = workspace.open_workflow(path.clone()).unwrap();
        assert_eq!(workspace.workflow_count(), 1);

        // Open the same file again
        let id2 = workspace.open_workflow(path.clone()).unwrap();
        assert_eq!(
            id1, id2,
            "Opening the same file should return the same workflow ID"
        );
        assert_eq!(
            workspace.workflow_count(),
            1,
            "Should not create duplicate workflow"
        );
        assert_eq!(
            workspace.active_workflow_id(),
            Some(id2),
            "Should make the workflow active"
        );
    }

    #[test]
    fn test_is_file_open() {
        let mut workspace = Workspace::new();
        let temp_file = create_test_workflow_file();
        let path = temp_file.path().to_path_buf();

        assert!(!workspace.is_file_open(&path));

        workspace.open_workflow(path.clone()).unwrap();
        assert!(workspace.is_file_open(&path));
    }

    // ============================================================================
    // File Saving Tests
    // ============================================================================

    #[test]
    fn test_save_new_workflow() {
        let mut workspace = Workspace::new();
        let temp_dir = TempDir::new().unwrap();
        let save_path = temp_dir.path().join("new_workflow.json");

        // Create a new workflow
        let id = workspace.create_new_workflow();

        // Initially no file path
        let doc = workspace.get_workflow(id).unwrap();
        assert!(doc.file_path.is_none());

        // Save as new file
        let result = workspace.save_workflow_as(id, save_path.clone());
        assert!(result.is_ok(), "Should successfully save new workflow");

        // Verify file exists and has correct path
        assert!(save_path.exists());
        let doc = workspace.get_workflow(id).unwrap();
        assert_eq!(doc.file_path, Some(save_path.clone()));
        assert!(!doc.is_modified);

        // Verify file content
        let content = fs::read_to_string(&save_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["version"], "1.0");
        assert!(parsed["name"].as_str().unwrap().starts_with("Untitled"));
    }

    #[test]
    fn test_save_existing_workflow() {
        let mut workspace = Workspace::new();
        let temp_file = create_test_workflow_file();
        let path = temp_file.path().to_path_buf();

        // Open existing workflow
        let id = workspace.open_workflow(path.clone()).unwrap();

        // Mark as modified
        workspace.mark_modified(id);
        assert!(workspace.get_workflow(id).unwrap().is_modified);

        // Save
        let result = workspace.save_workflow(id);
        assert!(result.is_ok(), "Should successfully save existing workflow");

        // Verify modified flag is cleared
        assert!(!workspace.get_workflow(id).unwrap().is_modified);
    }

    #[test]
    fn test_save_as_new_path() {
        let mut workspace = Workspace::new();
        let temp_file = create_test_workflow_file();
        let original_path = temp_file.path().to_path_buf();

        let temp_dir = TempDir::new().unwrap();
        let new_path = temp_dir.path().join("saved_as.json");

        // Open existing workflow
        let id = workspace.open_workflow(original_path.clone()).unwrap();

        // Save as new path
        let result = workspace.save_workflow_as(id, new_path.clone());
        assert!(result.is_ok(), "Should successfully save as new path");

        // Verify new file exists
        assert!(new_path.exists());

        // Verify path updated
        let doc = workspace.get_workflow(id).unwrap();
        assert_eq!(doc.file_path, Some(new_path));
        assert!(!doc.is_modified);
    }

    #[test]
    fn test_save_with_no_path() {
        let mut workspace = Workspace::new();

        // Create a new workflow without saving
        let id = workspace.create_new_workflow();

        // Try to save without setting path
        let result = workspace.save_workflow(id);
        assert!(result.is_err(), "Should fail to save without file path");

        let error_msg = result.unwrap_err();
        assert!(
            error_msg.contains("No file path set") || error_msg.contains("save_workflow_as"),
            "Error message should mention missing path: {}",
            error_msg
        );
    }

    #[test]
    fn test_save_nonexistent_workflow() {
        let mut workspace = Workspace::new();
        let temp_dir = TempDir::new().unwrap();
        let save_path = temp_dir.path().join("test.json");

        // Try to save non-existent workflow
        let result = workspace.save_workflow_as(999, save_path);
        assert!(result.is_err(), "Should fail to save non-existent workflow");

        let error_msg = result.unwrap_err();
        assert!(
            error_msg.contains("Workflow not found"),
            "Error message should mention workflow not found: {}",
            error_msg
        );
    }

    #[test]
    fn test_save_preserves_data() {
        let mut workspace = Workspace::new();
        let temp_dir = TempDir::new().unwrap();
        let save_path = temp_dir.path().join("preserved_data.json");

        // Create workflow with a specific name
        let id = workspace.create_new_workflow();
        {
            let doc = workspace.get_workflow_mut(id).unwrap();
            doc.name = "Custom Workflow Name".to_string();
        }

        // Save
        workspace.save_workflow_as(id, save_path.clone()).unwrap();

        // Open in new workspace
        let mut new_workspace = Workspace::new();
        let loaded_id = new_workspace.open_workflow(save_path).unwrap();

        let loaded_doc = new_workspace.get_workflow(loaded_id).unwrap();
        assert_eq!(loaded_doc.name, "Custom Workflow Name");
    }

    // ============================================================================
    // Workspace Management Tests
    // ============================================================================

    #[test]
    fn test_create_new_workflow() {
        let mut workspace = Workspace::new();
        assert_eq!(workspace.workflow_count(), 0);
        assert_eq!(workspace.active_workflow_id(), None);

        let id1 = workspace.create_new_workflow();
        assert_eq!(workspace.workflow_count(), 1);
        assert_eq!(workspace.active_workflow_id(), Some(id1));

        let id2 = workspace.create_new_workflow();
        assert_eq!(workspace.workflow_count(), 2);
        assert_eq!(workspace.active_workflow_id(), Some(id2));
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_open_multiple_workflows() {
        let mut workspace = Workspace::new();

        let temp_file1 = create_test_workflow_file();
        let temp_file2 = create_test_workflow_file();
        let path1 = temp_file1.path().to_path_buf();
        let path2 = temp_file2.path().to_path_buf();

        let id1 = workspace.open_workflow(path1).unwrap();
        let id2 = workspace.open_workflow(path2).unwrap();

        assert_eq!(workspace.workflow_count(), 2);
        assert_ne!(id1, id2);
        assert_eq!(workspace.active_workflow_id(), Some(id2));
    }

    #[test]
    fn test_switch_between_workflows() {
        let mut workspace = Workspace::new();

        let id1 = workspace.create_new_workflow();
        let id2 = workspace.create_new_workflow();
        let id3 = workspace.create_new_workflow();

        assert_eq!(workspace.active_workflow_id(), Some(id3));

        workspace.set_active_workflow(id1);
        assert_eq!(workspace.active_workflow_id(), Some(id1));

        workspace.set_active_workflow(id2);
        assert_eq!(workspace.active_workflow_id(), Some(id2));

        // Try to set non-existent workflow as active
        workspace.set_active_workflow(999);
        assert_eq!(
            workspace.active_workflow_id(),
            Some(id2),
            "Active workflow should not change for invalid ID"
        );
    }

    #[test]
    fn test_close_workflow() {
        let mut workspace = Workspace::new();

        let id1 = workspace.create_new_workflow();
        let id2 = workspace.create_new_workflow();
        let id3 = workspace.create_new_workflow();

        assert_eq!(workspace.workflow_count(), 3);
        assert_eq!(workspace.active_workflow_id(), Some(id3));

        // Close the active workflow
        let closed = workspace.close_workflow(id3);
        assert!(closed);
        assert_eq!(workspace.workflow_count(), 2);
        assert_ne!(workspace.active_workflow_id(), Some(id3));

        // Close a non-active workflow
        workspace.set_active_workflow(id1);
        let closed = workspace.close_workflow(id2);
        assert!(closed);
        assert_eq!(workspace.workflow_count(), 1);
        assert_eq!(workspace.active_workflow_id(), Some(id1));

        // Close the last workflow
        let closed = workspace.close_workflow(id1);
        assert!(closed);
        assert_eq!(workspace.workflow_count(), 0);
        assert_eq!(workspace.active_workflow_id(), None);
    }

    #[test]
    fn test_close_nonexistent_workflow() {
        let mut workspace = Workspace::new();
        let id = workspace.create_new_workflow();

        let closed = workspace.close_workflow(999);
        assert!(closed, "Closing non-existent workflow should return true");
        assert_eq!(workspace.workflow_count(), 1, "Should not affect existing workflows");
        assert_eq!(workspace.active_workflow_id(), Some(id));
    }

    #[test]
    fn test_mark_workflow_as_modified() {
        let mut workspace = Workspace::new();
        let id = workspace.create_new_workflow();

        let doc = workspace.get_workflow(id).unwrap();
        assert!(!doc.is_modified);

        workspace.mark_modified(id);

        let doc = workspace.get_workflow(id).unwrap();
        assert!(doc.is_modified);
    }

    #[test]
    fn test_mark_nonexistent_workflow_modified() {
        let mut workspace = Workspace::new();
        let id = workspace.create_new_workflow();

        // Should not panic
        workspace.mark_modified(999);

        // Original workflow should be unaffected
        let doc = workspace.get_workflow(id).unwrap();
        assert!(!doc.is_modified);
    }

    #[test]
    fn test_get_workflow_immutable() {
        let mut workspace = Workspace::new();
        let id1 = workspace.create_new_workflow();
        let id2 = workspace.create_new_workflow();

        let doc1 = workspace.get_workflow(id1);
        assert!(doc1.is_some());
        assert_eq!(doc1.unwrap().id, id1);

        let doc2 = workspace.get_workflow(id2);
        assert!(doc2.is_some());
        assert_eq!(doc2.unwrap().id, id2);

        let doc_none = workspace.get_workflow(999);
        assert!(doc_none.is_none());
    }

    #[test]
    fn test_get_workflow_mutable() {
        let mut workspace = Workspace::new();
        let id = workspace.create_new_workflow();

        {
            let doc = workspace.get_workflow_mut(id).unwrap();
            doc.name = "Modified Name".to_string();
        }

        let doc = workspace.get_workflow(id).unwrap();
        assert_eq!(doc.name, "Modified Name");
    }

    #[test]
    fn test_get_active_workflow() {
        let mut workspace = Workspace::new();

        assert!(workspace.get_active_workflow().is_none());

        let id1 = workspace.create_new_workflow();
        let active = workspace.get_active_workflow();
        assert!(active.is_some());
        assert_eq!(active.unwrap().id, id1);

        let id2 = workspace.create_new_workflow();
        let active = workspace.get_active_workflow();
        assert!(active.is_some());
        assert_eq!(active.unwrap().id, id2);
    }

    #[test]
    fn test_get_active_workflow_mut() {
        let mut workspace = Workspace::new();
        let id = workspace.create_new_workflow();

        {
            let active = workspace.get_active_workflow_mut().unwrap();
            active.name = "Active Modified".to_string();
        }

        let doc = workspace.get_workflow(id).unwrap();
        assert_eq!(doc.name, "Active Modified");
    }

    #[test]
    fn test_workflow_ids() {
        let mut workspace = Workspace::new();

        let ids = workspace.workflow_ids();
        assert_eq!(ids.len(), 0);

        let id1 = workspace.create_new_workflow();
        let id2 = workspace.create_new_workflow();
        let id3 = workspace.create_new_workflow();

        let ids = workspace.workflow_ids();
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
        assert!(ids.contains(&id3));
    }

    #[test]
    fn test_workflows_iterator() {
        let mut workspace = Workspace::new();

        workspace.create_new_workflow();
        workspace.create_new_workflow();
        workspace.create_new_workflow();

        let count = workspace.workflows().count();
        assert_eq!(count, 3);

        // Verify all workflows are accessible
        for doc in workspace.workflows() {
            assert!(doc.id > 0);
        }
    }

    // ============================================================================
    // Workflow Document Tests
    // ============================================================================

    #[test]
    fn test_workflow_document_display_name() {
        let mut workspace = Workspace::new();
        let id = workspace.create_new_workflow();

        // Without file path, use name
        let doc = workspace.get_workflow(id).unwrap();
        let display_name = doc.display_name();
        assert!(display_name.starts_with("Untitled"));

        // With file path, use filename
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("my_workflow.json");
        workspace.save_workflow_as(id, path).unwrap();

        let doc = workspace.get_workflow(id).unwrap();
        let display_name = doc.display_name();
        assert_eq!(display_name, "my_workflow.json");
    }

    #[test]
    fn test_workflow_document_display_title() {
        let mut workspace = Workspace::new();
        let id = workspace.create_new_workflow();

        // Unmodified
        let doc = workspace.get_workflow(id).unwrap();
        let title = doc.display_title();
        assert!(!title.contains('*'));

        // Modified
        workspace.mark_modified(id);
        let doc = workspace.get_workflow(id).unwrap();
        let title = doc.display_title();
        assert!(title.ends_with('*'));
    }

    // ============================================================================
    // Error Handling Tests
    // ============================================================================

    #[test]
    fn test_invalid_file_path() {
        let mut workspace = Workspace::new();

        // Empty path component
        let invalid_path = PathBuf::from("");
        let result = workspace.open_workflow(invalid_path);
        assert!(result.is_err());

        // Path with invalid characters (platform-specific)
        #[cfg(unix)]
        {
            let invalid_path = PathBuf::from("/tmp/\0invalid.json");
            let result = workspace.open_workflow(invalid_path);
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_save_to_invalid_directory() {
        let mut workspace = Workspace::new();
        let id = workspace.create_new_workflow();

        let invalid_path = PathBuf::from("/nonexistent_directory_12345/workflow.json");
        let result = workspace.save_workflow_as(id, invalid_path);
        assert!(result.is_err());

        let error_msg = result.unwrap_err();
        assert!(
            error_msg.contains("Failed to write file")
            || error_msg.contains("No such file or directory"),
            "Error message should mention write failure: {}",
            error_msg
        );
    }

    #[test]
    fn test_corrupted_json_recovery() {
        let mut workspace = Workspace::new();

        // Create a file with truncated JSON
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"{\"version\": \"1.0\", \"name\": \"Test").unwrap();
        temp_file.flush().unwrap();
        let path = temp_file.path().to_path_buf();

        let result = workspace.open_workflow(path);
        assert!(result.is_err());
        assert_eq!(workspace.workflow_count(), 0, "Should not create partial workflow");
    }

    #[test]
    fn test_empty_file() {
        let mut workspace = Workspace::new();

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.flush().unwrap();
        let path = temp_file.path().to_path_buf();

        let result = workspace.open_workflow(path);
        assert!(result.is_err());
        assert_eq!(workspace.workflow_count(), 0);
    }

    // ============================================================================
    // Integration Tests
    // ============================================================================

    #[test]
    fn test_complete_workflow_lifecycle() {
        let mut workspace = Workspace::new();
        let temp_dir = TempDir::new().unwrap();

        // 1. Create new workflow
        let id = workspace.create_new_workflow();
        assert_eq!(workspace.workflow_count(), 1);
        assert!(!workspace.get_workflow(id).unwrap().is_modified);

        // 2. Modify workflow
        workspace.mark_modified(id);
        assert!(workspace.get_workflow(id).unwrap().is_modified);

        // 3. Save workflow
        let save_path = temp_dir.path().join("lifecycle_test.json");
        workspace.save_workflow_as(id, save_path.clone()).unwrap();
        assert!(!workspace.get_workflow(id).unwrap().is_modified);
        assert!(save_path.exists());

        // 4. Modify again
        workspace.mark_modified(id);

        // 5. Save existing
        workspace.save_workflow(id).unwrap();
        assert!(!workspace.get_workflow(id).unwrap().is_modified);

        // 6. Close workflow
        workspace.close_workflow(id);
        assert_eq!(workspace.workflow_count(), 0);

        // 7. Reopen workflow
        let new_id = workspace.open_workflow(save_path).unwrap();
        assert_eq!(workspace.workflow_count(), 1);
        assert!(!workspace.get_workflow(new_id).unwrap().is_modified);
    }

    #[test]
    fn test_multiple_workflows_independence() {
        let mut workspace = Workspace::new();
        let temp_dir = TempDir::new().unwrap();

        // Create three workflows
        let id1 = workspace.create_new_workflow();
        let id2 = workspace.create_new_workflow();
        let id3 = workspace.create_new_workflow();

        // Modify only id2
        workspace.mark_modified(id2);

        // Save all
        workspace.save_workflow_as(id1, temp_dir.path().join("w1.json")).unwrap();
        workspace.save_workflow_as(id2, temp_dir.path().join("w2.json")).unwrap();
        workspace.save_workflow_as(id3, temp_dir.path().join("w3.json")).unwrap();

        // Verify states
        assert!(!workspace.get_workflow(id1).unwrap().is_modified);
        assert!(!workspace.get_workflow(id2).unwrap().is_modified); // Cleared by save
        assert!(!workspace.get_workflow(id3).unwrap().is_modified);

        // Close id2
        workspace.close_workflow(id2);
        assert_eq!(workspace.workflow_count(), 2);

        // Verify id1 and id3 still exist
        assert!(workspace.get_workflow(id1).is_some());
        assert!(workspace.get_workflow(id3).is_some());
    }

    #[test]
    fn test_save_load_roundtrip_with_data() {
        let mut workspace = Workspace::new();
        let temp_dir = TempDir::new().unwrap();
        let save_path = temp_dir.path().join("roundtrip.json");

        // Create and customize workflow
        let id = workspace.create_new_workflow();
        {
            let doc = workspace.get_workflow_mut(id).unwrap();
            doc.name = "Roundtrip Test".to_string();
            // In a real scenario, we would add nodes to the snarl here
        }

        // Save
        workspace.save_workflow_as(id, save_path.clone()).unwrap();

        // Create new workspace and load
        let mut new_workspace = Workspace::new();
        let loaded_id = new_workspace.open_workflow(save_path).unwrap();

        // Verify data integrity
        let loaded_doc = new_workspace.get_workflow(loaded_id).unwrap();
        assert_eq!(loaded_doc.name, "Roundtrip Test");
        assert!(!loaded_doc.is_modified);
    }

    #[test]
    fn test_workspace_default() {
        let workspace = Workspace::default();
        assert_eq!(workspace.workflow_count(), 0);
        assert_eq!(workspace.active_workflow_id(), None);
    }
}
