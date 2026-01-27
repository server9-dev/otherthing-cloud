# Workspace Tests Documentation

## Overview

The `workspace_tests.rs` file contains comprehensive tests for the workspace file loading/saving functionality in the ABCDODAF UI system. This test suite ensures robust file I/O operations and workspace management.

## Test File Location

```
tests/workspace_tests.rs
```

## Running the Tests

To run all workspace tests:

```bash
cargo test --features ui workspace_tests
```

To run a specific test:

```bash
cargo test --features ui workspace_tests::test_load_valid_workflow_file
```

To run with verbose output:

```bash
cargo test --features ui workspace_tests -- --nocapture
```

## Test Coverage

### 1. File Loading Tests (11 tests)

#### `test_load_valid_workflow_file`
- **Purpose**: Verifies that a valid workflow JSON file can be loaded successfully
- **Validates**:
  - Workflow count increments
  - Active workflow is set correctly
  - Workflow metadata is preserved
  - File path is stored
  - Modified flag is false

#### `test_load_missing_file`
- **Purpose**: Ensures proper error handling when attempting to load a non-existent file
- **Validates**:
  - Returns error
  - Error message mentions "Failed to read file"
  - Workspace state remains clean (no partial workflows)

#### `test_load_invalid_json`
- **Purpose**: Tests behavior when loading a file with malformed JSON
- **Validates**:
  - Returns error
  - Error message mentions "Failed to parse JSON"
  - No workflow is created

#### `test_load_wrong_schema`
- **Purpose**: Verifies handling of JSON files that don't match the expected schema
- **Validates**:
  - Returns error
  - Error message mentions parsing failure or missing fields
  - No workflow is created

#### `test_load_already_open_file`
- **Purpose**: Tests that opening the same file twice returns the existing workflow
- **Validates**:
  - Same workflow ID returned
  - No duplicate workflows created
  - File becomes active
  - Workflow count remains 1

#### `test_is_file_open`
- **Purpose**: Verifies the file open state checking functionality
- **Validates**:
  - Returns false for unopened files
  - Returns true for opened files

### 2. File Saving Tests (7 tests)

#### `test_save_new_workflow`
- **Purpose**: Tests saving a newly created workflow to a file
- **Validates**:
  - File is created on disk
  - File path is stored in workflow
  - Modified flag is cleared
  - File content is valid JSON
  - Workflow metadata is preserved

#### `test_save_existing_workflow`
- **Purpose**: Verifies saving an already-loaded workflow
- **Validates**:
  - Modified flag is cleared after save
  - File is updated

#### `test_save_as_new_path`
- **Purpose**: Tests "Save As" functionality with a new file path
- **Validates**:
  - New file is created
  - File path is updated in workflow
  - Modified flag is cleared
  - Original file remains unchanged

#### `test_save_with_no_path`
- **Purpose**: Ensures proper error when trying to save without a file path
- **Validates**:
  - Returns error
  - Error message mentions "No file path set" or "save_workflow_as"

#### `test_save_nonexistent_workflow`
- **Purpose**: Tests error handling when trying to save a non-existent workflow
- **Validates**:
  - Returns error
  - Error message mentions "Workflow not found"

#### `test_save_preserves_data`
- **Purpose**: Verifies that workflow data survives save/load roundtrip
- **Validates**:
  - Custom names are preserved
  - Data integrity is maintained

#### `test_save_to_invalid_directory`
- **Purpose**: Tests error handling when saving to an invalid directory path
- **Validates**:
  - Returns error
  - Error message mentions write failure

### 3. Workspace Management Tests (12 tests)

#### `test_create_new_workflow`
- **Purpose**: Tests creating new empty workflows
- **Validates**:
  - Workflow count increments
  - Each workflow gets unique ID
  - New workflow becomes active

#### `test_open_multiple_workflows`
- **Purpose**: Verifies that multiple workflows can be open simultaneously
- **Validates**:
  - Multiple files can be opened
  - Each gets unique ID
  - Last opened becomes active

#### `test_switch_between_workflows`
- **Purpose**: Tests switching the active workflow
- **Validates**:
  - Active workflow can be changed
  - Invalid IDs are ignored
  - Active state is tracked correctly

#### `test_close_workflow`
- **Purpose**: Verifies workflow closing functionality
- **Validates**:
  - Workflow is removed
  - Count decrements
  - Active workflow updates if needed
  - Other workflows remain unaffected

#### `test_close_nonexistent_workflow`
- **Purpose**: Tests closing a workflow that doesn't exist
- **Validates**:
  - Returns true (no-op)
  - Existing workflows unaffected

#### `test_mark_workflow_as_modified`
- **Purpose**: Tests the modified flag functionality
- **Validates**:
  - Flag starts as false
  - Flag can be set to true
  - Flag is tracked per workflow

#### `test_mark_nonexistent_workflow_modified`
- **Purpose**: Ensures no panic when marking non-existent workflow
- **Validates**:
  - No panic occurs
  - Other workflows unaffected

#### `test_get_workflow_immutable`
- **Purpose**: Tests immutable workflow retrieval
- **Validates**:
  - Returns Some for valid IDs
  - Returns None for invalid IDs
  - Correct workflow is returned

