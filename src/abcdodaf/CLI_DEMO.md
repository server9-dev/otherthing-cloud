# ABCDODAF CLI - Demonstration

This document demonstrates all features of the abcdodaf-cli tool.

## Setup

First, build the CLI tool:

```bash
cargo build --bin abcdodaf-cli
```

## 1. Basic Validation

Validate a workflow file:

```bash
$ ./target/debug/abcdodaf-cli validate test_bpmn_workflow.json
Success: Workflow 'test_bpmn_workflow.json' is valid!
```

Validate with JSON output:

```bash
$ ./target/debug/abcdodaf-cli validate test_bpmn_workflow.json --json
{
  "success": true,
  "file": "test_bpmn_workflow.json",
  "errors": [],
  "summary": {
    "errors": 0,
    "warnings": 0,
    "info": 0
  }
}
```

## 2. Workflow Information

Display basic workflow information:

```bash
$ ./target/debug/abcdodaf-cli info test_bpmn_workflow.json
Workflow Information

File: test_bpmn_workflow.json
Process ID: test_workflow
Process Name: Test Workflow
Version: 1.0.0
Executable: true
Process Type: agent_workflow

Statistics

Total Nodes: 6
Total Flows: 6
Start Events: 1
End Events: 1
Tasks: 3
Gateways: 1

Node Types:
  startEvent: 1
  serviceTask: 3
  exclusiveGateway: 1
  endEvent: 1
```

Display detailed information with node details:

```bash
$ ./target/debug/abcdodaf-cli info test_bpmn_workflow.json --detailed
# ... includes all node details with inputs, outputs, and types
```

Display only statistics:

```bash
$ ./target/debug/abcdodaf-cli info test_bpmn_workflow.json --stats-only
Statistics

Total Nodes: 6
Total Flows: 6
Start Events: 1
End Events: 1
Tasks: 3
Gateways: 1

Node Types:
  startEvent: 1
  serviceTask: 3
  exclusiveGateway: 1
  endEvent: 1
```

## 3. Format Conversion

Convert (pretty-print) a workflow:

```bash
$ ./target/debug/abcdodaf-cli convert test_bpmn_workflow.json output.json --format bpmn-json
Success: Converted 'test_bpmn_workflow.json' to 'output.json'
```

## 4. Auto-Fix Workflow Issues

Create a workflow with issues:

```json
{
  "bpmn_process": {
    "id": "  needs_fixing  ",
    "name": "",
    "version": "1",
    "isExecutable": true,
    "processType": "test"
  },
  "workflow_steps": [
    {
      "id": "  start  ",
      "name": "",
      "type": "startEvent"
    },
    {
      "id": "  end  ",
      "name": "",
      "type": "endEvent"
    }
  ],
  "sequence_flows": [
    {
      "id": "flow1",
      "sourceRef": "  start  ",
      "targetRef": "  end  "
    }
  ]
}
```

Validate to see the issues:

```bash
$ ./target/debug/abcdodaf-cli validate broken.json
[WARNING] [MissingField] Process name is empty (bpmn_process.name)
  Suggestion: Provide a descriptive name for the process
[WARNING] [InvalidValue] Invalid version format: '1' (bpmn_process.version)
  Suggestion: Use semantic versioning format (e.g., '1.0.0')
[WARNING] [MissingField] Node '  start  ' has no name (  start  )
  Suggestion: Provide a descriptive name for the node
[WARNING] [MissingField] Node '  end  ' has no name (  end  )
  Suggestion: Provide a descriptive name for the node

Validation summary: 0 error(s), 4 warning(s), 0 info message(s)
```

Fix the issues in-place:

```bash
$ ./target/debug/abcdodaf-cli fix broken.json --in-place
Success: Applied 6 fix(es) to 'broken.json'

$ ./target/debug/abcdodaf-cli validate broken.json
Success: Workflow 'broken.json' is valid!
```

Preview fixes without modifying the file:

```bash
$ ./target/debug/abcdodaf-cli fix broken.json
# Outputs the fixed JSON to stdout
```

## 5. List and Batch Validate

List all workflow files in a directory:

```bash
$ ./target/debug/abcdodaf-cli list . --validate
Workflow Files

Directory: .
Total Files: 3
Valid: 3
With Errors: 0

✓ ./test_bpmn_workflow.json
✓ ./broken.json
✓ ./output.json
```

List with JSON output:

```bash
$ ./target/debug/abcdodaf-cli list . --validate --json
{
  "directory": ".",
  "total_files": 3,
  "valid_files": 3,
  "files_with_errors": 0,
  "files": [
    {
      "path": "./test_bpmn_workflow.json",
      "status": "valid",
      "errors": 0,
      "warnings": 0
    },
    {
      "path": "./broken.json",
      "status": "valid",
      "errors": 0,
      "warnings": 0
    },
    {
      "path": "./output.json",
      "status": "valid",
      "errors": 0,
      "warnings": 0
    }
  ]
}
```

List only files with errors:

```bash
$ ./target/debug/abcdodaf-cli list . --validate --errors-only
# Only shows files with validation errors
```

## 6. Error Scenarios

### File Not Found

