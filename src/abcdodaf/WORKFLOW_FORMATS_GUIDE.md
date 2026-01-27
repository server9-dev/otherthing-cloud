# Workflow JSON Formats Guide

This guide documents the two JSON formats used by the ABCDODAF library for workflow representation.

## Format Overview

The library supports two JSON formats for workflows:

1. **BPMN JSON Format** - Human-readable format for defining workflows with steps and flows
2. **Snarl Format** - Internal graph representation used by the editor

## 1. BPMN JSON Format

### Purpose
- Portable, human-readable workflow definitions
- Import workflows from external sources
- Agent task definitions with sequential steps
- Can be converted to Snarl format for editing

### Schema

```json
{
  "bpmn_process": {
    "id": "string",
    "name": "string",
    "version": "string",
    "isExecutable": true,
    "processType": "string"
  },
  "workflow_steps": [
    {
      "id": "unique_step_id",
      "name": "Human readable name",
      "type": "startEvent|serviceTask|userTask|exclusiveGateway|parallelGateway|endEvent",
      "taskType": "optional: analysis|research|design|code_generation|integration|testing|documentation",
      "eventType": "optional: none|message|timer|error|signal",
      "implementation": "optional: implementation details",
      "inputs": ["optional array of input names"],
      "outputs": ["optional array of output names"],
      "duration_minutes": 0
    }
  ],
  "sequence_flows": [
    {
      "id": "flow_id",
      "sourceRef": "source_step_id",
      "targetRef": "target_step_id",
      "name": "optional: flow name",
      "condition": "optional: condition expression"
    }
  ],
  "task_metadata": {
    "optional": "task-specific metadata"
  },
  "dodaf_metadata": {
    "optional": "DoDAF annotations"
  }
}
```

### Required Fields

**bpmn_process:**
- `id` - Unique process identifier
- `name` - Process name
- `version` - Version string (default: "1.0.0")
- `isExecutable` - Boolean (default: true)

**workflow_steps:**
- `id` - Unique step identifier
- `name` - Step name
- `type` - Step type (see supported types below)

**sequence_flows:**
- `id` - Unique flow identifier
- `sourceRef` - Must match a step id
- `targetRef` - Must match a step id

### Supported Step Types

**Events:**
- `startEvent` - Process start point
- `endEvent` - Process end point
- `intermediateEvent` - Event during process

**Tasks:**
- `serviceTask` - Automated service task
- `userTask` - Human task
- `sendTask` - Send message
- `receiveTask` - Receive message
- `businessRuleTask` - Decision/rule execution
- `scriptTask` - Script execution

**Gateways:**
- `exclusiveGateway` - XOR decision (one path)
- `parallelGateway` - AND split/join (all paths)
- `inclusiveGateway` - OR decision (multiple paths)
- `eventBasedGateway` - Event-driven routing

### Example: Simple Process

```json
{
  "bpmn_process": {
    "id": "simple_approval",
    "name": "Simple Approval Process",
    "version": "1.0.0",
    "isExecutable": true,
    "processType": "approval_workflow"
  },
  "workflow_steps": [
    {
      "id": "start",
      "name": "Start",
      "type": "startEvent",
      "eventType": "none"
    },
    {
      "id": "review_task",
      "name": "Review Request",
      "type": "userTask",
      "taskType": "review",
      "duration_minutes": 30
    },
    {
      "id": "decision",
      "name": "Approved?",
      "type": "exclusiveGateway"
    },
    {
      "id": "approve_task",
      "name": "Process Approval",
      "type": "serviceTask",
      "taskType": "processing"
    },
    {
      "id": "reject_task",
      "name": "Send Rejection",
      "type": "sendTask"
    },
    {
      "id": "end_approve",
      "name": "Approved",
      "type": "endEvent"
    },
    {
      "id": "end_reject",
      "name": "Rejected",
      "type": "endEvent"
    }
  ],
  "sequence_flows": [
    {"id": "f1", "sourceRef": "start", "targetRef": "review_task"},
    {"id": "f2", "sourceRef": "review_task", "targetRef": "decision"},
    {"id": "f3", "sourceRef": "decision", "targetRef": "approve_task", "name": "Yes"},
    {"id": "f4", "sourceRef": "decision", "targetRef": "reject_task", "name": "No"},
    {"id": "f5", "sourceRef": "approve_task", "targetRef": "end_approve"},
    {"id": "f6", "sourceRef": "reject_task", "targetRef": "end_reject"}
  ]
}
```

### Loading BPMN JSON

Use the `bpmn_json_loader` module:

```rust
use abcdodaf::ui::bpmn_json_loader::BpmnJsonLoader;

// Load from file
let loader = BpmnJsonLoader::new();
let snarl = loader.load_from_file("workflow.json")?;

// Convert to workspace format
let workspace_file = loader.to_workspace_format(bpmn_workflow);
```

## 2. Snarl Format

