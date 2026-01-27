//! CMMN Case Instance Runtime Engine
//!
//! This module implements the execution semantics for CMMN 1.1 cases,
//! including plan item lifecycle management, sentry evaluation, and event handling.

use super::cmmn::*;
use crate::error::{AbcdodafError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Plan item state in the case lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanItemState {
    /// Initial state
    Available,
    /// Ready to be activated
    Enabled,
    /// Currently active
    Active,
    /// Suspended (can be resumed)
    Suspended,
    /// Completed successfully
    Completed,
    /// Terminated without completion
    Terminated,
    /// Failed
    Failed,
}

/// Event that can occur in a case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CaseEvent {
    /// Manual activation of a discretionary item
    DiscretionaryItemActivated {
        item_id: String,
        triggered_at: chrono::DateTime<chrono::Utc>,
    },
    /// Plan item state change
    PlanItemStateChanged {
        item_id: String,
        old_state: PlanItemState,
        new_state: PlanItemState,
        triggered_at: chrono::DateTime<chrono::Utc>,
    },
    /// Case file item updated
    CaseFileItemUpdated {
        item_id: String,
        new_value: serde_json::Value,
        triggered_at: chrono::DateTime<chrono::Utc>,
    },
    /// External signal received
    SignalReceived {
        signal_name: String,
        data: Option<serde_json::Value>,
        triggered_at: chrono::DateTime<chrono::Utc>,
    },
    /// Timer fired
    TimerFired {
        listener_id: String,
        triggered_at: chrono::DateTime<chrono::Utc>,
    },
}

/// State tracking for a plan item instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanItemInstance {
    pub id: String,
    pub plan_item_id: String,
    pub state: PlanItemState,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub properties: HashMap<String, serde_json::Value>,
}

/// A case instance at runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseInstance {
    pub id: String,
    pub case_model_id: String,
    pub state: CaseInstanceState,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// All plan item instances in this case
    pub plan_items: HashMap<String, PlanItemInstance>,
    /// Current case file data
    pub case_file: HashMap<String, serde_json::Value>,
    /// Audit trail of all events
    pub event_history: Vec<CaseEvent>,
}

/// State of the case instance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaseInstanceState {
    /// Case is running
    Active,
    /// Case is suspended
    Suspended,
    /// Case has completed
    Completed,
    /// Case terminated without completion
    Terminated,
}

/// Sentry evaluation context
#[derive(Debug, Clone)]
pub struct SentryContext<'a> {
    pub plan_items: &'a HashMap<String, PlanItemInstance>,
    pub case_file: &'a HashMap<String, serde_json::Value>,
    pub case_instance: &'a CaseInstance,
}

impl CaseInstance {
    /// Create a new case instance
    pub fn new(case_model_id: impl Into<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            case_model_id: case_model_id.into(),
            state: CaseInstanceState::Active,
            created_at: now,
            completed_at: None,
            plan_items: HashMap::new(),
            case_file: HashMap::new(),
            event_history: Vec::new(),
        }
    }

    /// Record an event in the audit trail
    pub fn record_event(&mut self, event: CaseEvent) {
        debug!("Recording event in case {}: {:?}", self.id, event);
        self.event_history.push(event);
    }

    /// Get a plan item instance by its ID
    pub fn get_plan_item(&self, item_id: &str) -> Option<&PlanItemInstance> {
        self.plan_items.get(item_id)
    }

    /// Get mutable reference to a plan item
    pub fn get_plan_item_mut(&mut self, item_id: &str) -> Option<&mut PlanItemInstance> {
        self.plan_items.get_mut(item_id)
    }

    /// Create a new plan item instance
    pub fn create_plan_item(
        &mut self,
        plan_item_id: String,
        initial_state: PlanItemState,
    ) -> String {
        let instance = PlanItemInstance {
            id: uuid::Uuid::new_v4().to_string(),
            plan_item_id,
            state: initial_state,
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            properties: HashMap::new(),
        };
        let instance_id = instance.id.clone();
        self.plan_items.insert(instance_id.clone(), instance);
        instance_id
    }

    /// Change state of a plan item
    pub fn change_plan_item_state(
        &mut self,
        item_instance_id: &str,
        new_state: PlanItemState,
    ) -> Result<()> {
        // First phase: extract data we need
        let (old_state, plan_item_id) = if let Some(item) = self.plan_items.get_mut(item_instance_id) {
            let old_state = item.state;
            item.state = new_state;

            // Update timestamps
            match new_state {
                PlanItemState::Active if item.started_at.is_none() => {
                    item.started_at = Some(chrono::Utc::now());
                }
                PlanItemState::Completed | PlanItemState::Terminated | PlanItemState::Failed => {
                    item.completed_at = Some(chrono::Utc::now());
                }
                _ => {}
            }

            (old_state, item.plan_item_id.clone())
        } else {
            return Err(AbcdodafError::WorkflowError(format!(
                "Plan item instance {} not found",
                item_instance_id
            )));
        };

        // Second phase: record the event (no longer borrowing plan_items)
        self.record_event(CaseEvent::PlanItemStateChanged {
            item_id: plan_item_id,
            old_state,
            new_state,
            triggered_at: chrono::Utc::now(),
        });

        Ok(())
    }

    /// Update case file item
    pub fn update_case_file(&mut self, key: String, value: serde_json::Value) {
        self.case_file.insert(key.clone(), value.clone());
        self.record_event(CaseEvent::CaseFileItemUpdated {
            item_id: key,
            new_value: value,
            triggered_at: chrono::Utc::now(),
        });
    }

    /// Check if case is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.state,
            CaseInstanceState::Completed | CaseInstanceState::Terminated
        )
    }

    /// Get all plan items in a specific state
    pub fn get_items_in_state(&self, state: PlanItemState) -> Vec<&PlanItemInstance> {
        self.plan_items
            .values()
            .filter(|item| item.state == state)
            .collect()
    }

    /// Check if all required plan items are completed
    pub fn check_completion(&self, case_model: &CmmnCase) -> bool {
        for plan_item in &case_model.case_plan_model.plan_items {
            let plan_item_id = match plan_item {
                PlanItem::HumanTask { id, required: true, .. } => Some(id),
                PlanItem::ProcessTask { id, required: true, .. } => Some(id),
                _ => None,
            };

            if let Some(id) = plan_item_id {
                let completed = self.plan_items.values().any(|pi| {
                    pi.plan_item_id == *id && pi.state == PlanItemState::Completed
                });

                if !completed {
                    return false;
                }
            }
        }

        true
    }
}

