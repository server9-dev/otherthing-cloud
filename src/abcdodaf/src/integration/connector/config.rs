//! Connector configuration

use super::auth::{ApiKeyConfig, OAuthConfig, JwtConfig, BasicAuthConfig};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

/// Authentication configuration enum
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(tag = "type")]
pub enum AuthConfig {
    /// API key authentication
    #[serde(rename = "api_key")]
    ApiKey(ApiKeyConfig),
    /// OAuth 2.0 authentication
    #[serde(rename = "oauth2")]
    OAuth2(OAuthConfig),
    /// JWT authentication
    #[serde(rename = "jwt")]
    Jwt(JwtConfig),
    /// Basic HTTP authentication
    #[serde(rename = "basic")]
    Basic(BasicAuthConfig),
    /// No authentication
    #[serde(rename = "none")]
    #[default]
    None,
}

impl AuthConfig {
    /// Create an API key config
    pub fn api_key(key: impl Into<String>, header: impl Into<String>) -> Self {
        Self::ApiKey(ApiKeyConfig::new(key, header))
    }

    /// Create an OAuth2 config
    pub fn oauth2(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        token_endpoint: impl Into<String>,
    ) -> Self {
        Self::OAuth2(OAuthConfig::new(client_id, client_secret, token_endpoint))
    }

    /// Create a JWT config
    pub fn jwt(token: impl Into<String>) -> Self {
        Self::Jwt(JwtConfig::new(token))
    }

    /// Create a basic auth config
    pub fn basic(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self::Basic(BasicAuthConfig::new(username, password))
    }
}

/// Connector configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorConfig {
    /// Connector name
    pub name: String,
    /// Connector type (e.g., "rest_api", "postgresql", "mysql", "rabbitmq")
    pub connector_type: String,
    /// Authentication configuration
    #[serde(default)]
    pub auth: AuthConfig,
    /// Connection parameters
    #[serde(default)]
    pub params: HashMap<String, Value>,
    /// Connector-specific configuration
    #[serde(default)]
    pub config: HashMap<String, Value>,
    /// Timeout for operations in seconds
    pub timeout_secs: Option<u64>,
    /// Enable connection pooling
    pub enable_pooling: Option<bool>,
    /// Pool size (if pooling enabled)
    pub pool_size: Option<usize>,
    /// Enable automatic retries
    pub enable_retries: Option<bool>,
    /// Maximum retry attempts
    pub max_retries: Option<usize>,
    /// Enable circuit breaker
    pub enable_circuit_breaker: Option<bool>,
    /// Circuit breaker threshold (failure count)
    pub circuit_breaker_threshold: Option<usize>,
    /// Circuit breaker timeout (seconds)
    pub circuit_breaker_timeout_secs: Option<u64>,
    /// Enable logging
    pub enable_logging: Option<bool>,
    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl ConnectorConfig {
    /// Create a new connector configuration
    pub fn new(name: impl Into<String>, connector_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            connector_type: connector_type.into(),
            auth: AuthConfig::None,
            params: HashMap::new(),
            config: HashMap::new(),
            timeout_secs: Some(30),
            enable_pooling: Some(true),
            pool_size: Some(10),
            enable_retries: Some(true),
            max_retries: Some(3),
            enable_circuit_breaker: Some(true),
            circuit_breaker_threshold: Some(5),
            circuit_breaker_timeout_secs: Some(60),
            enable_logging: Some(true),
            metadata: HashMap::new(),
        }
    }

    /// Set authentication
    pub fn with_auth(mut self, auth: AuthConfig) -> Self {
        self.auth = auth;
        self
    }

    /// Add a parameter
    pub fn with_param(mut self, key: impl Into<String>, value: Value) -> Self {
        self.params.insert(key.into(), value);
        self
    }

    /// Add connector-specific config
    pub fn with_config(mut self, key: impl Into<String>, value: Value) -> Self {
        self.config.insert(key.into(), value);
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = Some(secs);
        self
    }

    /// Enable/disable pooling
    pub fn with_pooling(mut self, enabled: bool, pool_size: usize) -> Self {
        self.enable_pooling = Some(enabled);
        self.pool_size = Some(pool_size);
        self
    }

    /// Enable/disable retries
    pub fn with_retries(mut self, enabled: bool, max_retries: usize) -> Self {
        self.enable_retries = Some(enabled);
        self.max_retries = Some(max_retries);
        self
    }

    /// Enable/disable circuit breaker
    pub fn with_circuit_breaker(
        mut self,
        enabled: bool,
        threshold: usize,
        timeout_secs: u64,
    ) -> Self {
        self.enable_circuit_breaker = Some(enabled);
        self.circuit_breaker_threshold = Some(threshold);
        self.circuit_breaker_timeout_secs = Some(timeout_secs);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Get timeout as Duration
    pub fn get_timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_secs.unwrap_or(30))
    }

    /// Get pool size with default
    pub fn get_pool_size(&self) -> usize {
        self.pool_size.unwrap_or(10)
    }

    /// Get max retries with default
    pub fn get_max_retries(&self) -> usize {
        self.max_retries.unwrap_or(3)
    }

    /// Get circuit breaker threshold with default
    pub fn get_circuit_breaker_threshold(&self) -> usize {
        self.circuit_breaker_threshold.unwrap_or(5)
    }

    /// Get circuit breaker timeout with default
    pub fn get_circuit_breaker_timeout(&self) -> Duration {
        Duration::from_secs(self.circuit_breaker_timeout_secs.unwrap_or(60))
    }
}

