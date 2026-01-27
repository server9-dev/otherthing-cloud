//! Integration Layer for BPM+ Triple Threat
//!
//! This module provides the glue that connects BPMN, CMMN, and DMN together.

use super::{bpmn::*, cmmn::*, dmn::*, BpmPlusModel};
use serde::{Deserialize, Serialize};

/// Cross-standard integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardIntegration {
    pub id: String,
    pub source: IntegrationSource,
    pub target: IntegrationTarget,
    pub integration_type: IntegrationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IntegrationSource {
    /// BPMN Process
    Process(String),
    /// CMMN Case
    Case(String),
    /// DMN Decision
    Decision(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IntegrationTarget {
    /// BPMN Task within a process
    ProcessTask { process_id: String, task_id: String },
    /// CMMN Plan Item within a case
    CasePlanItem { case_id: String, item_id: String },
    /// Another decision (for decision chaining)
    DecisionInput { decision_id: String, input_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntegrationType {
    /// DMN decision called from BPMN Business Rule Task
    DecisionTask,
    /// BPMN process called from CMMN Process Task
    ProcessTask,
    /// CMMN case called from CMMN Case Task
    CaseTask,
    /// CMMN case called from BPMN Call Activity
    CallActivity,
    /// Decision output feeds another decision input
    DecisionChain,
}

impl StandardIntegration {
    pub fn validate(&self, model: &BpmPlusModel) -> Result<(), String> {
        // Validate source exists
        match &self.source {
            IntegrationSource::Process(id) => {
                if !model.processes.iter().any(|p| &p.id == id) {
                    return Err(format!("Source process '{}' not found in model", id));
                }
            },
            IntegrationSource::Case(id) => {
                if !model.cases.iter().any(|c| &c.id == id) {
                    return Err(format!("Source case '{}' not found in model", id));
                }
            },
            IntegrationSource::Decision(id) => {
                if !model.decisions.iter().any(|d| &d.id == id) {
                    return Err(format!("Source decision '{}' not found in model", id));
                }
            },
        }

        // Validate target exists
        match &self.target {
            IntegrationTarget::ProcessTask { process_id, task_id } => {
                if let Some(process) = model.processes.iter().find(|p| &p.id == process_id) {
                    if !process.has_element(task_id) {
                        return Err(format!(
                            "Task '{}' not found in process '{}'",
                            task_id, process_id
                        ));
                    }
                } else {
                    return Err(format!("Target process '{}' not found", process_id));
                }
            },
            IntegrationTarget::CasePlanItem { case_id, item_id } => {
                if let Some(case) = model.cases.iter().find(|c| &c.id == case_id) {
                    if !case.has_plan_item(item_id) {
                        return Err(format!(
                            "Plan item '{}' not found in case '{}'",
                            item_id, case_id
                        ));
                    }
                } else {
                    return Err(format!("Target case '{}' not found", case_id));
                }
            },
            IntegrationTarget::DecisionInput { decision_id, .. } => {
                if !model.decisions.iter().any(|d| &d.id == decision_id) {
                    return Err(format!("Target decision '{}' not found", decision_id));
                }
            },
        }

        Ok(())
    }
}

/// Builder for creating integrated BPM+ models
pub struct BpmPlusBuilder {
    model: BpmPlusModel,
}

impl BpmPlusBuilder {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self { model: BpmPlusModel::new(name, description) }
    }

    pub fn add_process(mut self, process: BpmnProcess) -> Self {
        self.model.add_process(process);
        self
    }

    pub fn add_case(mut self, case: CmmnCase) -> Self {
        self.model.add_case(case);
        self
    }

    pub fn add_decision(mut self, decision: DmnDecision) -> Self {
        self.model.add_decision(decision);
        self
    }

    /// Link a DMN decision to a BPMN Business Rule Task
    pub fn link_decision_to_process(
        mut self,
        decision_id: impl Into<String>,
        process_id: impl Into<String>,
        task_id: impl Into<String>,
    ) -> Self {
        self.model.link_decision_to_process(
            &decision_id.into(),
            &process_id.into(),
            &task_id.into(),
        );
        self
    }

    /// Link a DMN decision to a CMMN Decision Task
    pub fn link_decision_to_case(
        mut self,
        decision_id: impl Into<String>,
        case_id: impl Into<String>,
        item_id: impl Into<String>,
    ) -> Self {
        self.model
            .link_decision_to_case(&decision_id.into(), &case_id.into(), &item_id.into());
        self
    }

    /// Link a BPMN process to a CMMN Process Task
    pub fn link_process_to_case(
        mut self,
        process_id: impl Into<String>,
        case_id: impl Into<String>,
        item_id: impl Into<String>,
    ) -> Self {
        let process_id = process_id.into();
        let case_id = case_id.into();
        let item_id = item_id.into();
        self.model.integrations.push(StandardIntegration {
            id: format!("{}_{}", process_id, item_id),
            source: IntegrationSource::Process(process_id),
            target: IntegrationTarget::CasePlanItem { case_id, item_id },
            integration_type: IntegrationType::ProcessTask,
        });
        self
    }

    pub fn build(self) -> Result<BpmPlusModel, String> {
        self.model.validate()?;
        Ok(self.model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_validation() {
        let mut model = BpmPlusModel::new("Test Model", "Test");

        // Add a process
        let mut process = BpmnProcess::new("proc1", "Test Process");
        process.add_flow_element(FlowElement::StartEvent {
            id: "start".to_string(),
            name: "Start".to_string(),
            outgoing: vec!["flow1".to_string()],
        });
        process.add_flow_element(FlowElement::BusinessRuleTask {
            id: "task1".to_string(),
            name: "Decision Task".to_string(),
            decision_ref: Some("decision1".to_string()),
            incoming: vec!["flow1".to_string()],
            outgoing: vec!["flow2".to_string()],
        });
        process.add_flow_element(FlowElement::EndEvent {
            id: "end".to_string(),
            name: "End".to_string(),
            incoming: vec!["flow2".to_string()],
        });
        process.add_flow_element(FlowElement::SequenceFlow {
            id: "flow1".to_string(),
            name: None,
            source_ref: "start".to_string(),
            target_ref: "task1".to_string(),
            condition: None,
        });
        process.add_flow_element(FlowElement::SequenceFlow {
            id: "flow2".to_string(),
            name: None,
            source_ref: "task1".to_string(),
            target_ref: "end".to_string(),
            condition: None,
        });
        model.add_process(process);

        // Add a decision
        let decision = DmnDecision::new("decision1", "Test Decision");
        model.add_decision(decision);

        // Link them
        model.link_decision_to_process("decision1", "proc1", "task1");

        // Validate
        assert!(model.validate().is_ok());
    }

    #[test]
    fn test_builder_pattern() {
        let mut process = BpmnProcess::new("proc1", "Process");
        process.add_flow_element(FlowElement::StartEvent {
            id: "start".to_string(),
            name: "Start".to_string(),
            outgoing: vec![],
        });
        process.add_flow_element(FlowElement::EndEvent {
            id: "end".to_string(),
            name: "End".to_string(),
            incoming: vec![],
        });

        let decision = DmnDecision::new("decision1", "Decision");

        let model = BpmPlusBuilder::new("Test", "Test model")
            .add_process(process)
            .add_decision(decision)
            .build();

        assert!(model.is_ok());
    }
}
