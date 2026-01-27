//! Discretionary Items System for CMMN
//!
//! Implements user-controlled activation of discretionary plan items.
//! This allows case workers to dynamically add activities to the case plan
//! based on their knowledge and judgment.

use super::cmmn::*;
use crate::error::{AbcdodafError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

/// A discretionary item that can be optionally activated by case workers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscretionaryItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    /// The actual plan item definition
    pub plan_item: PlanItem,
    /// Whether this item is available for activation
    pub is_available: bool,
    /// Authorization rules - who can activate this item
    pub authorized_roles: Vec<String>,
    /// When was this item added to the discretionary pool
    pub added_at: chrono::DateTime<chrono::Utc>,
}

/// Group of related discretionary items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscretionaryItemGroup {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub items: Vec<DiscretionaryItem>,
}

/// Activation request for a discretionary item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscretionaryActivation {
    pub id: String,
    pub discretionary_item_id: String,
    pub activated_by: String,
    pub activated_at: chrono::DateTime<chrono::Utc>,
    pub reason: Option<String>,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Policy for discretionary item activation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivationPolicy {
    /// Item can only be activated with explicit permission
    Restricted { required_role: String },
    /// Item can be activated by any authorized user
    Authorized { roles: Vec<String> },
    /// Item can be freely activated by any case worker
    Open,
    /// Item can only be activated based on case context
    Contextual { condition: String },
}

/// Manager for discretionary items in a case
pub struct DiscretionaryItemManager {
    /// Available discretionary items
    items: HashMap<String, DiscretionaryItem>,
    /// Grouped discretionary items
    groups: HashMap<String, DiscretionaryItemGroup>,
    /// Activation history
    activations: Vec<DiscretionaryActivation>,
    /// Activation policies
    policies: HashMap<String, ActivationPolicy>,
}

