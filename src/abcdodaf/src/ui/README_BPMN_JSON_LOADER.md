# BPMN JSON Loader Module

Quick reference for using the BPMN JSON to Snarl converter.

## Quick Start

```rust
use abcdodaf::ui::bpmn_json_loader::load_workflow_file;

// Load a BPMN JSON workflow file
let workflow = load_workflow_file("workflows/my_workflow.json")?;

// Access the converted data
println!("Loaded: {}", workflow.name);
println!("Nodes: {}", workflow.snarl.node_ids().count());
println!("Connections: {}", workflow.snarl.wires().count());
```

## What Does This Module Do?

Converts BPMN workflow files from this format:

```json
{
  "bpmn_process": { ... },
  "workflow_steps": [ ... ],
  "sequence_flows": [ ... ]
}
```

To the Snarl format that the workspace expects:

```json
{
  "version": "1.0",
  "name": "Workflow Name",
  "snarl": { "nodes": {...}, "wires": [...] }
}
```

## Common Use Cases

### Load from File

```rust
let workflow = load_workflow_file("path/to/workflow.json")?;
```

### Load from String

```rust
use abcdodaf::ui::bpmn_json_loader::convert_bpmn_json;

let json = r#"{ ... }"#;
let workflow = convert_bpmn_json(json)?;
```

### Custom Layout

```rust
use abcdodaf::ui::bpmn_json_loader::{BpmnJsonConverter, LayoutConfig};

let layout = LayoutConfig {
    start_x: 100.0,
    start_y: 100.0,
    horizontal_spacing: 250.0,
    vertical_spacing: 200.0,
    gateway_offset_x: 150.0,
};

let converter = BpmnJsonConverter::with_layout(layout);
let workflow = converter.load_from_file("workflow.json")?;
```

### Integration with Workspace

```rust
use abcdodaf::ui::{Workspace, WorkflowDocument, load_workflow_file};
use std::path::PathBuf;

let mut workspace = Workspace::new();
let path = PathBuf::from("workflow.json");
let workflow = load_workflow_file(&path)?;

let id = workspace.next_id;
workspace.next_id += 1;

let mut doc = WorkflowDocument::new(id, workflow.name);
doc.snarl = workflow.snarl;
doc.file_path = Some(path);
doc.is_modified = false;

workspace.workflows.insert(id, doc);
workspace.active_workflow = Some(id);
```

## Supported Node Types

| BPMN Type | Description |
|-----------|-------------|
| `startEvent` | Workflow start point |
| `endEvent` | Workflow end point |
| `serviceTask` | Automated task |
| `userTask` | User interaction task |
| `scriptTask` | Script execution task |
| `manualTask` | Manual task |
| `businessRuleTask` | Business rules task |
| `exclusiveGateway` | XOR decision point |
| `parallelGateway` | Parallel split/join |
| `inclusiveGateway` | OR gateway |
| `intermediateCatchEvent` | Catching event |
| `intermediateThrowEvent` | Throwing event |

## Example: Complete BPMN JSON

```json
{
  "bpmn_process": {
    "id": "example_process",
    "name": "Example Workflow",
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
      "name": "Process Data",
      "type": "serviceTask",
      "taskType": "research",
      "implementation": "process_data_method",
      "inputs": ["raw_data"],
      "outputs": ["processed_data"]
    },
    {
      "id": "end_event",
      "name": "Complete",
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
}
```

## Error Handling

```rust
match load_workflow_file("workflow.json") {
    Ok(workflow) => {
        println!("Success! Loaded: {}", workflow.name);
    }
    Err(e) => {
        eprintln!("Error: {}", e);
        // Common errors:
        // - "Failed to read file: ..."
        // - "Failed to parse JSON: ..."
        // - "Source node not found: ..."
        // - "Target node not found: ..."
    }
}
```

## Run the Example

```bash
cargo run --example bpmn_json_converter_example --features ui
```

## Full Documentation

See [../../docs/BPMN_JSON_LOADER.md](../../docs/BPMN_JSON_LOADER.md) for complete documentation.

## Testing

```bash
cargo test --features ui bpmn_json_loader
```

## Module Files

- `bpmn_json_loader.rs` - Main converter implementation
- `../../examples/bpmn_json_converter_example.rs` - Usage examples
- `../../tests/bpmn_json_loader_test.rs` - Integration tests
- `../../docs/BPMN_JSON_LOADER.md` - Full documentation
