//! BPMN 2.0 Complete Element Definitions
//!
//! Comprehensive BPMN 2.0 element types aligned with the official OMG specification.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// BPMN 2.0 Diagram root
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BpmnDiagram {
    pub id: String,
    pub name: Option<String>,
    pub documentation: Option<String>,
    pub processes: Vec<BpmnProcess>,
    pub collaborations: Vec<Collaboration>,
    pub data_stores: Vec<DataStore>,
    pub messages: Vec<Message>,
    pub signals: Vec<Signal>,
    /// Diagram Interchange (visual layout information)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagram_info: Option<BpmnDI>,
}

/// BPMN Process definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BpmnProcess {
    pub id: String,
    pub name: Option<String>,
    pub documentation: Option<String>,
    pub is_executable: bool,
    pub process_type: ProcessType,

    // Flow elements
    pub start_events: Vec<StartEvent>,
    pub end_events: Vec<EndEvent>,
    pub intermediate_events: Vec<IntermediateEvent>,
    pub tasks: Vec<BpmnTask>,
    pub subprocesses: Vec<Subprocess>,
    pub gateways: Vec<BpmnGateway>,

    // Connecting objects
    pub sequence_flows: Vec<SequenceFlow>,

    // Data
    pub data_objects: Vec<DataObject>,
    pub data_associations: Vec<DataAssociation>,

    // Artifacts
    pub text_annotations: Vec<TextAnnotation>,
    pub groups: Vec<Group>,

    // Swimlanes
    pub lanes: Vec<Lane>,

    pub metadata: HashMap<String, serde_json::Value>,
}

/// Process type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessType {
    None,
    Public,
    Private,
}

// ============================================================================
// EVENTS
// ============================================================================

/// Start Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartEvent {
    pub id: String,
    pub name: Option<String>,
    pub documentation: Option<String>,
    pub event_definition: Option<EventDefinition>,
    pub is_interrupting: bool,
}

/// End Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndEvent {
    pub id: String,
    pub name: Option<String>,
    pub documentation: Option<String>,
    pub event_definition: Option<EventDefinition>,
}

/// Intermediate Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntermediateEvent {
    pub id: String,
    pub name: Option<String>,
    pub documentation: Option<String>,
    pub event_definition: Option<EventDefinition>,
    pub is_catching: bool,               // true = catching, false = throwing
    pub is_interrupting: bool,           // for boundary events
    pub attached_to_ref: Option<String>, // for boundary events
}

/// Event Definition (12 types per BPMN 2.0)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventDefinition {
    None,
    Message { message_ref: Option<String> },
    Timer { time_expression: String },
    Signal { signal_ref: Option<String> },
    Error { error_ref: Option<String> },
    Escalation { escalation_ref: Option<String> },
    Cancel,
    Compensation,
    Conditional { condition: String },
    Link { target: Option<String>, source: Option<String> },
    Terminate,
    Multiple { events: Vec<EventDefinition>, is_parallel: bool },
}

// ============================================================================
// TASKS
// ============================================================================

/// BPMN Task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BpmnTask {
    pub id: String,
    pub name: Option<String>,
    pub documentation: Option<String>,
    pub task_type: BpmnTaskType,
    pub default_flow: Option<String>,
    pub io_specification: Option<IoSpecification>,
    pub properties: HashMap<String, serde_json::Value>,
    pub loop_characteristics: Option<LoopCharacteristics>,
    pub is_for_compensation: bool,
}

/// Task Type (8 types per BPMN 2.0)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BpmnTaskType {
    /// Abstract/None - Generic task
    Abstract,
    /// User Task - Human performer with process-aware application
    User { implementation: Option<String>, rendering: Option<String> },
    /// Service Task - Automated service/web service
    Service { implementation: Option<String>, operation_ref: Option<String> },
    /// Manual Task - Human work without system support
    Manual,
    /// Script Task - Automated script execution
    Script { script_format: String, script: String },
    /// Business Rule Task - Business rules engine
    BusinessRule { implementation: Option<String>, rule_ref: Option<String> },
    /// Send Task - Sends message
    Send { message_ref: Option<String>, operation_ref: Option<String> },
    /// Receive Task - Receives message
    Receive { message_ref: Option<String>, operation_ref: Option<String>, instantiate: bool },
}

/// Subprocess
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subprocess {
    pub id: String,
    pub name: Option<String>,
    pub documentation: Option<String>,
    pub subprocess_type: SubprocessType,
    pub triggered_by_event: bool,
    pub process: Option<BpmnProcess>,   // For embedded subprocesses
    pub called_element: Option<String>, // For call activities
    pub loop_characteristics: Option<LoopCharacteristics>,
}

/// Subprocess Type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubprocessType {
    /// Embedded subprocess
    Embedded,
    /// Call activity (reusable subprocess)
    CallActivity,
    /// Event subprocess
    EventSubprocess,
    /// Transaction (all-or-nothing)
    Transaction,
    /// Ad-hoc (flexible execution)
    AdHoc,
}

/// Loop characteristics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LoopCharacteristics {
    /// Sequential loop
    Standard { loop_condition: Option<String>, test_before: bool, loop_maximum: Option<i32> },
    /// Parallel multi-instance
    MultiInstance {
        is_sequential: bool,
        loop_cardinality: Option<i32>,
        loop_data_input_ref: Option<String>,
        loop_data_output_ref: Option<String>,
        completion_condition: Option<String>,
    },
}