impl DiscretionaryItemManager {
    /// Create a new discretionary item manager
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            groups: HashMap::new(),
            activations: Vec::new(),
            policies: HashMap::new(),
        }
    }

    /// Add a discretionary item
    pub fn add_discretionary_item(&mut self, item: DiscretionaryItem) -> &mut Self {
        debug!("Adding discretionary item: {}", item.id);
        self.items.insert(item.id.clone(), item);
        self
    }

    /// Add multiple discretionary items as a group
    pub fn add_discretionary_group(&mut self, group: DiscretionaryItemGroup) -> &mut Self {
        debug!("Adding discretionary item group: {}", group.id);
        for item in &group.items {
            self.items.insert(item.id.clone(), item.clone());
        }
        self.groups.insert(group.id.clone(), group);
        self
    }

    /// Get a discretionary item
    pub fn get_item(&self, item_id: &str) -> Option<&DiscretionaryItem> {
        self.items.get(item_id)
    }

    /// List all available discretionary items
    pub fn list_available_items(&self) -> Vec<&DiscretionaryItem> {
        self.items
            .values()
            .filter(|item| item.is_available)
            .collect()
    }

    /// Check if a user can activate a discretionary item
    pub fn can_activate(
        &self,
        item_id: &str,
        user_role: &str,
    ) -> Result<bool> {
        let item = self.items.get(item_id)
            .ok_or_else(|| AbcdodafError::WorkflowError(format!("Item {} not found", item_id)))?;

        if !item.is_available {
            return Ok(false);
        }

        // Check authorization
        if item.authorized_roles.is_empty() {
            return Ok(true);
        }

        Ok(item.authorized_roles.iter().any(|role| role == user_role))
    }

    /// Activate a discretionary item
    pub fn activate_item(
        &mut self,
        item_id: &str,
        user_id: String,
        user_role: String,
        reason: Option<String>,
    ) -> Result<DiscretionaryActivation> {
        // Check authorization
        if !self.can_activate(item_id, &user_role)? {
            return Err(AbcdodafError::WorkflowError(
                format!("User {} is not authorized to activate item {}", user_id, item_id)
            ));
        }

        // Create activation record
        let activation = DiscretionaryActivation {
            id: uuid::Uuid::new_v4().to_string(),
            discretionary_item_id: item_id.to_string(),
            activated_by: user_id,
            activated_at: chrono::Utc::now(),
            reason,
            properties: HashMap::new(),
        };

        self.activations.push(activation.clone());

        // Mark item as no longer available in its original pool
        // (though it can be re-activated if defined as repeatable)
        if let Some(item) = self.items.get_mut(item_id) {
            if !Self::is_plan_item_repeatable(&item.plan_item) {
                item.is_available = false;
            }
        }

        debug!("Activated discretionary item: {}", item_id);
        Ok(activation)
    }

    /// Get activation history
    pub fn get_activation_history(&self) -> &[DiscretionaryActivation] {
        &self.activations
    }

    /// Set activation policy for an item
    pub fn set_activation_policy(
        &mut self,
        item_id: &str,
        policy: ActivationPolicy,
    ) -> &mut Self {
        self.policies.insert(item_id.to_string(), policy);
        self
    }

    /// Get suggested discretionary items based on case context
    pub fn get_suggested_items(
        &self,
        case_context: &HashMap<String, serde_json::Value>,
    ) -> Vec<&DiscretionaryItem> {
        self.items
            .values()
            .filter(|item| {
                item.is_available && self.matches_context(item, case_context)
            })
            .collect()
    }

    /// Check if an item matches the case context
    fn matches_context(
        &self,
        _item: &DiscretionaryItem,
        _context: &HashMap<String, serde_json::Value>,
    ) -> bool {
        // In production, would evaluate contextual conditions
        // based on case file items and state
        true
    }

    /// Disable a discretionary item
    pub fn disable_item(&mut self, item_id: &str) -> Result<()> {
        if let Some(item) = self.items.get_mut(item_id) {
            item.is_available = false;
            Ok(())
        } else {
            Err(AbcdodafError::WorkflowError(format!("Item {} not found", item_id)))
        }
    }

    /// Re-enable a discretionary item
    pub fn enable_item(&mut self, item_id: &str) -> Result<()> {
        if let Some(item) = self.items.get_mut(item_id) {
            item.is_available = true;
            Ok(())
        } else {
            Err(AbcdodafError::WorkflowError(format!("Item {} not found", item_id)))
        }
    }

    /// Get group of discretionary items
    pub fn get_group(&self, group_id: &str) -> Option<&DiscretionaryItemGroup> {
        self.groups.get(group_id)
    }

    /// Get all groups
    pub fn list_groups(&self) -> Vec<&DiscretionaryItemGroup> {
        self.groups.values().collect()
    }

    fn is_plan_item_repeatable(item: &PlanItem) -> bool {
        match item {
            PlanItem::HumanTask { repeatable, .. } => *repeatable,
            PlanItem::Stage { .. } => true, // Stages can be re-entered
            _ => false,
        }
    }
}

impl Default for DiscretionaryItemManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for discretionary items
pub struct DiscretionaryItemBuilder {
    id: String,
    name: String,
    description: Option<String>,
    plan_item: Option<PlanItem>,
    authorized_roles: Vec<String>,
}

