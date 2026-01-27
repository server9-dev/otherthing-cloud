//! DoDAF 2.02 SV-1: Systems Interface Description
//!
//! Specifies composition and interaction of systems, showing how resources
//! are structured and interact to realize the operational architecture.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// SV-1 Systems Interface Description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemsInterfaceDescription {
    /// Unique identifier
    pub id: String,
    /// Name of the systems description
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Version number
    pub version: String,

    /// Systems in the architecture
    pub systems: Vec<System>,
    /// Interfaces between systems
    pub interfaces: Vec<SystemInterface>,
    /// Ports on systems
    pub ports: Vec<SystemPort>,
    /// Aggregates of systems
    pub system_aggregates: Vec<SystemAggregate>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// System - a distinct computational or physical resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct System {
    /// System ID
    pub id: String,
    /// System name
    pub name: String,
    /// System description
    pub description: Option<String>,
    /// Type of system
    pub system_type: SystemType,

    /// Functions this system performs
    pub functions: Vec<String>,
    /// Ports provided by this system
    pub ports: Vec<String>,
    /// Sub-systems (for hierarchical systems)
    pub sub_systems: Vec<String>,
    /// Location where system is deployed
    pub located_at: Option<String>,

    /// Additional properties
    pub properties: HashMap<String, serde_json::Value>,
}

/// Type of system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemType {
    /// Hardware system
    Hardware,
    /// Software system
    Software,
    /// Hybrid hardware/software system
    Hybrid,
    /// Manual system (people-based)
    Manual,
    /// Organic system (natural/biological)
    Organic,
}

/// Interface between systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInterface {
    /// Interface ID
    pub id: String,
    /// Interface name
    pub name: Option<String>,
    /// Source system ID
    pub source_system: String,
    /// Target system ID
    pub target_system: String,
    /// Source port ID
    pub source_port: String,
    /// Target port ID
    pub target_port: String,

    /// Type of interface
    pub interface_type: InterfaceType,
    /// Interface description
    pub description: Option<String>,
}

/// Type of system interface
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterfaceType {
    /// Data interface
    Data,
    /// Control interface
    Control,
    /// Service interface
    Service,
    /// Physical connection
    Physical,
    /// Power interface
    Power,
}

/// Port on a system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPort {
    /// Port ID
    pub id: String,
    /// Port name
    pub name: String,
    /// Type of port
    pub port_type: PortType,
    /// System this port belongs to
    pub belongs_to_system: String,

    /// Communication protocol
    pub protocol: Option<String>,
    /// Direction of data flow
    pub direction: PortDirection,
    /// Cardinality (how many connections allowed)
    pub cardinality: Option<String>,

    /// Additional properties
    pub properties: HashMap<String, serde_json::Value>,
}

/// Type of port
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortType {
    /// Input port
    Input,
    /// Output port
    Output,
    /// Bidirectional port
    Bidirectional,
    /// Control port
    Control,
    /// Data port
    Data,
    /// Power port
    Power,
}

/// Direction of data/control flow
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortDirection {
    /// Inbound only
    In,
    /// Outbound only
    Out,
    /// Both inbound and outbound
    InOut,
}

/// Aggregate of systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemAggregate {
    /// Aggregate ID
    pub id: String,
    /// Aggregate name
    pub name: String,
    /// Member system IDs
    pub member_systems: Vec<String>,
    /// Type of aggregate
    pub aggregate_type: String,
}

// ============================================================================
// Helper implementations
// ============================================================================

impl SystemsInterfaceDescription {
    /// Create a new systems interface description
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            version: "1.0".to_string(),
            systems: Vec::new(),
            interfaces: Vec::new(),
            ports: Vec::new(),
            system_aggregates: Vec::new(),
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
    pub fn add_system(mut self, system: System) -> Self {
        self.systems.push(system);
        self
    }

    /// Add an interface
    pub fn add_interface(mut self, interface: SystemInterface) -> Self {
        self.interfaces.push(interface);
        self
    }

    /// Add a port
    pub fn add_port(mut self, port: SystemPort) -> Self {
        self.ports.push(port);
        self
    }

    /// Add a system aggregate
    pub fn add_aggregate(mut self, aggregate: SystemAggregate) -> Self {
        self.system_aggregates.push(aggregate);
        self
    }

    /// Get all ports for a system
    pub fn get_system_ports(&self, system_id: &str) -> Vec<&SystemPort> {
        self.ports.iter().filter(|p| p.belongs_to_system == system_id).collect()
    }

