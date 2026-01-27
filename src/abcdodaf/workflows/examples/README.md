# Example BPMN Workflows

This directory contains example workflow files that can be loaded directly into the ABCDODAF UI.

## Available Workflows

### 1. simple_process.json
A basic three-node workflow demonstrating the minimal BPMN structure:
- **Start Event** → **User Task** → **End Event**
- Perfect for testing basic loading and rendering
- Use case: Simple linear process flow

### 2. decision_flow.json
Demonstrates exclusive gateway (XOR) decision logic:
- **Start Event** → **User Task** → **Exclusive Gateway** → Two paths:
  - Path A: Process Approval → End
  - Path B: Send Rejection → End
- Use case: Approval/rejection workflows

### 3. parallel_tasks.json
Shows parallel execution with fork/join pattern:
- **Start Event** → **Parallel Gateway (Split)** → Two parallel tasks:
  - **Service Task**: Process Data
  - **Send Task**: Send Notification
- **Parallel Gateway (Join)** → **End Event**
- Use case: Concurrent task execution with synchronization

### 4. complex_workflow.json
A realistic order processing workflow with multiple patterns:
- Message start event
- Business rule validation
- Exclusive gateway for validation decision
- Parallel processing (inventory, payment, shipping)
- Data object reference
- Multiple end events
- Use case: E-commerce order processing

### 5. dodaf_example.json
Intelligence analysis workflow with full DoDAF metadata:
- Complete DoDAF OV-5 operational activity metadata
- Security classifications (SECRET, TOP SECRET)
- Cost and duration estimates
- Performer assignments (roles, organizations, systems)
- Compartmented information markings
- Data store with security domain
- Use case: Defense/intelligence operational workflows

## File Format

All workflows use the standard serialization format:

```json
{
  "version": "1.0",
  "name": "Workflow Name",
  "snarl": {
    "nodes": [...],
    "wires": [...]
  }
}
```

## Loading Workflows

To load a workflow in the ABCDODAF UI:

1. Launch the application
2. Click **File** → **Open**
3. Navigate to this directory
4. Select a workflow file
5. The workflow will be displayed in the visual editor

## Node Structure

Each node contains:
- **id**: Unique identifier
- **pos**: [x, y] position in the canvas
- **value**: The EnhancedBpmnNode data
  - **node_type**: BPMN element type (Task, Gateway, Event, etc.)
  - **visual**: Display properties (color, markers)
  - **dodaf_metadata**: Optional DoDAF operational metadata
  - **properties**: Custom key-value properties

## Wire Structure

Wires (connections) define the sequence flow:
- **out_node**: Source node index
- **out_pin**: Output pin number (0 or 1 for gateways)
- **in_node**: Target node index
- **in_pin**: Input pin number (always 0 for single inputs)

## DoDAF Metadata Structure

The `dodaf_example.json` demonstrates full DoDAF metadata:

```json
"dodaf_metadata": {
  "activity_ref": "OV-5.1",
  "performer": {
    "id": "ORG-001",
    "name": "Organization Name",
    "type": "Organization|Role|System"
  },
  "cost": {
    "amount": 150.0,
    "currency": "USD"
  },
  "duration": {
    "value": 30,
    "unit": "Minutes|Hours|Days"
  },
  "security_domain": {
    "classification": "SECRET|TOP_SECRET|UNCLASSIFIED",
    "compartments": ["SI", "TK"],
    "dissemination_controls": ["NOFORN"],
    "releasability": ["USA", "FVEY"]
  },
  "dodaf_properties": {
    "custom_key": "custom_value"
  }
}
```

## Extending Examples

To create your own workflows:
1. Start with a simple example
2. Add nodes at appropriate positions
3. Connect nodes using wire indices
4. Add DoDAF metadata as needed
5. Validate JSON syntax before loading

## Validation

All files are valid JSON and can be validated with:
```bash
python3 -m json.tool workflow.json
```

Or in Rust:
```bash
cargo run --example validate_workflow workflow.json
```
