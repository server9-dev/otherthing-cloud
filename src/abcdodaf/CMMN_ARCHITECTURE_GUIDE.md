# CMMN 1.1 Architecture & Integration Guide

## System Architecture Overview

```
┌──────────────────────────────────────────────────────────────────┐
│                      ABCDODAF Library                            │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────┐      ┌─────────────────┐                  │
│  │  BPMN Module    │      │  CMMN Module    │                  │
│  │  (Processes)    │      │  (Cases)        │                  │
│  │                 │      │                 │                  │
│  │ - Process Model │      │ - Case Model    │                  │
│  │ - Tasks         │◄────►│ - Plan Items    │                  │
│  │ - Flows         │      │ - Sentries      │                  │
│  │ - Gateways      │      │ - Case File     │                  │
│  │ - Executor      │      │                 │                  │
│  └─────────────────┘      └─────────────────┘                  │
│           │                       │                             │
│           │      Integration      │                             │
│           └───────────────────────┘                             │
│                                                                  │
│  ┌──────────────────────────────────────────────────┐           │
│  │        CMMN Runtime & Support Modules            │           │
│  ├──────────────────────────────────────────────────┤           │
│  │                                                  │           │
│  │  ┌────────────┐  ┌────────────┐  ┌──────────┐  │           │
│  │  │ Runtime    │  │ Discretio- │  │XML I/O   │  │           │
│  │  │ Engine     │  │ nary Items │  │(Ser/De)  │  │           │
│  │  │            │  │            │  │          │  │           │
│  │  │ - Instances│  │ - Manager  │  │ - Export │  │           │
│  │  │ - Lifecycle│  │ - Auth     │  │ - Import │  │           │
│  │  │ - Sentry   │  │ - Activation│ │ - Roundtrip│ │           │
│  │  │ - File Ops │  │ - History  │  │          │  │           │
│  │  └────────────┘  └────────────┘  └──────────┘  │           │
│  │                                                  │           │
│  └──────────────────────────────────────────────────┘           │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

## Module Structure

### 1. Core CMMN Module (`cmmn.rs`)
**Responsibility**: Data model definition and validation

```
CmmnCase
├── id: String
├── name: String
├── description: Option<String>
├── case_plan_model
│   ├── id: String
│   ├── plan_items: Vec<PlanItem>
│   └── sentries: Vec<Sentry>
└── case_file
    └── items: HashMap<String, CaseFileItem>

PlanItem (Enum)
├── HumanTask { ... }
├── ProcessTask { ... }
├── CaseTask { ... }
├── DecisionTask { ... }
├── Milestone { ... }
├── Stage { ... }
└── EventListener { ... }

Sentry
├── id: String
├── on_parts: Vec<OnPart>
└── if_part: Option<String>
```

**Key Operations**:
- Case creation and configuration
- Plan item management
- Sentry definition
- Case file item setup
- Full validation

### 2. Runtime Engine (`cmmn_runtime.rs`)
**Responsibility**: Case execution and instance management

```
CaseRuntimeEngine
├── instances: Arc<RwLock<HashMap<String, CaseInstance>>>
└── case_models: Arc<RwLock<HashMap<String, CmmnCase>>>

CaseInstance
├── id: String
├── state: CaseInstanceState
├── plan_items: HashMap<String, PlanItemInstance>
├── case_file: HashMap<String, Value>
└── event_history: Vec<CaseEvent>

SentryEvaluator
├── evaluate_sentry(sentry, context) -> bool
├── evaluate_on_part(on_part, context) -> bool
└── evaluate_condition(condition, context) -> bool
```

**State Machine**:
```
Available ─→ Enabled ─→ Active ─→ Completed
    ↓         ↓         ↓
  Reset    Disabled  Suspended
              ↓         ↓
            Available  Terminated/Failed
```

**Key Operations**:
- Register case models
- Start case instances
- Create plan items
- Manage plan item lifecycle
- Update case file data
- Evaluate sentries
- Track audit trail
- Detect completion

### 3. Discretionary Items (`cmmn_discretionary.rs`)
**Responsibility**: User-controlled optional activities

```
DiscretionaryItemManager
├── items: HashMap<String, DiscretionaryItem>
├── groups: HashMap<String, DiscretionaryItemGroup>
├── activations: Vec<DiscretionaryActivation>
└── policies: HashMap<String, ActivationPolicy>

DiscretionaryItem
├── id: String
├── plan_item: PlanItem
├── is_available: bool
├── authorized_roles: Vec<String>
└── added_at: DateTime

ActivationPolicy
├── Restricted { required_role }
├── Authorized { roles }
├── Open
└── Contextual { condition }
```

**Key Operations**:
- Add discretionary items
- Check authorization
- Activate items
- Track activations
- Manage item groups
- Enable/disable items
- Control availability

### 4. XML Serialization (`cmmn_xml.rs`)
**Responsibility**: Standard format import/export

**Export Path**:
```
CmmnCase
    ↓
CmmnXmlExporter::to_xml()
    ↓