/// IO Specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoSpecification {
    pub data_inputs: Vec<DataInput>,
    pub data_outputs: Vec<DataOutput>,
    pub input_sets: Vec<Vec<String>>,
    pub output_sets: Vec<Vec<String>>,
}

/// Data Input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataInput {
    pub id: String,
    pub name: Option<String>,
    pub item_subject_ref: Option<String>,
    pub is_collection: bool,
}

/// Data Output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataOutput {
    pub id: String,
    pub name: Option<String>,
    pub item_subject_ref: Option<String>,
    pub is_collection: bool,
}

// ============================================================================
// GATEWAYS
// ============================================================================

/// Gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BpmnGateway {
    pub id: String,
    pub name: Option<String>,
    pub documentation: Option<String>,
    pub gateway_type: BpmnGatewayType,
    pub gateway_direction: GatewayDirection,
    pub default_flow: Option<String>,
}

/// Gateway Type (6 types per BPMN 2.0)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BpmnGatewayType {
    /// Exclusive (XOR) - one path taken
    Exclusive,
    /// Parallel (AND) - all paths taken/synchronized
    Parallel,
    /// Inclusive (OR) - one or more paths
    Inclusive,
    /// Event-Based - decision based on events
    EventBased { instantiate: bool },
    /// Parallel Event-Based - multiple events
    ParallelEventBased,
    /// Complex - advanced custom logic
    Complex { activation_condition: Option<String> },
}

/// Gateway Direction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GatewayDirection {
    Unspecified,
    Converging,
    Diverging,
    Mixed,
}

// ============================================================================
// CONNECTING OBJECTS
// ============================================================================

/// Sequence Flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceFlow {
    pub id: String,
    pub name: Option<String>,
    pub source_ref: String,
    pub target_ref: String,
    pub condition_expression: Option<String>,
    pub is_immediate: bool,
}

/// Message Flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageFlow {
    pub id: String,
    pub name: Option<String>,
    pub source_ref: String,
    pub target_ref: String,
    pub message_ref: Option<String>,
}

/// Association
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Association {
    pub id: String,
    pub source_ref: String,
    pub target_ref: String,
    pub direction: AssociationDirection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssociationDirection {
    None,
    One,
    Both,
}

/// Data Association
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataAssociation {
    pub id: String,
    pub source_ref: Option<String>,
    pub target_ref: String,
    pub transformation: Option<String>,
    pub assignment: Option<String>,
}

// ============================================================================
// DATA ELEMENTS
// ============================================================================

/// Data Object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataObject {
    pub id: String,
    pub name: Option<String>,
    pub item_subject_ref: Option<String>,
    pub is_collection: bool,
    pub data_state: Option<String>,
}

/// Data Store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataStore {
    pub id: String,
    pub name: Option<String>,
    pub capacity: Option<i32>,
    pub is_unlimited: bool,
    pub item_subject_ref: Option<String>,
}

/// Message Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub name: Option<String>,
    pub item_ref: Option<String>,
}

/// Signal Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub id: String,
    pub name: Option<String>,
    pub structure_ref: Option<String>,
}

// ============================================================================
// ARTIFACTS
// ============================================================================

/// Text Annotation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextAnnotation {
    pub id: String,
    pub text: String,
    pub text_format: String, // MIME type, default "text/plain"
}

/// Group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    pub category_value_ref: Option<String>,
}

// ============================================================================
// SWIMLANES
// ============================================================================

/// Collaboration (contains pools)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collaboration {
    pub id: String,
    pub name: Option<String>,
    pub participants: Vec<Participant>,
    pub message_flows: Vec<MessageFlow>,
    pub artifacts: Vec<Artifact>,
}

/// Participant (Pool)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id: String,
    pub name: Option<String>,
    pub process_ref: Option<String>,
    pub interface_ref: Option<String>,
    pub participant_multiplicity: Option<ParticipantMultiplicity>,
}

/// Participant Multiplicity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantMultiplicity {
    pub minimum: i32,
    pub maximum: i32,
}

/// Lane
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lane {
    pub id: String,
    pub name: Option<String>,
    pub partition_element_ref: Option<String>,
    pub child_lanes: Vec<Lane>,
    pub flow_node_refs: Vec<String>,
}

/// Artifact (grouping for text annotations, groups, and associations)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Artifact {
    TextAnnotation(TextAnnotation),
    Group(Group),
    Association(Association),
}

// ============================================================================
// DIAGRAM INTERCHANGE (DI)
// ============================================================================

/// BPMN Diagram Interchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BpmnDI {
    pub diagrams: Vec<BpmnDiagramInfo>,
}

/// Diagram Info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BpmnDiagramInfo {
    pub id: String,
    pub name: Option<String>,
    pub plane: BpmnPlane,
}

/// Plane (2D surface)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BpmnPlane {
    pub id: String,
    pub bpmn_element: String,
    pub shapes: Vec<BpmnShape>,
    pub edges: Vec<BpmnEdge>,
}

/// Shape (visual node)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BpmnShape {
    pub id: String,
    pub bpmn_element: String,
    pub bounds: Bounds,
    pub is_horizontal: bool,
    pub is_expanded: bool,
    pub is_marker_visible: bool,
    pub label: Option<BpmnLabel>,
}

/// Edge (visual connection)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BpmnEdge {
    pub id: String,
    pub bpmn_element: String,
    pub waypoints: Vec<Point>,
    pub label: Option<BpmnLabel>,
}

/// Bounds (rectangle)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Point (coordinate)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// Label
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BpmnLabel {
    pub bounds: Bounds,
}
