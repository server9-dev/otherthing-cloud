# DoDAF 2.02 Expansion Design Document

## Task #2: Expand DoDAF 2.02 Views Beyond OV-5

**Status**: Research and Design Phase
**Date**: 2026-01-27

---

## Executive Summary

This document outlines the comprehensive expansion of the ABCDODAF library to support multiple DoDAF 2.02 viewpoints beyond the currently implemented OV-5 (Operational Activity Decomposition). The expansion includes Operational Views (OV-1, OV-2, OV-3, OV-6a/b/c), Systems Views (SV-1, SV-2, SV-4), and Capability Views (CV-1, CV-2).

---

## 1. Research Findings

### 1.1 DoDAF 2.02 Viewpoint Overview

DoDAF (Department of Defense Architecture Framework) 2.02 provides a comprehensive set of views for describing architecture from different perspectives:

#### Operational Viewpoint (OV)
- **OV-1**: High-Level Operational Concept Graphic
- **OV-2**: Operational Node Connectivity & Resource Flow
- **OV-3**: Operational Information Exchange Matrix
- **OV-5**: Operational Activity Model (IMPLEMENTED)
- **OV-6a**: Operational Rules Model
- **OV-6b**: Operational State Transition Description
- **OV-6c**: Operational Event-Trace Description

#### Systems Viewpoint (SV)
- **SV-1**: Systems Interface Description
- **SV-2**: Systems Resource Flow Description
- **SV-4**: Systems Functionality Description

#### Capability Viewpoint (CV)
- **CV-1**: Vision (Strategic Context)
- **CV-2**: Capability Taxonomy

### 1.2 Current Implementation Status

**Implemented:**
- OV-5: Full implementation with activities, performers, resource flows, and business rules
- OV-6c: Event-trace description partially implemented
- OV-2: Operational nodes defined in OV-5
- Basic capability and services views

**To Implement:**
- OV-1: High-level operational concept visualization and metadata
- OV-2: Full operational node connectivity model
- OV-3: Resource flow matrix with attributes
- OV-6a: Business rules and constraints formalization
- OV-6b: State transition diagrams
- SV-1: Systems and interfaces
- SV-2: Systems resource flows and communications
- SV-4: Systems functionality decomposition
- CV-1: Capability vision and strategic context
- CV-2: Capability taxonomy and hierarchy

---

## 2. Architectural Design

### 2.1 Module Structure

```
src/dodaf/
├── mod.rs                    (Updated: Add new module exports)
├── ov5.rs                    (Existing: OV-5 implementation)
├── ov1.rs                    (New: OV-1 High-Level Concept)
├── ov2.rs                    (New: OV-2 Node Connectivity)
├── ov3.rs                    (New: OV-3 Information Exchange Matrix)
├── ov6.rs                    (New: OV-6 Rules/States/Events)
├── sv1.rs                    (New: SV-1 Systems Interface)
├── sv2.rs                    (New: SV-2 Systems Resource Flow)
├── sv4.rs                    (New: SV-4 Systems Functionality)
├── cv1.rs                    (New: CV-1 Vision)
├── cv2.rs                    (New: CV-2 Capability Taxonomy)
├── operational.rs            (Existing: Basic operational structures)
├── capability.rs             (Existing: Basic capability structures)
└── services.rs               (Existing: Basic services structures)
```

### 2.2 Cross-View Integration Architecture

```
┌────────────────────────────────────────────────────────┐
│              DoDAF 2.02 Architecture                   │
├────────────────────────────────────────────────────────┤
│                                                        │
│  ┌─────────────────────────────────────────────────┐  │
│  │         Operational Viewpoint (OV)              │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────────┐   │  │
│  │  │  OV-1    │ │  OV-2    │ │  OV-3/5/6    │   │  │
│  │  │Concept   │ │Nodes     │ │Activity/     │   │  │
│  │  │Graphic   │ │Flows     │ │Rules/Events  │   │  │
│  │  └──────────┘ └──────────┘ └──────────────┘   │  │
│  └──────────────────────────────────────────────────┘  │
│                      ↓↑                               │
│  ┌─────────────────────────────────────────────────┐  │
│  │         Systems Viewpoint (SV)                  │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────────┐   │  │
│  │  │  SV-1    │ │  SV-2    │ │  SV-4        │   │  │
│  │  │Interfaces│ │Resource  │ │Functionality │   │  │
│  │  │          │ │Flows     │ │              │   │  │
│  │  └──────────┘ └──────────┘ └──────────────┘   │  │
│  └──────────────────────────────────────────────────┘  │
│                      ↓↑                               │
│  ┌─────────────────────────────────────────────────┐  │
│  │         Capability Viewpoint (CV)               │  │
│  │  ┌──────────┐ ┌──────────────────────────┐   │  │
│  │  │  CV-1    │ │  CV-2 Taxonomy           │   │  │
│  │  │Vision    │ │  (Capabilities)          │   │  │
│  │  └──────────┘ └──────────────────────────┘   │  │
│  └──────────────────────────────────────────────────┘  │
│                                                        │
└────────────────────────────────────────────────────────┘
              Integration with BPMN Engine
```

