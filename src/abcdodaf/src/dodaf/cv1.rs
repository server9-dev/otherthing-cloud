//! DoDAF 2.02 CV-1: Capability Vision
//!
//! Defines strategic context for capabilities over a period of time.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// CV-1 Capability Vision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityVision {
    /// Unique identifier
    pub id: String,
    /// Vision name
    pub name: String,
    /// Vision description
    pub description: String,
    /// Version number
    pub version: String,

    // Strategic Elements
    /// Vision statement
    pub vision_statement: String,
    /// Mission areas affected by this vision
    pub mission_areas: Vec<String>,
    /// Strategic objectives
    pub strategic_objectives: Vec<StrategicObjective>,
    /// Capability increments realizing this vision
    pub capability_increments: Vec<CapabilityIncrement>,
    /// Timeframe for this vision
    pub timeframe: TimeFrame,

    // Relationships
    /// Related capability IDs (CV-2)
    pub related_capabilities: Vec<String>,
    /// Supporting operational views (OV)
    pub supporting_operations: Vec<String>,
    /// Supporting system views (SV)
    pub supporting_systems: Vec<String>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Strategic objective supporting the vision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategicObjective {
    /// Objective ID
    pub id: String,
    /// Objective name
    pub name: String,
    /// Objective description
    pub description: String,
    /// Priority/importance
    pub priority: i32,
    /// Measures of effectiveness
    pub measures_of_effectiveness: Vec<String>,
}

/// Capability increment - incremental delivery of capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityIncrement {
    /// Increment ID
    pub id: String,
    /// Increment name
    pub name: String,
    /// Increment description
    pub description: Option<String>,
    /// Increment version
    pub version: String,
    /// Start date
    pub start_date: DateTime<Utc>,
    /// End date (if applicable)
    pub end_date: Option<DateTime<Utc>>,
    /// Capabilities enabled by this increment
    pub capabilities_enabled: Vec<String>,
    /// Dependencies on other increments
    pub dependencies: Vec<String>,
}

/// Timeframe specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeFrame {
    /// Near-term timeframe description
    pub near_term: Option<String>,
    /// Mid-term timeframe description
    pub mid_term: Option<String>,
    /// Far-term timeframe description
    pub far_term: Option<String>,
}

// ============================================================================
// Helper implementations
// ============================================================================

impl CapabilityVision {
    /// Create a new capability vision
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        vision_statement: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            version: "1.0".to_string(),
            vision_statement: vision_statement.into(),
            mission_areas: Vec::new(),
            strategic_objectives: Vec::new(),
            capability_increments: Vec::new(),
            timeframe: TimeFrame::default(),
            related_capabilities: Vec::new(),
            supporting_operations: Vec::new(),
            supporting_systems: Vec::new(),
            created_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Add a mission area
    pub fn add_mission_area(mut self, mission_area: impl Into<String>) -> Self {
        self.mission_areas.push(mission_area.into());
        self
    }

    /// Add a strategic objective
    pub fn add_objective(mut self, objective: StrategicObjective) -> Self {
        self.strategic_objectives.push(objective);
        self
    }

    /// Add a capability increment
    pub fn add_increment(mut self, increment: CapabilityIncrement) -> Self {
        self.capability_increments.push(increment);
        self
    }

    /// Set timeframe
    pub fn with_timeframe(mut self, timeframe: TimeFrame) -> Self {
        self.timeframe = timeframe;
        self
    }

    /// Add related capability
    pub fn add_related_capability(mut self, capability_id: impl Into<String>) -> Self {
        self.related_capabilities.push(capability_id.into());
        self
    }

    /// Add supporting operation
    pub fn add_supporting_operation(mut self, operation_id: impl Into<String>) -> Self {
        self.supporting_operations.push(operation_id.into());
        self
    }

    /// Add supporting system
    pub fn add_supporting_system(mut self, system_id: impl Into<String>) -> Self {
        self.supporting_systems.push(system_id.into());
        self
    }
}

