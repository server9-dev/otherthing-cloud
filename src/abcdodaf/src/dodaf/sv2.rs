//! DoDAF 2.02 SV-2: Systems Resource Flow Description
//!
//! Documents communication systems, links, networks, and media supporting
//! systems and their interfaces.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// SV-2 Systems Resource Flow Description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemsResourceFlowDescription {
    /// Unique identifier
    pub id: String,
    /// Name of the description
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Version number
    pub version: String,

    /// Systems IDs that are connected
    pub systems: Vec<String>,
    /// Communication systems in the architecture
    pub communications_systems: Vec<CommunicationSystem>,
    /// Communication links between systems
    pub communications_links: Vec<CommunicationLink>,
    /// Communication networks
    pub communications_networks: Vec<CommunicationNetwork>,
    /// Information flows over communication systems
    pub information_flows: Vec<SystemInformationFlow>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Communication system - media and protocol for information exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationSystem {
    /// System ID
    pub id: String,
    /// System name
    pub name: String,
    /// System description
    pub description: Option<String>,
    /// Type of communication
    pub system_type: CommunicationType,
    /// Type of media used
    pub media_type: MediaType,
    /// Protocol used
    pub protocol: String,
}

/// Type of communication
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommunicationType {
    /// Voice communication
    Voice,
    /// Data communication
    Data,
    /// Video communication
    Video,
    /// Messaging/email
    Messaging,
    /// Service interface
    ServiceInterface,
}

/// Type of communication media
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaType {
    /// Radio transmission
    Radio,
    /// Wire/cable transmission
    Wire,
    /// Fiber optic transmission
    Fiber,
    /// Satellite transmission
    Satellite,
    /// Wireless transmission
    Wireless,
    /// Custom media type
    Custom(String),
}

/// Physical or logical communication link between systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationLink {
    /// Link ID
    pub id: String,
    /// Link name
    pub name: Option<String>,
    /// Source system ID
    pub from_system: String,
    /// Target system ID
    pub to_system: String,
    /// Communication system used for this link
    pub communication_system: String,

    /// Bandwidth capacity
    pub bandwidth: Option<String>,
    /// Latency specification
    pub latency: Option<String>,
    /// Reliability requirement (0-1.0)
    pub reliability: Option<f64>,
}

/// Communication network - collection of linked systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationNetwork {
    /// Network ID
    pub id: String,
    /// Network name
    pub name: String,
    /// Network description
    pub description: Option<String>,
    /// Type of network
    pub network_type: NetworkType,
    /// Member system IDs
    pub member_systems: Vec<String>,
    /// Communication link IDs in this network
    pub communication_links: Vec<String>,
}

/// Type of communication network
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkType {
    /// Local Area Network
    LAN,
    /// Wide Area Network
    WAN,
    /// Internet-based network
    Internet,
    /// Intranet (internal network)
    Intranet,
    /// Extranet (external network)
    Extranet,
    /// Peer-to-peer network
    P2P,
    /// Custom network type
    Custom(String),
}

/// Information flow over a communication system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInformationFlow {
    /// Flow ID
    pub id: String,
    /// Flow name
    pub name: Option<String>,
    /// Source system ID
    pub from_system: String,
    /// Target system ID
    pub to_system: String,
    /// Information element IDs flowing
    pub information_elements: Vec<String>,
    /// Communication link used
    pub communication_link: String,

    /// Flow attributes
    pub flow_attributes: FlowAttributes,
}

/// Attributes of an information flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowAttributes {
    /// Throughput requirement
    pub throughput: Option<String>,
    /// Latency requirement
    pub latency_requirement: Option<String>,
    /// Availability requirement (0-1.0)
    pub availability_requirement: Option<f64>,
    /// Security requirement
    pub security_requirement: Option<String>,
}

// ============================================================================
// Helper implementations
// ============================================================================

impl SystemsResourceFlowDescription {
    /// Create a new systems resource flow description
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            version: "1.0".to_string(),
            systems: Vec::new(),
            communications_systems: Vec::new(),
            communications_links: Vec::new(),
            communications_networks: Vec::new(),
            information_flows: Vec::new(),
            created_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add a system
    pub fn add_system(mut self, system_id: impl Into<String>) -> Self {
        self.systems.push(system_id.into());
        self
    }

    /// Add a communication system
    pub fn add_communication_system(mut self, system: CommunicationSystem) -> Self {
        self.communications_systems.push(system);
        self
    }

    /// Add a communication link
    pub fn add_link(mut self, link: CommunicationLink) -> Self {
        self.communications_links.push(link);
        self
    }

    /// Add a communication network
    pub fn add_network(mut self, network: CommunicationNetwork) -> Self {
        self.communications_networks.push(network);
        self
    }

    /// Add an information flow
    pub fn add_information_flow(mut self, flow: SystemInformationFlow) -> Self {
        self.information_flows.push(flow);
        self
    }

    /// Get links from a system
    pub fn get_outbound_links(&self, from_system: &str) -> Vec<&CommunicationLink> {
        self.communications_links
            .iter()
            .filter(|l| l.from_system == from_system)
            .collect()
    }

    /// Get links to a system
    pub fn get_inbound_links(&self, to_system: &str) -> Vec<&CommunicationLink> {
        self.communications_links.iter().filter(|l| l.to_system == to_system).collect()
    }
}

