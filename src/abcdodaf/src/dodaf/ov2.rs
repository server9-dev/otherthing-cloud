//! DoDAF 2.02 OV-2: Operational Node Connectivity & Resource Flow
//!
//! Depicts operational nodes, activities performed at each node,
//! and connectivities and information/resource flows between nodes.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OV-2 Operational Node Connectivity Description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalNodeConnectivity {
    /// Unique identifier
    pub id: String,
    /// Name of the description
    pub name: String,
    /// Description text
    pub description: Option<String>,
    /// Version number
    pub version: String,

    // Core Elements
    /// Operational nodes in the architecture
    pub nodes: Vec<OperationalNode>,
    /// Logical information requirements between nodes
    pub needlines: Vec<Needline>,
    /// Physical materiel requirements between nodes
    pub materialneedlines: Vec<MaterialNeedline>,
    /// Performer connections and movements
    pub performer_connections: Vec<PerformerConnection>,

    // Aggregated metrics
    /// Total number of nodes
    pub node_count: usize,
    /// Total number of flows
    pub flow_count: usize,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Operational node - location where activities are performed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalNode {
    /// Node ID
    pub id: String,
    /// Node name
    pub name: String,
    /// Node description
    pub description: Option<String>,
    /// Type of operational node
    pub node_type: OperationalNodeType,

    // Activities and Performers
    /// Activities performed at this node
    pub activities: Vec<String>,
    /// Performers (organizations, people) at this node
    pub performers: Vec<String>,

    // Location and Context
    /// Physical or logical location of node
    pub location: Option<crate::dodaf::ov5::Location>,
    /// Operational context/scenario
    pub operational_context: Option<String>,

    // Connections
    /// IDs of inbound flows (needlines)
    pub inbound_flows: Vec<String>,
    /// IDs of outbound flows (needlines)
    pub outbound_flows: Vec<String>,

    /// Additional properties
    pub properties: HashMap<String, serde_json::Value>,
}

/// Type of operational node
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationalNodeType {
    /// Organizational node
    Organization,
    /// System node
    System,
    /// Physical facility
    Facility,
    /// Platform (mobile facility)
    Platform,
    /// Custom node type
    Custom(String),
}

/// Needline - logical information requirement between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Needline {
    /// Needline ID
    pub id: String,
    /// Needline name
    pub name: String,
    /// Source node ID
    pub source_node: String,
    /// Target node ID
    pub target_node: String,
    /// Information elements exchanged
    pub information_elements: Vec<String>,
    /// Criticality of this needline
    pub criticality: Criticality,
    /// Frequency of exchange
    pub frequency: Option<crate::dodaf::ov5::Frequency>,
    /// Timeliness requirement (e.g., "Real-time", "Near real-time")
    pub timeliness: Option<String>,
}

/// Material needline - physical materiel requirement between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialNeedline {
    /// Needline ID
    pub id: String,
    /// Source node ID
    pub source_node: String,
    /// Target node ID
    pub target_node: String,
    /// Type of materiel
    pub materiel_type: String,
    /// Quantity of materiel
    pub quantity: Option<crate::dodaf::ov5::Quantity>,
    /// Mode of transportation
    pub transportation_mode: Option<String>,
}

/// Performer connection - movement or assignment of performers between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformerConnection {
    /// Connection ID
    pub id: String,
    /// Performer ID
    pub performer: String,
    /// Source node
    pub from_node: String,
    /// Target node
    pub to_node: String,
    /// Type of connection/movement
    pub connection_type: PerformerConnectionType,
}

/// Type of performer connection
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PerformerConnectionType {
    /// Performer assigned to node
    Assignment,
    /// Performer reassigned to different node
    Reassignment,
    /// Coordination between performers at different nodes
    Coordination,
    /// Reporting relationship
    Reporting,
}

/// Criticality level of information/resource flow
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Criticality {
    /// Mission-critical information
    Critical,
    /// Essential information
    Essential,
    /// Important information
    Important,
    /// Desirable information
    Desirable,
}

// ============================================================================
// Helper implementations
// ============================================================================

impl OperationalNodeConnectivity {
    /// Create a new operational node connectivity description
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            version: "1.0".to_string(),
            nodes: Vec::new(),
            needlines: Vec::new(),
            materialneedlines: Vec::new(),
            performer_connections: Vec::new(),
            node_count: 0,
            flow_count: 0,
            created_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add a node
    pub fn add_node(mut self, node: OperationalNode) -> Self {
        self.node_count += 1;
        self.nodes.push(node);
        self
    }

    /// Add a needline
    pub fn add_needline(mut self, needline: Needline) -> Self {
        self.flow_count += 1;
        self.needlines.push(needline);
        self
    }