/// Evaluates sentry conditions
pub struct SentryEvaluator;

impl SentryEvaluator {
    /// Evaluate if a sentry should fire
    pub fn evaluate_sentry(
        sentry: &Sentry,
        context: &SentryContext,
    ) -> bool {
        // Check on_parts (event-based conditions)
        let on_parts_satisfied = if sentry.on_parts.is_empty() {
            true
        } else {
            sentry.on_parts.iter().all(|on_part| {
                Self::evaluate_on_part(on_part, context)
            })
        };

        // Check if_part (condition expression)
        let if_part_satisfied = if let Some(condition) = &sentry.if_part {
            Self::evaluate_condition(condition, context)
        } else {
            true
        };

        on_parts_satisfied && if_part_satisfied
    }

    /// Evaluate a single on_part
    fn evaluate_on_part(on_part: &OnPart, context: &SentryContext) -> bool {
        // Find the referenced plan item
        let item = context.plan_items.values().find(|item| {
            item.plan_item_id == on_part.source_ref
        });

        if let Some(item) = item {
            match on_part.standard_event {
                StandardEvent::Create => item.state != PlanItemState::Available,
                StandardEvent::Enable => item.state == PlanItemState::Enabled,
                StandardEvent::Disable => item.state == PlanItemState::Available,
                StandardEvent::Start => item.state == PlanItemState::Active,
                StandardEvent::Complete => item.state == PlanItemState::Completed,
                StandardEvent::Terminate => item.state == PlanItemState::Terminated,
                StandardEvent::Suspend => item.state == PlanItemState::Suspended,
                StandardEvent::Resume => item.state == PlanItemState::Active,
            }
        } else {
            false
        }
    }

    /// Evaluate a condition expression (simplified implementation)
    fn evaluate_condition(condition: &str, context: &SentryContext) -> bool {
        // Simple condition evaluation - in production, would use proper expression language
        debug!("Evaluating condition: {}", condition);

        // Example conditions:
        // "${caseFile.priority} > 5" or "${planItem1.state} == COMPLETED"
        // This is a simplified parser - production would use proper expression language

        if condition.contains("caseFile") {
            // Extract variable and operator
            if let Some(start) = condition.find("${caseFile.") {
                let rest = &condition[start + 11..];
                if let Some(end) = rest.find("}") {
                    let var_name = &rest[..end];
                    if let Some(value) = context.case_file.get(var_name) {
                        // Simple comparison with number
                        if condition.contains("> 5") && value.is_number() {
                            if let Some(num) = value.as_f64() {
                                return num > 5.0;
                            }
                        }
                    }
                }
            }
        }

        // Default to true if condition can't be evaluated
        true
    }
}

/// Case runtime engine for executing CMMN cases
pub struct CaseRuntimeEngine {
    /// Active case instances
    instances: Arc<RwLock<HashMap<String, CaseInstance>>>,
    /// Case models
    case_models: Arc<RwLock<HashMap<String, CmmnCase>>>,
}

