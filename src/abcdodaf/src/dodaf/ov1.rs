//! DoDAF 2.02 OV-1: High-Level Operational Concept Graphic
//!
//! Provides graphical and textual description of operational concept including
//! high-level organizations, missions, geographic configuration, and external interactions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OV-1 High-Level Operational Concept Graphic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalConceptGraphic {
    /// Unique identifier
    pub id: String,
    /// Name of the operational concept
    pub name: String,
    /// Description of the concept
    pub description: Option<String>,
    /// Version number
    pub version: String,

    // Strategic Context
    /// Mission statement or purpose
    pub mission_statement: String,
    /// Operational scenarios
    pub operational_scenarios: Vec<OperationalScenario>,
    /// Strategic vision for operations
    pub strategic_vision: Option<String>,

    // High-Level Organizations
    /// Organizations involved in operations
    pub organizations: Vec<OperationalOrganization>,

    // Geographic Context
    /// Geographic distribution of operations
    pub geographic_config: Option<GeographicConfiguration>,

    // External Interactions
    /// External systems interacting with this architecture
    pub external_systems: Vec<ExternalSystem>,
    /// Interfaces to external systems
    pub interfaces: Vec<OperationalInterface>,

    // Relationships to Other Views
    /// Reference to CV-1 Vision
    pub relates_to_cv1: Option<String>,
    /// Reference to OV-2 Node Connectivity
    pub relates_to_ov2: Option<String>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Operational scenario - describes a specific mode of operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalScenario {
    /// Scenario ID
    pub id: String,
    /// Scenario name
    pub name: String,
    /// Scenario description
    pub description: String,
    /// Event that initiates this scenario
    pub initiating_event: Option<String>,
    /// Key activities performed in this scenario
    pub key_activities: Vec<String>,
    /// Expected duration
    pub duration: Option<crate::dodaf::ov5::Duration>,
}

/// High-level operational organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalOrganization {
    /// Organization ID
    pub id: String,
    /// Organization name
    pub name: String,
    /// Type of organization
    pub org_type: OrganizationType,
    /// Parent organization ID (for hierarchical relationships)
    pub parent_org: Option<String>,
    /// Responsibilities or mission of this organization
    pub responsibilities: Vec<String>,
    /// Primary location
    pub location: Option<crate::dodaf::ov5::Location>,
}

/// Type of operational organization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrganizationType {
    /// Command organization
    Command,
    /// Support organization
    Support,
    /// Intelligence organization
    Intelligence,
    /// Operations organization
    Operations,
    /// Logistics organization
    Logistics,
    /// Custom organization type
    Custom(String),
}

/// Geographic configuration of operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicConfiguration {
    /// Primary/main operational location
    pub primary_location: crate::dodaf::ov5::Location,
    /// Distributed operational nodes
    pub distributed_nodes: Vec<DistributedNode>,
    /// Communication network supporting geographic distribution
    pub communication_network: Option<String>,
}

/// Distributed operational node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedNode {
    /// Node name
    pub name: String,
    /// Physical location
    pub location: crate::dodaf::ov5::Location,
    /// Type of operational node
    pub node_type: crate::dodaf::ov5::OperationalNodeType,
    /// Organizations present at this node
    pub organizations_present: Vec<String>,
}

/// External system interacting with this architecture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSystem {
    /// External system ID
    pub id: String,
    /// System name
    pub name: String,
    /// Type of system
    pub system_type: String,
    /// System description
    pub description: Option<String>,
    /// IDs of systems this interfaces with
    pub interfaces_with: Vec<String>,
}

/// Interface between operational entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalInterface {
    /// Interface ID
    pub id: String,
    /// Interface name
    pub name: String,
    /// Source organization/system
    pub from_org: String,
    /// Target organization/system
    pub to_org: String,
    /// Type of interface
    pub interface_type: InterfaceType,
    /// Description of data/information flow
    pub data_flow: Option<String>,
}

/// Type of operational interface
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterfaceType {
    /// Information exchange interface
    Information,
    /// Physical/materiel interface
    Physical,
    /// Functional capability interface
    Functional,
    /// Service-based interface
    Service,
}

