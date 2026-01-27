//! DODAF Traceability Matrix
//!
//! Track relationships and dependencies between DODAF views, requirements,
//! capabilities, systems, and activities

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Traceability link between elements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceabilityLink {
    pub id: String,
    pub source: TraceabilityElement,
    pub target: TraceabilityElement,
    pub link_type: LinkType,
    pub rationale: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Element that can be traced
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceabilityElement {
    pub id: String,
    pub element_type: ElementType,
    pub name: String,
    pub description: Option<String>,
}

/// Type of traceable element
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElementType {
    /// Requirement
    Requirement,

    /// Capability (CV-2, CV-6)
    Capability,

    /// Operational Activity (OV-5)
    OperationalActivity,

    /// System (SV-1)
    System,

    /// Service (SvcV-1)
    Service,

    /// Standard
    Standard,

    /// System Function (SV-4)
    SystemFunction,

    /// Data Element (OV-7, DIV-2)
    DataElement,

    /// Performance Parameter
    PerformanceParameter,

    /// Test Case
    TestCase,

    /// Risk
    Risk,
}

/// Type of traceability link
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkType {
    /// Implements/realizes
    Implements,

    /// Derives from
    DerivesFrom,

    /// Depends on
    DependsOn,

    /// Verifies/validates
    Verifies,

    /// Allocates to
    AllocatesTo,

    /// Traces to
    TracesTo,

    /// Refines
    Refines,

    /// Satisfies
    Satisfies,

    /// Conflicts with
    ConflictsWith,

    /// Supports
    Supports,
}

/// Traceability matrix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceabilityMatrix {
    pub name: String,
    pub description: Option<String>,
    pub links: Vec<TraceabilityLink>,
    pub elements: HashMap<String, TraceabilityElement>,
}

impl TraceabilityMatrix {
    /// Create a new traceability matrix
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            links: Vec::new(),
            elements: HashMap::new(),
        }
    }

    /// Add an element to track
    pub fn add_element(&mut self, element: TraceabilityElement) {
        self.elements.insert(element.id.clone(), element);
    }

    /// Add a traceability link
    pub fn add_link(&mut self, link: TraceabilityLink) {
        // Ensure elements exist
        if !self.elements.contains_key(&link.source.id) {
            self.elements.insert(link.source.id.clone(), link.source.clone());
        }
        if !self.elements.contains_key(&link.target.id) {
            self.elements.insert(link.target.id.clone(), link.target.clone());
        }

        self.links.push(link);
    }

    /// Get all links from a source element
    pub fn get_links_from(&self, source_id: &str) -> Vec<&TraceabilityLink> {
        self.links
            .iter()
            .filter(|link| link.source.id == source_id)
            .collect()
    }

    /// Get all links to a target element
    pub fn get_links_to(&self, target_id: &str) -> Vec<&TraceabilityLink> {
        self.links
            .iter()
            .filter(|link| link.target.id == target_id)
            .collect()
    }

    /// Get all links of a specific type
    pub fn get_links_by_type(&self, link_type: LinkType) -> Vec<&TraceabilityLink> {
        self.links
            .iter()
            .filter(|link| link.link_type == link_type)
            .collect()
    }

    /// Find all elements that implement a capability
    pub fn get_capability_implementations(&self, capability_id: &str) -> Vec<&TraceabilityElement> {
        self.links
            .iter()
            .filter(|link| {
                link.target.id == capability_id
                    && link.target.element_type == ElementType::Capability
                    && link.link_type == LinkType::Implements
            })
            .map(|link| &link.source)
            .collect()
    }

    /// Find all requirements satisfied by an activity
    pub fn get_satisfied_requirements(&self, activity_id: &str) -> Vec<&TraceabilityElement> {
        self.links
            .iter()
            .filter(|link| {
                link.source.id == activity_id && link.link_type == LinkType::Satisfies
            })
            .map(|link| &link.target)
            .collect()
    }

    /// Check coverage: find elements with no traceability
    pub fn find_orphaned_elements(&self) -> Vec<&TraceabilityElement> {
        self.elements
            .values()
            .filter(|element| {
                !self.links.iter().any(|link| {
                    link.source.id == element.id || link.target.id == element.id
                })
            })
            .collect()
    }

    /// Generate coverage report
    pub fn coverage_report(&self) -> CoverageReport {
        let total_requirements = self
            .elements
            .values()
            .filter(|e| e.element_type == ElementType::Requirement)
            .count();

        let satisfied_requirements = self
            .links
            .iter()
            .filter(|link| {
                link.target.element_type == ElementType::Requirement
                    && link.link_type == LinkType::Satisfies
            })
            .map(|link| &link.target.id)
            .collect::<std::collections::HashSet<_>>()
            .len();

        let total_capabilities = self
            .elements
            .values()
            .filter(|e| e.element_type == ElementType::Capability)
            .count();

        let implemented_capabilities = self
            .links
            .iter()
            .filter(|link| {
                link.target.element_type == ElementType::Capability
                    && link.link_type == LinkType::Implements
            })
            .map(|link| &link.target.id)
            .collect::<std::collections::HashSet<_>>()
            .len();

        CoverageReport {
            total_elements: self.elements.len(),
            total_links: self.links.len(),
            total_requirements,
            satisfied_requirements,
            requirements_coverage: if total_requirements > 0 {
                (satisfied_requirements as f64 / total_requirements as f64) * 100.0
            } else {
                0.0
            },
            total_capabilities,
            implemented_capabilities,
            capabilities_coverage: if total_capabilities > 0 {
                (implemented_capabilities as f64 / total_capabilities as f64) * 100.0
            } else {
                0.0
            },
            orphaned_elements: self.find_orphaned_elements().len(),
        }
    }
}

