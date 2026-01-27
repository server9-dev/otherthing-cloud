# BPMN 2.0 XML Import/Export Implementation Guide

## Overview

This document describes the comprehensive BPMN 2.0 XML import/export system with Diagram Interchange (DI) support implemented for the ABCD ODAF library.

## Architecture

### Module Structure

```
bpmn/
├── xml_io.rs          # XML serialization/deserialization
├── file_io.rs         # File I/O operations
├── elements.rs        # BPMN element definitions
├── mod.rs             # Module exports
└── ...
```

### Core Components

#### 1. BpmnXmlSerializer (xml_io.rs)

Handles all XML serialization and deserialization for BPMN 2.0 diagrams.

**Key Features:**
- Produces valid BPMN 2.0 XML conforming to the OMG specification
- Full BPMN Diagram Interchange (DI) support
- Proper namespace handling (BPMN, BPMNDI, DC, DI)
- XML special character escaping
- Comprehensive error handling with detailed messages
- Extensible architecture for custom elements

**Public API:**
```rust
impl BpmnXmlSerializer {
    pub fn to_string(diagram: &BpmnDiagram) -> XmlResult<String>
    pub fn from_string(xml: &str) -> XmlResult<BpmnDiagram>
    pub fn write<W: Write>(diagram: &BpmnDiagram, writer: &mut W) -> XmlResult<()>
    pub fn read<R: Read>(reader: &mut R) -> XmlResult<BpmnDiagram>
}
```

**Supported Elements:**

1. **Events**
   - Start Event (with optional event definitions)
   - End Event
   - Intermediate Event (catching/throwing, boundary)
   - All 12 event types: None, Message, Timer, Signal, Error, Escalation, Cancel, Compensation, Conditional, Link, Terminate, Multiple

2. **Tasks**
   - Abstract Task
   - User Task
   - Service Task
   - Manual Task
   - Script Task (with script format)
   - Business Rule Task
   - Send Task
   - Receive Task

3. **Gateways**
   - Exclusive Gateway (XOR)
   - Parallel Gateway (AND)
   - Inclusive Gateway (OR)
   - Event-Based Gateway
   - Parallel Event-Based Gateway
   - Complex Gateway

4. **Subprocesses**
   - Embedded Subprocess
   - Call Activity
   - Event Subprocess
   - Transaction
   - Ad-hoc Subprocess

5. **Data Elements**
   - Data Object (single and collection)
   - Data Store
   - Data Association

6. **Artifacts**
   - Text Annotation
   - Group
   - Association

7. **Connections**
   - Sequence Flow (with optional conditions)
   - Message Flow
   - Data Association

8. **Swimlanes**
   - Collaboration
   - Participant
   - Lane (with nested lanes support)

9. **Diagram Interchange**
   - BPMNShape (node visual representation)
   - BPMNEdge (connection visual representation)
   - Bounds (x, y, width, height)
   - Waypoints (connection routing)
   - Labels (optional labels on shapes/edges)

#### 2. BpmnFileIo (file_io.rs)

High-level file operations for BPMN documents.

**Key Features:**
- Load/save .bpmn files with automatic format validation
- Automatic backup creation before overwriting
- Comprehensive error handling with path information
- Directory listing and file metadata operations
- Configurable save options

**Public API:**
```rust
impl BpmnFileIo {
    pub fn load<P: AsRef<Path>>(path: P) -> FileResult<BpmnDiagram>
    pub fn load_with_options<P: AsRef<Path>>(path: P, options: FileOptions) -> FileResult<BpmnDiagram>
    pub fn save<P: AsRef<Path>>(path: P, diagram: &BpmnDiagram) -> FileResult<()>
    pub fn save_with_options<P: AsRef<Path>>(path: P, diagram: &BpmnDiagram, options: FileOptions) -> FileResult<()>
    pub fn exists<P: AsRef<Path>>(path: P) -> bool
    pub fn absolute_path<P: AsRef<Path>>(path: P) -> FileResult<PathBuf>
    pub fn metadata<P: AsRef<Path>>(path: P) -> FileResult<fs::Metadata>
    pub fn list_in_directory<P: AsRef<Path>>(dir: P) -> FileResult<Vec<PathBuf>>
    pub fn delete<P: AsRef<Path>>(path: P) -> FileResult<()>
}
```

**FileOptions Configuration:**
```rust
pub struct FileOptions {
    pub create_backup: bool,           // Default: true
    pub backup_extension: String,      // Default: ".bak"
    pub fail_if_exists: bool,          // Default: false
    pub pretty_print: bool,            // Default: true
}
```

## BPMN 2.0 Namespace Standards

```rust
pub mod namespace {
    pub const BPMN2: &str = "http://www.omg.org/spec/BPMN/20100524/MODEL";
    pub const BPMNDI: &str = "http://www.omg.org/spec/BPMN/20100524/DI";
    pub const DC: &str = "http://www.omg.org/spec/DD/20100524/DC";
    pub const DI: &str = "http://www.omg.org/spec/DD/20100524/DI";
}
```

