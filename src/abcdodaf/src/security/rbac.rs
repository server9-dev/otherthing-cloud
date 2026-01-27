//! Role-Based Access Control (RBAC) system
//!
//! Provides role definition, management, and subject-to-role mapping for
//! implementing hierarchical access control in enterprise environments.

use crate::security::error::{SecurityError, SecurityResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Type of subject in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubjectType {
    /// Individual user
    User,
    /// Service account or application
    Service,
    /// System administrator
    Admin,
    /// External integration
    External,
}

/// Subject represents a principal (user, service, etc.) in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    /// Unique identifier for the subject
    pub id: String,
    /// Type of subject
    pub subject_type: SubjectType,
    /// Display name
    pub name: Option<String>,
    /// Email address (for users)
    pub email: Option<String>,
    /// Whether the subject is active
    pub is_active: bool,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Subject {
    /// Create a new subject
    pub fn new(id: impl Into<String>, subject_type: SubjectType) -> Self {
        Self {
            id: id.into(),
            subject_type,
            name: None,
            email: None,
            is_active: true,
            metadata: HashMap::new(),
        }
    }

    /// Set the subject's display name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the subject's email
    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    /// Add metadata field
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Deactivate the subject
    pub fn deactivate(mut self) -> Self {
        self.is_active = false;
        self
    }
}

/// Role definition with associated permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// Unique identifier for the role
    pub name: String,
    /// Human-readable description
    pub description: Option<String>,
    /// Permissions granted by this role
    pub permissions: Vec<String>,
    /// Parent role (for role hierarchy)
    pub parent_role: Option<Box<Role>>,
    /// Whether this is a system role
    pub is_system_role: bool,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last modified timestamp
    pub modified_at: chrono::DateTime<chrono::Utc>,
}

impl Role {
    /// Create a new role
    pub fn new(name: impl Into<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            name: name.into(),
            description: None,
            permissions: Vec::new(),
            parent_role: None,
            is_system_role: false,
            created_at: now,
            modified_at: now,
        }
    }

    /// Create a system role (cannot be deleted)
    pub fn system_role(name: impl Into<String>) -> Self {
        let mut role = Self::new(name);
        role.is_system_role = true;
        role
    }

    /// Set role description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a permission to the role
    pub fn add_permission(mut self, permission: impl Into<String>) -> Self {
        let perm = permission.into();
        if !self.permissions.contains(&perm) {
            self.permissions.push(perm);
        }
        self
    }

    /// Add multiple permissions
    pub fn add_permissions(mut self, permissions: Vec<String>) -> Self {
        for perm in permissions {
            if !self.permissions.contains(&perm) {
                self.permissions.push(perm);
            }
        }
        self
    }

    /// Set parent role for inheritance
    pub fn with_parent(mut self, parent: Role) -> Self {
        self.parent_role = Some(Box::new(parent));
        self
    }

    /// Get all permissions including inherited ones
    pub fn get_all_permissions(&self) -> Vec<String> {
        let mut perms = self.permissions.clone();
        if let Some(parent) = &self.parent_role {
            let parent_perms = parent.get_all_permissions();
            for perm in parent_perms {
                if !perms.contains(&perm) {
                    perms.push(perm);
                }
            }
        }
        perms
    }

    /// Check if role has a specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.get_all_permissions().contains(&permission.to_string())
    }

    /// Remove a permission from the role
    pub fn remove_permission(&mut self, permission: &str) {
        self.permissions.retain(|p| p != permission);
        self.modified_at = chrono::Utc::now();
    }
}