// ============================================================================
// Helper implementations
// ============================================================================

impl OperationalConceptGraphic {
    /// Create a new operational concept graphic
    pub fn new(id: impl Into<String>, name: impl Into<String>, mission: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            version: "1.0".to_string(),
            mission_statement: mission.into(),
            operational_scenarios: Vec::new(),
            strategic_vision: None,
            organizations: Vec::new(),
            geographic_config: None,
            external_systems: Vec::new(),
            interfaces: Vec::new(),
            relates_to_cv1: None,
            relates_to_ov2: None,
            created_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set strategic vision
    pub fn with_vision(mut self, vision: impl Into<String>) -> Self {
        self.strategic_vision = Some(vision.into());
        self
    }

    /// Add a scenario
    pub fn add_scenario(mut self, scenario: OperationalScenario) -> Self {
        self.operational_scenarios.push(scenario);
        self
    }

    /// Add an organization
    pub fn add_organization(mut self, org: OperationalOrganization) -> Self {
        self.organizations.push(org);
        self
    }

    /// Set geographic configuration
    pub fn with_geographic_config(mut self, config: GeographicConfiguration) -> Self {
        self.geographic_config = Some(config);
        self
    }

    /// Add an external system
    pub fn add_external_system(mut self, system: ExternalSystem) -> Self {
        self.external_systems.push(system);
        self
    }

    /// Add an interface
    pub fn add_interface(mut self, interface: OperationalInterface) -> Self {
        self.interfaces.push(interface);
        self
    }

    /// Set CV-1 reference
    pub fn with_cv1_reference(mut self, cv1_id: impl Into<String>) -> Self {
        self.relates_to_cv1 = Some(cv1_id.into());
        self
    }

    /// Set OV-2 reference
    pub fn with_ov2_reference(mut self, ov2_id: impl Into<String>) -> Self {
        self.relates_to_ov2 = Some(ov2_id.into());
        self
    }
}

impl OperationalOrganization {
    /// Create a new organization
    pub fn new(id: impl Into<String>, name: impl Into<String>, org_type: OrganizationType) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            org_type,
            parent_org: None,
            responsibilities: Vec::new(),
            location: None,
        }
    }

    /// Add a responsibility
    pub fn add_responsibility(mut self, resp: impl Into<String>) -> Self {
        self.responsibilities.push(resp.into());
        self
    }

    /// Set location
    pub fn with_location(mut self, location: crate::dodaf::ov5::Location) -> Self {
        self.location = Some(location);
        self
    }

    /// Set parent organization
    pub fn with_parent(mut self, parent_id: impl Into<String>) -> Self {
        self.parent_org = Some(parent_id.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_operational_concept() {
        let concept = OperationalConceptGraphic::new(
            "ov1_1",
            "Military Operations",
            "Execute joint operational mission",
        )
        .with_description("Joint command and control operations")
        .with_vision("Achieve operational dominance through integrated command");

        assert_eq!(concept.name, "Military Operations");
        assert_eq!(concept.mission_statement, "Execute joint operational mission");
        assert!(concept.description.is_some());
        assert!(concept.strategic_vision.is_some());
    }

    #[test]
    fn test_add_scenarios_and_organizations() {
        let scenario = OperationalScenario {
            id: "scen_1".to_string(),
            name: "Primary Scenario".to_string(),
            description: "Main operational scenario".to_string(),
            initiating_event: Some("Alert received".to_string()),
            key_activities: vec!["Assess".to_string(), "Plan".to_string(), "Execute".to_string()],
            duration: None,
        };

        let org =
            OperationalOrganization::new("org_1", "Command Center", OrganizationType::Command);

        let concept = OperationalConceptGraphic::new("ov1_1", "Test", "Test mission")
            .add_scenario(scenario)
            .add_organization(org);

        assert_eq!(concept.operational_scenarios.len(), 1);
        assert_eq!(concept.organizations.len(), 1);
    }

    #[test]
    fn test_interface_types() {
        assert_ne!(InterfaceType::Information, InterfaceType::Physical);
        assert_eq!(InterfaceType::Service, InterfaceType::Service);
    }
}
