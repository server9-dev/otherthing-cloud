# CMMN 1.1 Implementation for ABCDODAF

## Overview

This document describes the CMMN (Case Management Model and Notation) 1.1 implementation integrated into the ABCDODAF library. CMMN complements BPMN by providing mechanisms for managing knowledge-intensive, unstructured work that adapts based on context.

## Core Components

### 1. Case Model (`cmmn.rs`)

The base data model for CMMN cases with the following elements:

#### CmmnCase
- **id**: Unique case identifier
- **name**: Human-readable case name
- **description**: Optional case description
- **case_plan_model**: The plan definition for this case
- **case_file**: Data context for the case

#### PlanItem (Enum)
Represents executable elements within a case:

- **HumanTask**: Manual activity requiring human intervention
  - `performer`: Assigned role/user
  - `required`: Whether item must be completed
  - `repeatable`: Can be executed multiple times
  - Entry/exit criteria for conditional activation

- **ProcessTask**: Calls an external BPMN process
  - `process_ref`: Reference to BPMN process model
  - Can be required or optional

- **CaseTask**: Calls another CMMN case (nested)
  - `case_ref`: Reference to another case model

- **DecisionTask**: Evaluates a DMN decision
  - `decision_ref`: Reference to DMN decision model

- **Milestone**: Achievable goal/state in the case
  - Represents a significant point in case execution
  - Has entry criteria but no explicit execution

- **Stage**: Grouping container for related plan items
  - Can auto-complete when all items complete
  - Provides hierarchical organization

- **EventListener**: Responds to external events
  - `event_type`: Timer, User signal, or Signal

#### Sentry
Guards that control plan item lifecycle through conditions:

- **on_parts**: Event-based triggers
  - StandardEvent: Create, Enable, Disable, Start, Complete, Terminate, Suspend, Resume
  - source_ref: Plan item that triggers event

- **if_part**: Optional condition expression
  - Evaluated against case file data
  - Both on_parts AND if_part must be true

#### CaseFile
Data context for the case:

- **CaseFileItem**: Represents a data item
  - `name`: Item identifier
  - `definition_ref`: Optional reference to definition
  - `multiplicity`:
    - ZeroOrOne: At most one
    - ExactlyOne: Must exist
    - ZeroOrMore: Any number
    - OneOrMore: At least one

### 2. Case Instance Runtime (`cmmn_runtime.rs`)

Implements execution semantics for CMMN cases:

#### PlanItemState (Lifecycle)
```
Available → Enabled → Active → Completed
             ↓         ↓         ↓
          Available  Suspended Terminated/Failed
```

States:
- **Available**: Initial state, not yet enabled
- **Enabled**: Ready for execution
- **Active**: Currently executing
- **Suspended**: Paused, can be resumed
- **Completed**: Successfully finished
- **Terminated**: Ended without completion
- **Failed**: Execution error

#### CaseInstance
Represents a running case:

```rust
pub struct CaseInstance {
    pub id: String,
    pub case_model_id: String,
    pub state: CaseInstanceState,
    pub plan_items: HashMap<String, PlanItemInstance>,
    pub case_file: HashMap<String, serde_json::Value>,
    pub event_history: Vec<CaseEvent>,
}
```

#### CaseRuntimeEngine
Manages case execution:

```rust
let engine = CaseRuntimeEngine::new();

// Register case model
engine.register_case(case_model).await?;

// Start new instance
let instance_id = engine.start_case("case_model_id").await?;

// Manage plan items
engine.create_plan_item(&instance_id, plan_item_id).await?;
engine.activate_plan_item(&instance_id, &item_instance_id).await?;
engine.start_plan_item(&instance_id, &item_instance_id).await?;
engine.complete_plan_item(&instance_id, &item_instance_id).await?;

// Update case data
engine.update_case_file(&instance_id, "key", value).await?;

// Evaluate sentries
let fired = engine.evaluate_sentries(&instance_id, &case_model).await?;
```