### 2.3 Integration with BPMN

The existing BPMN-DoDAF mapper will be extended to support:

1. **OV-5 ↔ BPMN**: Existing mapping (activities ↔ tasks)
2. **OV-6c ↔ BPMN**: Event traces from process execution
3. **OV-6a ↔ BPMN**: Business rules and conditions
4. **OV-2 ↔ BPMN**: Pools and lanes as operational nodes
5. **SV-1 ↔ BPMN**: System interfaces as message flows
6. **SV-4 ↔ BPMN**: Functional decomposition from subprocesses

---

## 3. Data Structure Designs

### 3.1 OV-1: High-Level Operational Concept Graphic

**Purpose**: Provides graphical and textual description of operational concept, high-level organizations, missions, geographic configuration, and connectivity.

**Key Elements**:
- Strategic mission context
- High-level operational organizations
- Geographic/distributed configuration
- External system interactions
- Key operational scenarios

**Data Structure**:

```rust
pub struct OperationalConceptGraphic {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,

    // Strategic Context
    pub mission_statement: String,
    pub operational_scenarios: Vec<OperationalScenario>,
    pub strategic_vision: Option<String>,

    // High-Level Organizations
    pub organizations: Vec<OperationalOrganization>,

    // Geographic Context
    pub geographic_config: Option<GeographicConfiguration>,

    // External Interactions
    pub external_systems: Vec<ExternalSystem>,
    pub interfaces: Vec<OperationalInterface>,

    // Relationships to Other Views
    pub relates_to_cv1: Option<String>,  // Link to CV-1
    pub relates_to_ov2: Option<String>,  // Link to OV-2

    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

pub struct OperationalScenario {
    pub id: String,
    pub name: String,
    pub description: String,
    pub initiating_event: Option<String>,
    pub key_activities: Vec<String>,
    pub duration: Option<Duration>,
}

pub struct OperationalOrganization {
    pub id: String,
    pub name: String,
    pub org_type: OrganizationType,
    pub parent_org: Option<String>,
    pub responsibilities: Vec<String>,
    pub location: Option<Location>,
}

pub enum OrganizationType {
    Command,
    Support,
    Intelligence,
    Operations,
    Logistics,
    Custom(String),
}

pub struct GeographicConfiguration {
    pub primary_location: Location,
    pub distributed_nodes: Vec<DistributedNode>,
    pub communication_network: Option<String>,
}

pub struct DistributedNode {
    pub name: String,
    pub location: Location,
    pub node_type: OperationalNodeType,
    pub organizations_present: Vec<String>,
}

pub struct ExternalSystem {
    pub id: String,
    pub name: String,
    pub system_type: String,
    pub description: Option<String>,
    pub interfaces_with: Vec<String>,
}

pub struct OperationalInterface {
    pub id: String,
    pub name: String,
    pub from_org: String,
    pub to_org: String,
    pub interface_type: InterfaceType,
    pub data_flow: Option<String>,
}

pub enum InterfaceType {
    Information,
    Physical,
    Functional,
    Service,
}
```

### 3.2 OV-2: Operational Node Connectivity & Resource Flow

**Purpose**: Depicts operational nodes, activities performed at each node, and information/resource flows between nodes.

**Data Structure**:

```rust
pub struct OperationalNodeConnectivity {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,

    // Core Elements
    pub nodes: Vec<OperationalNode>,
    pub needlines: Vec<Needline>,  // Logical information requirements
    pub materialneedlines: Vec<MaterialNeedline>,
    pub performer_connections: Vec<PerformerConnection>,

    // Aggregated data
    pub node_count: usize,
    pub flow_count: usize,

    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

pub struct OperationalNode {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub node_type: OperationalNodeType,

    // Activities and Performers
    pub activities: Vec<String>,
    pub performers: Vec<String>,

    // Location and Context
    pub location: Option<Location>,
    pub operational_context: Option<String>,

    // Connections
    pub inbound_flows: Vec<String>,
    pub outbound_flows: Vec<String>,

    pub properties: HashMap<String, serde_json::Value>,
}

pub struct Needline {
    pub id: String,
    pub name: String,
    pub source_node: String,
    pub target_node: String,
    pub information_elements: Vec<String>,
    pub criticality: Criticality,
    pub frequency: Option<Frequency>,
    pub timeliness: Option<String>,
}

pub struct MaterialNeedline {
    pub id: String,
    pub source_node: String,
    pub target_node: String,
    pub materiel_type: String,
    pub quantity: Option<Quantity>,
    pub transportation_mode: Option<String>,
}

pub struct PerformerConnection {
    pub id: String,
    pub performer: String,
    pub from_node: String,
    pub to_node: String,
    pub connection_type: PerformerConnectionType,
}

pub enum PerformerConnectionType {
    Assignment,
    Reassignment,
    Coordination,
    Reporting,
}

pub enum Criticality {
    Critical,
    Essential,
    Important,
    Desirable,
}
```

### 3.3 OV-3: Operational Information Exchange Matrix

**Purpose**: Describes information exchanged between nodes/activities with attributes (media, quality, quantity, interoperability).

**Data Structure**:

```rust
pub struct InformationExchangeMatrix {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,

    // Matrix data
    pub exchanges: Vec<InformationElement>,
    pub exchange_pairs: Vec<ExchangePair>,

    // Aggregations
    pub row_headers: Vec<String>,  // Source nodes/activities
    pub column_headers: Vec<String>,  // Target nodes/activities

    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

pub struct InformationElement {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub information_type: InformationType,
    pub creator: Option<String>,
    pub consumer: Option<String>,
}

pub enum InformationType {
    Data,
    Signal,
    Document,
    Message,
    Command,
    Report,
    Custom(String),
}

pub struct ExchangePair {
    pub id: String,
    pub source: String,  // Source node/activity ID
    pub target: String,  // Target node/activity ID
    pub information_elements: Vec<String>,  // Information element IDs

    // OV-3 Attributes
    pub exchange_attributes: ExchangeAttributes,
    pub frequency: Option<Frequency>,
    pub criticality: Option<Criticality>,
}

pub struct ExchangeAttributes {
    /// When resource must be available
    pub timeliness: Option<String>,
    /// Quality characteristics
    pub accuracy: Option<f64>,
    pub completeness: Option<f64>,
    pub consistency: Option<f64>,
    /// Availability requirement
    pub availability: Option<f64>,
    /// Security classification
    pub protective_marking: Option<SecurityClassification>,
    /// Volume/Quantity
    pub quantity: Option<Quantity>,
    /// Transmission medium
    pub media: Option<ExchangeMedia>,
    /// Interoperability level
    pub interoperability_level: Option<InteroperabilityLevel>,
    /// Additional attributes
    pub custom_attributes: HashMap<String, String>,
}

pub enum ExchangeMedia {
    Electronic,
    Voice,
    Physical,
    VisualContact,
    Automated,
    Custom(String),
}

pub enum InteroperabilityLevel {
    Level0,  // Incompatible systems
    Level1,  // Manual/Off-line
    Level2,  // Supported data elements
    Level3,  // Automated data exchange
    Level4,  // Full interoperability
}
```

### 3.4 OV-6a: Operational Rules Model

**Purpose**: Identifies business rules that constrain operations and activities.

**Data Structure**:

