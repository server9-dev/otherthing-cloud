//! Permission model for workflows, tasks, and resources
//!
//! Provides fine-grained access control through permissions on different
//! resource types and operations.

use crate::security::error::{SecurityError, SecurityResult};
use crate::security::rbac::{RoleManager, Subject};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Types of resources that can be protected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    /// Workflow definition
    Workflow,
    /// Task definition
    Task,
    /// Execution instance
    Execution,
    /// Audit log
    AuditLog,
    /// Compliance report
    ComplianceReport,
    /// Secrets and credentials
    Secret,
    /// User and role management
    SecurityConfiguration,
}

impl ResourceType {
    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            ResourceType::Workflow => "workflow",
            ResourceType::Task => "task",
            ResourceType::Execution => "execution",
            ResourceType::AuditLog => "audit_log",
            ResourceType::ComplianceReport => "compliance_report",
            ResourceType::Secret => "secret",
            ResourceType::SecurityConfiguration => "security_config",
        }
    }
}

/// Permission for an action on a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    /// Resource type
    pub resource_type: ResourceType,
    /// Action (read, write, execute, delete, etc.)
    pub action: String,
    /// Resource ID (None means all resources of this type)
    pub resource_id: Option<String>,
    /// Whether this permission is granted or denied
    pub is_granted: bool,
    /// Conditions under which this permission applies
    pub conditions: Option<Vec<String>>,
}

impl Permission {
    /// Create a new permission
    pub fn new(resource_type: ResourceType, action: impl Into<String>) -> Self {
        Self {
            resource_type,
            action: action.into(),
            resource_id: None,
            is_granted: true,
            conditions: None,
        }
    }

    /// Create a permission for a specific resource
    pub fn for_resource(
        resource_type: ResourceType,
        action: impl Into<String>,
        resource_id: impl Into<String>,
    ) -> Self {
        Self {
            resource_type,
            action: action.into(),
            resource_id: Some(resource_id.into()),
            is_granted: true,
            conditions: None,
        }
    }

    /// Add conditions to the permission
    pub fn with_conditions(mut self, conditions: Vec<String>) -> Self {
        self.conditions = Some(conditions);
        self
    }

    /// Deny this permission
    pub fn deny(mut self) -> Self {
        self.is_granted = false;
        self
    }

    /// Convert to permission string format (e.g., "workflow:read")
    pub fn to_string_format(&self) -> String {
        format!("{}:{}", self.resource_type.as_str(), self.action)
    }
}

/// Resource-specific permission model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePermissionModel {
    /// Resource ID
    pub resource_id: String,
    /// Resource type
    pub resource_type: ResourceType,
    /// Owner of the resource
    pub owner_id: String,
    /// Explicit permissions granted
    pub permissions: HashMap<String, Vec<String>>, // subject_id -> permissions
    /// Explicit deny permissions
    pub denied_permissions: HashMap<String, Vec<String>>, // subject_id -> denied permissions
}

impl ResourcePermissionModel {
    /// Create a new resource permission model
    pub fn new(
        resource_id: impl Into<String>,
        resource_type: ResourceType,
        owner_id: impl Into<String>,
    ) -> Self {
        Self {
            resource_id: resource_id.into(),
            resource_type,
            owner_id: owner_id.into(),
            permissions: HashMap::new(),
            denied_permissions: HashMap::new(),
        }
    }

    /// Grant permission to a subject
    pub fn grant_permission(&mut self, subject_id: impl Into<String>, action: impl Into<String>) {
        let perms = self
            .permissions
            .entry(subject_id.into())
            .or_insert_with(Vec::new);
        let action_str = action.into();
        if !perms.contains(&action_str) {
            perms.push(action_str);
        }
    }

    /// Deny permission to a subject
    pub fn deny_permission(&mut self, subject_id: impl Into<String>, action: impl Into<String>) {
        let perms = self
            .denied_permissions
            .entry(subject_id.into())
            .or_insert_with(Vec::new);
        let action_str = action.into();
        if !perms.contains(&action_str) {
            perms.push(action_str);
        }
    }

