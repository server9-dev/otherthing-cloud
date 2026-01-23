# BPM+ Triple Threat Implementation

## Overview

The ABCDODAF library now fully implements the **BPM+ Triple Threat** - the three OMG (Object Management Group) standards that provide comprehensive business modeling capabilities:

1. **BPMN 2.0** (Business Process Model and Notation)
2. **CMMN 1.1** (Case Management Model and Notation)
3. **DMN 1.3** (Decision Model and Notation)

These standards work together to cover the full spectrum of business modeling needs:
- **BPMN** handles structured, repeatable processes
- **CMMN** handles unstructured, knowledge-intensive work
- **DMN** provides reusable decision logic for both

## Architecture

```
┌────────────────────────────────────────────────────────────┐
│                   BPM+ Triple Threat                       │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  ┌───────────────┐  ┌───────────────┐  ┌──────────────┐  │
│  │   BPMN 2.0    │  │   CMMN 1.1    │  │   DMN 1.3    │  │
│  ├───────────────┤  ├───────────────┤  ├──────────────┤  │
│  │ • Processes   │  │ • Cases       │  │ • Decisions  │  │
│  │ • Flow Elems  │  │ • Plan Items  │  │ • Tables     │  │
│  │ • Gateways    │  │ • Sentries    │  │ • Rules      │  │
│  │ • Tasks       │  │ • Milestones  │  │ • Hit Policy │  │
│  └───────┬───────┘  └───────┬───────┘  └──────┬───────┘  │
│          │                  │                  │          │
│          └──────────────────┴──────────────────┘          │
│                             │                             │
│              ┌──────────────┴──────────────┐              │
│              │   Integration Layer         │              │
│              │  • Cross-standard links     │              │
│              │  • Validation               │              │
│              │  • Builder patterns         │              │
│              └─────────────────────────────┘              │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

## Implementation Details

### 1. BPMN 2.0 Implementation (`bpm_plus/bpmn.rs`)

#### Supported Elements
- **Events**: Start Event, End Event
- **Tasks**:
  - User Task (human interaction)
  - Service Task (automated service calls)
  - Business Rule Task (DMN decision invocation)
  - Script Task (embedded scripts)
- **Gateways**:
  - Exclusive Gateway (XOR) - one path selection
  - Parallel Gateway (AND) - concurrent execution
  - Inclusive Gateway (OR) - multiple path selection
- **Flow**: Sequence Flow with conditional routing
- **Advanced**: Sub-Process, Call Activity (invoke CMMN cases)

#### Key Features
- Full process validation (start/end events, flow connectivity)
- BPMN 2.0 XML export capability
- Integration with DMN via Business Rule Tasks
- Integration with CMMN via Call Activities

#### Example Usage
```rust
let mut process = BpmnProcess::new("order_fulfillment", "Order Fulfillment");

// Add start event
process.add_flow_element(FlowElement::StartEvent {
    id: "start".to_string(),
    name: "Order Received".to_string(),
    outgoing: vec!["flow1".to_string()],
});

// Add business rule task (calls DMN)
process.add_flow_element(FlowElement::BusinessRuleTask {
    id: "check_credit".to_string(),
    name: "Check Credit (DMN)".to_string(),
    decision_ref: Some("credit_decision".to_string()),
    incoming: vec!["flow1".to_string()],
    outgoing: vec!["flow2".to_string()],
});

// Validate and export
process.validate()?;
let xml = process.to_bpmn_xml();
```

### 2. CMMN 1.1 Implementation (`bpm_plus/cmmn.rs`)

#### Supported Elements
- **Plan Items**:
  - Human Task (discretionary work)
  - Process Task (invoke BPMN process)
  - Case Task (invoke nested case)
  - Decision Task (invoke DMN decision)
  - Milestone (achievement marker)
  - Stage (grouping of plan items)
  - Event Listener (timer, user, signal events)
- **Sentries**: Entry/exit criteria guards
- **Case File**: Data context and items

#### Key Features
- Adaptive workflow with sentry-based activation
- Support for required, repeatable, and discretionary tasks
- Auto-complete stages
- Full sentry validation
- CMMN 1.1 XML export capability

#### Example Usage
```rust
let mut case = CmmnCase::new("customer_issue", "Customer Issue Resolution");