impl CommunicationSystem {
    /// Create a new communication system
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        comm_type: CommunicationType,
        media: MediaType,
        protocol: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            system_type: comm_type,
            media_type: media,
            protocol: protocol.into(),
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

impl CommunicationLink {
    /// Create a new communication link
    pub fn new(
        id: impl Into<String>,
        from_system: impl Into<String>,
        to_system: impl Into<String>,
        communication_system: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: None,
            from_system: from_system.into(),
            to_system: to_system.into(),
            communication_system: communication_system.into(),
            bandwidth: None,
            latency: None,
            reliability: None,
        }
    }

    /// Set name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set bandwidth
    pub fn with_bandwidth(mut self, bandwidth: impl Into<String>) -> Self {
        self.bandwidth = Some(bandwidth.into());
        self
    }

    /// Set latency
    pub fn with_latency(mut self, latency: impl Into<String>) -> Self {
        self.latency = Some(latency.into());
        self
    }

    /// Set reliability
    pub fn with_reliability(mut self, reliability: f64) -> Self {
        self.reliability = Some(reliability);
        self
    }
}

impl CommunicationNetwork {
    /// Create a new communication network
    pub fn new(id: impl Into<String>, name: impl Into<String>, network_type: NetworkType) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            network_type,
            member_systems: Vec::new(),
            communication_links: Vec::new(),
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add a member system
    pub fn add_member_system(mut self, system_id: impl Into<String>) -> Self {
        self.member_systems.push(system_id.into());
        self
    }

    /// Add a communication link
    pub fn add_link(mut self, link_id: impl Into<String>) -> Self {
        self.communication_links.push(link_id.into());
        self
    }
}

impl SystemInformationFlow {
    /// Create a new information flow
    pub fn new(
        id: impl Into<String>,
        from_system: impl Into<String>,
        to_system: impl Into<String>,
        communication_link: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: None,
            from_system: from_system.into(),
            to_system: to_system.into(),
            information_elements: Vec::new(),
            communication_link: communication_link.into(),
            flow_attributes: FlowAttributes::default(),
        }
    }

    /// Set name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Add an information element
    pub fn add_information_element(mut self, element_id: impl Into<String>) -> Self {
        self.information_elements.push(element_id.into());
        self
    }

    /// Set flow attributes
    pub fn with_attributes(mut self, attrs: FlowAttributes) -> Self {
        self.flow_attributes = attrs;
        self
    }
}

impl Default for FlowAttributes {
    fn default() -> Self {
        Self {
            throughput: None,
            latency_requirement: None,
            availability_requirement: None,
            security_requirement: None,
        }
    }
}

impl FlowAttributes {
    /// Create with availability requirement
    pub fn with_availability(availability: f64) -> Self {
        Self { availability_requirement: Some(availability), ..Default::default() }
    }

    /// Set throughput
    pub fn with_throughput(mut self, throughput: impl Into<String>) -> Self {
        self.throughput = Some(throughput.into());
        self
    }

    /// Set latency requirement
    pub fn with_latency(mut self, latency: impl Into<String>) -> Self {
        self.latency_requirement = Some(latency.into());
        self
    }

    /// Set security requirement
    pub fn with_security(mut self, security: impl Into<String>) -> Self {
        self.security_requirement = Some(security.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_resource_flow() {
        let desc = SystemsResourceFlowDescription::new("sv2_1", "Resource Flow")
            .with_description("Systems resource flows and communications");

        assert_eq!(desc.name, "Resource Flow");
        assert_eq!(desc.systems.len(), 0);
        assert_eq!(desc.communications_links.len(), 0);
    }

    #[test]
    fn test_communication_system() {
        let comm_sys = CommunicationSystem::new(
            "comm_1",
            "TCP/IP Network",
            CommunicationType::Data,
            MediaType::Fiber,
            "TCP/IP",
        )
        .with_description("High-speed fiber network");

        assert_eq!(comm_sys.system_type, CommunicationType::Data);
        assert_eq!(comm_sys.media_type, MediaType::Fiber);
    }

    #[test]
    fn test_communication_links() {
        let link = CommunicationLink::new("link_1", "sys_1", "sys_2", "comm_1")
            .with_bandwidth("100 Mbps")
            .with_latency("10ms")
            .with_reliability(0.99);

        assert_eq!(link.from_system, "sys_1");
        assert_eq!(link.to_system, "sys_2");
        assert_eq!(link.reliability, Some(0.99));
    }

    #[test]
    fn test_communication_network() {
        let network =
            CommunicationNetwork::new("net_1", "Corporate Network", NetworkType::Intranet)
                .add_member_system("sys_1")
                .add_member_system("sys_2")
                .add_link("link_1");

        assert_eq!(network.network_type, NetworkType::Intranet);
        assert_eq!(network.member_systems.len(), 2);
        assert_eq!(network.communication_links.len(), 1);
    }

    #[test]
    fn test_information_flow() {
        let attrs = FlowAttributes::with_availability(0.999).with_security("Top Secret");

        let flow = SystemInformationFlow::new("flow_1", "sys_1", "sys_2", "link_1")
            .add_information_element("ie_1")
            .with_attributes(attrs);

        assert_eq!(flow.information_elements.len(), 1);
        assert!(flow.flow_attributes.availability_requirement.is_some());
    }

    #[test]
    fn test_media_types() {
        assert_eq!(MediaType::Radio, MediaType::Radio);
        assert_ne!(MediaType::Radio, MediaType::Fiber);
    }
}