### Purpose
- Internal representation for the graph editor
- Direct serialization of editor state
- Preserves exact node positions and visual state
- Fastest format for save/load operations

### Schema

```json
{
  "version": "1.0",
  "name": "Workflow Name",
  "snarl": {
    "nodes": [
      {
        "id": "node_id",
        "pos": [x, y],
        "open": true,
        "value": {
          "id": "node_id",
          "node_type": {
            "StartEvent": {
              "name": "Start",
              "documentation": "Description",
              "event_definition": null,
              "is_interrupting": true
            }
          },
          "visual": {
            "color": null,
            "highlighted": false,
            "selected": false,
            "markers": []
          },
          "dodaf_metadata": null,
          "properties": {}
        }
      }
    ],
    "wires": [
      {
        "out_node": 0,
        "out_pin": 0,
        "in_node": 1,
        "in_pin": 0
      }
    ]
  }
}
```

### Required Fields

**Root:**
- `version` - Format version
- `name` - Workflow name
- `snarl` - Snarl graph object

**snarl:**
- `nodes` - Array of nodes
- `wires` - Array of connections

**nodes:**
- `id` - Unique node identifier
- `pos` - [x, y] position in editor
- `open` - Whether node is expanded
- `value` - Node data object
  - `id` - Node ID (matches parent)
  - `node_type` - Variant with node-specific data

**wires:**
- `out_node` - Source node index
- `out_pin` - Source pin index
- `in_node` - Target node index
- `in_pin` - Target pin index

### Node Types

Each node_type is a Rust enum variant with associated data:

**StartEvent:**
```json
{
  "StartEvent": {
    "name": "string",
    "documentation": "string",
    "event_definition": null | {
      "Message": {"message_ref": "string"}
    } | {
      "Timer": {"time_date": "string"}
    },
    "is_interrupting": true
  }
}
```

**Task:**
```json
{
  "Task": {
    "name": "string",
    "documentation": "string",
    "task_type": {
      "User": {"implementation": null, "rendering": null}
    } | {
      "Service": {"implementation": "string", "operation_ref": "string"}
    } | {
      "Send": {"message_ref": "string", "operation_ref": null}
    } | {
      "BusinessRule": {"implementation": "string", "rule_ref": "string"}
    },
    "loop_characteristics": null,
    "is_for_compensation": false
  }
}
```

**Gateway:**
```json
{
  "Gateway": {
    "name": "string",
    "documentation": "string",
    "gateway_type": "Exclusive" | "Parallel" | "Inclusive" | "EventBased",
    "gateway_direction": "Diverging" | "Converging"
  }
}
```

**EndEvent:**
```json
{
  "EndEvent": {
    "name": "string",
    "documentation": "string",
    "event_definition": null | {...}
  }
}
```

**DataObject:**
```json
{
  "DataObject": {
    "name": "string",
    "is_collection": false,
    "data_state": "string"
  }
}
```

**DataStore:**
```json
{
  "DataStore": {
    "name": "string",
    "is_unlimited": false,
    "capacity": null
  }
}
```

### Visual Properties

```json
{
  "visual": {
    "color": null | [r, g, b],
    "highlighted": false,
    "selected": false,
    "markers": []
  }
}
```

### DoDAF Metadata

Optional DoDAF annotations for each node:

```json
{
  "dodaf_metadata": {
    "activity_ref": "OV-5.1",
    "performer": {
      "id": "ORG-001",
      "name": "Organization Name",
      "type": "Organization" | "Role" | "System"
    },
    "cost": {
      "amount": 100.0,
      "currency": "USD"
    },
    "duration": {
      "value": 30,
      "unit": "Minutes" | "Hours" | "Days"
    },
    "security_domain": {
      "classification": "SECRET",
      "compartments": ["SI", "TK"],
      "dissemination_controls": ["NOFORN"],
      "releasability": []
    },
    "dodaf_properties": {
      "custom_key": "custom_value"
    }
  }
}
```

### Example: Simple Process (Snarl)

