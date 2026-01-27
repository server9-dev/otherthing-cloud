//! Enterprise-grade security, compliance, and audit logging for ABCD ODAF
//!
//! This module provides:
//! - Role-Based Access Control (RBAC) system
//! - Permission model for workflows and tasks
//! - Comprehensive audit logging system
//! - Encryption support for sensitive data
//! - Sensitive data masking and redaction
//! - Compliance reporting framework
//! - Security policy enforcement
//! - Secret management (credentials, API keys)
//! - Session management
//! - Security scanning for workflows
//! - Compliance validation rules
//! - Audit log export capabilities

pub mod rbac;
pub mod permissions;
pub mod audit;
pub mod encryption;
pub mod compliance;
pub mod secrets;
pub mod session;
pub mod security_policy;
pub mod audit_export;
pub mod error;

pub use error::{SecurityError, SecurityResult};
pub use rbac::{Role, RoleManager, Subject, SubjectType};
pub use permissions::{Permission, PermissionChecker, PermissionModel, ResourceType};
pub use audit::{AuditLogger, AuditEvent, AuditLevel, EventCategory};
pub use compliance::{ComplianceValidator, ComplianceRule, ComplianceReport};
pub use secrets::{SecretManager, SecretValue};
pub use session::{SessionManager, Session};
pub use security_policy::SecurityPolicyEngine;
pub use audit_export::AuditExporter;

/// Security context for evaluating access control and permissions
#[derive(Debug, Clone)]
pub struct SecurityContext {
    /// Subject requesting the action
    pub subject: Subject,
    /// Subject's assigned roles
    pub roles: Vec<Role>,
    /// Current session ID
    pub session_id: Option<String>,
    /// Timestamp of context creation
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl SecurityContext {
    /// Create a new security context
    pub fn new(subject: Subject) -> Self {
        Self {
            subject,
            roles: Vec::new(),
            session_id: None,
            created_at: chrono::Utc::now(),
        }
    }

    /// Add a role to the context
    pub fn with_role(mut self, role: Role) -> Self {
        self.roles.push(role);
        self
    }

    /// Add multiple roles
    pub fn with_roles(mut self, roles: Vec<Role>) -> Self {
        self.roles.extend(roles);
        self
    }

    /// Set session ID
    pub fn with_session(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Check if context has a specific role
    pub fn has_role(&self, role_name: &str) -> bool {
        self.roles.iter().any(|r| r.name == role_name)
    }

    /// Check if context has any of the given roles
    pub fn has_any_role(&self, role_names: &[&str]) -> bool {
        self.roles
            .iter()
            .any(|r| role_names.contains(&r.name.as_str()))
    }

    /// Get all permission names from assigned roles
    pub fn get_all_permissions(&self) -> Vec<String> {
        self.roles
            .iter()
            .flat_map(|r| r.permissions.iter().cloned())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_context_creation() {
        let subject = Subject::new("user1", SubjectType::User);
        let ctx = SecurityContext::new(subject);

        assert_eq!(ctx.subject.id, "user1");
        assert!(ctx.roles.is_empty());
        assert!(ctx.session_id.is_none());
    }

    #[test]
    fn test_security_context_with_roles() {
        let subject = Subject::new("user1", SubjectType::User);
        let role = Role::new("admin");
        let ctx = SecurityContext::new(subject)
            .with_role(role)
            .with_session("session123".to_string());

        assert_eq!(ctx.roles.len(), 1);
        assert!(ctx.has_role("admin"));
        assert_eq!(ctx.session_id, Some("session123".to_string()));
    }

    #[test]
    fn test_security_context_permissions() {
        let subject = Subject::new("user1", SubjectType::User);
        let mut role = Role::new("editor");
        role.permissions = vec!["workflow:read".to_string(), "workflow:edit".to_string()];

        let ctx = SecurityContext::new(subject).with_role(role);
        let perms = ctx.get_all_permissions();

        assert!(perms.contains(&"workflow:read".to_string()));
        assert!(perms.contains(&"workflow:edit".to_string()));
    }
}
