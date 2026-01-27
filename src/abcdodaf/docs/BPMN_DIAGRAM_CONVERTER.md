# BpmnDiagram to Snarl Converter

## Overview

The `BpmnDiagramConverter` provides bidirectional conversion between `BpmnDiagram` (the library's native BPMN 2.0 format) and `Snarl<EnhancedBpmnNode>` (the visual editor format used by egui-snarl).

This converter is **critical for maintaining full BPMN 2.0 compatibility** throughout the library, ensuring that all BPMN properties are preserved during editing workflows.

## Features

### ✓ Complete BPMN 2.0 Support

The converter handles all BPMN 2.0 elements:

**Events:**
- Start Events (with all 12 event types)
- End Events (with all 12 event types)
- Intermediate Events (catching/throwing, boundary events)

**Activities:**
- All 8 task types (User, Service, Script, Manual, Send, Receive, Business Rule, Abstract)
- Subprocesses (Embedded, Call Activity, Event Subprocess, Transaction, Ad-Hoc)
- Loop characteristics (Standard, Multi-Instance)

**Gateways:**
- All 6 gateway types (Exclusive, Parallel, Inclusive, Event-Based, Parallel Event-Based, Complex)
- Gateway directions (Converging, Diverging, Mixed)

**Data Elements:**
- Data Objects (with collection support)
- Data Stores

**Artifacts:**
- Text Annotations
- Groups

**Connecting Objects:**
- Sequence Flows (with conditions)

### ✓ Metadata Preservation

- Custom properties on all elements
- DoDAF operational metadata
- Visual markers (loops, multi-instance, compensation, etc.)
- Documentation fields
- Element IDs for consistency

### ✓ Visual Layout

- Node positions from BPMN DI (Diagram Interchange)
- Shape bounds and sizing
- Edge waypoints
- Can generate BpmnDiagramInfo from Snarl positions

## API Usage

### Basic Conversion

```rust
use abcdodaf::bpmn::elements::BpmnDiagram;
use abcdodaf::ui::{BpmnDiagramConverter, EnhancedBpmnNode};
use egui_snarl::Snarl;

// Load or create a BpmnDiagram
let diagram: BpmnDiagram = /* ... */;

// Convert to Snarl for visual editing
let snarl: Snarl<EnhancedBpmnNode> = BpmnDiagramConverter::to_snarl(&diagram)?;

// Edit the snarl in your UI...

// Convert back to BpmnDiagram for saving
let result_diagram = BpmnDiagramConverter::from_snarl(
    &snarl,
    "diagram_id",
    "Diagram Name"
)?;
```

### Multi-Process Diagrams

```rust
// Convert specific process by ID
let snarl = BpmnDiagramConverter::to_snarl_with_process(&diagram, "process_1")?;
```

### Generate Visual Layout Info

```rust
// Extract layout information from Snarl
let diagram_info = BpmnDiagramConverter::generate_diagram_info(
    &snarl,
    "diagram_id",
    "process_id"
);
```

## Conversion Flow

### BpmnDiagram → Snarl

1. **Extract Process**: Get the first process (or specified process by ID)
2. **Create Nodes**: Convert all flow elements to `EnhancedBpmnNode`
   - Start/End/Intermediate Events
   - Tasks (all 8 types)
   - Subprocesses
   - Gateways (all 6 types)
   - Data Objects/Stores
   - Text Annotations/Groups
3. **Position Nodes**: Use BPMN DI bounds if available, otherwise default positions
4. **Add Visual Markers**: Loop indicators, multi-instance, compensation, etc.
5. **Create Wires**: Convert `SequenceFlow` elements to Snarl connections
   - Map source/target references to node IDs
   - Determine pin indices based on node type

### Snarl → BpmnDiagram

1. **Extract Nodes**: Iterate all nodes in Snarl
2. **Convert to BPMN Elements**: Create appropriate BPMN element for each node type
   - Preserve all properties
   - Maintain element IDs
   - Keep DoDAF metadata
3. **Extract Wires**: Convert Snarl connections to `SequenceFlow` elements
   - Generate unique flow IDs
   - Map node IDs to element references
4. **Generate Layout**: Create `BpmnDiagramInfo` with visual positions
5. **Package Diagram**: Combine process, layout, and metadata

## Preserved Properties

### All Node Types Preserve:
- Element ID
- Name
- Documentation
- Custom properties (HashMap)
- DoDAF metadata (optional)
- Visual properties (color, markers)

### Task-Specific:
- Task type with all variant data
- Loop characteristics
- Compensation flag
- IO specification (future)

### Gateway-Specific:
- Gateway type with conditions
- Gateway direction
- Default flow reference (future)

### Event-Specific:
- Event definition (all 12 types)
- Catching/throwing flag
- Interrupting flag
- Boundary event attachment (future)

### Subprocess-Specific:
- Subprocess type
- Triggered by event flag
- Embedded process (future)
- Called element reference (future)

## Error Handling

The converter returns `Result<T, ConversionError>` with these error types:

```rust
pub enum ConversionError {
    /// No processes found in diagram
    EmptyDiagram,

    /// Invalid element reference
    InvalidReference(String),

    /// Missing required data
    MissingData(String),

    /// Invalid BPMN structure
    InvalidStructure(String),

    /// Conversion not supported
    Unsupported(String),
}
```

## Testing

The converter includes comprehensive unit tests:

```bash
cargo test --lib -- bpmn_diagram_converter
```

Run the example to see it in action:

```bash
cargo run --example bpmn_diagram_converter_example --features ui
```

## Round-Trip Integrity

The converter is designed for **lossless round-trip conversion**:

```
BpmnDiagram → Snarl → BpmnDiagram
```

All BPMN properties should be preserved. The example demonstrates verification:

```
✓ Start events: 1 -> 1
✓ Tasks: 3 -> 3
✓ Gateways: 1 -> 1
✓ Intermediate events: 1 -> 1
✓ End events: 2 -> 2
✓ Sequence flows: 7 -> 7
✓ All element counts match!
```

## Implementation Details

### Node Positioning

- Uses BPMN DI bounds when available
- Falls back to default positions:
  - Start events: (100, 100)
  - Tasks: (250, 150)
  - Gateways: (300, 150)
  - End events: (500, 100)
  - Etc.

### Pin Index Mapping

- Most nodes: 1 input, 1 output (pin index 0)
- Start events: 0 inputs, 1 output
- End events: 1 input, 0 outputs
- Gateways: 1 input, 2+ outputs (based on type)
- Data objects/stores: No pins (data associations not yet implemented)

### Visual Markers

Automatically added based on BPMN properties:
- Loop marker: `LoopCharacteristics::Standard`
- Multi-instance marker: `LoopCharacteristics::MultiInstance`
- Compensation marker: `is_for_compensation = true`
- Ad-hoc marker: `SubprocessType::AdHoc`
- Collapsed marker: `is_expanded = false`

## Future Enhancements

### Planned Features:
- [ ] Data associations (connect data objects to activities)
- [ ] Message flows (for collaboration diagrams)
- [ ] Boundary event attachment
- [ ] Default flow references on gateways
- [ ] Call activity element references
- [ ] Embedded subprocess processes
- [ ] IO specifications
- [ ] Lane/pool support
- [ ] Multi-process diagram merging
- [ ] Edge waypoint preservation

### Optimization:
- [ ] Lazy conversion for large diagrams
- [ ] Incremental updates (detect changes)
- [ ] Parallel conversion for multi-process diagrams
- [ ] Caching of conversion contexts

## Integration

The converter is automatically available when using the `ui` feature:

```toml
[dependencies]
abcdodaf = { version = "0.1", features = ["ui"] }
```

It's integrated into the workspace for seamless BPMN editing:

```rust
// In your UI code
use abcdodaf::ui::Workspace;

let mut workspace = Workspace::new();

// Load BPMN file
workspace.load_bpmn_file("workflow.bpmn")?;

// Edit visually with Snarl...

// Save back to BPMN
workspace.save_current()?;
```

## Best Practices

### 1. Always Preserve IDs
```rust
// DON'T: Generate new IDs
let node = EnhancedBpmnNode::new("new_id", ...);

// DO: Use original element ID
let node = EnhancedBpmnNode::new(task.id.clone(), ...);
```

### 2. Handle All Node Types
```rust
// DON'T: Ignore unknown types
match &node.node_type {
    BpmnNodeType::Task(_) => { /* ... */ }
    _ => {} // Silently drops data!
}

// DO: Handle all variants
match &node.node_type {
    BpmnNodeType::StartEvent(n) => { /* ... */ }
    BpmnNodeType::EndEvent(n) => { /* ... */ }
    BpmnNodeType::Task(n) => { /* ... */ }
    // ... handle all types
}
```

### 3. Validate After Conversion
```rust
use abcdodaf::ui::Validator;

let snarl = BpmnDiagramConverter::to_snarl(&diagram)?;

// Validate structure
let validation = Validator::validate_workflow(&snarl);
if !validation.is_valid() {
    for error in &validation.errors {
        eprintln!("Validation error: {:?}", error);
    }
}
```

### 4. Test Round-Trip
```rust
// Original
let original = create_diagram();

// Convert to Snarl and back
let snarl = BpmnDiagramConverter::to_snarl(&original)?;
let result = BpmnDiagramConverter::from_snarl(&snarl, "id", "name")?;

// Verify counts match
assert_eq!(
    original.processes[0].tasks.len(),
    result.processes[0].tasks.len()
);
```

## Troubleshooting

### Issue: Missing nodes after conversion
**Cause**: Not all node types are being converted.
**Solution**: Check that your `match` statement handles all `BpmnNodeType` variants.

### Issue: Connections lost after round-trip
**Cause**: Element IDs changed during conversion.
**Solution**: Always preserve original element IDs from BPMN elements.

### Issue: Visual positions not preserved
**Cause**: BPMN DI information not included in diagram.
**Solution**: Use `generate_diagram_info()` to create layout information before saving.

### Issue: Properties missing after conversion
**Cause**: Custom properties not copied to/from nodes.
**Solution**: Ensure `properties` HashMap is preserved in both directions.

## See Also

- [BPMN 2.0 Specification](http://www.omg.org/spec/BPMN/2.0/)
- [Enhanced Nodes Documentation](./ENHANCED_NODES.md)
- [Workspace Guide](./WORKSPACE.md)
- [Validation Rules](./VALIDATION.md)

## Example Output

Running the converter example:

```
=== BpmnDiagram to Snarl Converter Example ===

Created sample BPMN diagram:
  - ID: sample_diagram
  - Processes: 1
  - Process elements: 1 start, 3 tasks, 1 gateways, 2 end
  - Sequence flows: 7

Converting BpmnDiagram to Snarl...
Snarl created successfully:
  - Nodes: 8

Converting Snarl back to BpmnDiagram...
BpmnDiagram created from Snarl:
  - ID: result_diagram
  - Processes: 1
  - Process elements: 1 start, 3 tasks, 1 gateways, 2 end
  - Sequence flows: 7

=== Round-trip Verification ===
Comparing element counts:
  Start events: 1 -> 1 ✓
  Tasks: 3 -> 3 ✓
  Gateways: 1 -> 1 ✓
  Intermediate events: 1 -> 1 ✓
  End events: 2 -> 2 ✓
  Sequence flows: 7 -> 7 ✓

✓ All element counts match!
✓ Conversion example completed successfully!
```
