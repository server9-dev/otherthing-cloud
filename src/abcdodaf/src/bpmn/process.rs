//! BPMN process definition and builder

use crate::error::{AbcdodafError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// BPMN process definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Process {
    /// Process ID
    pub id: String,
    /// Process name
    pub name: String,
    /// Process description
    pub description: Option<String>,
    /// Process tasks
    pub tasks: Vec<Task>,
    /// Process flows (connections between tasks)
    pub flows: Vec<SequenceFlow>,
    /// Process gateways
    pub gateways: Vec<Gateway>,
    /// Process metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// A task in the process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Task ID
    pub id: String,
    /// Task name
    pub name: String,
    /// Task type
    pub task_type: TaskType,
    /// Task properties
    pub properties: HashMap<String, serde_json::Value>,
}

/// Type of task
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    /// User task (human interaction required)
    User,
    /// Service task (automated service call)
    Service,
    /// Script task (execute code)
    Script,
    /// Manual task (non-system work)
    Manual,
    /// Send task (message sending)
    Send,
    /// Receive task (message receiving)
    Receive,
}

/// Sequence flow connecting tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceFlow {
    /// Flow ID
    pub id: String,
    /// Source task/gateway ID
    pub source: String,
    /// Target task/gateway ID
    pub target: String,
    /// Condition expression (optional)
    pub condition: Option<String>,
}

/// Gateway for process flow control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gateway {
    /// Gateway ID
    pub id: String,
    /// Gateway name
    pub name: String,
    /// Gateway type
    pub gateway_type: GatewayType,
}

/// Type of gateway
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GatewayType {
    /// Exclusive (XOR) - one path
    Exclusive,
    /// Parallel (AND) - all paths
    Parallel,
    /// Inclusive (OR) - one or more paths
    Inclusive,
    /// Event-based - based on events
    EventBased,
}

/// Builder for creating BPMN processes
pub struct ProcessBuilder {
    process: Process,
}

impl ProcessBuilder {
    /// Create a new process builder
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            process: Process {
                id: id.into(),
                name: name.into(),
                description: None,
                tasks: Vec::new(),
                flows: Vec::new(),
                gateways: Vec::new(),
                metadata: HashMap::new(),
            },
        }
    }

    /// Set the process description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.process.description = Some(desc.into());
        self
    }

    /// Add a task to the process
    pub fn add_task(mut self, task: Task) -> Self {
        self.process.tasks.push(task);
        self
    }

    /// Add a user task
    pub fn add_user_task(self, id: impl Into<String>, name: impl Into<String>) -> Self {
        let task = Task {
            id: id.into(),
            name: name.into(),
            task_type: TaskType::User,
            properties: HashMap::new(),
        };
        self.add_task(task)
    }

    /// Add a service task
    pub fn add_service_task(self, id: impl Into<String>, name: impl Into<String>) -> Self {
        let task = Task {
            id: id.into(),
            name: name.into(),
            task_type: TaskType::Service,
            properties: HashMap::new(),
        };
        self.add_task(task)
    }

    /// Add a script task
    pub fn add_script_task(self, id: impl Into<String>, name: impl Into<String>) -> Self {
        let task = Task {
            id: id.into(),
            name: name.into(),
            task_type: TaskType::Script,
            properties: HashMap::new(),
        };
        self.add_task(task)
    }

    /// Add a sequence flow
    pub fn add_flow(
        mut self,
        id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        let flow = SequenceFlow {
            id: id.into(),
            source: source.into(),
            target: target.into(),
            condition: None,
        };
        self.process.flows.push(flow);
        self
    }

    /// Add a conditional flow
    pub fn add_conditional_flow(
        mut self,
        id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
        condition: impl Into<String>,
    ) -> Self {
        let flow = SequenceFlow {
            id: id.into(),
            source: source.into(),
            target: target.into(),
            condition: Some(condition.into()),
        };
        self.process.flows.push(flow);
        self
    }

    /// Add a gateway
    pub fn add_gateway(
        mut self,
        id: impl Into<String>,
        name: impl Into<String>,
        gateway_type: GatewayType,
    ) -> Self {
        let gateway = Gateway { id: id.into(), name: name.into(), gateway_type };
        self.process.gateways.push(gateway);
        self
    }

    /// Set metadata
    pub fn metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.process.metadata.insert(key.into(), value);
        self
    }

    /// Build the process
    pub fn build(self) -> Result<Process> {
        // Validate the process
        if self.process.tasks.is_empty() {
            return Err(AbcdodafError::BpmnError("Process must have at least one task".into()));
        }

        Ok(self.process)
    }
}

impl Process {
    /// Get a task by ID
    pub fn get_task(&self, id: &str) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == id)
    }

    /// Get a gateway by ID
    pub fn get_gateway(&self, id: &str) -> Option<&Gateway> {
        self.gateways.iter().find(|g| g.id == id)
    }

    /// Get flows from a specific source
    pub fn get_flows_from(&self, source: &str) -> Vec<&SequenceFlow> {
        self.flows.iter().filter(|f| f.source == source).collect()
    }

    /// Export to BPMN XML (simplified representation)
    pub fn to_xml(&self) -> Result<String> {
        // This is a simplified XML export
        // In production, use bpxe-bpmn-schema for proper BPMN 2.0 XML
        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push_str("\n<definitions xmlns=\"http://www.omg.org/spec/BPMN/20100524/MODEL\">");
        xml.push_str(&format!("\n  <process id=\"{}\" name=\"{}\">", self.id, self.name));

        for task in &self.tasks {
            xml.push_str(&format!(
                "\n    <{} id=\"{}\" name=\"{}\" />",
                match task.task_type {
                    TaskType::User => "userTask",
                    TaskType::Service => "serviceTask",
                    TaskType::Script => "scriptTask",
                    TaskType::Manual => "manualTask",
                    TaskType::Send => "sendTask",
                    TaskType::Receive => "receiveTask",
                },
                task.id,
                task.name
            ));
        }

        for flow in &self.flows {
            xml.push_str(&format!(
                "\n    <sequenceFlow id=\"{}\" sourceRef=\"{}\" targetRef=\"{}\" />",
                flow.id, flow.source, flow.target
            ));
        }

        xml.push_str("\n  </process>");
        xml.push_str("\n</definitions>");

        Ok(xml)
    }

    /// Export to JSON
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_builder() {
        let process = ProcessBuilder::new("test_process", "Test Process")
            .description("A test process")
            .add_user_task("task1", "User Task 1")
            .add_service_task("task2", "Service Task 1")
            .add_flow("flow1", "task1", "task2")
            .build()
            .unwrap();

        assert_eq!(process.id, "test_process");
        assert_eq!(process.name, "Test Process");
        assert_eq!(process.tasks.len(), 2);
        assert_eq!(process.flows.len(), 1);
    }

    #[test]
    fn test_process_validation() {
        let result = ProcessBuilder::new("empty", "Empty Process").build();
        assert!(result.is_err());
    }

    #[test]
    fn test_xml_export() {
        let process = ProcessBuilder::new("test", "Test")
            .add_user_task("t1", "Task 1")
            .build()
            .unwrap();

        let xml = process.to_xml().unwrap();
        assert!(xml.contains("userTask"));
        assert!(xml.contains("t1"));
    }
}