```rust
pub struct OperationalRulesModel {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,

    pub rules: Vec<OperationalRule>,
    pub rule_sets: Vec<RuleSet>,
    pub constraints: Vec<Constraint>,

    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

pub struct OperationalRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rule_type: RuleType,
    pub applicability: RuleApplicability,

    // Rule Logic
    pub condition: String,  // Logical expression or condition
    pub action: String,     // Action/consequence when true
    pub alternative_action: Option<String>,

    // Relationships
    pub applies_to_activities: Vec<String>,
    pub applies_to_performers: Vec<String>,

    // Context
    pub priority: i32,
    pub enabled: bool,
    pub enforcement_method: Option<String>,
}

pub enum RuleType {
    Policy,
    Regulation,
    Constraint,
    Guideline,
    Procedure,
    BusinessLogic,
}

pub enum RuleApplicability {
    Always,
    Conditional,
    PerScenario,
    PerMissionPhase,
    PerEnvironment,
}

pub struct RuleSet {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub rules: Vec<String>,  // Rule IDs
    pub evaluation_logic: EvaluationLogic,
}

pub enum EvaluationLogic {
    All,        // All rules must pass
    Any,        // At least one rule must pass
    Ordered,    // Rules evaluated in order, stop on first failure
    Weighted,   // Rules evaluated with weights
}

pub struct Constraint {
    pub id: String,
    pub name: String,
    pub constraint_type: ConstraintType,
    pub expression: String,
    pub applies_to: Vec<String>,  // Activity/node IDs
}

pub enum ConstraintType {
    Temporal,
    Resource,
    Sequence,
    Exclusion,
    Cardinality,
    Custom(String),
}
```

### 3.5 OV-6b: Operational State Transition Description

**Purpose**: Identifies responses of business processes to events and state transitions.

**Data Structure**:

```rust
pub struct StateTransitionDescription {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,

    pub states: Vec<OperationalState>,
    pub transitions: Vec<StateTransition>,
    pub events: Vec<TransitionEvent>,

    // State Machine Model
    pub initial_state: String,
    pub terminal_states: Vec<String>,

    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

pub struct OperationalState {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub state_type: StateType,

    // State Behavior
    pub entry_action: Option<String>,
    pub exit_action: Option<String>,
    pub activities_in_state: Vec<String>,

    // State Properties
    pub is_initial: bool,
    pub is_terminal: bool,
    pub is_composite: bool,
    pub parent_state: Option<String>,
}

pub enum StateType {
    Idle,
    Active,
    Processing,
    Waiting,
    Blocked,
    Error,
    Terminal,
    Custom(String),
}

pub struct StateTransition {
    pub id: String,
    pub name: Option<String>,
    pub source_state: String,
    pub target_state: String,

    // Trigger and Guard
    pub trigger_event: String,
    pub guard_condition: Option<String>,
    pub probability: Option<f64>,

    // Transition Actions
    pub action: Option<String>,
    pub action_sequence: Vec<String>,
}

pub struct TransitionEvent {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub event_type: EventType,
    pub triggers_transitions: Vec<String>,  // Transition IDs
}

pub enum EventType {
    External,
    Internal,
    Temporal,
    Signal,
    Condition,
    Error,
    Completion,
}
```

### 3.6 OV-6c: Operational Event-Trace Description (Enhanced)

**Purpose**: Time-ordered examination of resource flows as a result of a particular scenario.

**Enhancement**:

```rust
pub struct EventTraceDescription {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub scenario: String,

    pub trace_events: Vec<TraceEvent>,
    pub trace_objects: Vec<TraceObject>,
    pub event_relationships: Vec<EventRelationship>,

    // Scenario Context
    pub initiating_event: Option<String>,
    pub duration: Option<Duration>,
    pub success_criteria: Vec<String>,

    // Integration with Other Views
    pub bpmn_ref: Option<String>,  // Reference to BPMN process
    pub ov6b_ref: Option<String>,  // Reference to State Transition

    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

pub struct TraceObject {
    pub id: String,
    pub name: String,
    pub object_type: String,
    pub creation_event: String,
    pub terminal_event: Option<String>,
    pub properties: HashMap<String, serde_json::Value>,
}

pub struct EventRelationship {
    pub source_event_id: String,
    pub target_event_id: String,
    pub relationship_type: EventRelationshipType,
}

pub enum EventRelationshipType {
    Sequence,
    Causality,
    Parallel,
    Conditional,
    Synchronous,
}
```

### 3.7 SV-1: Systems Interface Description

**Purpose**: Specifies composition and interaction of systems, showing how resources are structured and interact.

**Data Structure**:

```rust
pub struct SystemsInterfaceDescription {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,

    pub systems: Vec<System>,
    pub interfaces: Vec<SystemInterface>,
    pub ports: Vec<SystemPort>,
    pub system_aggregates: Vec<SystemAggregate>,

    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

pub struct System {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub system_type: SystemType,

    pub functions: Vec<String>,
    pub ports: Vec<String>,
    pub sub_systems: Vec<String>,
    pub located_at: Option<String>,

    pub properties: HashMap<String, serde_json::Value>,
}

pub enum SystemType {
    Hardware,
    Software,
    Hybrid,
    Manual,
    Organic,
}

pub struct SystemInterface {
    pub id: String,
    pub name: Option<String>,
    pub source_system: String,
    pub target_system: String,
    pub source_port: String,
    pub target_port: String,

    pub interface_type: InterfaceType,
    pub description: Option<String>,
}

pub struct SystemPort {
    pub id: String,
    pub name: String,
    pub port_type: PortType,
    pub belongs_to_system: String,

    pub protocol: Option<String>,
    pub direction: PortDirection,
    pub cardinality: Option<String>,

    pub properties: HashMap<String, serde_json::Value>,
}

pub enum PortType {
    Input,
    Output,
    Bidirectional,
    Control,
    Data,
    Power,
}

pub enum PortDirection {
    In,
    Out,
    InOut,
}

pub struct SystemAggregate {
    pub id: String,
    pub name: String,
    pub member_systems: Vec<String>,
    pub aggregate_type: String,
}
```

### 3.8 SV-2: Systems Resource Flow Description

**Purpose**: Documents communication systems, links, networks, and media supporting systems and interfaces.

**Data Structure**:

```rust
pub struct SystemsResourceFlowDescription {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,

    pub systems: Vec<String>,  // System IDs
    pub communications_systems: Vec<CommunicationSystem>,
    pub communications_links: Vec<CommunicationLink>,
    pub communications_networks: Vec<CommunicationNetwork>,
    pub information_flows: Vec<SystemInformationFlow>,

    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

pub struct CommunicationSystem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub system_type: CommunicationType,
    pub media_type: MediaType,
    pub protocol: String,
}

pub enum CommunicationType {
    Voice,
    Data,
    Video,
    Messaging,
    ServiceInterface,
}

pub struct CommunicationLink {
    pub id: String,
    pub name: Option<String>,
    pub from_system: String,
    pub to_system: String,
    pub communication_system: String,
    pub bandwidth: Option<String>,
    pub latency: Option<String>,
    pub reliability: Option<f64>,
}

pub struct CommunicationNetwork {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub network_type: NetworkType,
    pub member_systems: Vec<String>,
    pub communication_links: Vec<String>,
}

pub enum NetworkType {
    LAN,
    WAN,
    Internet,
    Intranet,
    Extranet,
    P2P,
    Custom(String),
}

pub struct SystemInformationFlow {
    pub id: String,
    pub name: Option<String>,
    pub from_system: String,
    pub to_system: String,
    pub information_elements: Vec<String>,
    pub communication_link: String,
    pub flow_attributes: FlowAttributes,
}

pub struct FlowAttributes {
    pub throughput: Option<String>,
    pub latency_requirement: Option<String>,
    pub availability_requirement: Option<f64>,
    pub security_requirement: Option<String>,
}
```

### 3.9 SV-4: Systems Functionality Description

**Purpose**: Specifies functionality of systems, functional hierarchies, and data flows between functions.

**Data Structure**:

```rust
pub struct SystemsFunctionalityDescription {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,

    pub functions: Vec<SystemFunction>,
    pub function_flows: Vec<FunctionFlow>,
    pub data_flows: Vec<FunctionDataFlow>,
    pub functional_hierarchies: Vec<FunctionalHierarchy>,

    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

pub struct SystemFunction {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub function_type: FunctionType,
    pub belongs_to_system: String,

    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub allocated_to: Vec<String>,  // Systems that implement this

    pub parent_function: Option<String>,
    pub child_functions: Vec<String>,

    pub properties: HashMap<String, serde_json::Value>,
}

pub enum FunctionType {
    Primary,
    Supporting,
    Enabling,
    Operational,
    Maintenance,
    Management,
}

pub struct FunctionFlow {
    pub id: String,
    pub source_function: String,
    pub target_function: String,
    pub flow_type: String,
}

pub struct FunctionDataFlow {
    pub id: String,
    pub name: Option<String>,
    pub source_function: String,
    pub target_function: String,
    pub data_elements: Vec<String>,
    pub timing: Option<String>,
}

pub struct FunctionalHierarchy {
    pub root_function: String,
    pub levels: Vec<HierarchyLevel>,
}

pub struct HierarchyLevel {
    pub level: i32,
    pub functions: Vec<String>,
    pub parent_functions: HashMap<String, Vec<String>>,
}
```