#### `test_get_workflow_mutable`
- **Purpose**: Tests mutable workflow access
- **Validates**:
  - Workflow can be modified
  - Changes are persisted

#### `test_get_active_workflow`
- **Purpose**: Tests retrieving the active workflow
- **Validates**:
  - Returns None when no workflows
  - Returns correct workflow when active
  - Updates when active changes

#### `test_get_active_workflow_mut`
- **Purpose**: Tests mutable access to active workflow
- **Validates**:
  - Active workflow can be modified
  - Changes persist

#### `test_workflow_ids`
- **Purpose**: Tests retrieving all workflow IDs
- **Validates**:
  - Returns empty vec initially
  - Returns all IDs after creation
  - All IDs are present

#### `test_workflows_iterator`
- **Purpose**: Tests the workflows iterator
- **Validates**:
  - Correct count
  - All workflows accessible

### 4. Workflow Document Tests (2 tests)

#### `test_workflow_document_display_name`
- **Purpose**: Tests display name generation
- **Validates**:
  - Uses workflow name when no file path
  - Uses filename when file path exists

#### `test_workflow_document_display_title`
- **Purpose**: Tests display title with modification indicator
- **Validates**:
  - No asterisk when unmodified
  - Asterisk suffix when modified

### 5. Error Handling Tests (5 tests)

#### `test_invalid_file_path`
- **Purpose**: Tests handling of invalid file paths
- **Validates**:
  - Empty paths return error
  - Invalid characters cause error

#### `test_corrupted_json_recovery`
- **Purpose**: Tests handling of truncated/corrupted JSON
- **Validates**:
  - Returns error
  - No partial workflow created

#### `test_empty_file`
- **Purpose**: Tests loading an empty file
- **Validates**:
  - Returns error
  - No workflow created

### 6. Integration Tests (3 tests)

#### `test_complete_workflow_lifecycle`
- **Purpose**: Tests the complete workflow from creation to reload
- **Steps**:
  1. Create new workflow
  2. Modify it
  3. Save it
  4. Modify again
  5. Save existing
  6. Close it
  7. Reopen it
- **Validates**: Full lifecycle works correctly

#### `test_multiple_workflows_independence`
- **Purpose**: Ensures workflows don't interfere with each other
- **Validates**:
  - Independent modification flags
  - Independent saves
  - Closing one doesn't affect others

#### `test_save_load_roundtrip_with_data`
- **Purpose**: Tests data integrity through save/load cycle
- **Validates**:
  - Custom data survives roundtrip
  - Data is identical after reload

#### `test_workspace_default`
- **Purpose**: Tests default workspace construction
- **Validates**:
  - Default trait works correctly
  - Clean initial state

## Test Utilities

### Helper Functions

The test suite includes several helper functions for creating test fixtures:

- **`create_valid_workflow_json()`**: Creates a valid workflow JSON structure
- **`create_invalid_schema_json()`**: Creates JSON with wrong schema
- **`create_test_workflow_file()`**: Creates a temporary file with valid workflow
- **`create_invalid_json_file()`**: Creates a file with invalid JSON
- **`create_wrong_schema_file()`**: Creates a file with wrong schema JSON

### Test Isolation

All tests use `tempfile` crate for creating temporary files and directories, ensuring:
- Tests don't interfere with each other
- No leftover files after test completion
- Safe parallel test execution

## Test Statistics

- **Total Tests**: 40
- **File Loading**: 11 tests
- **File Saving**: 7 tests
- **Workspace Management**: 12 tests
- **Workflow Document**: 2 tests
- **Error Handling**: 5 tests
- **Integration**: 3 tests

## Dependencies

The test suite requires:
- `tempfile` crate (dev-dependency)
- `serde_json` for JSON manipulation
- `ui` feature flag enabled

## Error Scenarios Covered

1. **File System Errors**:
   - Missing files
   - Invalid paths
   - Directory doesn't exist
   - Permission issues (platform-specific)

2. **Data Format Errors**:
   - Invalid JSON syntax
   - Wrong schema
   - Corrupted/truncated data
   - Empty files

3. **State Errors**:
   - Non-existent workflow IDs
   - Missing file paths
   - Already-open files

## Code Coverage

The tests achieve high coverage of the `workspace.rs` module:
- ✅ All public methods
- ✅ Error paths
- ✅ Edge cases
- ✅ State transitions
- ✅ Data integrity

## Future Enhancements

Potential areas for additional testing:
1. Concurrent access to workspace
2. File watching/auto-reload
3. Undo/redo integration
4. Large file handling
5. File format migration
6. Backup/recovery mechanisms

## Running Tests in CI/CD

Example GitHub Actions configuration:

```yaml
- name: Run workspace tests
  run: |
    cargo test --features ui workspace_tests --verbose
```

## Troubleshooting

If tests fail:

1. **Check feature flag**: Ensure `--features ui` is specified
2. **Check dependencies**: Run `cargo update` for latest tempfile
3. **Check disk space**: Tests create temporary files
4. **Check permissions**: Ensure write access to temp directory
5. **Check isolation**: Run with `--test-threads=1` if needed

## Maintenance Notes

When modifying the workspace module:
1. Run the full test suite before committing
2. Add tests for new functionality
3. Update existing tests if behavior changes
4. Keep test documentation in sync with code