    /// Get all interfaces for a system
    pub fn get_system_interfaces(&self, system_id: &str) -> Vec<&SystemInterface> {
        self.interfaces
            .iter()
            .filter(|i| i.source_system == system_id || i.target_system == system_id)
            .collect()
    }
}

impl System {
    /// Create a new system
    pub fn new(id: impl Into<String>, name: impl Into<String>, system_type: SystemType) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            system_type,
            functions: Vec::new(),
            ports: Vec::new(),
            sub_systems: Vec::new(),
            located_at: None,
            properties: HashMap::new(),
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add a function
    pub fn add_function(mut self, function_id: impl Into<String>) -> Self {
        self.functions.push(function_id.into());
        self
    }

    /// Add a port
    pub fn add_port(mut self, port_id: impl Into<String>) -> Self {
        self.ports.push(port_id.into());
        self
    }

    /// Add a sub-system
    pub fn add_subsystem(mut self, subsystem_id: impl Into<String>) -> Self {
        self.sub_systems.push(subsystem_id.into());
        self
    }

    /// Set location
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.located_at = Some(location.into());
        self
    }
}

impl SystemPort {
    /// Create a new port
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        port_type: PortType,
        system_id: impl Into<String>,
        direction: PortDirection,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            port_type,
            belongs_to_system: system_id.into(),
            protocol: None,
            direction,
            cardinality: None,
            properties: HashMap::new(),
        }
    }

    /// Set protocol
    pub fn with_protocol(mut self, protocol: impl Into<String>) -> Self {
        self.protocol = Some(protocol.into());
        self
    }

    /// Set cardinality
    pub fn with_cardinality(mut self, cardinality: impl Into<String>) -> Self {
        self.cardinality = Some(cardinality.into());
        self
    }
}

impl SystemInterface {
    /// Create a new interface
    pub fn new(
        id: impl Into<String>,
        source_system: impl Into<String>,
        target_system: impl Into<String>,
        source_port: impl Into<String>,
        target_port: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: None,
            source_system: source_system.into(),
            target_system: target_system.into(),
            source_port: source_port.into(),
            target_port: target_port.into(),
            interface_type: InterfaceType::Data,
            description: None,
        }
    }

    /// Set name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set interface type
    pub fn with_type(mut self, iface_type: InterfaceType) -> Self {
        self.interface_type = iface_type;
        self
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_interface_description() {
        let desc = SystemsInterfaceDescription::new("sv1_1", "Systems Interface")
            .with_description("System architecture and interfaces");

        assert_eq!(desc.name, "Systems Interface");
        assert_eq!(desc.systems.len(), 0);
        assert_eq!(desc.interfaces.len(), 0);
    }

    #[test]
    fn test_systems_and_ports() {
        let sys1 = System::new("sys_1", "Command System", SystemType::Software);
        let sys2 = System::new("sys_2", "Sensor System", SystemType::Hardware);

        let port1 =
            SystemPort::new("port_1", "Command Out", PortType::Output, "sys_1", PortDirection::Out)
                .with_protocol("TCP");

        let port2 =
            SystemPort::new("port_2", "Data In", PortType::Input, "sys_2", PortDirection::In)
                .with_protocol("TCP");

        let desc = SystemsInterfaceDescription::new("sv1_1", "Test")
            .add_system(sys1)
            .add_system(sys2)
            .add_port(port1)
            .add_port(port2);

        assert_eq!(desc.systems.len(), 2);
        assert_eq!(desc.ports.len(), 2);
        assert_eq!(desc.get_system_ports("sys_1").len(), 1);
    }

    #[test]
    fn test_system_interface() {
        let iface = SystemInterface::new("if_1", "sys_1", "sys_2", "port_1", "port_2")
            .with_name("Command to Sensor")
            .with_type(InterfaceType::Data)
            .with_description("Sends commands to sensor");

        assert_eq!(iface.source_system, "sys_1");
        assert_eq!(iface.target_system, "sys_2");
        assert_eq!(iface.interface_type, InterfaceType::Data);
    }

    #[test]
    fn test_port_types() {
        assert_eq!(PortType::Input, PortType::Input);
        assert_ne!(PortType::Input, PortType::Output);
    }

    #[test]
    fn test_system_types() {
        assert_eq!(SystemType::Software, SystemType::Software);
        assert_ne!(SystemType::Hardware, SystemType::Software);
    }
}