### 3.10 CV-1: Capability Vision

**Purpose**: Defines strategic context for a group of capabilities.

**Data Structure**:

```rust
pub struct CapabilityVision {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,

    // Strategic Elements
    pub vision_statement: String,
    pub mission_areas: Vec<String>,
    pub strategic_objectives: Vec<StrategicObjective>,
    pub capability_increments: Vec<CapabilityIncrement>,
    pub timeframe: TimeFrame,

    // Relationships
    pub related_capabilities: Vec<String>,  // CV-2 relationships
    pub supporting_operations: Vec<String>,  // OV relationships
    pub supporting_systems: Vec<String>,    // SV relationships

    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

pub struct StrategicObjective {
    pub id: String,
    pub name: String,
    pub description: String,
    pub priority: i32,
    pub measures_of_effectiveness: Vec<String>,
}

pub struct CapabilityIncrement {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub capabilities_enabled: Vec<String>,
    pub dependencies: Vec<String>,
}

pub struct TimeFrame {
    pub near_term: Option<String>,  // 0-2 years
    pub mid_term: Option<String>,   // 2-5 years
    pub far_term: Option<String>,   // 5+ years
}
```

### 3.11 CV-2: Capability Taxonomy

**Purpose**: Organizes and hierarchically structures capabilities.

**Data Structure**:

```rust
pub struct CapabilityTaxonomy {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,

    pub capabilities: Vec<Capability>,
    pub hierarchy: TaxonomyHierarchy,
    pub capability_clusters: Vec<CapabilityCluster>,

    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

pub struct Capability {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub capability_type: CapabilityType,

    // Hierarchy
    pub parent_capability: Option<String>,
    pub child_capabilities: Vec<String>,
    pub hierarchy_level: i32,

    // Measures
    pub measures: Vec<CapabilityMeasure>,
    pub maturity_level: Option<i32>,
    pub readiness_level: Option<i32>,

    // Realization
    pub realized_by_operations: Vec<String>,  // OV relationships
    pub realized_by_systems: Vec<String>,     // SV relationships
    pub realized_by_services: Vec<String>,    // SvcV relationships

    pub properties: HashMap<String, serde_json::Value>,
}

pub struct CapabilityType {
    pub domain: String,  // e.g., "Command & Control", "Intelligence"
    pub category: String,  // e.g., "Communication", "Analysis"
}

pub struct CapabilityMeasure {
    pub id: String,
    pub name: String,
    pub metric: String,
    pub condition: Option<String>,  // Environmental condition
    pub target_value: Option<String>,
    pub current_value: Option<String>,
}

pub struct TaxonomyHierarchy {
    pub root_nodes: Vec<String>,
    pub levels: i32,
    pub relationships: Vec<HierarchyRelationship>,
}

pub struct HierarchyRelationship {
    pub parent_id: String,
    pub child_id: String,
    pub relationship_type: HierarchyRelationType,
}

pub enum HierarchyRelationType {
    Composition,
    Decomposition,
    Dependency,
    Specialization,
}

pub struct CapabilityCluster {
    pub id: String,
    pub name: String,
    pub members: Vec<String>,  // Capability IDs
    pub cluster_type: String,
}
```

---

## 4. Integration Strategy

### 4.1 BPMN-DoDAF Mapping Extensions

Extend the existing `BpmnDodafMapper` to support:

1. **OV-1**: Strategic context extraction from process metadata
2. **OV-2**: Pool/lane analysis → operational nodes
3. **OV-3**: Data object analysis → information exchange matrix
4. **OV-6a**: Conditional flows and constraints → business rules
5. **OV-6b**: Gateways and event handlers → state transitions
6. **OV-6c**: Full process execution trace (enhanced)
7. **SV-1**: Port definitions from data associations
8. **SV-4**: Task hierarchies → functional decomposition

### 4.2 Cross-View Traceability

Implement bidirectional traceability:

