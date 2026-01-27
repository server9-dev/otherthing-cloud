//! Connector error types and handling

use thiserror::Error;

/// Connector error type
#[derive(Error, Debug)]
pub enum ConnectorError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Connection error
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// Request error
    #[error("Request failed: {0}")]
    RequestError(String),

    /// Response error
    #[error("Response error: {0}")]
    ResponseError(String),

    /// Timeout error
    #[error("Request timeout after {timeout_secs}s")]
    Timeout { timeout_secs: u64 },

    /// Authentication error
    #[error("Authentication failed: {0}")]
    AuthError(String),

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Not found error
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Already exists error
    #[error("Resource already exists: {0}")]
    AlreadyExists(String),

    /// Rate limit error
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    /// Circuit breaker open
    #[error("Circuit breaker is open")]
    CircuitBreakerOpen,

    /// Max retries exceeded
    #[error("Max retries exceeded: {0}")]
    MaxRetriesExceeded(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Database error
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

impl ConnectorError {
    /// Create a configuration error
    pub fn config(msg: impl Into<String>) -> Self {
        Self::ConfigError(msg.into())
    }

    /// Create a connection error
    pub fn connection(msg: impl Into<String>) -> Self {
        Self::ConnectionError(msg.into())
    }

    /// Create a request error
    pub fn request(msg: impl Into<String>) -> Self {
        Self::RequestError(msg.into())
    }

    /// Create a timeout error
    pub fn timeout(secs: u64) -> Self {
        Self::Timeout { timeout_secs: secs }
    }

    /// Create an auth error
    pub fn auth(msg: impl Into<String>) -> Self {
        Self::AuthError(msg.into())
    }

    /// Create a validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::ValidationError(msg.into())
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::ConnectionError(_)
                | Self::RequestError(_)
                | Self::Timeout { .. }
                | Self::ResponseError(_)
        )
    }

    /// Check if error is a rate limit
    pub fn is_rate_limit(&self) -> bool {
        matches!(self, Self::RateLimit(_))
    }

    /// Check if error is authentication related
    pub fn is_auth_error(&self) -> bool {
        matches!(self, Self::AuthError(_))
    }
}

/// Result type for connector operations
pub type ConnectorResult<T> = Result<T, ConnectorError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = ConnectorError::config("test");
        assert_eq!(err.to_string(), "Configuration error: test");

        let err = ConnectorError::timeout(30);
        assert_eq!(err.to_string(), "Request timeout after 30s");
    }

    #[test]
    fn test_error_retryable() {
        assert!(ConnectorError::connection("error").is_retryable());
        assert!(ConnectorError::timeout(10).is_retryable());
        assert!(!ConnectorError::auth("error").is_retryable());
    }

    #[test]
    fn test_error_classification() {
        assert!(ConnectorError::auth("error").is_auth_error());
        assert!(ConnectorError::rate_limit("error").is_rate_limit());
    }

    impl ConnectorError {
        pub fn rate_limit(msg: impl Into<String>) -> Self {
            Self::RateLimit(msg.into())
        }
    }
}