XML Document (CMMN 1.1 compliant)
```

**Import Path**:
```
XML Document
    ↓
CmmnXmlImporter::from_xml()
    ↓
CmmnCase
```

**Standard Output**:
```xml
<definitions xmlns="http://www.omg.org/spec/CMMN/20151109/MODEL">
  <case id="..." name="...">
    <casePlanModel id="..." name="...">
      <planItem id="..." name="..." definitionRef="..." />
      <sentry id="..." name="...">
        <onPart>
          <standardEvent>Complete</standardEvent>
          <source ref="..." />
        </onPart>
      </sentry>
    </casePlanModel>
    <caseFileModel>
      <caseFileItem id="..." name="..." multiplicity="..." />
    </caseFileModel>
  </case>
</definitions>
```

### 5. BPMN Integration (`bpmn_cmmn_integration.rs`)
**Responsibility**: Cross-domain orchestration

```
BpmnCmmnIntegration
├── case_engine: CaseRuntimeEngine
└── case_calls: HashMap<String, CaseCallInstance>

CaseCallActivity
├── id: String
├── case_ref: String
├── input_mappings: HashMap<String, String>
├── output_mappings: HashMap<String, String>
└── wait_for_completion: bool

CaseCallInstance
├── id: String
├── process_instance_id: Uuid
├── case_instance_id: String
├── status: CallStatus
└── error: Option<String>

CallStatus
├── Started
├── Running
├── Completed
├── Terminated
└── Failed
```

**Integration Flow**:
```
┌─────────────────────────────────┐
│  BPMN Process Instance          │
│  ┌──────────────────────────┐   │
│  │ Call Activity Task       │   │
│  │ (Case Call Activity)     │   │
│  └──────────────────────────┘   │
└─────────┬───────────────────────┘
          │
          │ execute_case_call()
          │ Input Mapping
          ▼
┌─────────────────────────────────┐
│  CMMN Case Instance             │
│  ┌──────────────────────────┐   │
│  │ Plan Items               │   │
│  │ Case File (Mapped Data)  │   │
│  └──────────────────────────┘   │
│  [Case Executes]                │
└─────────┬───────────────────────┘
          │
          │ check_case_call_status()
          │ Output Mapping
          ▼
┌─────────────────────────────────┐
│  BPMN Process Instance          │
│  ┌──────────────────────────┐   │
│  │ Post-Call Task           │   │
│  │ (With Case Results)      │   │
│  └──────────────────────────┘   │
└─────────────────────────────────┘
```

**Variable Mapping**:
```
Process Side              Case Side
─────────────────────────────────────
request_priority  ───→  case_priority
customer_id       ───→  customer
                   ←──  result
                   ←──  resolution_status
```

## Execution Flow Examples

### Example 1: Simple Case Execution

```
START
  ├─ Create CmmnCase model
  ├─ Register with CaseRuntimeEngine
  ├─ Start instance
  ├─ Create plan items
  │  ├─ Task1 (Available)
  │  └─ Milestone1 (Available)
  ├─ Activate Task1 (Enabled)
  ├─ Start Task1 (Active)
  ├─ Complete Task1 (Completed)
  │  ├─ Evaluate sentries
  │  └─ Milestone1 entry sentry fires
  ├─ Activate Milestone1 (Enabled)
  ├─ Complete Milestone1 (Completed)
  └─ Case completes (all required items done)
END
```

### Example 2: Discretionary Item Activation

```
DURING CASE EXECUTION
  ├─ Case is executing
  ├─ Case worker sees optional QA review available
  ├─ Check authorization (QA team member)
  ├─ User activates QA review
  ├─ DiscretionaryItemManager:
  │  ├─ Record activation
  │  ├─ Create plan item for QA task
  │  └─ Mark item unavailable (non-repeatable)
  ├─ QA task executes
  └─ Continue case execution
```

### Example 3: BPMN Calling CMMN

```
BPMN PROCESS
  ├─ Start process
  ├─ ... execute tasks ...
  ├─ Call Activity: "Handle Complex Case"
  │  ├─ Map process variables to case file
  │  │  ├─ request_priority → case_priority
  │  │  └─ customer_id → customer
  │  ├─ Start CMMN case instance
  │  ├─ Monitor execution
  │  │  └─ Check status periodically
  │  ├─ Case completes
  │  ├─ Map outputs to process
  │  │  └─ case_result → result_variable
  │  └─ Return to process
  ├─ ... execute remaining tasks ...
  └─ End process
```

## Data Flow Diagrams

### Case Instance State Transitions

```
Planning Phase
───────────────────────────────────────
Available items → Discretionary selection → Item grouping

Execution Phase
───────────────────────────────────────
Enabled → Active → Complete/Terminate
   ↑        ↑
   └─ Suspended ← Resume

Monitoring Phase
───────────────────────────────────────
Sentry Evaluation → Event Triggering → Condition Check
   ↓
Case File Updates → Audit Trail Recording

Completion Phase
───────────────────────────────────────
All Required Items Done → Case Completed
Event History → Audit Report
```

### Variable Transformation

```
BPMN Process Variables
    ↓ (on case call)