```rust
pub struct ViewTraceability {
    pub id: String,
    pub source_view: ViewType,
    pub target_view: ViewType,
    pub mappings: Vec<ElementMapping>,
}

pub enum ViewType {
    OV1, OV2, OV3, OV5, OV6A, OV6B, OV6C,
    SV1, SV2, SV4,
    CV1, CV2,
}

pub struct ElementMapping {
    pub source_element_id: String,
    pub target_element_id: String,
    pub mapping_type: String,
    pub confidence: Option<f64>,
}
```

### 4.3 UI Visualization Concepts

**OV-1 Visualization**:
- Graphic diagram with organizations, missions, and external systems
- Hierarchical organization chart overlay
- Geographic map showing node locations

**OV-2 Visualization**:
- Node-and-link diagram with operational nodes
- Animated flow indicators for needlines
- Color coding by criticality/frequency

**OV-3 Visualization**:
- Matrix view with source/target nodes
- Heatmap showing information density
- Drill-down to exchange attributes

**OV-6a Visualization**:
- Rule list with rule type color coding
- Constraint visualization on activities
- Rule dependency graph

**OV-6b Visualization**:
- State diagram with transitions
- Timeline view of state changes
- Event-triggered transition highlighting

**OV-6c Visualization**:
- Sequence diagram/timeline
- Event flow with object lifecycle
- Scenario execution animation

**SV-1 Visualization**:
- Systems architecture diagram
- Port/interface connections highlighted
- System hierarchy tree

**SV-2 Visualization**:
- Network topology diagram
- Communication link bandwidth visualization
- Network paths highlighted

**SV-4 Visualization**:
- Functional decomposition tree
- Function call/data flow diagram
- Allocation matrix (functions ↔ systems)

**CV-1 Visualization**:
- Strategic vision timeline
- Capability increment roadmap
- Mission area relationships

**CV-2 Visualization**:
- Capability hierarchy tree
- Taxonomy browser
- Capability-to-operation traceability matrix

---

## 5. Implementation Roadmap

### Phase 1: Data Structures (Weeks 1-2)
- [ ] Create new module files (ov1.rs, ov2.rs, etc.)
- [ ] Implement all data structures
- [ ] Add serialization/deserialization (serde)
- [ ] Create builder patterns for complex types
- [ ] Add comprehensive tests

### Phase 2: BPMN Integration (Weeks 3-4)
- [ ] Extend BpmnDodafMapper for all views
- [ ] Implement bidirectional mapping
- [ ] Add traceability tracking
- [ ] Create integration tests

### Phase 3: UI Visualization (Weeks 5-6)
- [ ] Design visualization components for each view
- [ ] Implement egui rendering for each view type
- [ ] Add interactive features (zoom, pan, filter)
- [ ] Create view switching interface

### Phase 4: Examples & Documentation (Week 7)
- [ ] Create example files for each view type
- [ ] Add comprehensive documentation
- [ ] Create integration examples
- [ ] Performance optimization

---

## 6. Key Dependencies

- **serde/serde_json**: Serialization framework
- **chrono**: Date/time handling
- **uuid**: Unique identifier generation
- **egui** (for UI): Visualization
- **BPMN execution engine**: Integration point

---

## 7. Success Criteria

1. All 10 new views implemented with full data structures
2. Serialization/deserialization working for all views
3. BPMN mapper extended to support all views
4. At least 80% test coverage for new code
5. UI visualization concepts prototyped
6. Cross-view traceability functional
7. Performance: <100ms serialization for typical architecture
8. Example files for at least 5 complete architectures

---

## References

- [DoDAF 2.02 Official Documentation](https://dodcio.defense.gov/Library/DoD-Architecture-Framework/)
- [OV-1 High Level Operational Concept Graphic](https://dodcio.defense.gov/Library/DoD-Architecture-Framework/dodaf20_ov1/)
- [SV-2 Systems Resource Flow Description](https://dodcio.defense.gov/Library/DoD-Architecture-Framework/dodaf20_sv2/)
- [CV-2 Capability Taxonomy](https://dodcio.defense.gov/Library/DoD-Architecture-Framework/dodaf20_cv2/)
- DoDAF 2.0 Viewpoint Definitions (PDF)

---

**Document Status**: Design Phase Complete
**Next Action**: Begin Phase 1 Implementation