#### Sentry Evaluation
`SentryEvaluator` evaluates entry/exit criteria:

1. Check all on_parts (events)
2. Check if_part (condition)
3. Both must be satisfied for sentry to fire

Event evaluation checks plan item state transitions:
- Complete: Item reached Completed state
- Start: Item transitioned to Active
- Enable: Item transitioned to Enabled
- etc.

### 3. Discretionary Items (`cmmn_discretionary.rs`)

Enables case workers to dynamically add optional activities:

#### DiscretionaryItem
Optional plan item available for manual activation:

```rust
pub struct DiscretionaryItem {
    pub id: String,
    pub name: String,
    pub plan_item: PlanItem,
    pub is_available: bool,
    pub authorized_roles: Vec<String>,
    pub added_at: chrono::DateTime<chrono::Utc>,
}
```

#### DiscretionaryItemManager
Manages discretionary items and activations:

```rust
let mut manager = DiscretionaryItemManager::new();

// Add discretionary item
manager.add_discretionary_item(item);

// Check authorization
if manager.can_activate("item_id", user_role)? {
    // Activate item
    let activation = manager.activate_item(
        "item_id",
        user_id,
        user_role,
        reason,
    )?;
}

// List available items
let available = manager.list_available_items();

// Disable/enable items
manager.disable_item("item_id")?;
manager.enable_item("item_id")?;
```

Authorization:
- **Restricted**: Only specific role can activate
- **Authorized**: List of authorized roles
- **Open**: Any case worker can activate
- **Contextual**: Activation depends on case data

#### Activation History
All activations are tracked for audit purposes:

```rust
pub struct DiscretionaryActivation {
    pub id: String,
    pub discretionary_item_id: String,
    pub activated_by: String,
    pub activated_at: chrono::DateTime<chrono::Utc>,
    pub reason: Option<String>,
    pub properties: HashMap<String, serde_json::Value>,
}
```

### 4. XML Import/Export (`cmmn_xml.rs`)

Serialize and deserialize CMMN to/from standard XML format:

#### Export
```rust
let xml = CmmnXmlExporter::to_xml(&case)?;
```

Produces CMMN 1.1 compliant XML:
- Standard namespace: `http://www.omg.org/spec/CMMN/20151109/MODEL`
- Plan items as `<planItem>` elements
- Sentries as `<sentry>` elements
- Case file as `<caseFileModel>`

#### Import
```rust
let case = CmmnXmlImporter::from_xml(xml_content)?;
```

Parses XML back to case model:
- Handles multiple plan item types
- Reconstructs sentry relationships
- Preserves case file structure

### 5. BPMN-CMMN Integration (`bpmn_cmmn_integration.rs`)

Enables bidirectional process-case interaction:

#### Call CMMN from BPMN
```rust
let call_activity = CaseCallActivityBuilder::new("call1", "Execute Case", "case_id")
    .add_input_mapping("case_priority", "request_priority")
    .add_output_mapping("result", "case_result")
    .wait_for_completion(true)
    .build();
```

Variables are automatically mapped:
- **Input mappings**: BPMN variables → Case file items
- **Output mappings**: Case file items → BPMN variables

#### Case Call Instance
Tracks execution of case invoked from BPMN:

```rust
pub struct CaseCallInstance {
    pub id: String,
    pub call_activity_id: String,
    pub process_instance_id: uuid::Uuid,
    pub case_instance_id: String,
    pub status: CallStatus, // Started, Running, Completed, Terminated, Failed
}
```

#### Integration Engine
```rust
let integration = BpmnCmmnIntegration::new(case_engine);

// Execute case from BPMN
let call_instance = integration.execute_case_call(
    &call_activity,
    &process_instance
).await?;

// Check status
let updated = integration.check_case_call_status(
    &call_id,
    &call_activity
).await?;

// Get outputs when complete
let outputs = integration.get_case_call_output(
    &call_id,
    &call_activity
).await?;
```