[Input Mapping]
    ↓
Case File Items
    ↓ (during execution)
[Case Processing]
    ↓
Modified Case File
    ↓ (on case complete)
[Output Mapping]
    ↓
BPMN Process Variables (Updated)
```

## Concurrency Model

```
CaseRuntimeEngine (Arc<RwLock<>>)
    ├─ instances (RwLock)
    │  ├─ Read: get_case_instance, evaluate_sentries
    │  └─ Write: create_plan_item, update_case_file, change_state
    └─ case_models (RwLock)
       ├─ Read: case lookups
       └─ Write: register_case

Tokio Async Runtime
    ├─ All operations are non-blocking
    ├─ Multiple cases execute concurrently
    └─ Safe state sharing via Arc<RwLock<>>
```

## Error Handling

```
Operation Errors
├─ Case model not found
├─ Case instance not found
├─ Plan item not found
├─ Authorization denied
├─ State transition invalid
├─ Validation failed
└─ Condition evaluation error

Error Propagation
├─ Result<T> for all fallible operations
├─ ? operator for clean error flow
├─ Custom AbcdodafError::WorkflowError
└─ Detailed error messages for debugging
```

## Performance Characteristics

### Time Complexity
- Case registration: O(1)
- Instance creation: O(1)
- Plan item operations: O(1)
- Sentry evaluation: O(n) where n = on_parts count
- Case file operations: O(1)

### Space Complexity
- Instance: O(m + n) where m = plan items, n = case file items
- Case model: O(p + s) where p = plan items, s = sentries
- Runtime engine: O(i × m) where i = instances, m = avg items per instance

### Async Performance
- Non-blocking I/O
- Concurrent case execution
- Tokio efficient task scheduling
- RwLock allows multiple concurrent readers

## Testing Strategy

### Unit Tests (Per Module)
```
cmmn.rs
├─ Case creation
├─ Plan item validation
├─ Sentry validation
└─ Query operations

cmmn_runtime.rs
├─ Instance lifecycle
├─ State transitions
├─ Sentry evaluation
├─ Case file operations
└─ Completion detection

cmmn_discretionary.rs
├─ Item creation
├─ Authorization
├─ Activation workflow
└─ Grouping

cmmn_xml.rs
├─ Export correctness
├─ Import parsing
├─ Roundtrip integrity
└─ Special characters

bpmn_cmmn_integration.rs
├─ Call activity creation
├─ Variable mapping
├─ Status tracking
└─ Output extraction
```

### Integration Tests
- Multi-module scenarios
- BPMN-CMMN orchestration
- End-to-end workflows
- Error recovery

## Extension Points

### Adding New Plan Item Type

1. Add variant to `PlanItem` enum
2. Update `CmmnCase::has_plan_item()`
3. Implement XML export in `CmmnXmlExporter`
4. Implement XML import in `CmmnXmlImporter`
5. Add lifecycle handling if needed
6. Write tests

### Adding New Standard Event

1. Add variant to `StandardEvent` enum
2. Update `SentryEvaluator::evaluate_on_part()`
3. Document behavior
4. Add tests

### Custom Authorization Policy

1. Extend `ActivationPolicy` enum
2. Update authorization check
3. Implement policy evaluation
4. Add activation examples

## Best Practices

### Case Design
- Keep cases focused (single responsibility)
- Define clear entry criteria
- Use stages for logical grouping
- Document plan item purposes

### Sentry Design
- Combine on_parts for flexibility
- Use if_part for complex conditions
- Avoid circular sentry dependencies
- Test sentry combinations

### Variable Mapping (BPMN→CMMN)
- Use consistent naming conventions
- Map only necessary variables
- Document data transformations
- Validate input constraints

### Discretionary Items
- Provide clear descriptions
- Set appropriate authorization levels
- Consider item grouping
- Track activation reasons

### Error Handling
- Check case registration before execution
- Validate case models before use
- Handle authorization failures gracefully
- Log all errors for audit trail

## Deployment Considerations

### Production Setup
1. Register all case models on startup
2. Implement persistence layer
3. Set up monitoring/logging
4. Configure authorization system
5. Plan capacity for concurrent cases

### Scaling
- Horizontal: Multiple process instances
- Database: Store case instances
- Cache: Frequently accessed cases
- Events: Distributed event handling

### Monitoring
- Case execution metrics
- Sentry evaluation timing
- Discretionary activation tracking
- Authorization audit logs

## Related Standards

- **CMMN 1.1**: OMG Case Management standard
- **BPMN 2.0**: Process orchestration
- **DMN 1.2**: Decision logic
- **FEEL**: Expression language (future)

## References

- `/CMMN_IMPLEMENTATION.md` - Detailed implementation guide
- `/TASK_4_CMMN_COMPLETION.md` - Completion summary
- OMG CMMN 1.1 Specification: https://www.omg.org/spec/CMMN/1.1/

---

**Architecture Version**: 1.0
**Last Updated**: January 2026
**Status**: Production Ready