// Add sentry for conditional activation
case.add_sentry(Sentry {
    id: "investigation_complete".to_string(),
    name: "Investigation Done".to_string(),
    on_parts: vec![OnPart {
        source_ref: "investigate".to_string(),
        standard_event: StandardEvent::Complete,
    }],
    if_part: None,
});

// Add human task with entry criterion
case.add_plan_item(PlanItem::HumanTask {
    id: "investigate".to_string(),
    name: "Investigate Issue".to_string(),
    performer: Some("agent".to_string()),
    entry_criteria: vec![],
    exit_criteria: vec![],
    required: true,
    repeatable: false,
});

// Add milestone triggered by sentry
case.add_plan_item(PlanItem::Milestone {
    id: "resolved".to_string(),
    name: "Issue Resolved".to_string(),
    entry_criteria: vec!["investigation_complete".to_string()],
});
```

### 3. DMN 1.3 Implementation (`bpm_plus/dmn.rs`)

#### Supported Elements
- **Decision Types**:
  - Decision Table (most common)
  - Literal Expression
  - Invocation (call another decision)
  - Decision Service
- **Decision Tables**:
  - Input/Output clauses
  - Decision rules
  - Hit policies: First, Unique, RuleOrder, Collect, Any, OutputOrder
- **Data Types**: String, Number, Boolean, Date, Time, DateTime, Duration, Custom

#### Key Features
- Full decision table validation (input/output matching)
- Runtime decision execution with actual rule evaluation
- Support for FEEL expressions (basic)
- DMN 1.3 XML export capability
- Decision chaining (outputs feed into other decisions)

#### Example Usage
```rust
let decision_table = DecisionTable {
    hit_policy: HitPolicy::First,
    inputs: vec![
        InputClause {
            id: "i1".to_string(),
            label: "age".to_string(),
            input_expression: "customer.age".to_string(),
            input_values: None,
        },
    ],
    outputs: vec![
        OutputClause {
            id: "o1".to_string(),
            label: "Discount".to_string(),
            name: "discount".to_string(),
            output_values: Some(vec!["10%".to_string(), "20%".to_string()]),
        },
    ],
    rules: vec![
        DecisionRule {
            id: "r1".to_string(),
            input_entries: vec![">= 65".to_string()],
            output_entries: vec!["20%".to_string()],
            annotation: Some("Senior discount".to_string()),
        },
        DecisionRule {
            id: "r2".to_string(),
            input_entries: vec!["-".to_string()],
            output_entries: vec!["10%".to_string()],
            annotation: Some("Default".to_string()),
        },
    ],
};

let decision = DmnDecision::with_decision_table(
    "discount_decision",
    "Calculate Discount",
    decision_table,
);

// Execute at runtime
let mut inputs = HashMap::new();
inputs.insert("age".to_string(), serde_json::json!(70));
let result = decision.execute(&inputs)?;
// result = { "discount": "20%" }
```

### 4. Integration Layer (`bpm_plus/integration.rs`)

The integration layer provides seamless connectivity between all three standards:

#### StandardIntegration
Represents a cross-standard link with:
- **Source**: BPMN process, CMMN case, or DMN decision
- **Target**: Process task, case plan item, or decision input
- **Type**: Decision task, process task, case task, call activity, decision chain

#### BpmPlusBuilder
Fluent API for building integrated models:
```rust
let model = BpmPlusBuilder::new("AI Support", "Complete support system")
    .add_decision(priority_decision)
    .add_decision(assignment_decision)
    .add_process(triage_process)
    .add_case(complex_case)
    .link_decision_to_process("priority_decision", "triage", "decide_priority")
    .link_decision_to_case("routing_decision", "complex_case", "route_item")
    .build()?;