/// Manages roles and subject-to-role assignments
#[derive(Debug, Clone)]
pub struct RoleManager {
    /// Stored roles
    roles: Arc<RwLock<HashMap<String, Role>>>,
    /// Subject to roles mapping
    subject_roles: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl RoleManager {
    /// Create a new role manager
    pub fn new() -> Self {
        Self {
            roles: Arc::new(RwLock::new(HashMap::new())),
            subject_roles: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize with standard enterprise roles
    pub async fn with_enterprise_defaults() -> Self {
        let manager = Self::new();

        // Create standard roles
        let viewer = Role::system_role("viewer")
            .with_description("Read-only access to workflows and tasks")
            .add_permissions(vec![
                "workflow:read".to_string(),
                "task:read".to_string(),
                "audit:read".to_string(),
            ]);

        let editor = Role::system_role("editor")
            .with_description("Can create and modify workflows")
            .add_permissions(vec![
                "workflow:read".to_string(),
                "workflow:create".to_string(),
                "workflow:edit".to_string(),
                "task:read".to_string(),
                "task:execute".to_string(),
                "audit:read".to_string(),
            ]);

        let executor = Role::system_role("executor")
            .with_description("Can execute workflows and tasks")
            .add_permissions(vec![
                "workflow:read".to_string(),
                "workflow:execute".to_string(),
                "task:read".to_string(),
                "task:execute".to_string(),
                "execution:monitor".to_string(),
                "audit:read".to_string(),
            ]);

        let admin = Role::system_role("admin")
            .with_description("Full administrative access")
            .add_permissions(vec![
                "workflow:*".to_string(),
                "task:*".to_string(),
                "execution:*".to_string(),
                "audit:*".to_string(),
                "compliance:*".to_string(),
                "security:*".to_string(),
                "role:*".to_string(),
            ]);

        let auditor = Role::system_role("auditor")
            .with_description("Can read and export audit logs")
            .add_permissions(vec![
                "audit:read".to_string(),
                "audit:export".to_string(),
                "compliance:read".to_string(),
            ]);

        manager.create_role(viewer).await.ok();
        manager.create_role(editor).await.ok();
        manager.create_role(executor).await.ok();
        manager.create_role(admin).await.ok();
        manager.create_role(auditor).await.ok();

        manager
    }

    /// Create a new role
    pub async fn create_role(&self, role: Role) -> SecurityResult<()> {
        let mut roles = self.roles.write().await;
        roles.insert(role.name.clone(), role);
        Ok(())
    }

    /// Get a role by name
    pub async fn get_role(&self, role_name: &str) -> SecurityResult<Option<Role>> {
        let roles = self.roles.read().await;
        Ok(roles.get(role_name).cloned())
    }

    /// Update an existing role
    pub async fn update_role(&self, role: Role) -> SecurityResult<()> {
        let mut roles = self.roles.write().await;

        if !roles.contains_key(&role.name) {
            return Err(SecurityError::RoleNotFound(role.name));
        }

        let mut updated_role = role;
        updated_role.modified_at = chrono::Utc::now();
        roles.insert(updated_role.name.clone(), updated_role);
        Ok(())
    }

    /// Delete a role (system roles cannot be deleted)
    pub async fn delete_role(&self, role_name: &str) -> SecurityResult<()> {
        let mut roles = self.roles.write().await;

        if let Some(role) = roles.get(role_name) {
            if role.is_system_role {
                return Err(SecurityError::CannotDeleteSystemRole(role_name.to_string()));
            }
        }

        roles.remove(role_name);
        Ok(())
    }

    /// List all roles
    pub async fn list_roles(&self) -> SecurityResult<Vec<Role>> {
        let roles = self.roles.read().await;
        Ok(roles.values().cloned().collect())
    }

    /// Assign a role to a subject
    pub async fn assign_role(&self, subject_id: &str, role_name: &str) -> SecurityResult<()> {
        let roles = self.roles.read().await;
        if !roles.contains_key(role_name) {
            return Err(SecurityError::RoleNotFound(role_name.to_string()));
        }
        drop(roles);

        let mut subject_roles = self.subject_roles.write().await;
        let subject_role_list = subject_roles
            .entry(subject_id.to_string())
            .or_insert_with(Vec::new);

        if !subject_role_list.contains(&role_name.to_string()) {
            subject_role_list.push(role_name.to_string());
        }

        Ok(())
    }

    /// Assign multiple roles to a subject
    pub async fn assign_roles(&self, subject_id: &str, role_names: Vec<&str>) -> SecurityResult<()> {
        for role_name in role_names {
            self.assign_role(subject_id, role_name).await?;
        }
        Ok(())
    }

    /// Revoke a role from a subject
    pub async fn revoke_role(&self, subject_id: &str, role_name: &str) -> SecurityResult<()> {
        let mut subject_roles = self.subject_roles.write().await;

        if let Some(roles) = subject_roles.get_mut(subject_id) {
            roles.retain(|r| r != role_name);
        }

        Ok(())
    }

    /// Get all roles for a subject
    pub async fn get_subject_roles(&self, subject_id: &str) -> SecurityResult<Vec<Role>> {
        let subject_roles = self.subject_roles.read().await;
        let role_names = subject_roles.get(subject_id).cloned().unwrap_or_default();

        let roles = self.roles.read().await;
        let mut result = Vec::new();

        for role_name in role_names {
            if let Some(role) = roles.get(&role_name) {
                result.push(role.clone());
            }
        }

        Ok(result)
    }

    /// Check if subject has a specific role
    pub async fn has_role(&self, subject_id: &str, role_name: &str) -> SecurityResult<bool> {
        let subject_roles = self.subject_roles.read().await;
        Ok(subject_roles
            .get(subject_id)
            .map(|roles| roles.contains(&role_name.to_string()))
            .unwrap_or(false))
    }

    /// Get all permissions for a subject
    pub async fn get_subject_permissions(&self, subject_id: &str) -> SecurityResult<Vec<String>> {
        let roles = self.get_subject_roles(subject_id).await?;
        let mut permissions = Vec::new();

        for role in roles {
            for perm in role.get_all_permissions() {
                if !permissions.contains(&perm) {
                    permissions.push(perm);
                }
            }
        }

        Ok(permissions)
    }
}

impl Default for RoleManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subject_creation() {
        let subject = Subject::new("user123", SubjectType::User)
            .with_name("John Doe")
            .with_email("john@example.com");

        assert_eq!(subject.id, "user123");
        assert_eq!(subject.subject_type, SubjectType::User);
        assert_eq!(subject.name, Some("John Doe".to_string()));
        assert!(subject.is_active);
    }

    #[test]
    fn test_role_creation() {
        let role = Role::new("editor")
            .with_description("Editor role")
            .add_permission("workflow:read")
            .add_permission("workflow:edit");

        assert_eq!(role.name, "editor");
        assert_eq!(role.permissions.len(), 2);
        assert!(role.has_permission("workflow:read"));
        assert!(!role.is_system_role);
    }

    #[test]
    fn test_role_inheritance() {
        let base_role = Role::new("base")
            .add_permission("read");

        let derived_role = Role::new("derived")
            .add_permission("write")
            .with_parent(base_role);

        let perms = derived_role.get_all_permissions();
        assert!(perms.contains(&"read".to_string()));
        assert!(perms.contains(&"write".to_string()));
    }

    #[tokio::test]
    async fn test_role_manager() {
        let manager = RoleManager::with_enterprise_defaults().await;

        let roles = manager.list_roles().await.unwrap();
        assert!(!roles.is_empty());

        assert!(manager.get_role("admin").await.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_subject_role_assignment() {
        let manager = RoleManager::with_enterprise_defaults().await;
        let subject_id = "user1";

        manager.assign_role(subject_id, "viewer").await.unwrap();
        manager.assign_role(subject_id, "editor").await.unwrap();

        let roles = manager.get_subject_roles(subject_id).await.unwrap();
        assert_eq!(roles.len(), 2);

        assert!(manager.has_role(subject_id, "viewer").await.unwrap());
        assert!(manager.has_role(subject_id, "editor").await.unwrap());
        assert!(!manager.has_role(subject_id, "admin").await.unwrap());
    }

    #[tokio::test]
    async fn test_subject_permissions() {
        let manager = RoleManager::with_enterprise_defaults().await;
        let subject_id = "user1";

        manager.assign_role(subject_id, "editor").await.unwrap();

        let perms = manager.get_subject_permissions(subject_id).await.unwrap();
        assert!(perms.contains(&"workflow:read".to_string()));
        assert!(perms.contains(&"workflow:edit".to_string()));
    }

    #[tokio::test]
    async fn test_role_revocation() {
        let manager = RoleManager::with_enterprise_defaults().await;
        let subject_id = "user1";

        manager.assign_role(subject_id, "editor").await.unwrap();
        assert!(manager.has_role(subject_id, "editor").await.unwrap());

        manager.revoke_role(subject_id, "editor").await.unwrap();
        assert!(!manager.has_role(subject_id, "editor").await.unwrap());
    }
}
