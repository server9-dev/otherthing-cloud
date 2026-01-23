//! DoDAF 2.02 (Department of Defense Architecture Framework) integration
//!
//! This module implements the DoDAF 2.02 framework for architectural modeling,
//! focusing on operational activities, capabilities, and services viewpoints.
//!
//! ## DoDAF 2.02 Viewpoints
//!
//! - **Operational Viewpoint (OV)**: Operational scenarios, activities, and information flows
//! - **Capability Viewpoint (CV)**: Required capabilities and their relationships
//! - **Services Viewpoint (SvcV)**: Service-oriented architecture and service specifications
//! - **Systems Viewpoint (SV)**: Systems and their interactions (future implementation)
//! - **Data and Information Viewpoint (DIV)**: Data models and relationships (future implementation)

pub mod operational;
pub mod capability;
pub mod services;

pub use operational::{
    OperationalActivity, OperationalContext, OperationalView,
    ActivityType, MissionArea, InformationExchange,
};
pub use capability::{Capability, CapabilityView, CapabilityMapping, CapabilityType};
pub use services::{Service, ServiceView, ServiceSpecification, ServiceType};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// DoDAF architecture description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DodafArchitecture {
    /// Architecture name
    pub name: String,
    /// Architecture description
    pub description: Option<String>,
    /// Operational view
    pub operational_view: OperationalView,
    /// Capability view
    pub capability_view: CapabilityView,
    /// Services view
    pub services_view: ServiceView,
    /// Architecture metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl DodafArchitecture {
    /// Create a new DoDAF architecture
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            operational_view: OperationalView::new(),
            capability_view: CapabilityView::new(),
            services_view: ServiceView::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set architecture description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set operational view
    pub fn with_operational_view(mut self, view: OperationalView) -> Self {
        self.operational_view = view;
        self
    }

    /// Set capability view
    pub fn with_capability_view(mut self, view: CapabilityView) -> Self {
        self.capability_view = view;
        self
    }

    /// Set services view
    pub fn with_services_view(mut self, view: ServiceView) -> Self {
        self.services_view = view;
        self
    }

    /// Export to JSON
    pub fn to_json(&self) -> crate::error::Result<String> {
        serde_json::to_string_pretty(self).map_err(|e| e.into())
    }

    /// Export to YAML
    pub fn to_yaml(&self) -> crate::error::Result<String> {
        serde_yaml::to_string(self)
            .map_err(|e| crate::error::AbcdodafError::DodafError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dodaf_architecture_creation() {
        let arch = DodafArchitecture::new("Test Architecture")
            .with_description("A test DoDAF architecture");

        assert_eq!(arch.name, "Test Architecture");
        assert!(arch.description.is_some());
    }

    #[test]
    fn test_json_export() {
        let arch = DodafArchitecture::new("Test");
        let json = arch.to_json().unwrap();
        assert!(json.contains("Test"));
    }
}
