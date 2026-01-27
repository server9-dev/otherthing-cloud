//! DoDAF 2.02 OV-5 Operational Activity Model
//!
//! Implementation of DoDAF 2.02 Operational Viewpoint, specifically OV-5 (Operational Activity Model).
//! Aligns with DM2 (DoDAF Meta-Model) specifications.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OV-5 Operational Activity Model
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationalActivityModel {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,

    // Core elements
    pub activities: Vec<OperationalActivity>,
    pub resource_flows: Vec<ResourceFlow>,
    pub performers: Vec<Performer>,

    // Metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Operational Activity (DM2 Activity Entity)
///
/// An activity is a transformation that produces new resources from existing resources.
/// Activities are synonymous with Tasks in DM2.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationalActivity {
    // Core Attributes
    pub id: String,
    pub name: String,
    pub description: String,
    pub performer: PerformerRef,

    // Resource Flows
    pub input_resources: Vec<ResourceRef>,
    pub output_resources: Vec<ResourceRef>,

    // Operational Attributes
    pub cost: Option<Cost>,
    pub duration: Option<Duration>,
    pub frequency: Option<Frequency>,
    pub security_domain: Option<SecurityDomain>,
    pub location: Option<Location>,

    // Relationships
    pub predecessor_activities: Vec<String>, // Activity IDs
    pub successor_activities: Vec<String>,   // Activity IDs
    pub parent_activity: Option<String>,     // For hierarchical decomposition
    pub child_activities: Vec<String>,       // Sub-activities

    // Business Rules
    pub constraints: Vec<BusinessRule>,

    // Traceability
    pub capabilities: Vec<String>,           // Links to CV-6
    pub systems_services: Vec<String>,       // Links to SV-5a
    pub information_requirements: Vec<String>, // Links to DIV

    // Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Performer (DM2 Performer Entity)
///
/// A performer performs an activity. Types include: Person, Organization,
/// Service, ServiceInterface, System, and Interface (Port).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Performer {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub performer_type: PerformerType,
    pub properties: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PerformerType {
    Person {
        role: String,
        skills: Vec<String>,
    },
    Organization {
        org_type: String,
        parent_org: Option<String>,
    },
    Service {
        service_type: String,
        interface_ref: Option<String>,
    },
    ServiceInterface {
        protocol: String,
        endpoint: Option<String>,
    },
    System {
        system_type: String,
        capabilities: Vec<String>,
    },
    Interface {
        interface_type: String,
        port: Option<String>,
    },
}

/// Performer Reference
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerformerRef {
    pub performer_id: String,
    pub role: Option<String>,
}

/// Resource Flow (DM2 Resource Flow Exchange)
///
/// Represents behavioral and structural interactions between activities.
/// Temporal and results in flow/exchange of information, data, materiel, or performers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceFlow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,

    // Flow endpoints
    pub source_activity: String, // Activity ID
    pub target_activity: String, // Activity ID

    // Resource being flowed
    pub resource_type: ResourceType,
    pub resource_ref: Option<String>,

    // OV-3 Attributes
    pub attributes: ResourceFlowAttributes,

    // Needline (logical requirement)
    pub is_needline: bool,

    pub metadata: HashMap<String, serde_json::Value>,
}

/// Resource Type (types of resources that can flow)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    /// Information (data, messages)
    Information,
    /// Funding (budget, financial resources)
    Funding,
    /// Personnel (human resources)
    Personnel,
    /// Materiel (physical goods, equipment)
    Materiel,
}

/// Resource Flow Attributes (OV-3 attributes)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceFlowAttributes {
    /// When the resource must be available
    pub timeliness: Option<String>,
    /// Reliability and uptime requirements
    pub availability: Option<f64>,
    /// Security classification
    pub protective_marking: Option<SecurityClassification>,
    /// Authentication requirements
    pub non_repudiation: Option<bool>,
    /// Performance characteristics
    pub quality: Option<QualityMetrics>,
    /// Volume or amount
    pub quantity: Option<Quantity>,
    /// Transmission medium
    pub media: Option<String>,
    /// Required compatibility standards
    pub interoperability_level: Option<String>,
}

/// Resource Reference
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceRef {
    pub resource_id: String,
    pub resource_type: ResourceType,
    pub quantity: Option<Quantity>,
}

/// Cost information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cost {
    pub amount: f64,
    pub currency: String,
    pub cost_type: CostType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CostType {
    Fixed,
    Variable,
    Estimated,
    Actual,
}

/// Duration information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Duration {
    pub value: f64,
    pub unit: TimeUnit,
    pub is_estimated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeUnit {
    Seconds,
    Minutes,
    Hours,
    Days,
    Weeks,
    Months,
}

