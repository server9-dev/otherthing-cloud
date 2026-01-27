//! DODAF Performer Types
//!
//! Enhanced performer taxonomy for OV-4 (Organizational Relationships)
//! Supports mapping BPMN lanes and participants to DODAF performers

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Performer - entity that performs activities
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Performer {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub performer_type: PerformerType,
    pub metadata: HashMap<String, String>,
}

/// Comprehensive performer type taxonomy
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PerformerType {
    /// Individual person
    Person {
        role: String,
        organization: String,
        skills: Vec<String>,
        clearance_level: Option<String>,
    },

    /// Organizational unit
    Organization {
        name: String,
        org_type: OrganizationType,
        parent_org: Option<String>,
        location: Option<String>,
    },

    /// Software service
    Service {
        name: String,
        service_type: ServiceType,
        interface: String,
        endpoint: Option<String>,
        version: Option<String>,
    },

    /// Service interface (API, protocol)
    ServiceInterface {
        protocol: String,
        specification: String,
        operations: Vec<String>,
    },

    /// Physical or logical system
    System {
        name: String,
        system_type: SystemType,
        version: String,
        vendor: Option<String>,
    },

    /// Hardware/equipment
    Equipment {
        name: String,
        equipment_type: String,
        model: Option<String>,
        serial_number: Option<String>,
    },
}

/// Organization type categories
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrganizationType {
    Command,
    Agency,
    Department,
    Division,
    Branch,
    Team,
    Unit,
    Contractor,
    Partner,
}

/// Service type categories
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceType {
    WebService,
    RestApi,
    GraphQLApi,
    MessageQueue,
    Database,
    FileSystem,
    AIAgent,
    LLM,
    Custom(String),
}

/// System type categories
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemType {
    ComputerSystem,
    NetworkSystem,
    StorageSystem,
    SecuritySystem,
    CommunicationSystem,
    ApplicationSystem,
    DatabaseSystem,
}

/// Performer assignment to activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformerAssignment {
    pub activity_id: String,
    pub performer_id: String,
    pub assignment_type: AssignmentType,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub allocation_percentage: Option<f64>,
}

/// Type of performer assignment
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssignmentType {
    Primary,
    Secondary,
    Backup,
    Reviewer,
    Approver,
}

/// Mapping from BPMN to DODAF performers
pub struct PerformerMapper;

impl PerformerMapper {
    /// Create performer from BPMN lane
    pub fn from_bpmn_lane(lane_id: &str, lane_name: &str) -> Performer {
        // Heuristic: detect performer type from lane name
        let performer_type = if lane_name.contains("System") || lane_name.contains("Service") {
            PerformerType::System {
                name: lane_name.to_string(),
                system_type: SystemType::ApplicationSystem,
                version: "1.0".to_string(),
                vendor: None,
            }
        } else if lane_name.contains("API") || lane_name.contains("Agent") {
            PerformerType::Service {
                name: lane_name.to_string(),
                service_type: ServiceType::AIAgent,
                interface: "REST".to_string(),
                endpoint: None,
                version: None,
            }
        } else if lane_name.contains("Team") || lane_name.contains("Department") {
            PerformerType::Organization {
                name: lane_name.to_string(),
                org_type: OrganizationType::Team,
                parent_org: None,
                location: None,
            }
        } else {
            // Default to person
            PerformerType::Person {
                role: lane_name.to_string(),
                organization: "Unknown".to_string(),
                skills: vec![],
                clearance_level: None,
            }
        };

        Performer {
            id: lane_id.to_string(),
            name: lane_name.to_string(),
            description: None,
            performer_type,
            metadata: HashMap::new(),
        }
    }

    /// Create performer assignment from BPMN task
    pub fn assign_task_to_lane(
        task_id: &str,
        lane_id: &str,
        assignment_type: AssignmentType,
    ) -> PerformerAssignment {
        PerformerAssignment {
            activity_id: task_id.to_string(),
            performer_id: lane_id.to_string(),
            assignment_type,
            start_time: None,
            end_time: None,
            allocation_percentage: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performer_from_lane() {
        let performer = PerformerMapper::from_bpmn_lane("lane_1", "API Gateway Service");

        assert_eq!(performer.id, "lane_1");
        assert_eq!(performer.name, "API Gateway Service");

        if let PerformerType::Service { service_type, .. } = performer.performer_type {
            assert_eq!(service_type, ServiceType::AIAgent);
        } else {
            panic!("Expected Service performer type");
        }
    }

    #[test]
    fn test_performer_assignment() {
        let assignment = PerformerMapper::assign_task_to_lane(
            "task_1",
            "lane_1",
            AssignmentType::Primary,
        );

        assert_eq!(assignment.activity_id, "task_1");
        assert_eq!(assignment.performer_id, "lane_1");
        assert_eq!(assignment.assignment_type, AssignmentType::Primary);
    }
}