/// Coverage report for traceability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    pub total_elements: usize,
    pub total_links: usize,
    pub total_requirements: usize,
    pub satisfied_requirements: usize,
    pub requirements_coverage: f64,
    pub total_capabilities: usize,
    pub implemented_capabilities: usize,
    pub capabilities_coverage: f64,
    pub orphaned_elements: usize,
}

/// Builder for traceability links
pub struct TraceabilityLinkBuilder {
    link: TraceabilityLink,
}

impl TraceabilityLinkBuilder {
    /// Start building a link from a source element
    pub fn from(source: TraceabilityElement) -> Self {
        Self {
            link: TraceabilityLink {
                id: uuid::Uuid::new_v4().to_string(),
                source,
                target: TraceabilityElement {
                    id: String::new(),
                    element_type: ElementType::Requirement,
                    name: String::new(),
                    description: None,
                },
                link_type: LinkType::TracesTo,
                rationale: None,
                metadata: HashMap::new(),
            },
        }
    }

    /// Set the target element
    pub fn to(mut self, target: TraceabilityElement) -> Self {
        self.link.target = target;
        self
    }

    /// Set the link type
    pub fn link_type(mut self, link_type: LinkType) -> Self {
        self.link.link_type = link_type;
        self
    }

    /// Add rationale
    pub fn rationale(mut self, rationale: impl Into<String>) -> Self {
        self.link.rationale = Some(rationale.into());
        self
    }

    /// Build the link
    pub fn build(self) -> TraceabilityLink {
        self.link
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_traceability_matrix() {
        let mut matrix = TraceabilityMatrix::new("Test Matrix");

        let req = TraceabilityElement {
            id: "REQ-001".to_string(),
            element_type: ElementType::Requirement,
            name: "User Authentication".to_string(),
            description: None,
        };

        let activity = TraceabilityElement {
            id: "ACT-001".to_string(),
            element_type: ElementType::OperationalActivity,
            name: "Authenticate User".to_string(),
            description: None,
        };

        let link = TraceabilityLinkBuilder::from(activity.clone())
            .to(req.clone())
            .link_type(LinkType::Satisfies)
            .build();

        matrix.add_link(link);

        assert_eq!(matrix.elements.len(), 2);
        assert_eq!(matrix.links.len(), 1);

        let satisfied = matrix.get_satisfied_requirements("ACT-001");
        assert_eq!(satisfied.len(), 1);
        assert_eq!(satisfied[0].id, "REQ-001");
    }

    #[test]
    fn test_coverage_report() {
        let mut matrix = TraceabilityMatrix::new("Coverage Test");

        matrix.add_element(TraceabilityElement {
            id: "REQ-001".to_string(),
            element_type: ElementType::Requirement,
            name: "Req 1".to_string(),
            description: None,
        });

        matrix.add_element(TraceabilityElement {
            id: "REQ-002".to_string(),
            element_type: ElementType::Requirement,
            name: "Req 2".to_string(),
            description: None,
        });

        let activity = TraceabilityElement {
            id: "ACT-001".to_string(),
            element_type: ElementType::OperationalActivity,
            name: "Activity 1".to_string(),
            description: None,
        };

        let req1 = matrix.elements.get("REQ-001").unwrap().clone();
        let link = TraceabilityLinkBuilder::from(activity.clone())
            .to(req1)
            .link_type(LinkType::Satisfies)
            .build();

        matrix.add_link(link);

        let report = matrix.coverage_report();
        assert_eq!(report.total_requirements, 2);
        assert_eq!(report.satisfied_requirements, 1);
        assert_eq!(report.requirements_coverage, 50.0);
    }
}