impl CaseRuntimeEngine {
    /// Create a new case runtime engine
    pub fn new() -> Self {
        Self {
            instances: Arc::new(RwLock::new(HashMap::new())),
            case_models: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a case model
    pub async fn register_case(&self, case_model: CmmnCase) -> Result<()> {
        case_model.validate()
            .map_err(|e| AbcdodafError::WorkflowError(e))?;
        let mut models = self.case_models.write().await;
        models.insert(case_model.id.clone(), case_model);
        info!("Registered case model");
        Ok(())
    }

    /// Start a new case instance
    pub async fn start_case(&self, case_model_id: &str) -> Result<String> {
        let models = self.case_models.read().await;
        let _case_model = models.get(case_model_id).ok_or_else(|| {
            AbcdodafError::WorkflowError(format!("Case model '{}' not found", case_model_id))
        })?;

        let instance = CaseInstance::new(case_model_id);
        let instance_id = instance.id.clone();

        let mut instances = self.instances.write().await;
        instances.insert(instance_id.clone(), instance);

        info!("Started case instance: {}", instance_id);
        Ok(instance_id)
    }

    /// Get a case instance
    pub async fn get_case_instance(&self, instance_id: &str) -> Result<CaseInstance> {
        let instances = self.instances.read().await;
        instances
            .get(instance_id)
            .cloned()
            .ok_or_else(|| {
                AbcdodafError::WorkflowError(format!("Case instance '{}' not found", instance_id))
            })
    }

    /// Create a plan item in a case instance
    pub async fn create_plan_item(
        &self,
        instance_id: &str,
        plan_item_id: String,
    ) -> Result<String> {
        let mut instances = self.instances.write().await;
        if let Some(instance) = instances.get_mut(instance_id) {
            let item_instance_id = instance.create_plan_item(plan_item_id, PlanItemState::Available);
            Ok(item_instance_id)
        } else {
            Err(AbcdodafError::WorkflowError(format!(
                "Case instance '{}' not found",
                instance_id
            )))
        }
    }

    /// Activate a plan item (enables it for execution)
    pub async fn activate_plan_item(
        &self,
        instance_id: &str,
        item_instance_id: &str,
    ) -> Result<()> {
        let mut instances = self.instances.write().await;
        if let Some(instance) = instances.get_mut(instance_id) {
            instance.change_plan_item_state(item_instance_id, PlanItemState::Enabled)?;
            Ok(())
        } else {
            Err(AbcdodafError::WorkflowError(format!(
                "Case instance '{}' not found",
                instance_id
            )))
        }
    }

    /// Start executing a plan item
    pub async fn start_plan_item(
        &self,
        instance_id: &str,
        item_instance_id: &str,
    ) -> Result<()> {
        let mut instances = self.instances.write().await;
        if let Some(instance) = instances.get_mut(instance_id) {
            instance.change_plan_item_state(item_instance_id, PlanItemState::Active)?;
            Ok(())
        } else {
            Err(AbcdodafError::WorkflowError(format!(
                "Case instance '{}' not found",
                instance_id
            )))
        }
    }

    /// Complete a plan item
    pub async fn complete_plan_item(
        &self,
        instance_id: &str,
        item_instance_id: &str,
    ) -> Result<()> {
        let mut instances = self.instances.write().await;
        if let Some(instance) = instances.get_mut(instance_id) {
            instance.change_plan_item_state(item_instance_id, PlanItemState::Completed)?;

            // Check if case should be completed
            let models = self.case_models.read().await;
            if let Some(case_model) = models.get(&instance.case_model_id) {
                if instance.check_completion(case_model) {
                    instance.state = CaseInstanceState::Completed;
                    instance.completed_at = Some(chrono::Utc::now());
                    info!("Case instance {} completed", instance_id);
                }
            }
            Ok(())
        } else {
            Err(AbcdodafError::WorkflowError(format!(
                "Case instance '{}' not found",
                instance_id
            )))
        }
    }

    /// Update case file item
    pub async fn update_case_file(
        &self,
        instance_id: &str,
        key: String,
        value: serde_json::Value,
    ) -> Result<()> {
        let mut instances = self.instances.write().await;
        if let Some(instance) = instances.get_mut(instance_id) {
            instance.update_case_file(key, value);
            Ok(())
        } else {
            Err(AbcdodafError::WorkflowError(format!(
                "Case instance '{}' not found",
                instance_id
            )))
        }
    }

    /// Evaluate sentries and return IDs of sentries that fired
    pub async fn evaluate_sentries(
        &self,
        instance_id: &str,
        case_model: &CmmnCase,
    ) -> Result<Vec<String>> {
        let instances = self.instances.read().await;
        if let Some(instance) = instances.get(instance_id) {
            let context = SentryContext {
                plan_items: &instance.plan_items,
                case_file: &instance.case_file,
                case_instance: instance,
            };

            let fired_sentries: Vec<String> = case_model
                .case_plan_model
                .sentries
                .iter()
                .filter(|sentry| SentryEvaluator::evaluate_sentry(sentry, &context))
                .map(|sentry| sentry.id.clone())
                .collect();

            Ok(fired_sentries)
        } else {
            Err(AbcdodafError::WorkflowError(format!(
                "Case instance '{}' not found",
                instance_id
            )))
        }
    }

    /// Terminate a case instance
    pub async fn terminate_case(&self, instance_id: &str) -> Result<()> {
        let mut instances = self.instances.write().await;
        if let Some(instance) = instances.get_mut(instance_id) {
            instance.state = CaseInstanceState::Terminated;
            instance.completed_at = Some(chrono::Utc::now());
            info!("Case instance {} terminated", instance_id);
            Ok(())
        } else {
            Err(AbcdodafError::WorkflowError(format!(
                "Case instance '{}' not found",
                instance_id
            )))
        }
    }
}

impl Default for CaseRuntimeEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_item_instance_creation() {
        let mut case_instance = CaseInstance::new("case1");
        let item_id = case_instance.create_plan_item("task1".to_string(), PlanItemState::Available);

        assert!(!item_id.is_empty());
        assert_eq!(case_instance.plan_items.len(), 1);

        let item = case_instance.get_plan_item(&item_id).unwrap();
        assert_eq!(item.state, PlanItemState::Available);
    }

    #[test]
    fn test_case_file_update() {
        let mut case_instance = CaseInstance::new("case1");
        case_instance.update_case_file("priority".to_string(), serde_json::json!(5));

        assert_eq!(case_instance.case_file.get("priority").unwrap(), &serde_json::json!(5));
        assert_eq!(case_instance.event_history.len(), 1);
    }

    #[test]
    fn test_plan_item_state_transition() {
        let mut case_instance = CaseInstance::new("case1");
        let item_id = case_instance.create_plan_item("task1".to_string(), PlanItemState::Available);

        case_instance.change_plan_item_state(&item_id, PlanItemState::Enabled).unwrap();
        assert_eq!(case_instance.get_plan_item(&item_id).unwrap().state, PlanItemState::Enabled);

        case_instance.change_plan_item_state(&item_id, PlanItemState::Active).unwrap();
        assert!(case_instance.get_plan_item(&item_id).unwrap().started_at.is_some());
    }

    #[test]
    fn test_sentry_evaluation_on_parts() {
        let mut case = CaseInstance::new("case1");
        let item_id = case.create_plan_item("task1".to_string(), PlanItemState::Available);
        case.change_plan_item_state(&item_id, PlanItemState::Completed).unwrap();

        let sentry = Sentry {
            id: "s1".to_string(),
            name: "Task Complete Sentry".to_string(),
            on_parts: vec![OnPart {
                source_ref: "task1".to_string(),
                standard_event: StandardEvent::Complete,
            }],
            if_part: None,
        };

        let context = SentryContext {
            plan_items: &case.plan_items,
            case_file: &case.case_file,
            case_instance: &case,
        };

        assert!(SentryEvaluator::evaluate_sentry(&sentry, &context));
    }

    #[tokio::test]
    async fn test_case_runtime_engine() {
        let engine = CaseRuntimeEngine::new();

        let case_model = CmmnCase::new("case1", "Test Case");
        engine.register_case(case_model).await.unwrap();

        let instance_id = engine.start_case("case1").await.unwrap();
        let instance = engine.get_case_instance(&instance_id).await.unwrap();

        assert_eq!(instance.case_model_id, "case1");
        assert_eq!(instance.state, CaseInstanceState::Active);
    }

    #[tokio::test]
    async fn test_plan_item_lifecycle() {
        let engine = CaseRuntimeEngine::new();
        let case_model = CmmnCase::new("case1", "Test Case");
        engine.register_case(case_model).await.unwrap();

        let instance_id = engine.start_case("case1").await.unwrap();
        let item_instance_id = engine.create_plan_item(&instance_id, "task1".to_string()).await.unwrap();

        engine.activate_plan_item(&instance_id, &item_instance_id).await.unwrap();
        let instance = engine.get_case_instance(&instance_id).await.unwrap();
        assert_eq!(instance.get_plan_item(&item_instance_id).unwrap().state, PlanItemState::Enabled);

        engine.start_plan_item(&instance_id, &item_instance_id).await.unwrap();
        let instance = engine.get_case_instance(&instance_id).await.unwrap();
        assert_eq!(instance.get_plan_item(&item_instance_id).unwrap().state, PlanItemState::Active);
    }
}
