//! BPMN JSON Format Structures
//!
//! Defines the JSON schema for BPMN workflow files.
//! This is UI-independent and can be used in both CLI and GUI contexts.

use serde::{Deserialize, Serialize};

/// BPMN JSON workflow format (input format from workflow files)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BpmnJsonWorkflow {
    pub bpmn_process: BpmnProcessInfo,
    #[serde(default)]
    pub workflow_steps: Vec<WorkflowStep>,
    #[serde(default)]
    pub sequence_flows: Vec<SequenceFlow>,
    #[serde(default)]
    pub dodaf_metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub task_metadata: Option<serde_json::Value>,
}

/// BPMN process information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BpmnProcessInfo {
    pub id: String,
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_executable")]
    #[serde(rename = "isExecutable")]
    pub is_executable: bool,
    #[serde(default)]
    #[serde(rename = "processType")]
    pub process_type: String,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

fn default_executable() -> bool {
    true
}

/// Workflow step (BPMN element)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub step_type: String,
    #[serde(default)]
    #[serde(rename = "taskType")]
    pub task_type: Option<String>,
    #[serde(default)]
    #[serde(rename = "eventType")]
    pub event_type: Option<String>,
    #[serde(default)]
    pub implementation: Option<String>,
    #[serde(default)]
    pub inputs: Vec<String>,
    #[serde(default)]
    pub outputs: Vec<String>,
    #[serde(default)]
    pub duration_minutes: Option<i32>,
}

/// Sequence flow (connection between nodes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceFlow {
    pub id: String,
    #[serde(rename = "sourceRef")]
    pub source_ref: String,
    #[serde(rename = "targetRef")]
    pub target_ref: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub condition: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bpmn_json() {
        let json = r#"{
            "bpmn_process": {
                "id": "test_process",
                "name": "Test Process",
                "version": "1.0.0",
                "isExecutable": true,
                "processType": "agent_workflow"
            },
            "workflow_steps": [
                {
                    "id": "start_event",
                    "name": "Start",
                    "type": "startEvent"
                },
                {
                    "id": "task1",
                    "name": "Task 1",
                    "type": "serviceTask",
                    "taskType": "research"
                },
                {
                    "id": "end_event",
                    "name": "End",
                    "type": "endEvent"
                }
            ],
            "sequence_flows": [
                {
                    "id": "flow1",
                    "sourceRef": "start_event",
                    "targetRef": "task1"
                },
                {
                    "id": "flow2",
                    "sourceRef": "task1",
                    "targetRef": "end_event"
                }
            ]
        }"#;

        let result: Result<BpmnJsonWorkflow, _> = serde_json::from_str(json);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let workflow = result.unwrap();
        assert_eq!(workflow.bpmn_process.id, "test_process");
        assert_eq!(workflow.bpmn_process.name, "Test Process");
        assert_eq!(workflow.workflow_steps.len(), 3);
        assert_eq!(workflow.sequence_flows.len(), 2);
    }

    #[test]
    fn test_default_values() {
        let json = r#"{
            "bpmn_process": {
                "id": "test",
                "name": "Test"
            }
        }"#;

        let result: Result<BpmnJsonWorkflow, _> = serde_json::from_str(json);
        assert!(result.is_ok());

        let workflow = result.unwrap();
        assert_eq!(workflow.bpmn_process.version, "1.0.0");
        assert_eq!(workflow.bpmn_process.is_executable, true);
        assert_eq!(workflow.workflow_steps.len(), 0);
        assert_eq!(workflow.sequence_flows.len(), 0);
    }

    #[test]
    fn test_optional_fields() {
        let json = r#"{
            "bpmn_process": {
                "id": "test",
                "name": "Test"
            },
            "workflow_steps": [
                {
                    "id": "task1",
                    "name": "Task",
                    "type": "serviceTask",
                    "taskType": "research",
                    "implementation": "rust_function",
                    "inputs": ["input1", "input2"],
                    "outputs": ["output1"],
                    "duration_minutes": 30
                }
            ],
            "sequence_flows": []
        }"#;

        let result: Result<BpmnJsonWorkflow, _> = serde_json::from_str(json);
        assert!(result.is_ok());

        let workflow = result.unwrap();
        let task = &workflow.workflow_steps[0];
        assert_eq!(task.task_type, Some("research".to_string()));
        assert_eq!(task.implementation, Some("rust_function".to_string()));
        assert_eq!(task.inputs.len(), 2);
        assert_eq!(task.outputs.len(), 1);
        assert_eq!(task.duration_minutes, Some(30));
    }
}
