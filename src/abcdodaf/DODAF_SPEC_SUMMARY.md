# DoDAF 2.02 Specification Summary

## Overview
DoDAF 2.02 (Department of Defense Architecture Framework) is a comprehensive framework for modeling operational scenarios, activities, and resource flows, particularly useful for AI workforce orchestration.

## Operational View (OV) Products

### Core OV Products:
1. **OV-1**: High-Level Operational Concept Graphic
2. **OV-2**: Operational Resource Flow Description
3. **OV-3**: Operational Resource Flow Matrix
4. **OV-4**: Organizational Relationships Chart
5. **OV-5**: Operational Activity Model
   - **OV-5a**: Operational Activity Decomposition Tree
   - **OV-5b**: Operational Activity Model (flow-based)
6. **OV-6**: Operational Rules/Sequence Models
   - **OV-6a**: Operational Rules Model
   - **OV-6b**: State Transition Description
   - **OV-6c**: Event-Trace Description

## OV-5 Operational Activity Model (Key for ABCDODAF)

### Purpose
Describes operations conducted to achieve mission/business goals. Depicts:
- Operational activities (tasks)
- Input/output flows between activities
- Flows to/from activities outside scope

### Characteristics
- **OV-5a**: Hierarchical decomposition tree (parent-child relationships)
- **OV-5b**: Flow-based representation (sequences and interactions)

### Usage
- Delineates lines of responsibility
- Uncovers redundancy
- Supports streamlining decisions
- Identifies activities requiring scrutiny

## Data Structures (DM2 Meta-Model)

### Three Layers
1. **Conceptual Data Model (CDM)** - High-level concepts
2. **Logical Data Model (LDM)** - Technical details and attributes
3. **Physical Exchange Specification (PES)** - Implementation specs

### Key Entities

**Activity Entity:**
- Synonymous with Tasks
- Transformation producing new resources from existing ones
- Central to DM2 metamodel

**Performer Entity:**
- Types: Person, Organization, Service, ServiceInterface, System, Interface
- All inherit from Performer base class

**Resource Flow Exchange:**
- Behavioral/structural interactions between Activities
- Temporal flow/exchange of objects (information, data, materiel, performers)

## Required Attributes for Operational Activities

### Core Attributes
- **Name**: Unique identifier/label
- **Description**: Detailed explanation of purpose and execution
- **Performer**: Entity performing the activity

### Additional Attributes
- **Cost**: Dollar costs
- **Duration**: Time required
- **Frequency**: How often performed
- **Security Domain**: Classification requirements
- **Location**: Where performed

### Relationship Attributes
- **Input Resources**: Resources consumed
- **Output Resources**: Resources produced
- **Predecessor Activities**: Must complete before
- **Successor Activities**: Follow this activity
- **Constraints**: Business rules limiting activity

### Traceability Attributes
- **Capabilities**: Links to CV-6
- **Systems/Services**: Links to SV-5a
- **Information Requirements**: Links to DIV

## Resource Flows

### Types
- Information (data, messages)
- Funding (budget, financial)
- Personnel (human resources)
- Materiel (physical goods, equipment)

### Attributes (OV-3)
- Timeliness
- Availability
- Protective marking (security)
- Non-repudiation (authentication)
- Quality
- Quantity
- Media
- Interoperability level

## Activity Decomposition

### Hierarchical Structure
- Mission-level activities → Major operational processes → Specific tasks → Sub-tasks
- OV-5a provides tree structure
- Maintains logical coherence at each level

### Relationships
- Maps to OV-2 resource flows
- Maps to CV-6 capabilities
- Traces to SV-5a system functions

## DoDAF + BPMN Integration

### Official Support
- **OV-6c can be developed using BPMN 2.0**
- DoDAF Persona (Conformance Sub-Class) of BPMN 2.0 exists
- Standard format for notation and exchange

### Integration Benefits
- **Standardization**: Common notation
- **Interoperability**: Federated architectures
- **Executable Models**: BPMN 2.0 → WS-BPEL → BPMS
- **Tool Support**: Multiple tools support BPMN visualization of DM2

### Transformation Path
```
DoDAF OV-6c → BPMN 2.0 → WS-BPEL → BPMS Execution
```

### BPMN Primitives for DoDAF
- Subset of BPMN constructs sufficient for:
  - Requirements engineering (OV-6c)
  - Systems engineering (SvcV-10c, SV-10c)
- Some features (compensation activities) can be eliminated

## Implementation for ABCDODAF

### Type System Requirements

1. **Operational Activity Type** should include:
   ```rust
   struct OperationalActivity {
       id: String,
       name: String,
       description: String,
       performer: Performer,
       inputs: Vec<ResourceFlow>,
       outputs: Vec<ResourceFlow>,
       cost: Option<Cost>,
       duration: Option<Duration>,
       frequency: Option<Frequency>,
       security_domain: Option<SecurityClassification>,
       location: Option<Location>,
       predecessors: Vec<ActivityId>,
       successors: Vec<ActivityId>,
       constraints: Vec<BusinessRule>,
       capabilities: Vec<CapabilityId>,
       metadata: HashMap<String, Value>,
   }
   ```

2. **Resource Flow Type**:
   ```rust
   struct ResourceFlow {
       id: String,
       name: String,
       flow_type: FlowType, // Information, Funding, Personnel, Materiel
       source_activity: ActivityId,
       target_activity: ActivityId,
       attributes: FlowAttributes {
           timeliness: Option<String>,
           availability: Option<f64>,
           protective_marking: Option<Classification>,
           quality: Option<QualityMetrics>,
           quantity: Option<Quantity>,
       },
   }
   ```

3. **Performer Type**:
   ```rust
   enum Performer {
       Person(PersonPerformer),
       Organization(OrgPerformer),
       Service(ServicePerformer),
       System(SystemPerformer),
       Interface(InterfacePerformer),
   }
   ```

### BPMN Mapping
- BPMN Tasks → DoDAF Activities
- BPMN Sequence Flows → DoDAF Activity Flows
- BPMN Data Objects → DoDAF Resource Flows
- BPMN Pools/Lanes → DoDAF Performers/Organizations
- BPMN Gateways → DoDAF Decision Points

### OV-5 Visualization Strategy
- Use egui-snarl for OV-5b (activity flow model)
- Node types represent activities
- Edges represent resource flows
- Node properties include DoDAF attributes
- Support hierarchical decomposition (OV-5a) via nested views

## References
- [DoDAF 2.02 Official](https://dodcio.defense.gov/library/dod-architecture-framework/)
- [DoDAF Meta-Model (DM2)](https://dodcio.defense.gov/Library/DoD-Architecture-Framework/dodaf20_dm2/)
- [BPMN for DoD Enterprise](https://www.bpminstitute.org/resources/articles/modeling-net-centric-department-defense-dod-enterprise-using-bpmn-20)
- [OV-5 Operational Activity Model](https://dodcio.defense.gov/Library/DoD-Architecture-Framework/dodaf20_ov5ab/)
