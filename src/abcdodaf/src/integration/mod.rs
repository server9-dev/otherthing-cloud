//! Integration with external systems
//!
//! Provides integration capabilities with RhizOS MCP protocol, REST APIs, databases,
//! message queues, webhooks, and file systems

pub mod connector;
pub mod connectors;
pub mod mcp;
pub mod bpmn_dodaf_mapping;

pub use connector::{
    Connector, ConnectorRequest, ConnectorResponse, ConnectorConfig, ConnectorRegistry,
    ConnectionStatus, HealthStatus, AuthConfig,
};
pub use connectors::{
    RestApiConnector, PostgresConnector, MySqlConnector, SqliteConnector,
    OutgoingWebhookConnector, IncomingWebhookConnector, WebhookEvent,
    FileSystemConnector,
};
pub use mcp::McpIntegration;
pub use bpmn_dodaf_mapping::*;

use serde::{Deserialize, Serialize};

/// Integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    /// MCP integration enabled
    pub mcp_enabled: bool,
    /// MCP server endpoint
    pub mcp_endpoint: Option<String>,
    /// Custom integrations
    pub custom_integrations: Vec<CustomIntegration>,
}

/// Custom integration definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomIntegration {
    /// Integration name
    pub name: String,
    /// Integration type
    pub integration_type: String,
    /// Configuration
    pub config: serde_json::Value,
}

impl IntegrationConfig {
    /// Create default integration config
    pub fn new() -> Self {
        Self {
            mcp_enabled: false,
            mcp_endpoint: None,
            custom_integrations: Vec::new(),
        }
    }

    /// Enable MCP integration
    pub fn with_mcp(mut self, endpoint: impl Into<String>) -> Self {
        self.mcp_enabled = true;
        self.mcp_endpoint = Some(endpoint.into());
        self
    }

    /// Add custom integration
    pub fn add_integration(mut self, integration: CustomIntegration) -> Self {
        self.custom_integrations.push(integration);
        self
    }
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self::new()
    }
}
