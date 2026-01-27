//! Integration between BPMN and CMMN
//!
//! This module provides mechanisms to:
//! - Call CMMN cases from BPMN processes (Call Activity)
//! - Embed BPMN processes within CMMN plan items (Process Task)
//! - Share case file data with process variables
//! - Handle completion and error scenarios across boundaries

use super::cmmn_runtime::*;
use crate::bpmn::{Process, ProcessInstance};
use crate::error::{AbcdodafError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Call relationship from BPMN process to CMMN case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseCallActivity {
    pub id: String,
    pub name: String,
    /// Reference to the CMMN case to call
    pub case_ref: String,
    /// Input variable mappings: (case_file_item -> process_variable)
    pub input_mappings: HashMap<String, String>,
    /// Output variable mappings: (process_variable -> case_file_item)
    pub output_mappings: HashMap<String, String>,
    /// Whether to wait for case completion
    pub wait_for_completion: bool,
}

/// Represents an instance of a case called from BPMN
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseCallInstance {
    pub id: String,
    pub call_activity_id: String,
    pub process_instance_id: uuid::Uuid,
    pub case_instance_id: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub status: CallStatus,
    pub error: Option<String>,
}

/// Status of a case call
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CallStatus {
    /// Case has been started
    Started,
    /// Case is executing
    Running,
    /// Case has completed successfully
    Completed,
    /// Case was terminated early
    Terminated,
    /// Case encountered an error
    Failed,
}

/// BPMN-CMMN integration engine
pub struct BpmnCmmnIntegration {
    /// Case runtime engine
    case_engine: CaseRuntimeEngine,
    /// Active case calls
    case_calls: HashMap<String, CaseCallInstance>,
}

impl BpmnCmmnIntegration {
    /// Create a new BPMN-CMMN integration engine
    pub fn new(case_engine: CaseRuntimeEngine) -> Self {
        Self {
            case_engine,
            case_calls: HashMap::new(),
        }
    }

    /// Execute a BPMN-to-CMMN call activity
    pub async fn execute_case_call(
        &mut self,
        call_activity: &CaseCallActivity,
        process_instance: &ProcessInstance,
    ) -> Result<CaseCallInstance> {
        debug!("Executing case call activity: {}", call_activity.name);

        // Start a new case instance
        let case_instance_id = self.case_engine
            .start_case(&call_activity.case_ref)
            .await?;

        // Map input variables from process to case file
        for (case_item, process_var) in &call_activity.input_mappings {
            if let Some(value) = process_instance.get_variable(process_var) {
                self.case_engine
                    .update_case_file(&case_instance_id, case_item.clone(), value.clone())
                    .await?;
            }
        }

        let call_instance = CaseCallInstance {
            id: uuid::Uuid::new_v4().to_string(),
            call_activity_id: call_activity.id.clone(),
            process_instance_id: process_instance.id,
            case_instance_id: case_instance_id.clone(),
            started_at: chrono::Utc::now(),
            completed_at: None,
            status: CallStatus::Running,
            error: None,
        };

        let call_id = call_instance.id.clone();
        self.case_calls.insert(call_id.clone(), call_instance);

        info!("Started case call from BPMN: {}", call_id);
        Ok(self.case_calls[&call_id].clone())
    }

    /// Check and update case call status
    pub async fn check_case_call_status(
        &mut self,
        call_id: &str,
        _call_activity: &CaseCallActivity,
    ) -> Result<CaseCallInstance> {
        let call = self.case_calls.get(call_id)
            .ok_or_else(|| AbcdodafError::WorkflowError(format!("Call {} not found", call_id)))?
            .clone();

        let case_instance = self.case_engine
            .get_case_instance(&call.case_instance_id)
            .await?;

        let mut updated_call = call.clone();

        // Update status based on case state
        match case_instance.state {
            CaseInstanceState::Active | CaseInstanceState::Suspended => {
                updated_call.status = CallStatus::Running;
            }
            CaseInstanceState::Completed => {
                updated_call.status = CallStatus::Completed;
                updated_call.completed_at = Some(chrono::Utc::now());
            }
            CaseInstanceState::Terminated => {
                updated_call.status = CallStatus::Terminated;
                updated_call.completed_at = Some(chrono::Utc::now());
            }
        }

        self.case_calls.insert(call_id.to_string(), updated_call.clone());
        Ok(updated_call)
    }

    /// Get output variables from completed case call
    pub async fn get_case_call_output(
        &self,
        call_id: &str,
        call_activity: &CaseCallActivity,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let call = self.case_calls.get(call_id)
            .ok_or_else(|| AbcdodafError::WorkflowError(format!("Call {} not found", call_id)))?;

        if call.status != CallStatus::Completed {
            return Err(AbcdodafError::WorkflowError(
                format!("Case call {} is not completed", call_id)
            ));
        }

        let case_instance = self.case_engine
            .get_case_instance(&call.case_instance_id)
            .await?;

        let mut output_variables = HashMap::new();

        // Map output variables from case file to process variables
        for (process_var, case_item) in &call_activity.output_mappings {
            if let Some(value) = case_instance.case_file.get(case_item) {
                output_variables.insert(process_var.clone(), value.clone());
            }
        }

        Ok(output_variables)
    }

    /// Create a process task that calls BPMN from CMMN
    pub async fn create_process_task_call(
        &self,
        case_instance_id: &str,
        plan_item_id: String,
        process: &Process,
    ) -> Result<String> {
        // Create a plan item instance for the process task
        let item_instance_id = self.case_engine
            .create_plan_item(case_instance_id, plan_item_id)
            .await?;

        // In production, would execute the BPMN process here
        debug!("Created process task call for BPMN process: {}", process.id);

        Ok(item_instance_id)
    }

    /// Get case call instance
    pub fn get_case_call(&self, call_id: &str) -> Option<&CaseCallInstance> {
        self.case_calls.get(call_id)
    }

    /// Get all active case calls
    pub fn get_active_case_calls(&self) -> Vec<&CaseCallInstance> {
        self.case_calls
            .values()
            .filter(|call| call.status == CallStatus::Running)
            .collect()
    }

    /// Terminate a case call
    pub async fn terminate_case_call(&mut self, call_id: &str) -> Result<()> {
        if let Some(call) = self.case_calls.get_mut(call_id) {
            self.case_engine
                .terminate_case(&call.case_instance_id)
                .await?;
            call.status = CallStatus::Terminated;
            call.completed_at = Some(chrono::Utc::now());
            Ok(())
        } else {
            Err(AbcdodafError::WorkflowError(format!("Call {} not found", call_id)))
        }
    }
}

/// Variable mapper for BPMN-CMMN data transformation
pub struct BpmnCmmnVariableMapper;

impl BpmnCmmnVariableMapper {
    /// Map BPMN process variables to CMMN case file
    pub fn map_process_to_case(
        process_variables: &HashMap<String, serde_json::Value>,
        mappings: &HashMap<String, String>,
    ) -> HashMap<String, serde_json::Value> {
        let mut case_file = HashMap::new();

        for (case_item, process_var) in mappings {
            if let Some(value) = process_variables.get(process_var) {
                case_file.insert(case_item.clone(), value.clone());
            }
        }

        case_file
    }

    /// Map CMMN case file to BPMN process variables
    pub fn map_case_to_process(
        case_file: &HashMap<String, serde_json::Value>,
        mappings: &HashMap<String, String>,
    ) -> HashMap<String, serde_json::Value> {
        let mut process_variables = HashMap::new();

        for (process_var, case_item) in mappings {
            if let Some(value) = case_file.get(case_item) {
                process_variables.insert(process_var.clone(), value.clone());
            }
        }

        process_variables
    }

    /// Validate variable mappings exist in source
    pub fn validate_mappings(
        source_vars: &HashMap<String, serde_json::Value>,
        mappings: &HashMap<String, String>,
    ) -> Result<()> {
        for (_target, source_var) in mappings {
            if !source_vars.contains_key(source_var) {
                warn!("Source variable {} not found in mappings", source_var);
            }
        }
        Ok(())
    }
}

/// Builder for case call activities
pub struct CaseCallActivityBuilder {
    id: String,
    name: String,
    case_ref: String,
    input_mappings: HashMap<String, String>,
    output_mappings: HashMap<String, String>,
    wait_for_completion: bool,
}

impl CaseCallActivityBuilder {
    /// Create a new case call activity builder
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        case_ref: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            case_ref: case_ref.into(),
            input_mappings: HashMap::new(),
            output_mappings: HashMap::new(),
            wait_for_completion: true,
        }
    }

    /// Add input mapping
    pub fn add_input_mapping(mut self, case_item: impl Into<String>, process_var: impl Into<String>) -> Self {
        self.input_mappings.insert(case_item.into(), process_var.into());
        self
    }

    /// Add output mapping
    pub fn add_output_mapping(mut self, process_var: impl Into<String>, case_item: impl Into<String>) -> Self {
        self.output_mappings.insert(process_var.into(), case_item.into());
        self
    }

    /// Set whether to wait for case completion
    pub fn wait_for_completion(mut self, wait: bool) -> Self {
        self.wait_for_completion = wait;
        self
    }

    /// Build the case call activity
    pub fn build(self) -> CaseCallActivity {
        CaseCallActivity {
            id: self.id,
            name: self.name,
            case_ref: self.case_ref,
            input_mappings: self.input_mappings,
            output_mappings: self.output_mappings,
            wait_for_completion: self.wait_for_completion,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_call_activity_builder() {
        let activity = CaseCallActivityBuilder::new("call1", "Execute Case", "case1")
            .add_input_mapping("priority", "request_priority")
            .add_output_mapping("result", "case_result")
            .wait_for_completion(true)
            .build();

        assert_eq!(activity.id, "call1");
        assert_eq!(activity.case_ref, "case1");
        assert_eq!(activity.input_mappings.len(), 1);
        assert_eq!(activity.output_mappings.len(), 1);
    }

    #[test]
    fn test_variable_mapping_process_to_case() {
        let mut process_vars = HashMap::new();
        process_vars.insert("priority".to_string(), serde_json::json!(5));
        process_vars.insert("status".to_string(), serde_json::json!("pending"));

        let mut mappings = HashMap::new();
        mappings.insert("case_priority".to_string(), "priority".to_string());
        mappings.insert("case_status".to_string(), "status".to_string());

        let case_file = BpmnCmmnVariableMapper::map_process_to_case(&process_vars, &mappings);

        assert_eq!(case_file.len(), 2);
        assert_eq!(case_file.get("case_priority").unwrap(), &serde_json::json!(5));
    }

    #[test]
    fn test_variable_mapping_case_to_process() {
        let mut case_file = HashMap::new();
        case_file.insert("case_result".to_string(), serde_json::json!("success"));

        let mut mappings = HashMap::new();
        mappings.insert("result".to_string(), "case_result".to_string());

        let process_vars = BpmnCmmnVariableMapper::map_case_to_process(&case_file, &mappings);

        assert_eq!(process_vars.len(), 1);
        assert_eq!(process_vars.get("result").unwrap(), &serde_json::json!("success"));
    }
}
