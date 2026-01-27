# BPMN JSON to Snarl Converter

This module provides functionality to load BPMN workflow files (in JSON format) and convert them to Snarl format for use with the visual editor workspace.

## Overview

The BPMN JSON loader bridges the gap between two workflow file formats:

**Input Format** (BPMN JSON):
```json
{
  "bpmn_process": {...},
  "workflow_steps": [...],
  "sequence_flows": [...]
}
```

**Output Format** (Workspace/Snarl):
```json
{
  "version": "1.0",
  "name": "Workflow Name",
  "snarl": {
    "nodes": {...},
    "wires": [...]
  }
}
```

## Features

- ✅ Parses BPMN JSON workflow files
- ✅ Converts workflow steps to EnhancedBpmnNode instances
- ✅ Creates Snarl graph with proper connections
- ✅ Automatic node layout (vertical flow with configurable spacing)
- ✅ Supports all major BPMN element types
- ✅ Returns WorkflowFile struct compatible with Workspace

## Usage

### Basic Usage

```rust
use abcdodaf::ui::bpmn_json_loader::{load_workflow_file, BpmnJsonConverter};

// Simple: Load from file
let workflow = load_workflow_file("path/to/workflow.json")?;

// Advanced: Use converter with custom layout
let converter = BpmnJsonConverter::new();
let workflow = converter.load_from_file("path/to/workflow.json")?;
```

### Loading from String

```rust
use abcdodaf::ui::bpmn_json_loader::convert_bpmn_json;

let json = r#"{
    "bpmn_process": {
        "id": "my_process",
        "name": "My Workflow",
        "version": "1.0.0",
        "isExecutable": true
    },
    "workflow_steps": [
        {"id": "start", "name": "Start", "type": "startEvent"},
        {"id": "task1", "name": "Do Work", "type": "serviceTask"},
        {"id": "end", "name": "End", "type": "endEvent"}
    ],
    "sequence_flows": [
        {"id": "f1", "sourceRef": "start", "targetRef": "task1"},
        {"id": "f2", "sourceRef": "task1", "targetRef": "end"}
    ]
}"#;

let workflow = convert_bpmn_json(json)?;
```

### Integration with Workspace

```rust
use abcdodaf::ui::{Workspace, load_workflow_file};

let mut workspace = Workspace::new();

// Load a BPMN JSON file
let workflow = load_workflow_file("workflow.json")?;

// Create a new workflow document
let id = workspace.next_id;
workspace.next_id += 1;

let mut doc = WorkflowDocument::new(id, workflow.name);
doc.snarl = workflow.snarl;
doc.file_path = Some(PathBuf::from("workflow.json"));

workspace.workflows.insert(id, doc);
workspace.active_workflow = Some(id);
```

### Custom Layout Configuration

```rust
use abcdodaf::ui::bpmn_json_loader::{BpmnJsonConverter, LayoutConfig};

let layout = LayoutConfig {
    start_x: 50.0,
    start_y: 50.0,
    horizontal_spacing: 250.0,
    vertical_spacing: 200.0,
    gateway_offset_x: 150.0,
};

let converter = BpmnJsonConverter::with_layout(layout);
let workflow = converter.load_from_file("workflow.json")?;
```

## Supported Node Types

The converter supports the following BPMN element types:

| BPMN Type | JSON Type | EnhancedBpmnNode |
|-----------|-----------|------------------|
| Start Event | `startEvent` | `StartEvent` |
| End Event | `endEvent` | `EndEvent` |
| Intermediate Event | `intermediateCatchEvent` / `intermediateThrowEvent` | `IntermediateEvent` |
| Service Task | `serviceTask` | `Task(Service)` |
| User Task | `userTask` | `Task(User)` |
| Script Task | `scriptTask` | `Task(Script)` |
| Manual Task | `manualTask` | `Task(Manual)` |
| Business Rule Task | `businessRuleTask` | `Task(BusinessRule)` |
| Generic Task | `task` | `Task(Abstract)` |
| Exclusive Gateway | `exclusiveGateway` | `Gateway(Exclusive)` |
| Parallel Gateway | `parallelGateway` | `Gateway(Parallel)` |
| Inclusive Gateway | `inclusiveGateway` | `Gateway(Inclusive)` |

## BPMN JSON Format

### Complete Example

