# BPMN JSON Validation

Comprehensive JSON validation for BPMN workflow files, implemented in the core library and usable standalone (UI-independent).

## Overview

The BPMN JSON validation module provides robust validation capabilities for BPMN workflow JSON files. It validates:

- Required fields (bpmn_process, workflow_steps, sequence_flows)
- Valid node types according to BPMN 2.0 specification
- Node ID uniqueness
- Sequence flow references (sourceRef/targetRef point to existing nodes)
- Required fields for each node type
- Structural integrity (start/end events, connectivity)
- Gateway connection patterns

## Module Location

- **Core Module**: `src/bpmn/json_validation.rs`
- **JSON Format**: `src/bpmn/json_format.rs`
- **Public API**: Exported through `src/bpmn/mod.rs`

## Usage

### Basic Validation

```rust
use abcdodaf::bpmn::validate_bpmn_json;

let json = r#"{
    "bpmn_process": {
        "id": "my_process",
        "name": "My Process",
        "version": "1.0.0"
    },
    "workflow_steps": [
        {
            "id": "start",
            "name": "Start",
            "type": "startEvent"
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
            "sourceRef": "start",
            "targetRef": "end"
        }
    ]
}"#;

match validate_bpmn_json(json) {
    Ok(()) => println!("Valid workflow!"),
    Err(errors) => {
        for error in errors {
            println!("{}: {}", error.severity, error.message);
        }
    }
}
```

### Validating Parsed Workflows

```rust
use abcdodaf::bpmn::{validate_bpmn_workflow, BpmnJsonWorkflow};

let workflow: BpmnJsonWorkflow = serde_json::from_str(json)?;
match validate_bpmn_workflow(&workflow) {
    Ok(()) => println!("Valid!"),
    Err(errors) => handle_errors(errors),
}
```

### Error Handling

```rust
use abcdodaf::bpmn::{ValidationError, ErrorSeverity, ErrorCategory, ValidationSummary};

fn handle_validation_errors(errors: Vec<ValidationError>) {
    let summary = ValidationSummary::from_errors(&errors);

    println!("{}", summary.format());
    println!("Has errors: {}", summary.has_errors());

    for error in errors {
        match error.severity {
            ErrorSeverity::Error => {
                // Critical errors that prevent loading
                eprintln!("ERROR: {}", error.message);
            }
            ErrorSeverity::Warning => {
                // Non-critical issues
                println!("WARNING: {}", error.message);
            }
            ErrorSeverity::Info => {
                // Best practice suggestions
                println!("INFO: {}", error.message);
            }
        }

        if let Some(context) = error.context {
            println!("  Context: {}", context);
        }

        if let Some(suggestion) = error.suggestion {
            println!("  Suggestion: {}", suggestion);
        }
    }
}
```

## Validation Categories

### 1. Missing Fields
Detects required fields that are missing or empty:
- Process ID
- Process name
- Node IDs
- Node names
- Flow references

### 2. Invalid Values
Validates field values against BPMN 2.0 specification:
- Invalid node types
- Invalid event types
- Invalid task types
- Invalid version format

### 3. Broken References
Ensures all references point to existing elements:
- sourceRef in sequence flows
- targetRef in sequence flows

### 4. Duplicate IDs
Ensures uniqueness of identifiers:
- Node IDs must be unique
- Flow IDs must be unique

### 5. Structural Issues
Validates workflow structure:
- Must have at least one start event
- Must have at least one end event
- Start events should not have incoming flows
- End events should not have outgoing flows
- Nodes should be connected (not orphaned)

### 6. Type-Specific Validation
Validates requirements for specific node types:
- Service tasks should have taskType
- Script tasks should have implementation
- Events may have eventType

## Valid Node Types

According to BPMN 2.0 specification:

### Events
- `startEvent`
- `endEvent`
- `intermediateCatchEvent`
- `intermediateThrowEvent`
- `boundaryEvent`

### Tasks
- `task` (abstract/generic)
- `serviceTask`
- `userTask`
- `manualTask`
- `scriptTask`
- `businessRuleTask`
- `sendTask`
- `receiveTask`

### Gateways
- `exclusiveGateway` (XOR)
- `parallelGateway` (AND)
- `inclusiveGateway` (OR)
- `eventBasedGateway`
- `complexGateway`

### Subprocesses
- `subProcess`
- `callActivity`
- `transaction`
- `adHocSubProcess`

## Valid Task Types

For `serviceTask` nodes:
- `research`
- `design`
- `code_generation`
- `testing`
- `documentation`
- `integration`
- `debugging`
- `optimization`
- `deployment`
- `monitoring`

## Valid Event Types

For event nodes:
- `none` (default)
- `message`
- `timer`
- `error`
- `escalation`
- `cancel`
- `compensation`
- `conditional`
- `link`
- `signal`
- `terminate`
- `multiple`
- `parallelMultiple`

## Error Severity Levels

### Error (Critical)
Prevents the workflow from being loaded or executed:
- Missing start/end events
- Broken references
- Duplicate IDs
- Invalid node types

### Warning (Non-Critical)
Indicates potential issues but workflow can still be loaded:
- Missing optional fields
- Empty names
- Disconnected nodes
- Unknown task/event types

### Info (Best Practices)
Suggestions for improvement:
- Version format recommendations
- Gateway usage patterns
- Structural improvements

## Example Output

```
Validation summary: 2 error(s), 1 warning(s), 0 info message(s)

1. ERROR [BrokenReference] Sequence flow 'flow1' references non-existent target node: 'missing_task'
   Context: flow1
   Suggestion: Ensure the targetRef matches an existing node ID

2. ERROR [StructuralIssue] Workflow has no end event
   Suggestion: Add at least one endEvent node

3. WARNING [MissingField] Service task 'task1' has no taskType specified
   Context: task1
   Suggestion: Consider specifying a taskType: research, design, code_generation, ...
```

## Running the Example

```bash
cargo run --example json_validation_example
```

This example demonstrates:
1. Valid workflow validation
2. Missing start event detection
3. Broken reference detection
4. Duplicate ID detection
5. Invalid node type detection
6. Complex workflow with parallel gateways

## Testing

Run the validation tests:

```bash
cargo test bpmn::json_validation
cargo test bpmn::json_format
```

Test coverage includes:
- Valid workflow parsing
- Missing required fields
- Duplicate IDs
- Broken references
- Invalid node types
- Invalid version formats
- Gateway validation
- Event/task type validation

## Integration with UI

The UI module (`src/ui/bpmn_json_loader.rs`) now uses the core JSON format structures from `src/bpmn/json_format.rs`, ensuring consistency between validation and loading.

## CLI Integration

The validation module is designed to be used in CLI tools without any UI dependencies. See Task #4 for CLI tool implementation with clap.

## Future Enhancements

Potential improvements:
1. Custom validation rules via trait
2. Validation profiles (strict/lenient)
3. JSON schema generation
4. Validation for DoDAF metadata
5. Cross-reference validation (data objects, messages)
6. Cycle detection in flows
7. Reachability analysis
