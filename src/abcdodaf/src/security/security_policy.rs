//! Security policy enforcement engine
//!
//! Provides policy definition and enforcement for security requirements
//! across workflows and tasks.

use crate::security::error::{SecurityError, SecurityResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Security policy type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PolicyType {
    /// Password policy
    PasswordPolicy,
    /// Encryption policy
    EncryptionPolicy,
    /// Session policy
    SessionPolicy,
    /// Access policy
    AccessPolicy,
    /// Data retention policy
    DataRetentionPolicy,
    /// Audit policy
    AuditPolicy,
    /// Custom policy
    Custom,
}

impl std::fmt::Display for PolicyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PolicyType::PasswordPolicy => write!(f, "Password Policy"),
            PolicyType::EncryptionPolicy => write!(f, "Encryption Policy"),
            PolicyType::SessionPolicy => write!(f, "Session Policy"),
            PolicyType::AccessPolicy => write!(f, "Access Policy"),
            PolicyType::DataRetentionPolicy => write!(f, "Data Retention Policy"),
            PolicyType::AuditPolicy => write!(f, "Audit Policy"),
            PolicyType::Custom => write!(f, "Custom"),
        }
    }
}

/// Security policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    /// Unique policy ID
    pub id: String,
    /// Policy name
    pub name: String,
    /// Policy type
    pub policy_type: PolicyType,
    /// Policy description
    pub description: Option<String>,
    /// Policy rules (key-value pairs)
    pub rules: HashMap<String, String>,
    /// Whether policy is enforced
    pub is_enforced: bool,
    /// Whether policy is mandatory
    pub is_mandatory: bool,
    /// Policy creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Policy modification timestamp
    pub modified_at: chrono::DateTime<chrono::Utc>,
    /// Policy expiration
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl SecurityPolicy {
    /// Create a new security policy
    pub fn new(id: impl Into<String>, name: impl Into<String>, policy_type: PolicyType) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: id.into(),
            name: name.into(),
            policy_type,
            description: None,
            rules: HashMap::new(),
            is_enforced: true,
            is_mandatory: false,
            created_at: now,
            modified_at: now,
            expires_at: None,
        }
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a rule
    pub fn add_rule(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.rules.insert(key.into(), value.into());
        self
    }

    /// Set enforcement status
    pub fn set_enforced(mut self, enforced: bool) -> Self {
        self.is_enforced = enforced;
        self.modified_at = chrono::Utc::now();
        self
    }

    /// Mark as mandatory
    pub fn mark_mandatory(mut self) -> Self {
        self.is_mandatory = true;
        self
    }

    /// Set expiration
    pub fn with_expiration(mut self, expires_at: chrono::DateTime<chrono::Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// Check if policy is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expiry) = self.expires_at {
            chrono::Utc::now() > expiry
        } else {
            false
        }
    }

    /// Check if policy is active
    pub fn is_active(&self) -> bool {
        self.is_enforced && !self.is_expired()
    }
}

/// Password policy constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicyConfig {
    /// Minimum password length
    pub min_length: usize,
    /// Require uppercase letters
    pub require_uppercase: bool,
    /// Require lowercase letters
    pub require_lowercase: bool,
    /// Require numbers
    pub require_numbers: bool,
    /// Require special characters
    pub require_special_chars: bool,
    /// Password expiration days (0 = never)
    pub expiration_days: u32,
    /// Minimum password history to prevent reuse
    pub history_count: usize,
    /// Account lockout after N failed attempts
    pub lockout_threshold: u32,
    /// Lockout duration in minutes
    pub lockout_duration_minutes: u32,
}

impl Default for PasswordPolicyConfig {
    fn default() -> Self {
        Self {
            min_length: 12,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special_chars: true,
            expiration_days: 90,
            history_count: 5,
            lockout_threshold: 5,
            lockout_duration_minutes: 30,
        }
    }
}