```bash
$ ./target/debug/abcdodaf-cli validate nonexistent.json
Error: Failed to read file 'nonexistent.json': No such file or directory (os error 2)
$ echo $?
2
```

### Invalid JSON

```bash
$ echo "{invalid json}" > invalid.json
$ ./target/debug/abcdodaf-cli validate invalid.json
[ERROR] [InvalidValue] Failed to parse JSON: expected `:` at line 1 column 10
  Suggestion: Ensure the JSON is well-formed and matches the expected schema

Validation summary: 1 error(s), 0 warning(s), 0 info message(s)
$ echo $?
1
```

### Validation Errors

Create a workflow with errors:

```json
{
  "bpmn_process": {
    "id": "test",
    "name": "Test",
    "version": "1.0.0"
  },
  "workflow_steps": [
    {
      "id": "start",
      "name": "Start",
      "type": "startEvent"
    }
  ],
  "sequence_flows": [
    {
      "id": "flow1",
      "sourceRef": "start",
      "targetRef": "nonexistent"
    }
  ]
}
```

```bash
$ ./target/debug/abcdodaf-cli validate broken_refs.json
[ERROR] [BrokenReference] Sequence flow 'flow1' references non-existent target node: 'nonexistent' (flow1)
  Suggestion: Ensure the targetRef matches an existing node ID
[ERROR] [StructuralIssue] Workflow has no end event
  Suggestion: Add at least one endEvent node
[WARNING] [StructuralIssue] Start event 'start' has no outgoing flows (start)
  Suggestion: Start events should have at least one outgoing flow

Validation summary: 2 error(s), 1 warning(s), 0 info message(s)
$ echo $?
1
```

## 7. Advanced Usage

### Show only errors (hide warnings):

```bash
$ ./target/debug/abcdodaf-cli validate workflow.json --errors-only
```

### Exit with success even if there are warnings:

```bash
$ ./target/debug/abcdodaf-cli validate workflow.json --warn-as-success
$ echo $?
0
```

### Disable colored output:

```bash
$ ./target/debug/abcdodaf-cli validate workflow.json --no-color > report.txt
```

### Recursive directory scanning:

```bash
$ ./target/debug/abcdodaf-cli list workflows/ --recursive --validate
```

## 8. Integration Examples

### CI/CD Pipeline

```bash
#!/bin/bash
# Validate all workflows in CI

set -e

echo "Validating workflows..."
./target/release/abcdodaf-cli list workflows/ --validate --recursive

if [ $? -eq 0 ]; then
    echo "✓ All workflows are valid"
    exit 0
else
    echo "✗ Workflow validation failed"
    exit 1
fi
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

WORKFLOWS=$(git diff --cached --name-only --diff-filter=ACM | grep '\.json$' | grep workflow)

if [ -z "$WORKFLOWS" ]; then
    exit 0
fi

echo "Validating modified workflows..."

for file in $WORKFLOWS; do
    ./target/debug/abcdodaf-cli validate "$file" --errors-only
    if [ $? -ne 0 ]; then
        echo "Validation failed for $file"
        exit 1
    fi
done

echo "All workflows valid"
exit 0
```

### Batch Processing Script

```bash
#!/bin/bash
# Fix all workflows in a directory

for file in workflows/**/*.json; do
    echo "Processing $file..."

    # Try to fix
    ./target/debug/abcdodaf-cli fix "$file" --in-place

    # Validate
    if ./target/debug/abcdodaf-cli validate "$file" --errors-only; then
        echo "  ✓ $file is valid"
    else
        echo "  ✗ $file has errors"
    fi
done
```

### JSON Processing with jq

```bash
# Get summary of all workflows
./target/debug/abcdodaf-cli list workflows/ --validate --json | \
  jq '{
    total: .total_files,
    valid: .valid_files,
    invalid: .files_with_errors,
    error_files: [.files[] | select(.status == "invalid") | .path]
  }'
```

Output:
```json
{
  "total": 10,
  "valid": 8,
  "invalid": 2,
  "error_files": [
    "./workflows/broken1.json",
    "./workflows/broken2.json"
  ]
}
```

## 9. Help and Documentation

Get help for any command:

```bash
$ ./target/debug/abcdodaf-cli --help
$ ./target/debug/abcdodaf-cli validate --help
$ ./target/debug/abcdodaf-cli info --help
$ ./target/debug/abcdodaf-cli convert --help
$ ./target/debug/abcdodaf-cli fix --help
$ ./target/debug/abcdodaf-cli list --help
```

Get version information:

```bash
$ ./target/debug/abcdodaf-cli --version
abcdodaf-cli 0.1.0
```

## Exit Codes

- `0`: Success - workflow is valid or operation completed successfully
- `1`: Validation error - workflow has errors or warnings
- `2`: File error - file not found, permission denied, or JSON parse error

## Summary

The abcdodaf-cli tool provides comprehensive functionality for:

1. **Validation**: Ensure BPMN JSON workflows are valid
2. **Information**: Display workflow metadata and statistics
3. **Conversion**: Convert between workflow formats
4. **Auto-fixing**: Automatically fix common issues
5. **Batch processing**: Validate multiple workflows at once
6. **Automation**: JSON output for CI/CD integration
7. **Error reporting**: Detailed, actionable error messages

For more information, see `CLI_README.md`.