```json
{
  "version": "1.0",
  "name": "Simple Process",
  "snarl": {
    "nodes": [
      {
        "id": "start_1",
        "pos": [100.0, 200.0],
        "open": true,
        "value": {
          "id": "start_1",
          "node_type": {
            "StartEvent": {
              "name": "Start",
              "documentation": "Process begins",
              "event_definition": null,
              "is_interrupting": true
            }
          },
          "visual": {
            "color": null,
            "highlighted": false,
            "selected": false,
            "markers": []
          },
          "dodaf_metadata": null,
          "properties": {}
        }
      },
      {
        "id": "task_1",
        "pos": [300.0, 200.0],
        "open": true,
        "value": {
          "id": "task_1",
          "node_type": {
            "Task": {
              "name": "Process Request",
              "documentation": "Main task",
              "task_type": {
                "User": {
                  "implementation": null,
                  "rendering": null
                }
              },
              "loop_characteristics": null,
              "is_for_compensation": false
            }
          },
          "visual": {
            "color": null,
            "highlighted": false,
            "selected": false,
            "markers": []
          },
          "dodaf_metadata": null,
          "properties": {}
        }
      },
      {
        "id": "end_1",
        "pos": [500.0, 200.0],
        "open": true,
        "value": {
          "id": "end_1",
          "node_type": {
            "EndEvent": {
              "name": "End",
              "documentation": "Process complete",
              "event_definition": null
            }
          },
          "visual": {
            "color": null,
            "highlighted": false,
            "selected": false,
            "markers": []
          },
          "dodaf_metadata": null,
          "properties": {}
        }
      }
    ],
    "wires": [
      {
        "out_node": 0,
        "out_pin": 0,
        "in_node": 1,
        "in_pin": 0
      },
      {
        "out_node": 1,
        "out_pin": 0,
        "in_node": 2,
        "in_pin": 0
      }
    ]
  }
}
```

### Loading Snarl Format

Direct deserialization:

```rust
use abcdodaf::ui::workspace::WorkflowFile;
use std::fs;

let content = fs::read_to_string("workflow.json")?;
let workflow: WorkflowFile = serde_json::from_str(&content)?;

// Access the snarl
let snarl = workflow.snarl;
```

## Format Comparison

| Feature | BPMN JSON | Snarl |
|---------|-----------|-------|
| **Human Readable** | ✅ High | ⚠️ Medium |
| **Compact** | ✅ Yes | ❌ Verbose |
| **Portable** | ✅ Standard | ❌ Custom |
| **Editor Ready** | ❌ Needs conversion | ✅ Direct use |
| **Preserves Layout** | ❌ No | ✅ Yes |
| **File Size** | ✅ Small | ❌ Large |
| **Edit by Hand** | ✅ Easy | ❌ Complex |

## Format Conversion

### BPMN JSON → Snarl

```rust
use abcdodaf::ui::bpmn_json_loader::BpmnJsonLoader;

let loader = BpmnJsonLoader::new();
let snarl = loader.load_from_file("workflow.json")?;
```

The conversion:
1. Creates Snarl nodes from workflow_steps
2. Automatically positions nodes in a grid layout
3. Creates Snarl wires from sequence_flows
4. Maps BPMN types to node_type variants

### Snarl → BPMN JSON

Currently not implemented. Snarl format is considered the "source of truth" after editing.

## Best Practices

### When to Use BPMN JSON
- ✅ Initial workflow definition
- ✅ Version control (more diff-friendly)
- ✅ Human editing
- ✅ Importing from external tools
- ✅ Agent-generated workflows
- ✅ Documentation examples

### When to Use Snarl
- ✅ Saving editor state
- ✅ Preserving visual layout
- ✅ Internal workspace files
- ✅ Direct workspace loading
- ✅ Runtime execution

### File Naming Conventions

**BPMN JSON:**
- `task_*.json` - Agent task definitions
- `workflow_*.json` - General workflows
- `*_TEMPLATE.json` - Templates

**Snarl:**
- `*.snarl.json` - Explicit Snarl format
- `workspace_*.json` - Saved workspaces
- No suffix = Snarl format (legacy)

## Validation

### BPMN JSON Validation

Required checks:
- ✅ All workflow_steps have unique IDs
- ✅ All sequence_flows reference valid step IDs
- ✅ All steps have required fields (id, name, type)
- ✅ No circular dependencies (optional)
- ✅ Start and end events present (recommended)

### Snarl Validation

Required checks:
- ✅ All nodes have unique IDs
- ✅ All wire indices are within bounds
- ✅ All nodes have valid node_type
- ✅ Node positions are valid floats

## Migration

If you have workflows in BPMN JSON format that need to be used in the editor:

1. Use `BpmnJsonLoader` to convert to Snarl
2. Save the Snarl format for future editing
3. Keep BPMN JSON for version control

```rust
// One-time conversion
let loader = BpmnJsonLoader::new();
let snarl = loader.load_from_file("workflow.json")?;

// Save as Snarl
let workspace_file = WorkflowFile {
    version: "1.0".to_string(),
    name: "My Workflow".to_string(),
    snarl,
};

let json = serde_json::to_string_pretty(&workspace_file)?;
fs::write("workflow.snarl.json", json)?;
```

## File Locations

**Standard Locations:**
```
/workflows/
  /agent_tasks/        # BPMN JSON format (agent workflows)
  /examples/           # Snarl format (example workflows)
  /templates/          # BPMN JSON format (templates)
  /workspaces/         # Snarl format (saved workspaces)
```

## Summary

- **BPMN JSON**: Best for creating and sharing workflows
- **Snarl**: Best for editor operations and visual layout
- Both formats are valid and serve different purposes
- Use conversion tools to move between formats
- Choose format based on use case, not preference