impl PasswordPolicyConfig {
    /// Validate a password against the policy
    pub fn validate_password(&self, password: &str) -> SecurityResult<()> {
        // Check length
        if password.len() < self.min_length {
            return Err(SecurityError::Other(format!(
                "Password must be at least {} characters",
                self.min_length
            )));
        }

        // Check uppercase
        if self.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            return Err(SecurityError::Other(
                "Password must contain uppercase letters".to_string(),
            ));
        }

        // Check lowercase
        if self.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            return Err(SecurityError::Other(
                "Password must contain lowercase letters".to_string(),
            ));
        }

        // Check numbers
        if self.require_numbers && !password.chars().any(|c| c.is_numeric()) {
            return Err(SecurityError::Other("Password must contain numbers".to_string()));
        }

        // Check special characters
        if self.require_special_chars {
            let special_chars = "!@#$%^&*()_+-=[]{}|;:,.<>?";
            if !password.chars().any(|c| special_chars.contains(c)) {
                return Err(SecurityError::Other(
                    "Password must contain special characters".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// Security policy engine
#[derive(Clone)]
pub struct SecurityPolicyEngine {
    /// Stored policies
    policies: Arc<RwLock<HashMap<String, SecurityPolicy>>>,
    /// Password policy configuration
    password_policy: Arc<RwLock<PasswordPolicyConfig>>,
}

impl SecurityPolicyEngine {
    /// Create a new policy engine
    pub fn new() -> Self {
        Self {
            policies: Arc::new(RwLock::new(HashMap::new())),
            password_policy: Arc::new(RwLock::new(PasswordPolicyConfig::default())),
        }
    }

    /// Initialize with standard policies
    pub async fn with_standard_policies() -> Self {
        let engine = Self::new();

        // Password policy
        let password_policy = SecurityPolicy::new(
            "pwd_policy",
            "Default Password Policy",
            PolicyType::PasswordPolicy,
        )
        .with_description("Enterprise-grade password security policy")
        .add_rule("min_length", "12")
        .add_rule("require_uppercase", "true")
        .add_rule("require_lowercase", "true")
        .add_rule("require_numbers", "true")
        .add_rule("require_special_chars", "true")
        .add_rule("expiration_days", "90")
        .mark_mandatory();

        // Encryption policy
        let encryption_policy = SecurityPolicy::new(
            "enc_policy",
            "Default Encryption Policy",
            PolicyType::EncryptionPolicy,
        )
        .with_description("Require AES-256-GCM for sensitive data")
        .add_rule("algorithm", "AES-256-GCM")
        .add_rule("tls_required", "true")
        .add_rule("tls_version", "1.3")
        .mark_mandatory();

        // Session policy
        let session_policy = SecurityPolicy::new(
            "session_policy",
            "Default Session Policy",
            PolicyType::SessionPolicy,
        )
        .with_description("Session timeout and lifecycle management")
        .add_rule("default_timeout_hours", "24")
        .add_rule("max_idle_minutes", "60")
        .add_rule("extend_on_activity", "true")
        .add_rule("max_concurrent_sessions", "5");

        // Audit policy
        let audit_policy =
            SecurityPolicy::new("audit_policy", "Default Audit Policy", PolicyType::AuditPolicy)
                .with_description("Comprehensive audit logging")
                .add_rule("log_level", "INFO")
                .add_rule("max_events", "100000")
                .add_rule("track_authentication", "true")
                .add_rule("track_authorization", "true")
                .add_rule("track_data_access", "true")
                .mark_mandatory();

        // Data retention policy
        let retention_policy = SecurityPolicy::new(
            "retention_policy",
            "Default Retention Policy",
            PolicyType::DataRetentionPolicy,
        )
        .with_description("Data retention period for audit logs")
        .add_rule("audit_log_retention_days", "365")
        .add_rule("access_log_retention_days", "90");

        engine.add_policy(password_policy).await.ok();
        engine.add_policy(encryption_policy).await.ok();
        engine.add_policy(session_policy).await.ok();
        engine.add_policy(audit_policy).await.ok();
        engine.add_policy(retention_policy).await.ok();

        engine
    }

    /// Add a policy
    pub async fn add_policy(&self, policy: SecurityPolicy) -> SecurityResult<()> {
        let mut policies = self.policies.write().await;
        policies.insert(policy.id.clone(), policy);
        Ok(())
    }

    /// Get a policy
    pub async fn get_policy(&self, policy_id: &str) -> SecurityResult<Option<SecurityPolicy>> {
        let policies = self.policies.read().await;
        Ok(policies.get(policy_id).cloned())
    }

    /// Update a policy
    pub async fn update_policy(&self, policy: SecurityPolicy) -> SecurityResult<()> {
        let mut policies = self.policies.write().await;
        let mut updated = policy;
        updated.modified_at = chrono::Utc::now();
        policies.insert(updated.id.clone(), updated);
        Ok(())
    }

    /// Get policies by type
    pub async fn get_policies_by_type(
        &self,
        policy_type: PolicyType,
    ) -> SecurityResult<Vec<SecurityPolicy>> {
        let policies = self.policies.read().await;
        Ok(policies
            .values()
            .filter(|p| p.policy_type == policy_type && p.is_active())
            .cloned()
            .collect())
    }

    /// Get all active policies
    pub async fn get_active_policies(&self) -> SecurityResult<Vec<SecurityPolicy>> {
        let policies = self.policies.read().await;
        Ok(policies.values().filter(|p| p.is_active()).cloned().collect())
    }

    /// Check if policy is enforced
    pub async fn is_policy_enforced(&self, policy_id: &str) -> SecurityResult<bool> {
        if let Some(policy) = self.get_policy(policy_id).await? {
            Ok(policy.is_active())
        } else {
            Ok(false)
        }
    }

    /// Get password policy configuration
    pub async fn get_password_policy(&self) -> SecurityResult<PasswordPolicyConfig> {
        let policy = self.password_policy.read().await;
        Ok(policy.clone())
    }

    /// Update password policy configuration
    pub async fn update_password_policy(&self, config: PasswordPolicyConfig) -> SecurityResult<()> {
        let mut policy = self.password_policy.write().await;
        *policy = config;
        Ok(())
    }

    /// Validate password against policy
    pub async fn validate_password(&self, password: &str) -> SecurityResult<()> {
        let policy = self.password_policy.read().await;
        policy.validate_password(password)
    }

    /// List all policies
    pub async fn list_policies(&self) -> SecurityResult<Vec<SecurityPolicy>> {
        let policies = self.policies.read().await;
        Ok(policies.values().cloned().collect())
    }

    /// Check policy compliance
    pub async fn check_compliance(&self, policy_id: &str) -> SecurityResult<bool> {
        if let Some(policy) = self.get_policy(policy_id).await? {
            Ok(!policy.is_expired())
        } else {
            Ok(false)
        }
    }
}

impl Default for SecurityPolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_policy_creation() {
        let policy = SecurityPolicy::new("policy1", "Test Policy", PolicyType::AccessPolicy)
            .with_description("A test policy")
            .add_rule("rule1", "value1");

        assert_eq!(policy.id, "policy1");
        assert!(!policy.is_expired());
    }

    #[test]
    fn test_password_policy_validation() {
        let policy = PasswordPolicyConfig::default();

        assert!(policy.validate_password("ValidPass123!").is_ok());
        assert!(policy.validate_password("short").is_err());
        assert!(policy.validate_password("nouppercase123!").is_err());
    }

    #[tokio::test]
    async fn test_security_policy_engine() {
        let engine = SecurityPolicyEngine::with_standard_policies().await;

        let policies = engine.list_policies().await.unwrap();
        assert!(!policies.is_empty());
    }

    #[tokio::test]
    async fn test_get_policies_by_type() {
        let engine = SecurityPolicyEngine::with_standard_policies().await;

        let policies = engine.get_policies_by_type(PolicyType::AuditPolicy).await.unwrap();

        assert!(!policies.is_empty());
    }

    #[tokio::test]
    async fn test_password_validation() {
        let engine = SecurityPolicyEngine::with_standard_policies().await;

        assert!(engine.validate_password("ValidPass123!").await.is_ok());
        assert!(engine.validate_password("weak").await.is_err());
    }
}