## Usage Examples

### Basic Export

```rust
use abcdodaf::bpmn::{
    elements::*,
    xml_io::BpmnXmlSerializer,
    file_io::BpmnFileIo
};
use std::collections::HashMap;

// Create a BPMN diagram
let diagram = BpmnDiagram {
    id: "my_process".to_string(),
    name: Some("My Process".to_string()),
    documentation: None,
    processes: vec![
        BpmnProcess {
            id: "proc1".to_string(),
            name: Some("Process 1".to_string()),
            documentation: None,
            is_executable: true,
            process_type: ProcessType::Private,
            start_events: vec![StartEvent {
                id: "start1".to_string(),
                name: Some("Start".to_string()),
                documentation: None,
                event_definition: None,
                is_interrupting: true,
            }],
            end_events: vec![EndEvent {
                id: "end1".to_string(),
                name: Some("End".to_string()),
                documentation: None,
                event_definition: None,
            }],
            // ... other elements ...
            tasks: vec![],
            gateways: vec![],
            intermediate_events: vec![],
            subprocesses: vec![],
            sequence_flows: vec![],
            data_objects: vec![],
            data_associations: vec![],
            text_annotations: vec![],
            groups: vec![],
            lanes: vec![],
            metadata: HashMap::new(),
        }
    ],
    collaborations: vec![],
    data_stores: vec![],
    messages: vec![],
    signals: vec![],
};

// Export to XML string
let xml = BpmnXmlSerializer::to_string(&diagram)?;

// Save to file
BpmnFileIo::save("my_process.bpmn", &diagram)?;
```

### Basic Import

```rust
// Load from file
let diagram = BpmnFileIo::load("my_process.bpmn")?;

// Access loaded elements
for process in &diagram.processes {
    println!("Process: {}", process.id);
    println!("  Start events: {}", process.start_events.len());
    println!("  Tasks: {}", process.tasks.len());
}
```

### Advanced Features

#### Custom Save Options

```rust
use abcdodaf::bpmn::file_io::{BpmnFileIo, FileOptions};

let options = FileOptions {
    create_backup: true,
    backup_extension: ".bak".to_string(),
    fail_if_exists: false,
    pretty_print: true,
};

BpmnFileIo::save_with_options("my_process.bpmn", &diagram, options)?;
```

#### File Management

```rust
// List all BPMN files in directory
let files = BpmnFileIo::list_in_directory("./processes")?;
for file in files {
    println!("Found: {}", file.display());
}

// Get file metadata
let metadata = BpmnFileIo::metadata("my_process.bpmn")?;
println!("File size: {} bytes", metadata.len());

// Delete file
BpmnFileIo::delete("old_process.bpmn")?;
```

## Round-Trip Capability

The implementation supports full round-trip conversion:

```
BPMN Diagram → XML Export → File Save → File Load → XML Parse → BPMN Diagram
```

This ensures that:
1. All element data is preserved
2. All attributes are maintained
3. Visual layout (DI information) is retained
4. No information loss during cycles

### Validation

Round-trip validation can be performed:

```rust
use abcdodaf::bpmn::xml_io::validation;

// Verify that export → import → export produces identical XML
validation::validate_roundtrip(&diagram)?;
```

## Error Handling

### XML Errors

```rust
pub enum XmlError {
    ParseError(String),
    SerializationError(String),
    ElementNotFound(String),
    InvalidElement(String),
    MissingAttribute { element: String, attribute: String },
    InvalidCoordinate { value: String, reason: String },
    IoError(String),
    ValidationError(String),
}
```

### File Errors

```rust
pub enum FileError {
    NotFound(PathBuf),
    DirectoryNotFound(PathBuf),
    InvalidExtension { path: PathBuf, expected: &'static str, found: String },
    Io(String),
    Xml(String),
    AlreadyExists(PathBuf),
    PermissionDenied(PathBuf),
}
```

### Error Handling Pattern