## Usage Examples

### Example 1: Simple Case Creation

```rust
use abcdodaf::bpm_plus::*;

let mut case = CmmnCase::new("customer_service", "Customer Service Case");

// Add a required human task
case.add_plan_item(PlanItem::HumanTask {
    id: "accept_task".to_string(),
    name: "Accept Customer Inquiry".to_string(),
    performer: Some("agent".to_string()),
    documentation: Some("Initial assessment".to_string()),
    entry_criteria: vec![],
    exit_criteria: vec![],
    required: true,
    repeatable: false,
});

// Add a milestone
case.add_plan_item(PlanItem::Milestone {
    id: "resolved".to_string(),
    name: "Case Resolved".to_string(),
    entry_criteria: vec!["sentry_resolved".to_string()],
});

// Add a sentry
case.add_sentry(Sentry {
    id: "sentry_resolved".to_string(),
    name: "On Task Complete".to_string(),
    on_parts: vec![OnPart {
        source_ref: "accept_task".to_string(),
        standard_event: StandardEvent::Complete,
    }],
    if_part: None,
});
```

### Example 2: Case Instance Execution

```rust
let runtime = CaseRuntimeEngine::new();
runtime.register_case(case_model).await?;

// Start case
let instance_id = runtime.start_case("customer_service").await?;

// Create and manage plan items
let task_id = runtime.create_plan_item(&instance_id, "accept_task".to_string()).await?;

// Activate for execution
runtime.activate_plan_item(&instance_id, &task_id).await?;
runtime.start_plan_item(&instance_id, &task_id).await?;

// Update case data
runtime.update_case_file(&instance_id, "priority".to_string(), serde_json::json!(5)).await?;

// Complete the task
runtime.complete_plan_item(&instance_id, &task_id).await?;

// Get instance
let instance = runtime.get_case_instance(&instance_id).await?;
assert_eq!(instance.state, CaseInstanceState::Completed);
```

### Example 3: Discretionary Items

```rust
let mut manager = DiscretionaryItemManager::new();

let qa_review = DiscretionaryItemBuilder::new("qa_review", "QA Review")
    .with_description("Optional quality assurance")
    .with_plan_item(PlanItem::HumanTask {
        id: "qa_task".to_string(),
        name: "Review".to_string(),
        performer: Some("qa_team".to_string()),
        documentation: None,
        entry_criteria: vec![],
        exit_criteria: vec![],
        required: false,
        repeatable: false,
    })
    .add_authorized_role("qa_team")
    .add_authorized_role("supervisor")
    .build()?;

manager.add_discretionary_item(qa_review);

// User activates the discretionary item
if manager.can_activate("qa_review", "qa_team")? {
    manager.activate_item(
        "qa_review",
        "user_123".to_string(),
        "qa_team".to_string(),
        Some("High priority case".to_string()),
    )?;
}
```

### Example 4: XML Export/Import

```rust
// Export to XML
let xml = CmmnXmlExporter::to_xml(&case_model)?;

// Save to file
std::fs::write("case.cmmn.xml", xml)?;

// Import from file
let xml_content = std::fs::read_to_string("case.cmmn.xml")?;
let imported_case = CmmnXmlImporter::from_xml(&xml_content)?;
```

### Example 5: BPMN-CMMN Integration

```rust
// From BPMN process, call CMMN case
let call_activity = CaseCallActivityBuilder::new(
    "call_case",
    "Call Customer Service Case",
    "customer_service"
)
.add_input_mapping("case_priority", "request_priority")
.add_input_mapping("case_customer", "customer_id")
.add_output_mapping("resolution", "case_resolution")
.wait_for_completion(true)
.build();

let integration = BpmnCmmnIntegration::new(case_engine);

// Execute from BPMN
let call_instance = integration.execute_case_call(
    &call_activity,
    &process_instance
).await?;

// Monitor execution
loop {
    let status = integration.check_case_call_status(
        &call_instance.id,
        &call_activity
    ).await?;

    if status.status == CallStatus::Completed {
        // Get outputs
        let outputs = integration.get_case_call_output(
            &call_instance.id,
            &call_activity
        ).await?;
        break;
    }
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
}
```