/// Frequency information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Frequency {
    pub occurrences: i32,
    pub per_time_unit: TimeUnit,
}

/// Security Domain
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityDomain {
    pub classification: SecurityClassification,
    pub access_control: Vec<String>,
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityClassification {
    Unclassified,
    Confidential,
    Secret,
    TopSecret,
    Custom(String),
}

/// Location information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Location {
    pub name: String,
    pub location_type: LocationType,
    pub coordinates: Option<Coordinates>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocationType {
    Physical,
    Virtual,
    Distributed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}

/// Business Rule (from OV-6a)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BusinessRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rule_type: RuleType,
    pub expression: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleType {
    Constraint,
    Guideline,
    Policy,
    Regulation,
}

/// Quality Metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub accuracy: Option<f64>,
    pub completeness: Option<f64>,
    pub consistency: Option<f64>,
    pub timeliness: Option<f64>,
    pub custom_metrics: HashMap<String, f64>,
}

/// Quantity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Quantity {
    pub value: f64,
    pub unit: String,
}

/// OV-5a Decomposition Tree Node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecompositionNode {
    pub activity_id: String,
    pub level: i32,
    pub parent_id: Option<String>,
    pub children: Vec<String>,
    pub sequence_order: Option<i32>,
}

/// Activity Decomposition Tree (OV-5a)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActivityDecompositionTree {
    pub id: String,
    pub name: String,
    pub root_activities: Vec<String>,
    pub nodes: Vec<DecompositionNode>,
}

// ============================================================================
// OV-6c Event-Trace Description (BPMN Integration)
// ============================================================================

/// OV-6c Event-Trace Description
///
/// Describes time-ordered sequence of activities and events in specific scenarios.
/// Can be represented using BPMN 2.0.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventTraceDescription {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub scenario: String,
    pub trace_events: Vec<TraceEvent>,
    pub bpmn_ref: Option<String>, // Reference to BPMN process if using BPMN representation
}

/// Trace Event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceEvent {
    pub timestamp: DateTime<Utc>,
    pub sequence_number: i32,
    pub event_type: TraceEventType,
    pub activity_id: String,
    pub performer_id: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraceEventType {
    ActivityStart,
    ActivityEnd,
    MessageSent,
    MessageReceived,
    DecisionPoint,
    ResourceAllocated,
    ResourceReleased,
}

// ============================================================================
// OV-2 Operational Resource Flow (Needlines)
// ============================================================================

/// OV-2 Operational Node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationalNode {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub node_type: OperationalNodeType,
    pub activities: Vec<String>, // Activity IDs performed at this node
    pub location: Option<Location>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationalNodeType {
    Organization,
    System,
    Facility,
    Platform,
}

/// OV-2 Model
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationalResourceFlowDescription {
    pub id: String,
    pub name: String,
    pub nodes: Vec<OperationalNode>,
    pub needlines: Vec<ResourceFlow>,
}

// ============================================================================
// Helper implementations
// ============================================================================

impl OperationalActivity {
    pub fn new(id: impl Into<String>, name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            performer: PerformerRef {
                performer_id: String::new(),
                role: None,
            },
            input_resources: Vec::new(),
            output_resources: Vec::new(),
            cost: None,
            duration: None,
            frequency: None,
            security_domain: None,
            location: None,
            predecessor_activities: Vec::new(),
            successor_activities: Vec::new(),
            parent_activity: None,
            child_activities: Vec::new(),
            constraints: Vec::new(),
            capabilities: Vec::new(),
            systems_services: Vec::new(),
            information_requirements: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}

impl Performer {
    pub fn new_person(id: impl Into<String>, name: impl Into<String>, role: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            performer_type: PerformerType::Person {
                role: role.into(),
                skills: Vec::new(),
            },
            properties: HashMap::new(),
        }
    }

    pub fn new_system(id: impl Into<String>, name: impl Into<String>, system_type: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            performer_type: PerformerType::System {
                system_type: system_type.into(),
                capabilities: Vec::new(),
            },
            properties: HashMap::new(),
        }
    }
}

impl ResourceFlow {
    pub fn new(
        id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
        resource_type: ResourceType,
    ) -> Self {
        Self {
            id: id.into(),
            name: String::new(),
            description: None,
            source_activity: source.into(),
            target_activity: target.into(),
            resource_type,
            resource_ref: None,
            attributes: ResourceFlowAttributes::default(),
            is_needline: false,
            metadata: HashMap::new(),
        }
    }
}

impl Default for ResourceFlowAttributes {
    fn default() -> Self {
        Self {
            timeliness: None,
            availability: None,
            protective_marking: None,
            non_repudiation: None,
            quality: None,
            quantity: None,
            media: None,
            interoperability_level: None,
        }
    }
}
