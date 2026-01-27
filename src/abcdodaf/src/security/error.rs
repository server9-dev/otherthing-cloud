//! Security-related error types

use thiserror::Error;

/// Security operation result type
pub type SecurityResult<T> = Result<T, SecurityError>;

/// Security-related errors
#[derive(Error, Debug)]
pub enum SecurityError {
    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Role not found
    #[error("Role not found: {0}")]
    RoleNotFound(String),

    /// Cannot delete system role
    #[error("Cannot delete system role: {0}")]
    CannotDeleteSystemRole(String),

    /// Resource not found
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    /// Resource already exists
    #[error("Resource already exists: {0}")]
    ResourceAlreadyExists(String),

    /// Invalid session
    #[error("Invalid session: {0}")]
    InvalidSession(String),

    /// Session expired
    #[error("Session expired")]
    SessionExpired,

    /// Invalid credentials
    #[error("Invalid credentials")]
    InvalidCredentials,

    /// Secret not found
    #[error("Secret not found: {0}")]
    SecretNotFound(String),

    /// Encryption error
    #[error("Encryption error: {0}")]
    EncryptionError(String),

    /// Decryption error
    #[error("Decryption error: {0}")]
    DecryptionError(String),

    /// Compliance violation
    #[error("Compliance violation: {0}")]
    ComplianceViolation(String),

    /// Invalid policy
    #[error("Invalid security policy: {0}")]
    InvalidPolicy(String),

    /// Audit error
    #[error("Audit error: {0}")]
    AuditError(String),

    /// Configuration error
    #[error("Security configuration error: {0}")]
    ConfigurationError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Other error
    #[error("Security error: {0}")]
    Other(String),
}

impl From<String> for SecurityError {
    fn from(s: String) -> Self {
        SecurityError::Other(s)
    }
}

impl From<&str> for SecurityError {
    fn from(s: &str) -> Self {
        SecurityError::Other(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_error_display() {
        let err = SecurityError::PermissionDenied("workflow:edit".to_string());
        assert_eq!(err.to_string(), "Permission denied: workflow:edit");

        let err = SecurityError::SessionExpired;
        assert_eq!(err.to_string(), "Session expired");
    }

    #[test]
    fn test_security_error_conversion() {
        let err: SecurityError = "test error".into();
        assert!(matches!(err, SecurityError::Other(_)));
    }
}
