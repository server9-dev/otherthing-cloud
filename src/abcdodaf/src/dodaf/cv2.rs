//! DoDAF 2.02 CV-2: Capability Taxonomy
//!
//! Organizes and hierarchically structures capabilities.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// CV-2 Capability Taxonomy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityTaxonomy {
    /// Unique identifier
    pub id: String,
    /// Taxonomy name
    pub name: String,
    /// Taxonomy description
    pub description: Option<String>,
    /// Version number
    pub version: String,

    /// Capabilities in taxonomy
    pub capabilities: Vec<Capability>,
    /// Hierarchical organization
    pub hierarchy: TaxonomyHierarchy,
    /// Capability clusters
    pub capability_clusters: Vec<CapabilityCluster>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Capability - an ability to perform a specific function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Capability ID
    pub id: String,
    /// Capability name
    pub name: String,
    /// Capability description
    pub description: Option<String>,
    /// Type of capability
    pub capability_type: CapabilityType,

    // Hierarchy
    /// Parent capability ID
    pub parent_capability: Option<String>,
    /// Child capability IDs
    pub child_capabilities: Vec<String>,
    /// Level in hierarchy (0 = root)
    pub hierarchy_level: i32,

    // Measures
    /// Measures of performance
    pub measures: Vec<CapabilityMeasure>,
    /// Maturity level (1-5)
    pub maturity_level: Option<i32>,
    /// Readiness level (0-100%)
    pub readiness_level: Option<i32>,

    // Realization
    /// Operational views (OV) that realize this capability
    pub realized_by_operations: Vec<String>,
    /// Systems (SV) that realize this capability
    pub realized_by_systems: Vec<String>,
    /// Services (SvcV) that realize this capability
    pub realized_by_services: Vec<String>,

    /// Additional properties
    pub properties: HashMap<String, serde_json::Value>,
}

/// Capability type and categorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityType {
    /// Domain (e.g., "Command & Control", "Intelligence")
    pub domain: String,
    /// Category (e.g., "Communication", "Analysis")
    pub category: String,
}

/// Measure of capability performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityMeasure {
    /// Measure ID
    pub id: String,
    /// Measure name
    pub name: String,
    /// Metric description
    pub metric: String,
    /// Environmental condition for this measure
    pub condition: Option<String>,
    /// Target value
    pub target_value: Option<String>,
    /// Current value
    pub current_value: Option<String>,
}

/// Hierarchical organization of capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxonomyHierarchy {
    /// Root capability IDs
    pub root_nodes: Vec<String>,
    /// Maximum hierarchy depth
    pub levels: i32,
    /// Relationships in hierarchy
    pub relationships: Vec<HierarchyRelationship>,
}

/// Relationship in the capability hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyRelationship {
    /// Parent capability ID
    pub parent_id: String,
    /// Child capability ID
    pub child_id: String,
    /// Type of relationship
    pub relationship_type: HierarchyRelationType,
}

/// Type of hierarchical relationship
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HierarchyRelationType {
    /// Child is a composition of parent
    Composition,
    /// Child is a decomposition of parent
    Decomposition,
    /// Child depends on parent
    Dependency,
    /// Child is a specialization of parent
    Specialization,
}

/// Cluster of related capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityCluster {
    /// Cluster ID
    pub id: String,
    /// Cluster name
    pub name: String,
    /// Member capability IDs
    pub members: Vec<String>,
    /// Type of cluster
    pub cluster_type: String,
}

// ============================================================================
// Helper implementations
// ============================================================================

impl CapabilityTaxonomy {
    /// Create a new capability taxonomy
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            version: "1.0".to_string(),
            capabilities: Vec::new(),
            hierarchy: TaxonomyHierarchy {
                root_nodes: Vec::new(),
                levels: 0,
                relationships: Vec::new(),
            },
            capability_clusters: Vec::new(),
            created_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add a capability
    pub fn add_capability(mut self, capability: Capability) -> Self {
        if capability.parent_capability.is_none() {
            self.hierarchy.root_nodes.push(capability.id.clone());
        }
        self.hierarchy.levels = self.hierarchy.levels.max(capability.hierarchy_level + 1);
        self.capabilities.push(capability);
        self
    }

    /// Add a cluster
    pub fn add_cluster(mut self, cluster: CapabilityCluster) -> Self {
        self.capability_clusters.push(cluster);
        self
    }

    /// Get capabilities by domain
    pub fn get_capabilities_by_domain(&self, domain: &str) -> Vec<&Capability> {
        self.capabilities
            .iter()
            .filter(|c| c.capability_type.domain == domain)
            .collect()
    }

    /// Get capabilities by category
    pub fn get_capabilities_by_category(&self, category: &str) -> Vec<&Capability> {
        self.capabilities
            .iter()
            .filter(|c| c.capability_type.category == category)
            .collect()
    }

    /// Get children of a capability
    pub fn get_children(&self, capability_id: &str) -> Vec<&Capability> {
        self.capabilities
            .iter()
            .filter(|c| c.parent_capability.as_ref() == Some(&capability_id.to_string()))
            .collect()
    }

    /// Get root capabilities
    pub fn get_root_capabilities(&self) -> Vec<&Capability> {
        self.capabilities.iter().filter(|c| c.parent_capability.is_none()).collect()
    }
}

impl Capability {
    /// Create a new capability
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        domain: impl Into<String>,
        category: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            capability_type: CapabilityType { domain: domain.into(), category: category.into() },
            parent_capability: None,
            child_capabilities: Vec::new(),
            hierarchy_level: 0,
            measures: Vec::new(),
            maturity_level: None,
            readiness_level: None,
            realized_by_operations: Vec::new(),
            realized_by_systems: Vec::new(),
            realized_by_services: Vec::new(),
            properties: HashMap::new(),
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set parent capability
    pub fn with_parent(mut self, parent_id: impl Into<String>, level: i32) -> Self {
        self.parent_capability = Some(parent_id.into());
        self.hierarchy_level = level;
        self
    }

