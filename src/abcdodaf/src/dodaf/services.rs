//! Services Viewpoint (SvcV) - DoDAF 2.02
//!
//! Describes service-oriented architecture, service specifications,
//! and service interactions that support operational and capability requirements.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Services View container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceView {
    /// Services
    pub services: Vec<Service>,
    /// Service specifications
    pub specifications: Vec<ServiceSpecification>,
    /// Service interactions
    pub interactions: Vec<ServiceInteraction>,
}

impl ServiceView {
    /// Create a new services view
    pub fn new() -> Self {
        Self {
            services: Vec::new(),
            specifications: Vec::new(),
            interactions: Vec::new(),
        }
    }

    /// Add a service
    pub fn add_service(mut self, service: Service) -> Self {
        self.services.push(service);
        self
    }

    /// Add a service specification
    pub fn add_specification(mut self, spec: ServiceSpecification) -> Self {
        self.specifications.push(spec);
        self
    }

    /// Add a service interaction
    pub fn add_interaction(mut self, interaction: ServiceInteraction) -> Self {
        self.interactions.push(interaction);
        self
    }
}

impl Default for ServiceView {
    fn default() -> Self {
        Self::new()
    }
}

/// A service in the architecture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    /// Service ID
    pub id: String,
    /// Service name
    pub name: String,
    /// Service description
    pub description: Option<String>,
    /// Service type
    pub service_type: ServiceType,
    /// Provides capabilities (capability IDs)
    pub provides_capabilities: Vec<String>,
    /// Service endpoint
    pub endpoint: Option<String>,
    /// Service metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Type of service
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceType {
    /// AI/ML inference service
    AiInference,
    /// Data processing service
    DataProcessing,
    /// Storage service
    Storage,
    /// Communication/messaging service
    Messaging,
    /// Orchestration service
    Orchestration,
    /// Authentication/authorization service
    Security,
    /// Custom service type
    Custom(String),
}

/// Service specification defining interface and behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSpecification {
    /// Specification ID
    pub id: String,
    /// Service ID this spec applies to
    pub service_id: String,
    /// Operations/methods
    pub operations: Vec<ServiceOperation>,
    /// Service level agreements
    pub sla: Option<ServiceLevelAgreement>,
}

/// A service operation/method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceOperation {
    /// Operation name
    pub name: String,
    /// Operation description
    pub description: Option<String>,
    /// Input parameters
    pub inputs: Vec<Parameter>,
    /// Output parameters
    pub outputs: Vec<Parameter>,
    /// Operation constraints
    pub constraints: HashMap<String, serde_json::Value>,
}

/// Parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: String,
    /// Is required
    pub required: bool,
    /// Default value
    pub default: Option<serde_json::Value>,
}

/// Service Level Agreement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceLevelAgreement {
    /// Availability percentage (e.g., 99.9)
    pub availability: f64,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Throughput (requests per second)
    pub throughput_rps: Option<u32>,
    /// Additional metrics
    pub metrics: HashMap<String, serde_json::Value>,
}

/// Interaction between services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInteraction {
    /// Interaction ID
    pub id: String,
    /// Source service ID
    pub source_service: String,
    /// Target service ID
    pub target_service: String,
    /// Interaction type
    pub interaction_type: InteractionType,
    /// Message/data exchanged
    pub message: Option<String>,
}

/// Type of service interaction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InteractionType {
    /// Synchronous request-response
    RequestResponse,
    /// Asynchronous message
    AsyncMessage,
    /// Event notification
    Event,
    /// Streaming data
    Stream,
}

impl Service {
    /// Create a new service
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        service_type: ServiceType,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            service_type,
            provides_capabilities: Vec::new(),
            endpoint: None,
            metadata: HashMap::new(),
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set endpoint
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    /// Add provided capability
    pub fn provides(mut self, capability_id: impl Into<String>) -> Self {
        self.provides_capabilities.push(capability_id.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_creation() {
        let service = Service::new("svc1", "AI Service", ServiceType::AiInference)
            .with_description("ML inference service")
            .with_endpoint("http://localhost:8080")
            .provides("nlp_capability");

        assert_eq!(service.id, "svc1");
        assert_eq!(service.service_type, ServiceType::AiInference);
        assert_eq!(service.provides_capabilities.len(), 1);
    }

    #[test]
    fn test_service_view() {
        let view = ServiceView::new()
            .add_service(Service::new("s1", "Service 1", ServiceType::DataProcessing));

        assert_eq!(view.services.len(), 1);
    }
}
