//! Operational Viewpoint (OV) - DoDAF 2.02
//!
//! Describes operational scenarios, activities, and information exchanges
//! required to accomplish missions and objectives.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Operational View container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalView {
    /// Mission areas
    pub mission_areas: Vec<MissionArea>,
    /// Operational activities
    pub activities: Vec<OperationalActivity>,
    /// Information exchanges
    pub information_exchanges: Vec<InformationExchange>,
}

impl OperationalView {
    /// Create a new operational view
    pub fn new() -> Self {
        Self {
            mission_areas: Vec::new(),
            activities: Vec::new(),
            information_exchanges: Vec::new(),
        }
    }

    /// Add a mission area
    pub fn add_mission_area(mut self, mission: MissionArea) -> Self {
        self.mission_areas.push(mission);
        self
    }

    /// Add an operational activity
    pub fn add_activity(mut self, activity: OperationalActivity) -> Self {
        self.activities.push(activity);
        self
    }

    /// Add an information exchange
    pub fn add_information_exchange(mut self, exchange: InformationExchange) -> Self {
        self.information_exchanges.push(exchange);
        self
    }
}

impl Default for OperationalView {
    fn default() -> Self {
        Self::new()
    }
}

/// Mission area grouping operational activities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionArea {
    /// Mission area ID
    pub id: String,
    /// Mission area name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Related capability IDs
    pub capabilities: Vec<String>,
}

/// Operational activity in DoDAF
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalActivity {
    /// Activity ID
    pub id: String,
    /// Activity name
    pub name: String,
    /// Activity description
    pub description: Option<String>,
    /// Activity type
    pub activity_type: ActivityType,
    /// Performing nodes (agents, humans, systems)
    pub performers: Vec<String>,
    /// Input information elements
    pub inputs: Vec<String>,
    /// Output information elements
    pub outputs: Vec<String>,
    /// Success criteria
    pub success_criteria: Vec<String>,
    /// Resource requirements
    pub resources: HashMap<String, serde_json::Value>,
}

/// Type of operational activity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityType {
    /// Automated activity (fully agentic)
    Automated,
    /// Human-performed activity
    Manual,
    /// Hybrid (human + AI collaboration)
    Hybrid,
    /// Decision-making activity
    Decision,
    /// Information processing
    Processing,
    /// Communication/coordination
    Communication,
}

/// Information exchange between operational nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InformationExchange {
    /// Exchange ID
    pub id: String,
    /// Source node/activity
    pub source: String,
    /// Target node/activity
    pub target: String,
    /// Information elements exchanged
    pub information_elements: Vec<String>,
    /// Exchange mechanism (API, message queue, etc.)
    pub mechanism: String,
}

/// Operational context for workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalContext {
    /// Context ID
    pub id: Uuid,
    /// Mission area
    pub mission_area: Option<String>,
    /// Required capabilities
    pub required_capabilities: Vec<String>,
    /// Available resources
    pub resources: HashMap<String, serde_json::Value>,
    /// Performance constraints
    pub constraints: HashMap<String, serde_json::Value>,
    /// Context metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl OperationalContext {
    /// Create a new operational context
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            mission_area: None,
            required_capabilities: Vec::new(),
            resources: HashMap::new(),
            constraints: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set mission area
    pub fn with_mission_area(mut self, mission: impl Into<String>) -> Self {
        self.mission_area = Some(mission.into());
        self
    }

    /// Add required capability
    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.required_capabilities.push(capability.into());
        self
    }

    /// Add resource
    pub fn with_resource(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.resources.insert(key.into(), value);
        self
    }

    /// Add constraint
    pub fn with_constraint(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.constraints.insert(key.into(), value);
        self
    }
}

impl Default for OperationalContext {
    fn default() -> Self {
        Self::new()
    }
}

impl OperationalActivity {
    /// Create a new operational activity
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            activity_type: ActivityType::Processing,
            performers: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            success_criteria: Vec::new(),
            resources: HashMap::new(),
        }
    }

    /// Set activity type
    pub fn with_type(mut self, activity_type: ActivityType) -> Self {
        self.activity_type = activity_type;
        self
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add a performer
    pub fn add_performer(mut self, performer: impl Into<String>) -> Self {
        self.performers.push(performer.into());
        self
    }

    /// Add an input
    pub fn add_input(mut self, input: impl Into<String>) -> Self {
        self.inputs.push(input.into());
        self
    }

    /// Add an output
    pub fn add_output(mut self, output: impl Into<String>) -> Self {
        self.outputs.push(output.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operational_context() {
        let context = OperationalContext::new()
            .with_mission_area("AI Workforce Operations")
            .with_capability("Natural Language Processing")
            .with_resource("max_tokens", serde_json::json!(1000));

        assert_eq!(context.mission_area, Some("AI Workforce Operations".to_string()));
        assert_eq!(context.required_capabilities.len(), 1);
        assert_eq!(context.resources.len(), 1);
    }

    #[test]
    fn test_operational_activity() {
        let activity = OperationalActivity::new("act1", "Process Request")
            .with_type(ActivityType::Automated)
            .add_performer("AI Agent")
            .add_input("user_request")
            .add_output("processed_result");

        assert_eq!(activity.activity_type, ActivityType::Automated);
        assert_eq!(activity.performers.len(), 1);
    }
}