    /// Add child capability
    pub fn add_child(mut self, child_id: impl Into<String>) -> Self {
        self.child_capabilities.push(child_id.into());
        self
    }

    /// Set maturity level (1-5)
    pub fn with_maturity_level(mut self, level: i32) -> Self {
        self.maturity_level = Some(level);
        self
    }

    /// Set readiness level (0-100%)
    pub fn with_readiness_level(mut self, level: i32) -> Self {
        self.readiness_level = Some(level);
        self
    }

    /// Add a measure
    pub fn add_measure(mut self, measure: CapabilityMeasure) -> Self {
        self.measures.push(measure);
        self
    }

    /// Add realizing operation
    pub fn add_realizing_operation(mut self, operation_id: impl Into<String>) -> Self {
        self.realized_by_operations.push(operation_id.into());
        self
    }

    /// Add realizing system
    pub fn add_realizing_system(mut self, system_id: impl Into<String>) -> Self {
        self.realized_by_systems.push(system_id.into());
        self
    }

    /// Add realizing service
    pub fn add_realizing_service(mut self, service_id: impl Into<String>) -> Self {
        self.realized_by_services.push(service_id.into());
        self
    }

    /// Get realization count
    pub fn realization_count(&self) -> usize {
        self.realized_by_operations.len()
            + self.realized_by_systems.len()
            + self.realized_by_services.len()
    }
}

impl CapabilityMeasure {
    /// Create a new capability measure
    pub fn new(id: impl Into<String>, name: impl Into<String>, metric: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            metric: metric.into(),
            condition: None,
            target_value: None,
            current_value: None,
        }
    }

    /// Set condition
    pub fn with_condition(mut self, condition: impl Into<String>) -> Self {
        self.condition = Some(condition.into());
        self
    }

    /// Set target value
    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target_value = Some(target.into());
        self
    }

    /// Set current value
    pub fn with_current(mut self, current: impl Into<String>) -> Self {
        self.current_value = Some(current.into());
        self
    }

    /// Calculate performance percentage
    pub fn performance_percentage(&self) -> Option<f64> {
        match (&self.current_value, &self.target_value) {
            (Some(curr), Some(target)) => {
                if let (Ok(c), Ok(t)) = (curr.parse::<f64>(), target.parse::<f64>()) {
                    if t > 0.0 {
                        Some((c / t) * 100.0)
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            _ => None,
        }
    }
}

impl CapabilityCluster {
    /// Create a new capability cluster
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        cluster_type: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            members: Vec::new(),
            cluster_type: cluster_type.into(),
        }
    }

    /// Add a member capability
    pub fn add_member(mut self, capability_id: impl Into<String>) -> Self {
        self.members.push(capability_id.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_taxonomy() {
        let taxonomy = CapabilityTaxonomy::new("cv2_1", "Command & Control Capability Taxonomy")
            .with_description("Hierarchical taxonomy of C&C capabilities");

        assert_eq!(taxonomy.name, "Command & Control Capability Taxonomy");
        assert_eq!(taxonomy.capabilities.len(), 0);
    }

    #[test]
    fn test_capabilities_hierarchy() {
        let parent =
            Capability::new("cap_1", "Command", "Command & Control", "Core").with_maturity_level(3);

        let child1 = Capability::new("cap_1_1", "Plan", "Command & Control", "Planning")
            .with_parent("cap_1", 1);

        let child2 = Capability::new("cap_1_2", "Execute", "Command & Control", "Execution")
            .with_parent("cap_1", 1);

        let taxonomy = CapabilityTaxonomy::new("cv2_1", "Taxonomy")
            .add_capability(parent)
            .add_capability(child1)
            .add_capability(child2);

        assert_eq!(taxonomy.capabilities.len(), 3);
        assert_eq!(taxonomy.hierarchy.levels, 2);
        assert_eq!(taxonomy.get_root_capabilities().len(), 1);
    }

    #[test]
    fn test_capability_measures() {
        let measure = CapabilityMeasure::new("m_1", "Response Time", "milliseconds")
            .with_condition("Normal operations")
            .with_target("100")
            .with_current("85");

        assert!(measure.performance_percentage().is_some());
        assert_eq!(measure.performance_percentage(), Some(85.0));
    }

    #[test]
    fn test_capability_realization() {
        let cap = Capability::new("cap_1", "Sensing", "Intelligence", "Sensor")
            .add_realizing_operation("ov_sense")
            .add_realizing_system("sys_radar")
            .add_realizing_service("svc_sensor");

        assert_eq!(cap.realization_count(), 3);
    }

    #[test]
    fn test_capability_cluster() {
        let cluster = CapabilityCluster::new("cluster_1", "ISR Capabilities", "Mission Area")
            .add_member("cap_sense")
            .add_member("cap_analyze")
            .add_member("cap_disseminate");

        assert_eq!(cluster.members.len(), 3);
    }

    #[test]
    fn test_query_by_domain() {
        let cap1 = Capability::new("cap_1", "Command", "C2", "Core");
        let cap2 = Capability::new("cap_2", "Intelligence", "Intelligence", "Core");
        let cap3 = Capability::new("cap_3", "Plan", "C2", "Planning");

        let taxonomy = CapabilityTaxonomy::new("cv2_1", "Test")
            .add_capability(cap1)
            .add_capability(cap2)
            .add_capability(cap3);

        assert_eq!(taxonomy.get_capabilities_by_domain("C2").len(), 2);
        assert_eq!(taxonomy.get_capabilities_by_domain("Intelligence").len(), 1);
    }
}