impl StrategicObjective {
    /// Create a new strategic objective
    pub fn new(id: impl Into<String>, name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            priority: 1,
            measures_of_effectiveness: Vec::new(),
        }
    }

    /// Set priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Add measure of effectiveness
    pub fn add_measure(mut self, measure: impl Into<String>) -> Self {
        self.measures_of_effectiveness.push(measure.into());
        self
    }
}

impl CapabilityIncrement {
    /// Create a new capability increment
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        start_date: DateTime<Utc>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            version: "1.0".to_string(),
            start_date,
            end_date: None,
            capabilities_enabled: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set end date
    pub fn with_end_date(mut self, end_date: DateTime<Utc>) -> Self {
        self.end_date = Some(end_date);
        self
    }

    /// Add enabled capability
    pub fn add_enabled_capability(mut self, capability_id: impl Into<String>) -> Self {
        self.capabilities_enabled.push(capability_id.into());
        self
    }

    /// Add dependency
    pub fn add_dependency(mut self, increment_id: impl Into<String>) -> Self {
        self.dependencies.push(increment_id.into());
        self
    }
}

impl Default for TimeFrame {
    fn default() -> Self {
        Self {
            near_term: None,
            mid_term: None,
            far_term: None,
        }
    }
}

impl TimeFrame {
    /// Create a timeframe with all periods
    pub fn with_all(near: impl Into<String>, mid: impl Into<String>, far: impl Into<String>) -> Self {
        Self {
            near_term: Some(near.into()),
            mid_term: Some(mid.into()),
            far_term: Some(far.into()),
        }
    }

    /// Set near-term timeframe
    pub fn with_near_term(mut self, near_term: impl Into<String>) -> Self {
        self.near_term = Some(near_term.into());
        self
    }

    /// Set mid-term timeframe
    pub fn with_mid_term(mut self, mid_term: impl Into<String>) -> Self {
        self.mid_term = Some(mid_term.into());
        self
    }

    /// Set far-term timeframe
    pub fn with_far_term(mut self, far_term: impl Into<String>) -> Self {
        self.far_term = Some(far_term.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_vision() {
        let vision = CapabilityVision::new(
            "cv1_1",
            "2026-2030 Vision",
            "Strategic capability vision for 2026-2030 period",
            "Achieve integrated cyber-physical capabilities",
        );

        assert_eq!(vision.name, "2026-2030 Vision");
        assert!(vision.description.contains("Strategic"));
    }

    #[test]
    fn test_vision_with_objectives() {
        let obj1 = StrategicObjective::new(
            "obj_1",
            "System Integration",
            "Integrate disparate systems",
        )
        .with_priority(1)
        .add_measure("Integration success rate");

        let vision = CapabilityVision::new(
            "cv1_1",
            "Vision",
            "Test",
            "Test vision",
        )
        .add_objective(obj1);

        assert_eq!(vision.strategic_objectives.len(), 1);
        assert_eq!(vision.strategic_objectives[0].priority, 1);
    }

    #[test]
    fn test_capability_increments() {
        let start = Utc::now();
        let end = start + chrono::Duration::days(365);

        let inc = CapabilityIncrement::new("inc_1", "Phase 1", start)
            .with_end_date(end)
            .add_enabled_capability("cap_1")
            .add_enabled_capability("cap_2");

        assert_eq!(inc.capabilities_enabled.len(), 2);
        assert!(inc.end_date.is_some());
    }

    #[test]
    fn test_timeframe() {
        let tf = TimeFrame::with_all("0-2 years", "2-5 years", "5+ years");

        assert!(tf.near_term.is_some());
        assert!(tf.mid_term.is_some());
        assert!(tf.far_term.is_some());
    }

    #[test]
    fn test_vision_relationships() {
        let vision = CapabilityVision::new("cv1_1", "Vision", "Test", "Test")
            .add_mission_area("Command & Control")
            .add_related_capability("cap_1")
            .add_supporting_system("sys_1");

        assert_eq!(vision.mission_areas.len(), 1);
        assert_eq!(vision.related_capabilities.len(), 1);
        assert_eq!(vision.supporting_systems.len(), 1);
    }
}