```rust
// Using ? operator for automatic error propagation
match BpmnFileIo::load("my_process.bpmn") {
    Ok(diagram) => {
        // Process diagram
    }
    Err(FileError::NotFound(path)) => {
        eprintln!("File not found: {}", path.display());
    }
    Err(FileError::Xml(msg)) => {
        eprintln!("XML error: {}", msg);
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

## XML Structure Example

Generated XML follows BPMN 2.0 standards:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<definitions id="my_diagram" name="My Process"
    xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
    xmlns:bpmndi="http://www.omg.org/spec/BPMN/20100524/DI"
    xmlns:dc="http://www.omg.org/spec/DD/20100524/DC"
    xmlns:di="http://www.omg.org/spec/DD/20100524/DI">

    <message id="msg1" name="Test Message" />
    <signal id="sig1" name="Test Signal" />

    <process id="proc1" name="Process 1" isExecutable="true" processType="Private">
        <documentation>A test process</documentation>

        <startEvent id="start1" name="Start" />
        <userTask id="task1" name="Review" />
        <exclusiveGateway id="gate1" name="Approved?" default="reject_flow" />
        <endEvent id="end_approved" name="Approved" />
        <endEvent id="end_rejected" name="Rejected" />

        <sequenceFlow id="flow1" sourceRef="start1" targetRef="task1" />
        <sequenceFlow id="flow2" sourceRef="task1" targetRef="gate1" />
        <sequenceFlow id="approve_flow" name="Yes" sourceRef="gate1"
            targetRef="end_approved">
            <conditionExpression>approved == true</conditionExpression>
        </sequenceFlow>
        <sequenceFlow id="reject_flow" name="No" sourceRef="gate1"
            targetRef="end_rejected" />

        <dataObject id="doc1" name="Document" />
        <textAnnotation id="note1" textFormat="text/plain">
            <text>Review document for approval</text>
        </textAnnotation>
    </process>

    <bpmndi:BPMNDiagram id="Diagram_my_diagram" name="My Process">
        <bpmndi:BPMNPlane id="Plane_proc1" bpmnElement="proc1">
            <bpmndi:BPMNShape id="Shape_start1" bpmnElement="start1">
                <dc:Bounds x="100" y="80" width="36" height="36" />
            </bpmndi:BPMNShape>
            <bpmndi:BPMNShape id="Shape_task1" bpmnElement="task1">
                <dc:Bounds x="200" y="80" width="100" height="80" />
            </bpmndi:BPMNShape>
            <bpmndi:BPMNEdge id="Edge_flow1" bpmnElement="flow1">
                <di:waypoint x="136" y="98" />
                <di:waypoint x="200" y="120" />
            </bpmndi:BPMNEdge>
        </bpmndi:BPMNPlane>
    </bpmndi:BPMNDiagram>

</definitions>
```

## Integration with Workspace

The BPMN import/export system can be integrated with the workspace save/load system:

```rust
// In workspace implementation
impl Workspace {
    pub fn save_as_bpmn(&self, workflow_id: WorkflowId, path: &Path) -> Result<()> {
        let document = self.get_document(workflow_id)?;
        let diagram = convert_workflow_to_bpmn(&document.snarl)?;
        BpmnFileIo::save(path, &diagram)?;
        Ok(())
    }

    pub fn load_bpmn(&mut self, path: &Path) -> Result<WorkflowId> {
        let diagram = BpmnFileIo::load(path)?;
        let snarl = convert_bpmn_to_workflow(&diagram)?;
        let id = self.create_new_workflow();
        let doc = self.get_mut_document(id)?;
        doc.snarl = snarl;
        Ok(id)
    }
}
```

## Validation (Future Enhancement)

The validation module provides comprehensive BPMN validation:

```rust
use abcdodaf::bpmn::validation::BpmnValidator;

// Validate entire diagram
match BpmnValidator::validate(&diagram) {
    Ok(()) => println!("Diagram is valid"),
    Err(errors) => {
        for error in errors {
            eprintln!("Validation error: {}", error);
        }
    }
}

// Validate just structure
let structural_errors = BpmnValidator::validate_structural(&diagram);

// Validate just semantics
let semantic_errors = BpmnValidator::validate_semantic(&diagram);
```

## Testing

Comprehensive tests are provided in:
- `tests/bpmn_xml_tests.rs` - XML serialization tests
- `examples/bpmn_io_roundtrip.rs` - Round-trip example

Run tests with:
```bash
cargo test --lib bpmn::xml_io
cargo test --lib bpmn::file_io
cargo test --test bpmn_xml_tests
```

## Performance Considerations

1. **XML Serialization**: O(n) where n is number of elements
2. **Memory Usage**: Entire diagram loaded in memory
3. **File I/O**: Buffered reading/writing for efficiency
4. **Character Escaping**: Minimal overhead with string replacement

## Future Enhancements

1. **Streaming XML Parser**: For large diagrams
2. **XSD Validation**: Against BPMN 2.0 XSD schema
3. **XML Pretty Printing**: Formatted XML output
4. **Extension Support**: Custom elements and attributes
5. **Import Filters**: Selective element import
6. **Export Templates**: Customizable XML generation

## Standards Compliance

- BPMN 2.0.2 (ISO/IEC 19510)
- BPMN Diagram Interchange (DI) standard
- XML 1.0 specification
- Namespace handling per W3C standards

## References

- [OMG BPMN 2.0 Specification](http://www.omg.org/spec/BPMN/2.0/)
- [BPMN 2.0.2](https://www.omg.org/spec/BPMN/2.0.2/)
- [BPMN Reference](https://camunda.com/bpmn/reference/)
- [Diagram Interchange Standard](https://www.omg.org/spec/DI/1.0/)