impl Default for ConnectorConfig {
    fn default() -> Self {
        Self::new("unnamed", "unknown")
    }
}

/// Connector execution context
#[derive(Debug, Clone)]
pub struct ConnectorContext {
    /// Execution ID
    pub execution_id: String,
    /// Workflow ID
    pub workflow_id: Option<String>,
    /// Task ID
    pub task_id: Option<String>,
    /// User context
    pub user_id: Option<String>,
    /// Request metadata
    pub metadata: HashMap<String, String>,
    /// Additional context variables
    pub variables: HashMap<String, Value>,
}

impl ConnectorContext {
    /// Create a new connector context
    pub fn new(execution_id: impl Into<String>) -> Self {
        Self {
            execution_id: execution_id.into(),
            workflow_id: None,
            task_id: None,
            user_id: None,
            metadata: HashMap::new(),
            variables: HashMap::new(),
        }
    }

    /// Set workflow ID
    pub fn with_workflow(mut self, workflow_id: impl Into<String>) -> Self {
        self.workflow_id = Some(workflow_id.into());
        self
    }

    /// Set task ID
    pub fn with_task(mut self, task_id: impl Into<String>) -> Self {
        self.task_id = Some(task_id.into());
        self
    }

    /// Set user ID
    pub fn with_user(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Add context variable
    pub fn with_variable(mut self, key: impl Into<String>, value: Value) -> Self {
        self.variables.insert(key.into(), value);
        self
    }

    /// Get context variable
    pub fn get_variable(&self, key: &str) -> Option<&Value> {
        self.variables.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_connector_config_creation() {
        let config = ConnectorConfig::new("test", "rest_api")
            .with_timeout(60)
            .with_param("url", json!("https://example.com"))
            .with_auth(AuthConfig::api_key("key123", "X-API-Key"));

        assert_eq!(config.name, "test");
        assert_eq!(config.connector_type, "rest_api");
        assert_eq!(config.timeout_secs, Some(60));
        assert_eq!(
            config.params.get("url"),
            Some(&json!("https://example.com"))
        );
    }

    #[test]
    fn test_connector_context() {
        let ctx = ConnectorContext::new("exec-1")
            .with_workflow("wf-1")
            .with_task("task-1")
            .with_user("user-1")
            .with_variable("key", json!("value"));

        assert_eq!(ctx.execution_id, "exec-1");
        assert_eq!(ctx.workflow_id, Some("wf-1".to_string()));
        assert_eq!(ctx.task_id, Some("task-1".to_string()));
        assert_eq!(ctx.user_id, Some("user-1".to_string()));
        assert_eq!(ctx.get_variable("key"), Some(&json!("value")));
    }

    #[test]
    fn test_timeout_duration() {
        let config = ConnectorConfig::new("test", "rest_api").with_timeout(45);
        let timeout = config.get_timeout();
        assert_eq!(timeout.as_secs(), 45);
    }
}
