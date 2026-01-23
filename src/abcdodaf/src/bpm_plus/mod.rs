//! # BPM+ Triple Threat Implementation
//!
//! This module implements the three OMG (Object Management Group) standards that form the BPM+ methodology:
//!
//! 1. **BPMN** (Business Process Model and Notation) - Process flows and orchestration
//! 2. **CMMN** (Case Management Model and Notation) - Knowledge work and adaptive cases
//! 3. **DMN** (Decision Model and Notation) - Business decision logic
//!
//! These three standards work together to provide comprehensive business modeling:
//! - BPMN handles structured, repeatable processes
//! - CMMN handles unstructured, knowledge-intensive work
//! - DMN provides reusable decision logic for both

pub mod bpmn;
pub mod cmmn;
pub mod dmn;
pub mod integration;

pub use bpmn::*;
pub use cmmn::*;
pub use dmn::*;
pub use integration::*;

use serde::{Deserialize, Serialize};

/// The BPM+ Triple Threat - combines all three OMG standards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BpmPlusModel {
    /// BPMN process models
    pub processes: Vec<bpmn::BpmnProcess>,

    /// CMMN case models
    pub cases: Vec<cmmn::CmmnCase>,

    /// DMN decision models
    pub decisions: Vec<dmn::DmnDecision>,

    /// Cross-standard integrations
    pub integrations: Vec<integration::StandardIntegration>,

    /// Metadata
    pub metadata: ModelMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: Option<String>,
    pub created: chrono::DateTime<chrono::Utc>,
    pub modified: chrono::DateTime<chrono::Utc>,
}

impl BpmPlusModel {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            processes: Vec::new(),
            cases: Vec::new(),
            decisions: Vec::new(),
            integrations: Vec::new(),
            metadata: ModelMetadata {
                name: name.into(),
                version: "1.0.0".to_string(),
                description: description.into(),
                author: None,
                created: now,
                modified: now,
            },
        }
    }

    /// Add a BPMN process to the model
    pub fn add_process(&mut self, process: bpmn::BpmnProcess) {
        self.processes.push(process);
        self.metadata.modified = chrono::Utc::now();
    }

    /// Add a CMMN case to the model
    pub fn add_case(&mut self, case: cmmn::CmmnCase) {
        self.cases.push(case);
        self.metadata.modified = chrono::Utc::now();
    }

    /// Add a DMN decision to the model
    pub fn add_decision(&mut self, decision: dmn::DmnDecision) {
        self.decisions.push(decision);
        self.metadata.modified = chrono::Utc::now();
    }

    /// Link a decision to a process task
    pub fn link_decision_to_process(&mut self, decision_id: &str, process_id: &str, task_id: &str) {
        self.integrations.push(integration::StandardIntegration {
            id: format!("{}_{}", decision_id, task_id),
            source: integration::IntegrationSource::Decision(decision_id.to_string()),
            target: integration::IntegrationTarget::ProcessTask {
                process_id: process_id.to_string(),
                task_id: task_id.to_string(),
            },
            integration_type: integration::IntegrationType::DecisionTask,
        });
        self.metadata.modified = chrono::Utc::now();
    }

    /// Link a decision to a case
    pub fn link_decision_to_case(&mut self, decision_id: &str, case_id: &str, item_id: &str) {
        self.integrations.push(integration::StandardIntegration {
            id: format!("{}_{}", decision_id, item_id),
            source: integration::IntegrationSource::Decision(decision_id.to_string()),
            target: integration::IntegrationTarget::CasePlanItem {
                case_id: case_id.to_string(),
                item_id: item_id.to_string(),
            },
            integration_type: integration::IntegrationType::DecisionTask,
        });
        self.metadata.modified = chrono::Utc::now();
    }

    /// Validate the entire model
    pub fn validate(&self) -> Result<(), String> {
        // Validate all processes
        for process in &self.processes {
            process.validate()?;
        }

        // Validate all cases
        for case in &self.cases {
            case.validate()?;
        }

        // Validate all decisions
        for decision in &self.decisions {
            decision.validate()?;
        }

        // Validate integrations
        for integration in &self.integrations {
            integration.validate(&self)?;
        }

        Ok(())
    }

    /// Export to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Export to YAML
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bpm_plus_model_creation() {
        let model = BpmPlusModel::new("Test Model", "A test BPM+ model");
        assert_eq!(model.metadata.name, "Test Model");
        assert_eq!(model.processes.len(), 0);
        assert_eq!(model.cases.len(), 0);
        assert_eq!(model.decisions.len(), 0);
    }
}