## Key Design Decisions

### 1. Async/Await for Concurrency
- All runtime operations are async using Tokio
- Enables non-blocking case instance execution
- Supports concurrent cases in production

### 2. Type-Safe State Transitions
- PlanItemState enum ensures valid transitions
- Compiler enforces correct state handling
- Prevents invalid state combinations

### 3. Flexible Plan Items
- Enum-based design allows extensibility
- Each item type has appropriate properties
- Easy to add new item types

### 4. Audit Trail
- All events recorded in case instance
- Complete history of case execution
- Supports compliance and debugging

### 5. Separation of Concerns
- Runtime engine handles execution
- Discrete modules for specific concerns
- Clean integration points with BPMN

## Limitations and Future Enhancements

### Current Limitations
1. Condition expressions use simplified parsing
   - Future: Use proper expression language (FEEL)
2. No persistence layer
   - Future: Add database backend for durability
3. No event correlation
   - Future: Advanced event matching and correlation
4. Limited timing constraints
   - Future: Proper timer event handling

### Future Enhancements
1. **FEEL (Friendly Enough Expression Language)**
   - Full expression evaluation
   - Complex conditions and transformations

2. **Persistence**
   - Database backend for cases
   - Event sourcing
   - Snapshots for performance

3. **Advanced Monitoring**
   - Real-time case analytics
   - Performance metrics
   - Audit reporting

4. **Enhanced Discretionary Items**
   - ML-based suggestions
   - Contextual item recommendation
   - Predictive activation

5. **Integration Improvements**
   - Full BPMN-CMMN orchestration
   - Data binding and expressions
   - Cross-boundary transactions

## Testing

Comprehensive test suites cover:
- Case model validation
- Sentry evaluation scenarios
- Plan item lifecycle transitions
- Discretionary item activation
- XML serialization roundtrips
- BPMN-CMMN integration scenarios

Run tests with:
```bash
cargo test --lib bpm_plus::
```

## References

- OMG CMMN 1.1 Specification: https://www.omg.org/spec/CMMN/1.1/
- BPMN 2.0 Standard
- DMN 1.2 (Decision Model and Notation)

## Architecture Diagram

```
┌─────────────────────────────────────────────────────┐
│                  CMMN Case Model                    │
│  ┌────────────┐  ┌────────────┐  ┌──────────────┐  │
│  │ Plan Items │  │  Sentries  │  │  Case File   │  │
│  └────────────┘  └────────────┘  └──────────────┘  │
└──────────────────────┬──────────────────────────────┘
                       │
        ┌──────────────┴──────────────┐
        │                             │
   ┌────▼─────────┐          ┌───────▼────┐
   │  Runtime     │          │  XML I/O   │
   │  Engine      │          │  (Ser/De)  │
   └────┬─────────┘          └────────────┘
        │
        ├─────────┬────────────────┬───────────┐
        │         │                │           │
   ┌────▼─┐  ┌────▼─────┐  ┌──────▼──┐  ┌────▼──┐
   │Sentry│  │Discretio-│  │BPMN-CMMN│  │Event  │
   │Eval. │  │nary Items│  │Integr.  │  │Mgmt.  │
   └──────┘  └──────────┘  └─────────┘  └───────┘
```

## Contributing

To extend CMMN implementation:

1. Add new plan item types in `cmmn.rs`
2. Implement lifecycle in `cmmn_runtime.rs`
3. Add tests for new functionality
4. Update XML support in `cmmn_xml.rs`
5. Document in this file

## License

MIT or Apache 2.0 (same as ABCDODAF library)