    /// Add a materiel needline
    pub fn add_materiel_needline(mut self, needline: MaterialNeedline) -> Self {
        self.flow_count += 1;
        self.materialneedlines.push(needline);
        self
    }

    /// Add a performer connection
    pub fn add_performer_connection(mut self, connection: PerformerConnection) -> Self {
        self.performer_connections.push(connection);
        self
    }
}

impl OperationalNode {
    /// Create a new operational node
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        node_type: OperationalNodeType,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            node_type,
            activities: Vec::new(),
            performers: Vec::new(),
            location: None,
            operational_context: None,
            inbound_flows: Vec::new(),
            outbound_flows: Vec::new(),
            properties: HashMap::new(),
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set location
    pub fn with_location(mut self, location: crate::dodaf::ov5::Location) -> Self {
        self.location = Some(location);
        self
    }

    /// Set operational context
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.operational_context = Some(context.into());
        self
    }

    /// Add an activity
    pub fn add_activity(mut self, activity_id: impl Into<String>) -> Self {
        self.activities.push(activity_id.into());
        self
    }

    /// Add a performer
    pub fn add_performer(mut self, performer_id: impl Into<String>) -> Self {
        self.performers.push(performer_id.into());
        self
    }

    /// Register inbound flow
    pub fn add_inbound_flow(mut self, flow_id: impl Into<String>) -> Self {
        self.inbound_flows.push(flow_id.into());
        self
    }

    /// Register outbound flow
    pub fn add_outbound_flow(mut self, flow_id: impl Into<String>) -> Self {
        self.outbound_flows.push(flow_id.into());
        self
    }
}

impl Needline {
    /// Create a new needline
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
        criticality: Criticality,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            source_node: source.into(),
            target_node: target.into(),
            information_elements: Vec::new(),
            criticality,
            frequency: None,
            timeliness: None,
        }
    }

    /// Add an information element
    pub fn add_information_element(mut self, element: impl Into<String>) -> Self {
        self.information_elements.push(element.into());
        self
    }

    /// Set frequency
    pub fn with_frequency(mut self, freq: crate::dodaf::ov5::Frequency) -> Self {
        self.frequency = Some(freq);
        self
    }

    /// Set timeliness
    pub fn with_timeliness(mut self, timeliness: impl Into<String>) -> Self {
        self.timeliness = Some(timeliness.into());
        self
    }
}

impl MaterialNeedline {
    /// Create a new material needline
    pub fn new(
        id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
        materiel_type: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            source_node: source.into(),
            target_node: target.into(),
            materiel_type: materiel_type.into(),
            quantity: None,
            transportation_mode: None,
        }
    }

    /// Set quantity
    pub fn with_quantity(mut self, quantity: crate::dodaf::ov5::Quantity) -> Self {
        self.quantity = Some(quantity);
        self
    }

    /// Set transportation mode
    pub fn with_transport_mode(mut self, mode: impl Into<String>) -> Self {
        self.transportation_mode = Some(mode.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_node_connectivity() {
        let desc = OperationalNodeConnectivity::new("ov2_1", "Node Connectivity")
            .with_description("Operational node connectivity description");

        assert_eq!(desc.name, "Node Connectivity");
        assert_eq!(desc.node_count, 0);
        assert_eq!(desc.flow_count, 0);
    }

    #[test]
    fn test_add_nodes_and_needlines() {
        let node1 =
            OperationalNode::new("node1", "Command Center", OperationalNodeType::Organization);
        let node2 = OperationalNode::new("node2", "Field Unit", OperationalNodeType::Organization);

        let needline =
            Needline::new("nl1", "Command Flow", "node1", "node2", Criticality::Critical)
                .add_information_element("tactical_order");

        let desc = OperationalNodeConnectivity::new("ov2_1", "Test")
            .add_node(node1)
            .add_node(node2)
            .add_needline(needline);

        assert_eq!(desc.node_count, 2);
        assert_eq!(desc.flow_count, 1);
        assert_eq!(desc.nodes.len(), 2);
        assert_eq!(desc.needlines.len(), 1);
    }

    #[test]
    fn test_criticality_ordering() {
        assert_eq!(Criticality::Critical, Criticality::Critical);
        assert_ne!(Criticality::Critical, Criticality::Desirable);
    }

    #[test]
    fn test_performer_connections() {
        let conn = PerformerConnection {
            id: "pc1".to_string(),
            performer: "performer1".to_string(),
            from_node: "node1".to_string(),
            to_node: "node2".to_string(),
            connection_type: PerformerConnectionType::Reassignment,
        };

        assert_eq!(conn.connection_type, PerformerConnectionType::Reassignment);
    }
}