    /// Check if subject has permission
    pub fn has_permission(&self, subject_id: &str, action: &str) -> bool {
        // Explicit deny takes precedence
        if let Some(denied) = self.denied_permissions.get(subject_id) {
            if denied.contains(&action.to_string()) {
                return false;
            }
        }

        // Check explicit grants
        if let Some(granted) = self.permissions.get(subject_id) {
            if granted.contains(&action.to_string()) {
                return true;
            }
        }

        false
    }
}

/// Permission checker for enforcing access control
#[derive(Debug, Clone)]
pub struct PermissionChecker {
    role_manager: Arc<RoleManager>,
    resource_permissions: Arc<RwLock<HashMap<String, ResourcePermissionModel>>>,
}

impl PermissionChecker {
    /// Create a new permission checker
    pub fn new(role_manager: Arc<RoleManager>) -> Self {
        Self {
            role_manager,
            resource_permissions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if subject can perform action
    pub async fn can_perform(
        &self,
        subject: &Subject,
        resource_type: ResourceType,
        action: &str,
    ) -> SecurityResult<bool> {
        // Inactive subjects cannot perform any action
        if !subject.is_active {
            return Ok(false);
        }

        // Get subject permissions from roles
        let permissions = self.role_manager.get_subject_permissions(&subject.id).await?;

        // Check for wildcard permission (admin)
        if permissions.contains(&format!("{}:*", resource_type.as_str())) {
            return Ok(true);
        }

        // Check for specific permission
        let permission_str = format!("{}:{}", resource_type.as_str(), action);
        Ok(permissions.contains(&permission_str))
    }

    /// Check if subject can perform action on specific resource
    pub async fn can_perform_on_resource(
        &self,
        subject: &Subject,
        resource_type: ResourceType,
        action: &str,
        resource_id: &str,
    ) -> SecurityResult<bool> {
        // Inactive subjects cannot perform any action
        if !subject.is_active {
            return Ok(false);
        }

        // First check role-based permissions
        if self.can_perform(subject, resource_type, action).await? {
            return Ok(true);
        }

        // Then check resource-specific permissions
        let resource_permissions = self.resource_permissions.read().await;
        if let Some(model) = resource_permissions.get(resource_id) {
            if model.has_permission(&subject.id, action) {
                return Ok(true);
            }

            // Check if owner
            if model.owner_id == subject.id {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Grant permission on a specific resource
    pub async fn grant_resource_permission(
        &self,
        resource_id: &str,
        subject_id: &str,
        action: &str,
    ) -> SecurityResult<()> {
        let mut resource_permissions = self.resource_permissions.write().await;

        if let Some(model) = resource_permissions.get_mut(resource_id) {
            model.grant_permission(subject_id, action);
            Ok(())
        } else {
            Err(SecurityError::ResourceNotFound(resource_id.to_string()))
        }
    }

    /// Deny permission on a specific resource
    pub async fn deny_resource_permission(
        &self,
        resource_id: &str,
        subject_id: &str,
        action: &str,
    ) -> SecurityResult<()> {
        let mut resource_permissions = self.resource_permissions.write().await;

        if let Some(model) = resource_permissions.get_mut(resource_id) {
            model.deny_permission(subject_id, action);
            Ok(())
        } else {
            Err(SecurityError::ResourceNotFound(resource_id.to_string()))
        }
    }

    /// Create a new protected resource
    pub async fn create_protected_resource(
        &self,
        resource_id: impl Into<String>,
        resource_type: ResourceType,
        owner_id: impl Into<String>,
    ) -> SecurityResult<()> {
        let mut resource_permissions = self.resource_permissions.write().await;
        let resource_id_str = resource_id.into();

        if resource_permissions.contains_key(&resource_id_str) {
            return Err(SecurityError::ResourceAlreadyExists(resource_id_str.clone()));
        }

        resource_permissions.insert(
            resource_id_str.clone(),
            ResourcePermissionModel::new(
                resource_id_str,
                resource_type,
                owner_id.into(),
            ),
        );

        Ok(())
    }

    /// Get resource permission model
    pub async fn get_resource_model(&self, resource_id: &str) -> SecurityResult<Option<ResourcePermissionModel>> {
        let resource_permissions = self.resource_permissions.read().await;
        Ok(resource_permissions.get(resource_id).cloned())
    }
}

/// Permission model combining role-based and resource-based access control
#[derive(Debug, Clone)]
pub struct PermissionModel {
    permission_checker: Arc<PermissionChecker>,
}

impl PermissionModel {
    /// Create a new permission model
    pub fn new(role_manager: Arc<RoleManager>) -> Self {
        Self {
            permission_checker: Arc::new(PermissionChecker::new(role_manager)),
        }
    }

    /// Get the underlying permission checker
    pub fn checker(&self) -> Arc<PermissionChecker> {
        self.permission_checker.clone()
    }

    /// Verify subject can perform action
    pub async fn verify_permission(
        &self,
        subject: &Subject,
        resource_type: ResourceType,
        action: &str,
    ) -> SecurityResult<()> {
        if self
            .permission_checker
            .can_perform(subject, resource_type, action)
            .await?
        {
            Ok(())
        } else {
            Err(SecurityError::PermissionDenied(format!(
                "{}:{}",
                resource_type.as_str(),
                action
            )))
        }
    }

    /// Verify subject can perform action on resource
    pub async fn verify_resource_permission(
        &self,
        subject: &Subject,
        resource_type: ResourceType,
        action: &str,
        resource_id: &str,
    ) -> SecurityResult<()> {
        if self
            .permission_checker
            .can_perform_on_resource(subject, resource_type, action, resource_id)
            .await?
        {
            Ok(())
        } else {
            Err(SecurityError::PermissionDenied(format!(
                "{}:{} on {}",
                resource_type.as_str(),
                action,
                resource_id
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_creation() {
        let perm = Permission::new(ResourceType::Workflow, "read");
        assert_eq!(perm.to_string_format(), "workflow:read");
        assert!(perm.is_granted);
    }

    #[test]
    fn test_resource_permission_model() {
        let mut model = ResourcePermissionModel::new("workflow1", ResourceType::Workflow, "owner1");

        model.grant_permission("user1", "read");
        model.grant_permission("user1", "edit");

        assert!(model.has_permission("user1", "read"));
        assert!(model.has_permission("user1", "edit"));
        assert!(!model.has_permission("user2", "read"));
    }

    #[test]
    fn test_resource_permission_deny() {
        let mut model = ResourcePermissionModel::new("workflow1", ResourceType::Workflow, "owner1");

        model.grant_permission("user1", "read");
        model.deny_permission("user1", "read");

        assert!(!model.has_permission("user1", "read"));
    }

    #[tokio::test]
    async fn test_permission_checker() {
        let role_manager = Arc::new(RoleManager::with_enterprise_defaults().await);
        let checker = PermissionChecker::new(role_manager.clone());

        role_manager.assign_role("user1", "editor").await.unwrap();

        let subject = Subject::new("user1", crate::security::rbac::SubjectType::User);

        assert!(checker
            .can_perform(&subject, ResourceType::Workflow, "read")
            .await
            .unwrap());

        assert!(!checker
            .can_perform(&subject, ResourceType::SecurityConfiguration, "read")
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_permission_model() {
        let role_manager = Arc::new(RoleManager::with_enterprise_defaults().await);
        let model = PermissionModel::new(role_manager.clone());

        role_manager.assign_role("user1", "viewer").await.unwrap();

        let subject = Subject::new("user1", crate::security::rbac::SubjectType::User);

        assert!(model
            .verify_permission(&subject, ResourceType::Workflow, "read")
            .await
            .is_ok());

        assert!(model
            .verify_permission(&subject, ResourceType::Workflow, "edit")
            .await
            .is_err());
    }
}
