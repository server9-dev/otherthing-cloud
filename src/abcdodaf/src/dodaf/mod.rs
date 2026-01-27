//! DoDAF 2.02 (Department of Defense Architecture Framework) integration
//!
//! This module implements the DoDAF 2.02 framework for architectural modeling,
//! including operational, systems, and capability viewpoints.
//!
//! ## DoDAF 2.02 Viewpoints
//!
//! ### Operational Viewpoint (OV)
//! - **OV-1**: High-Level Operational Concept Graphic
//! - **OV-2**: Operational Node Connectivity & Resource Flow
//! - **OV-3**: Operational Information Exchange Matrix
//! - **OV-5**: Operational Activity Model
//! - **OV-6a**: Operational Rules Model
//! - **OV-6b**: Operational State Transition Description
//! - **OV-6c**: Operational Event-Trace Description
//!
//! ### Systems Viewpoint (SV)
//! - **SV-1**: Systems Interface Description
//! - **SV-2**: Systems Resource Flow Description
//! - **SV-4**: Systems Functionality Description
//!
//! ### Capability Viewpoint (CV)
//! - **CV-1**: Capability Vision
//! - **CV-2**: Capability Taxonomy
//!
//! ### Legacy Viewpoints
//! - **Capability**: Basic capability structures
//! - **Services**: Service-oriented architecture

// Operational Views
pub mod operational;
pub mod ov1;
pub mod ov2;
pub mod ov3;
pub mod ov5;
pub mod ov6;

// Systems Views
pub mod sv1;
pub mod sv2;
pub mod sv4;

// Capability Views
pub mod cv1;
pub mod cv2;

// Legacy modules
pub mod capability;
pub mod services;

// Enhanced modules for executor
pub mod performers;
pub mod resource_flows;
pub mod security;
pub mod traceability;

pub use operational::{
    OperationalActivity, OperationalContext, OperationalView,
    ActivityType, MissionArea, InformationExchange,
};
pub use ov1::{OperationalConceptGraphic, OperationalOrganization, OrganizationType};
pub use ov2::{OperationalNodeConnectivity, OperationalNode, Needline, Criticality};
pub use ov3::{InformationExchangeMatrix, InformationElement, ExchangePair};
pub use ov5::*;
pub use ov6::{OperationalRulesModel, StateTransitionDescription, OperationalRule, RuleType};
pub use sv1::{SystemsInterfaceDescription, System, SystemInterface, SystemPort};
pub use sv2::{SystemsResourceFlowDescription, CommunicationSystem, CommunicationLink};
pub use sv4::{SystemsFunctionalityDescription, SystemFunction, FunctionDataFlow};
pub use cv1::{CapabilityVision, StrategicObjective, CapabilityIncrement};
pub use cv2::{CapabilityTaxonomy, Capability, CapabilityMeasure};
pub use capability::{Capability as BaseCapability, CapabilityView, CapabilityMapping, CapabilityType as BaseCapabilityType};
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
