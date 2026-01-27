//! Capability Viewpoint (CV) - DoDAF 2.02
//!
//! Describes capabilities, their compositions, and relationships to
//! operational activities and organizational roles.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Capability View container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityView {
    /// Capabilities
    pub capabilities: Vec<Capability>,
    /// Capability mappings to operational activities
    pub mappings: Vec<CapabilityMapping>,
}

impl CapabilityView {
    /// Create a new capability view
    pub fn new() -> Self {
        Self { capabilities: Vec::new(), mappings: Vec::new() }
    }

    /// Add a capability
    pub fn add_capability(mut self, capability: Capability) -> Self {
        self.capabilities.push(capability);
        self
    }

    /// Add a capability mapping
    pub fn add_mapping(mut self, mapping: CapabilityMapping) -> Self {
        self.mappings.push(mapping);
        self
    }

    /// Find capability by ID
    pub fn get_capability(&self, id: &str) -> Option<&Capability> {
        self.capabilities.iter().find(|c| c.id == id)
    }
}

impl Default for CapabilityView {
    fn default() -> Self {
        Self::new()
    }
}

/// A capability required or provided by the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Capability ID
    pub id: String,
    /// Capability name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Capability type
    pub capability_type: CapabilityType,
    /// Parent capability (for hierarchical capabilities)
    pub parent_id: Option<String>,
    /// Required resources
    pub required_resources: Vec<String>,
    /// Performance metrics
    pub metrics: HashMap<String, serde_json::Value>,
}

/// Type of capability
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityType {
    /// Cognitive capability (AI/ML)
    Cognitive,
    /// Physical capability (hardware, execution)
    Physical,
    /// Information capability (data processing, storage)
    Information,
    /// Communication capability (networking, messaging)
    Communication,
    /// Organizational capability (coordination, governance)
    Organizational,
}

/// Mapping between capabilities and operational activities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityMapping {
    /// Capability ID
    pub capability_id: String,
    /// Operational activity ID
    pub activity_id: String,
    /// Mapping type
    pub mapping_type: MappingType,
}

/// Type of capability-to-activity mapping
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MappingType {
    /// Capability is required by the activity
    Required,
    /// Capability is optional for the activity
    Optional,
    /// Capability enables the activity
    Enables,
    /// Capability is produced by the activity
    Produced,
}

impl Capability {
    /// Create a new capability
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        capability_type: CapabilityType,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            capability_type,
            parent_id: None,
            required_resources: Vec::new(),
            metrics: HashMap::new(),
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set parent capability
    pub fn with_parent(mut self, parent_id: impl Into<String>) -> Self {
        self.parent_id = Some(parent_id.into());
        self
    }

    /// Add required resource
    pub fn add_resource(mut self, resource: impl Into<String>) -> Self {
        self.required_resources.push(resource.into());
        self
    }

    /// Add performance metric
    pub fn add_metric(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metrics.insert(key.into(), value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_creation() {
        let cap = Capability::new("cap1", "NLP", CapabilityType::Cognitive)
            .with_description("Natural Language Processing")
            .add_resource("LLM Model")
            .add_metric("accuracy", serde_json::json!(0.95));

        assert_eq!(cap.id, "cap1");
        assert_eq!(cap.capability_type, CapabilityType::Cognitive);
        assert_eq!(cap.required_resources.len(), 1);
    }

    #[test]
    fn test_capability_view() {
        let view = CapabilityView::new().add_capability(Capability::new(
            "c1",
            "Cap1",
            CapabilityType::Cognitive,
        ));

        assert_eq!(view.capabilities.len(), 1);
        assert!(view.get_capability("c1").is_some());
    }
}