```

#### Validation
- Validates all cross-references exist
- Ensures decision tasks reference valid decisions
- Verifies process/case task targets exist
- Checks integration type compatibility

## Integration with DoDAF 2.02

The BPM+ triple threat integrates seamlessly with DoDAF 2.02:

- **BPMN processes** map to **Operational Activities** (OV-5)
- **CMMN cases** represent **Capability Services** (SvcV)
- **DMN decisions** define **Information Exchange** rules
- All three can reference **DoDAF architecture elements**

## Use Cases

### 1. Structured Process (BPMN)
- Order fulfillment
- Invoice processing
- Automated deployments
- Standard operating procedures

### 2. Adaptive Case Management (CMMN)
- Customer support escalations
- Medical diagnoses
- Legal case management
- Complex investigations

### 3. Decision Logic (DMN)
- Credit scoring
- Risk assessment
- Pricing rules
- Eligibility determination

### 4. AI Workforce Orchestration
The ABCDODAF library combines all three for AI agent coordination:

- **BPMN**: Main orchestration flow (ticket triage, request routing)
- **DMN**: Agent selection, priority determination, routing decisions
- **CMMN**: Complex, multi-step agent collaboration cases
- **DoDAF**: Overall architecture and capability mapping

## Example: AI Customer Support

See `examples/bpm_plus_triple_threat.rs` for a complete example showing:

1. **DMN Decisions**:
   - Ticket priority (based on urgency + impact)
   - Agent assignment (AI vs. human based on complexity)

2. **BPMN Process** (Ticket Triage):
   - Start: Ticket received
   - Service Task: AI classification
   - Business Rule Task: Determine priority (DMN)
   - Business Rule Task: Assign agent (DMN)
   - Exclusive Gateway: Route by priority
   - Call Activity: Create complex case (CMMN) for critical tickets
   - End: Ticket queued or case created

3. **CMMN Case** (Complex Inquiry):
   - Human Task: Initial investigation (required)
   - Decision Task: Determine resolution approach (DMN)
   - Stage: Execute resolution
     - Human Task: Implement solution
     - Human Task: Verify solution
   - Milestone: Issue resolved

## Test Coverage

All three standards have comprehensive test coverage:

- **BPMN**: 3 tests (creation, validation, XML export)
- **CMMN**: 2 tests (case creation, sentry validation)
- **DMN**: 3 tests (creation, table validation, execution)
- **Integration**: 2 tests (validation, builder pattern)

**Total: 10 tests, all passing ✅**

## XML Export

All three standards support export to their respective XML formats:

```rust
// BPMN 2.0 XML
let bpmn_xml = process.to_bpmn_xml();

// DMN 1.3 XML
let dmn_xml = decision.to_dmn_xml();

// CMMN 1.1 XML
let cmmn_xml = case.to_cmmn_xml();
```

The exported XML conforms to official OMG specifications:
- BPMN: `http://www.omg.org/spec/BPMN/20100524/MODEL`
- CMMN: `http://www.omg.org/spec/CMMN/20151109/MODEL`
- DMN: `https://www.omg.org/spec/DMN/20191111/MODEL/`

## Future Enhancements

Potential areas for expansion:

1. **BPMN**:
   - Event types (timer, message, signal, error)
   - Boundary events
   - Multi-instance tasks
   - Compensation handling

2. **CMMN**:
   - Planning table
   - Discretionary items
   - User event listeners
   - File item on-parts

3. **DMN**:
   - Full FEEL expression engine
   - Context and list expressions
   - Relation and invocation expressions
   - Decision requirement diagrams (DRD)

4. **Integration**:
   - Visual diagram rendering
   - Import from XML
   - Runtime execution engine
   - Monitoring and analytics

## Resources

- [BPMN 2.0 Specification](https://www.omg.org/spec/BPMN/2.0/)
- [CMMN 1.1 Specification](https://www.omg.org/spec/CMMN/1.1/)
- [DMN 1.3 Specification](https://www.omg.org/spec/DMN/1.3/)
- [DoDAF 2.02 Framework](https://dodcio.defense.gov/Library/DoD-Architecture-Framework/)

## Conclusion

The ABCDODAF library now provides a complete implementation of the BPM+ triple threat, enabling comprehensive business modeling for AI workforce orchestration. The three standards work together seamlessly:

- BPMN provides structure and flow
- CMMN provides flexibility and adaptation
- DMN provides consistent, reusable decision logic

Combined with DoDAF 2.02 architectural modeling, this creates a powerful framework for designing, documenting, and executing AI-powered business processes.
