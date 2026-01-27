//! Standards registry and management
//!
//! Central registry for DODAF, BPMN, and custom standards with version management.

use super::rules::{RuleDefinition, RuleSet};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Standard version information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StandardVersion {
    /// Version string (e.g., "2.02", "2.0")
    pub version: String,
    /// Release date
    pub release_date: Option<String>,
    /// Whether this is the current/active version
    pub is_current: bool,
    /// Deprecation notice
    pub deprecated: bool,
}

/// Standard definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Standard {
    /// Standard identifier (e.g., "DODAF", "BPMN")
    pub id: String,
    /// Full name
    pub name: String,
    /// Description
    pub description: String,
    /// Version information
    pub version: StandardVersion,
    /// Official specification URL
    pub specification_url: Option<String>,
    /// Rule sets for this standard
    pub rule_sets: Vec<String>,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

impl Standard {
    /// Create a new standard
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        version: StandardVersion,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            version,
            specification_url: None,
            rule_sets: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set specification URL
    pub fn with_spec_url(mut self, url: impl Into<String>) -> Self {
        self.specification_url = Some(url.into());
        self
    }

    /// Add rule set reference
    pub fn with_rule_set(mut self, rule_set_id: impl Into<String>) -> Self {
        self.rule_sets.push(rule_set_id.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Registry of standards and their rules
pub struct StandardsRegistry {
    /// Registered standards
    standards: HashMap<String, Standard>,
    /// Registered rule sets
    rule_sets: HashMap<String, RuleSet>,
}

impl StandardsRegistry {
    /// Create a new standards registry
    pub fn new() -> Self {
        Self { standards: HashMap::new(), rule_sets: HashMap::new() }
    }

    /// Register a standard
    pub fn register_standard(&mut self, standard: Standard) {
        self.standards.insert(standard.id.clone(), standard);
    }

    /// Register a rule set
    pub fn register_rule_set(&mut self, rule_set: RuleSet) {
        self.rule_sets.insert(rule_set.id.clone(), rule_set);
    }

    /// Get a standard by ID
    pub fn get_standard(&self, id: &str) -> Option<&Standard> {
        self.standards.get(id)
    }

    /// Get a rule set by ID
    pub fn get_rule_set(&self, id: &str) -> Option<&RuleSet> {
        self.rule_sets.get(id)
    }

    /// Get all rules for a standard
    pub fn get_standard_rules(&self, standard_id: &str) -> Vec<RuleDefinition> {
        if let Some(standard) = self.standards.get(standard_id) {
            standard
                .rule_sets
                .iter()
                .filter_map(|rs_id| self.rule_sets.get(rs_id))
                .flat_map(|rs| rs.rules.clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// List all standards
    pub fn list_standards(&self) -> Vec<&Standard> {
        self.standards.values().collect()
    }

    /// List current (non-deprecated) standards
    pub fn list_current_standards(&self) -> Vec<&Standard> {
        self.standards
            .values()
            .filter(|s| s.version.is_current && !s.version.deprecated)
            .collect()
    }

    /// Create default registry with DODAF and BPMN standards
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();

        // DODAF 2.02
        let dodaf = Standard::new(
            "DODAF",
            "Department of Defense Architecture Framework",
            "Enterprise architecture framework for the Department of Defense",
            StandardVersion {
                version: "2.02".to_string(),
                release_date: Some("2009-08".to_string()),
                is_current: true,
                deprecated: false,
            },
        )
        .with_spec_url("https://dodcio.defense.gov/Library/DoD-Architecture-Framework/")
        .with_metadata("organization", "US Department of Defense")
        .with_metadata("scope", "Enterprise Architecture");

        // BPMN 2.0
        let bpmn = Standard::new(
            "BPMN",
            "Business Process Model and Notation",
            "OMG standard for business process modeling",
            StandardVersion {
                version: "2.0".to_string(),
                release_date: Some("2011-01".to_string()),
                is_current: true,
                deprecated: false,
            },
        )
        .with_spec_url("https://www.omg.org/spec/BPMN/2.0/")
        .with_metadata("organization", "Object Management Group")
        .with_metadata("scope", "Process Modeling");

        registry.register_standard(dodaf);
        registry.register_standard(bpmn);

        registry
    }
}

impl Default for StandardsRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_creation() {
        let standard = Standard::new(
            "TEST",
            "Test Standard",
            "A test standard",
            StandardVersion {
                version: "1.0".to_string(),
                release_date: None,
                is_current: true,
                deprecated: false,
            },
        )
        .with_spec_url("https://example.com/spec")
        .with_metadata("key", "value");

        assert_eq!(standard.id, "TEST");
        assert!(standard.specification_url.is_some());
        assert_eq!(standard.metadata.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_standards_registry() {
        let registry = StandardsRegistry::with_defaults();

        assert!(registry.get_standard("DODAF").is_some());
        assert!(registry.get_standard("BPMN").is_some());

        let current = registry.list_current_standards();
        assert_eq!(current.len(), 2);
    }
}