```json
{
  "bpmn_process": {
    "id": "process_id",
    "name": "Process Name",
    "version": "1.0.0",
    "isExecutable": true,
    "processType": "agent_workflow"
  },
  "workflow_steps": [
    {
      "id": "start_event",
      "name": "Start",
      "type": "startEvent",
      "eventType": "none"
    },
    {
      "id": "service_task_1",
      "name": "Process Data",
      "type": "serviceTask",
      "taskType": "research",
      "implementation": "process_data_method",
      "inputs": ["raw_data"],
      "outputs": ["processed_data"],
      "duration_minutes": 5
    },
    {
      "id": "exclusive_gateway",
      "name": "Check Result",
      "type": "exclusiveGateway"
    },
    {
      "id": "end_event",
      "name": "Complete",
      "type": "endEvent",
      "eventType": "none"
    }
  ],
  "sequence_flows": [
    {
      "id": "flow_1",
      "sourceRef": "start_event",
      "targetRef": "service_task_1"
    },
    {
      "id": "flow_2",
      "sourceRef": "service_task_1",
      "targetRef": "exclusive_gateway"
    },
    {
      "id": "flow_3",
      "sourceRef": "exclusive_gateway",
      "targetRef": "end_event",
      "name": "Success",
      "condition": "result == 'success'"
    }
  ],
  "dodaf_metadata": {
    "operational_activity": {
      "id": "OA-001",
      "name": "Activity Name"
    }
  }
}
```

### Workflow Step Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Yes | Unique identifier for the step |
| `name` | string | Yes | Display name |
| `type` | string | Yes | BPMN element type (see supported types) |
| `taskType` | string | No | Specific task type (for service tasks) |
| `eventType` | string | No | Event type (for events) |
| `implementation` | string | No | Implementation reference |
| `inputs` | array | No | Input data elements |
| `outputs` | array | No | Output data elements |
| `duration_minutes` | number | No | Expected duration |

### Sequence Flow Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Yes | Unique flow identifier |
| `sourceRef` | string | Yes | Source node ID |
| `targetRef` | string | Yes | Target node ID |
| `name` | string | No | Flow label |
| `condition` | string | No | Condition expression (for gateways) |

## Automatic Layout

The converter uses a breadth-first traversal algorithm to automatically position nodes:

1. **Start nodes** are placed at `(start_x, start_y)`
2. **Each level** is positioned horizontally with `horizontal_spacing`
3. **Multiple nodes per level** are stacked vertically with `vertical_spacing`
4. **Connections** are created automatically based on `sequence_flows`

### Default Layout Values

- `start_x`: 100.0
- `start_y`: 100.0
- `horizontal_spacing`: 200.0
- `vertical_spacing`: 150.0
- `gateway_offset_x`: 150.0

## Error Handling

The converter provides descriptive error messages for common issues:

```rust
match load_workflow_file("workflow.json") {
    Ok(workflow) => {
        println!("Loaded: {}", workflow.name);
    }
    Err(e) => {
        eprintln!("Error: {}", e);
        // Possible errors:
        // - "Failed to read file: <io_error>"
        // - "Failed to parse JSON: <json_error>"
        // - "Source node not found: <node_id>"
        // - "Target node not found: <node_id>"
    }
}
```

## Example: Complete Integration

```rust
use abcdodaf::ui::{Workspace, bpmn_json_loader::load_workflow_file};
use std::path::PathBuf;

fn open_bpmn_workflow(workspace: &mut Workspace, path: PathBuf) -> Result<(), String> {
    // Load and convert BPMN JSON
    let workflow = load_workflow_file(&path)?;

    // Create workspace document
    let id = workspace.next_id;
    workspace.next_id += 1;

    let mut doc = WorkflowDocument::new(id, workflow.name);
    doc.snarl = workflow.snarl;
    doc.file_path = Some(path);
    doc.is_modified = false;

    // Add to workspace
    workspace.workflows.insert(id, doc);
    workspace.active_workflow = Some(id);

    Ok(())
}
```

## Testing

Run the included example to see the converter in action:

```bash
cargo run --example bpmn_json_converter_example --features ui
```

Run tests:

```bash
cargo test --features ui bpmn_json_loader
```

## Future Enhancements

Potential improvements for future versions:

- [ ] Support for data objects and data stores
- [ ] Boundary events (attached to activities)
- [ ] Subprocess expansion/collapse
- [ ] Text annotations
- [ ] Custom node properties preservation
- [ ] Bi-directional conversion (Snarl → BPMN JSON)
- [ ] Layout optimization for complex graphs
- [ ] Support for pools and lanes
- [ ] Event definitions (timer, message, signal, etc.)

## See Also

- [EnhancedBpmnNode Documentation](./ENHANCED_NODES.md)
- [Workspace Documentation](./WORKSPACE.md)
- [BPMN 2.0 Specification](https://www.omg.org/spec/BPMN/2.0/)
- [Workflow Template](../workflows/WORKFLOW_TEMPLATE.json)
