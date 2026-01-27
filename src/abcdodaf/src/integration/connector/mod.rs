//! Connector framework for external system integration
//!
//! Provides extensible traits and base types for building connectors to various external systems

pub mod error;
pub mod auth;
pub mod config;
pub mod retry;
pub mod circuit_breaker;
pub mod registry;

pub use error::{ConnectorError, ConnectorResult};
pub use auth::{AuthStrategy, OAuthConfig, JwtConfig, ApiKeyConfig};
pub use config::{ConnectorConfig, ConnectorContext, AuthConfig};
pub use retry::RetryPolicy;
pub use circuit_breaker::{CircuitBreaker, CircuitBreakerState};
pub use registry::ConnectorRegistry;

use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fmt;

/// Connection status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    /// Connector is disconnected
    Disconnected,
    /// Connector is connecting
    Connecting,
    /// Connector is connected
    Connected,
    /// Connector is reconnecting
    Reconnecting,
    /// Connector encountered an error
    Error,
    /// Connector is closed
    Closed,
}

impl fmt::Display for ConnectionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disconnected => write!(f, "Disconnected"),
            Self::Connecting => write!(f, "Connecting"),
            Self::Connected => write!(f, "Connected"),
            Self::Reconnecting => write!(f, "Reconnecting"),
            Self::Error => write!(f, "Error"),
            Self::Closed => write!(f, "Closed"),
        }
    }
}

/// Connector capability trait
///
/// Defines the interface for a connector to external systems
#[async_trait]
pub trait Connector: Send + Sync {
    /// Get the connector type identifier
    fn connector_type(&self) -> &str;

    /// Get the connector name
    fn name(&self) -> &str;

    /// Get current connection status
    fn status(&self) -> ConnectionStatus;

    /// Initialize the connector
    async fn initialize(&mut self) -> ConnectorResult<()>;

    /// Close the connector and cleanup resources
    async fn close(&mut self) -> ConnectorResult<()>;

    /// Execute a request against the connector
    async fn execute(&self, request: ConnectorRequest) -> ConnectorResult<ConnectorResponse>;

    /// Health check to verify the connector is working
    async fn health_check(&self) -> ConnectorResult<HealthStatus>;

    /// Get connector metadata
    fn metadata(&self) -> ConnectorMetadata {
        ConnectorMetadata {
            connector_type: self.connector_type().to_string(),
            name: self.name().to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            capabilities: Vec::new(),
            status: self.status(),
        }
    }

    /// Optional: Get connector statistics
    fn statistics(&self) -> ConnectorStats {
        ConnectorStats::default()
    }
}

/// Request to execute against a connector
#[derive(Debug, Clone)]
pub struct ConnectorRequest {
    /// Request identifier
    pub id: String,
    /// Request operation/method
    pub operation: String,
    /// Request parameters
    pub parameters: HashMap<String, Value>,
    /// Request body/payload
    pub body: Option<Value>,
    /// Request headers/metadata
    pub headers: HashMap<String, String>,
    /// Request timeout in seconds
    pub timeout_secs: Option<u64>,
}

impl ConnectorRequest {
    /// Create a new connector request
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            operation: operation.into(),
            parameters: HashMap::new(),
            body: None,
            headers: HashMap::new(),
            timeout_secs: None,
        }
    }

    /// Add a parameter
    pub fn with_param(mut self, key: impl Into<String>, value: Value) -> Self {
        self.parameters.insert(key.into(), value);
        self
    }

    /// Set the request body
    pub fn with_body(mut self, body: Value) -> Self {
        self.body = Some(body);
        self
    }

    /// Add a header
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = Some(secs);
        self
    }
}

/// Response from a connector
#[derive(Debug, Clone)]
pub struct ConnectorResponse {
    /// Response identifier (matches request ID)
    pub id: String,
    /// HTTP-like status code
    pub status_code: u16,
    /// Response status message
    pub status_message: String,
    /// Response body
    pub body: Value,
    /// Response headers/metadata
    pub headers: HashMap<String, String>,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
}

impl ConnectorResponse {
    /// Create a new connector response
    pub fn new(id: impl Into<String>, status_code: u16) -> Self {
        Self {
            id: id.into(),
            status_code,
            status_message: format!("Status {}", status_code),
            body: json!({}),
            headers: HashMap::new(),
            execution_time_ms: 0,
        }
    }

    /// Check if response indicates success (2xx status code)
    pub fn is_success(&self) -> bool {
        self.status_code >= 200 && self.status_code < 300
    }

    /// Set response body
    pub fn with_body(mut self, body: Value) -> Self {
        self.body = body;
        self
    }

    /// Add response header
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set execution time
    pub fn with_execution_time(mut self, ms: u64) -> Self {
        self.execution_time_ms = ms;
        self
    }
}

/// Health check status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    /// Connector is healthy
    Healthy,
    /// Connector is unhealthy but recovering
    Degraded(String),
    /// Connector is unhealthy
    Unhealthy(String),
}

/// Connector metadata
#[derive(Debug, Clone)]
pub struct ConnectorMetadata {
    /// Connector type
    pub connector_type: String,
    /// Human-readable name
    pub name: String,
    /// Version
    pub version: String,
    /// Description
    pub description: String,
    /// Supported capabilities
    pub capabilities: Vec<String>,
    /// Current status
    pub status: ConnectionStatus,
}

/// Connector statistics
#[derive(Debug, Clone, Default)]
pub struct ConnectorStats {
    /// Total requests processed
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Average response time (ms)
    pub avg_response_time_ms: f64,
    /// Last error message
    pub last_error: Option<String>,
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Total bytes received
    pub bytes_received: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_creation() {
        let req = ConnectorRequest::new("GET")
            .with_param("id", json!("123"))
            .with_timeout(30);

        assert_eq!(req.operation, "GET");
        assert_eq!(req.timeout_secs, Some(30));
        assert_eq!(req.parameters.get("id"), Some(&json!("123")));
    }

    #[test]
    fn test_response_success() {
        let resp = ConnectorResponse::new("req-1", 200);
        assert!(resp.is_success());

        let resp = ConnectorResponse::new("req-2", 400);
        assert!(!resp.is_success());

        let resp = ConnectorResponse::new("req-3", 500);
        assert!(!resp.is_success());
    }

    #[test]
    fn test_connection_status_display() {
        assert_eq!(ConnectionStatus::Connected.to_string(), "Connected");
        assert_eq!(ConnectionStatus::Disconnected.to_string(), "Disconnected");
    }
}