impl DiscretionaryItemBuilder {
    /// Create a new discretionary item builder
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            plan_item: None,
            authorized_roles: Vec::new(),
        }
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the plan item
    pub fn with_plan_item(mut self, plan_item: PlanItem) -> Self {
        self.plan_item = Some(plan_item);
        self
    }

    /// Add authorized role
    pub fn add_authorized_role(mut self, role: impl Into<String>) -> Self {
        self.authorized_roles.push(role.into());
        self
    }

    /// Build the discretionary item
    pub fn build(self) -> Result<DiscretionaryItem> {
        let plan_item = self.plan_item
            .ok_or_else(|| AbcdodafError::WorkflowError("Plan item is required".to_string()))?;

        Ok(DiscretionaryItem {
            id: self.id,
            name: self.name,
            description: self.description,
            plan_item,
            is_available: true,
            authorized_roles: self.authorized_roles,
            added_at: chrono::Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discretionary_item_creation() {
        let item = DiscretionaryItemBuilder::new("item1", "Optional Review")
            .with_description("Optional code review")
            .with_plan_item(PlanItem::HumanTask {
                id: "task1".to_string(),
                name: "Code Review".to_string(),
                performer: Some("reviewer".to_string()),
                documentation: None,
                entry_criteria: vec![],
                exit_criteria: vec![],
                required: false,
                repeatable: false,
            })
            .add_authorized_role("reviewer")
            .add_authorized_role("lead")
            .build()
            .unwrap();

        assert_eq!(item.id, "item1");
        assert_eq!(item.name, "Optional Review");
        assert!(item.is_available);
        assert_eq!(item.authorized_roles.len(), 2);
    }

    #[test]
    fn test_discretionary_item_manager() {
        let mut manager = DiscretionaryItemManager::new();

        let item = DiscretionaryItemBuilder::new("item1", "Review")
            .with_plan_item(PlanItem::HumanTask {
                id: "task1".to_string(),
                name: "Review".to_string(),
                performer: None,
                documentation: None,
                entry_criteria: vec![],
                exit_criteria: vec![],
                required: false,
                repeatable: false,
            })
            .add_authorized_role("reviewer")
            .build()
            .unwrap();

        manager.add_discretionary_item(item);

        assert_eq!(manager.list_available_items().len(), 1);
        assert!(manager.can_activate("item1", "reviewer").unwrap());
        assert!(!manager.can_activate("item1", "other").unwrap());
    }

    #[test]
    fn test_discretionary_activation() {
        let mut manager = DiscretionaryItemManager::new();

        let item = DiscretionaryItemBuilder::new("item1", "Review")
            .with_plan_item(PlanItem::HumanTask {
                id: "task1".to_string(),
                name: "Review".to_string(),
                performer: None,
                documentation: None,
                entry_criteria: vec![],
                exit_criteria: vec![],
                required: false,
                repeatable: false,
            })
            .build()
            .unwrap();

        manager.add_discretionary_item(item);

        let activation = manager.activate_item(
            "item1",
            "user1".to_string(),
            "case_worker".to_string(),
            Some("Quality check required".to_string()),
        ).unwrap();

        assert_eq!(activation.discretionary_item_id, "item1");
        assert_eq!(activation.activated_by, "user1");
        assert!(activation.reason.is_some());
        assert_eq!(manager.get_activation_history().len(), 1);
    }

    #[test]
    fn test_discretionary_item_group() {
        let mut manager = DiscretionaryItemManager::new();

        let items = vec![
            DiscretionaryItemBuilder::new("item1", "Review")
                .with_plan_item(PlanItem::HumanTask {
                    id: "task1".to_string(),
                    name: "Review".to_string(),
                    performer: None,
                    documentation: None,
                    entry_criteria: vec![],
                    exit_criteria: vec![],
                    required: false,
                    repeatable: false,
                })
                .build()
                .unwrap(),
            DiscretionaryItemBuilder::new("item2", "Audit")
                .with_plan_item(PlanItem::HumanTask {
                    id: "task2".to_string(),
                    name: "Audit".to_string(),
                    performer: None,
                    documentation: None,
                    entry_criteria: vec![],
                    exit_criteria: vec![],
                    required: false,
                    repeatable: false,
                })
                .build()
                .unwrap(),
        ];

        let group = DiscretionaryItemGroup {
            id: "group1".to_string(),
            name: "Quality Assurance".to_string(),
            description: Some("QA discretionary items".to_string()),
            items,
        };

        manager.add_discretionary_group(group);

        assert_eq!(manager.list_groups().len(), 1);
        assert_eq!(manager.list_available_items().len(), 2);
    }

    #[test]
    fn test_discretionary_item_disable() {
        let mut manager = DiscretionaryItemManager::new();

        let item = DiscretionaryItemBuilder::new("item1", "Review")
            .with_plan_item(PlanItem::HumanTask {
                id: "task1".to_string(),
                name: "Review".to_string(),
                performer: None,
                documentation: None,
                entry_criteria: vec![],
                exit_criteria: vec![],
                required: false,
                repeatable: false,
            })
            .build()
            .unwrap();

        manager.add_discretionary_item(item);
        assert_eq!(manager.list_available_items().len(), 1);

        manager.disable_item("item1").unwrap();
        assert_eq!(manager.list_available_items().len(), 0);

        manager.enable_item("item1").unwrap();
        assert_eq!(manager.list_available_items().len(), 1);
    }
}
