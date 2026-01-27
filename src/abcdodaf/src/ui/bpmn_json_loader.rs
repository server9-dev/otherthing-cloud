//! BPMN JSON to Snarl Converter
//!
//! Converts BPMN workflow files (with workflow_steps and sequence_flows)
//! into Snarl format for use with the workspace editor.

use crate::bpmn::elements::{BpmnGatewayType, BpmnTaskType, GatewayDirection};
use crate::bpmn::json_format::{BpmnJsonWorkflow, WorkflowStep};
use crate::ui::enhanced_nodes::{BpmnNodeType, EnhancedBpmnNode, GatewayNode, TaskNode};
use egui::Pos2;
use egui_snarl::{InPinId, NodeId, OutPinId, Snarl};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{debug, error, info, warn};

/// Workspace file format (output format for workspace)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowFile {
    pub version: String,
    pub name: String,
    pub snarl: Snarl<EnhancedBpmnNode>,
}

/// Layout configuration for automatic positioning
#[derive(Debug, Clone)]
pub struct LayoutConfig {
    pub start_x: f32,
    pub start_y: f32,
    pub horizontal_spacing: f32,
    pub vertical_spacing: f32,
    pub gateway_offset_x: f32,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            start_x: 100.0,
            start_y: 100.0,
            horizontal_spacing: 200.0,
            vertical_spacing: 150.0,
            gateway_offset_x: 150.0,
        }
    }
}

/// BPMN JSON to Snarl converter
pub struct BpmnJsonConverter {
    layout_config: LayoutConfig,
}

impl BpmnJsonConverter {
    /// Create a new converter with default layout
    pub fn new() -> Self {
        Self { layout_config: LayoutConfig::default() }
    }

    /// Create a new converter with custom layout
    pub fn with_layout(layout_config: LayoutConfig) -> Self {
        Self { layout_config }
    }

    /// Load BPMN JSON from file and convert to WorkflowFile
    pub fn load_from_file<P: AsRef<Path>>(&self, path: P) -> Result<WorkflowFile, String> {
        debug!("Loading BPMN JSON from {:?}", path.as_ref());

        let content = fs::read_to_string(path.as_ref()).map_err(|e| {
            error!("Failed to read file {:?}: {}", path.as_ref(), e);
            format!("Failed to read file: {}", e)
        })?;

        self.load_from_string(&content)
    }

    /// Load BPMN JSON from string and convert to WorkflowFile
    pub fn load_from_string(&self, json_content: &str) -> Result<WorkflowFile, String> {
        debug!("Parsing BPMN JSON content");

        let bpmn_workflow: BpmnJsonWorkflow = serde_json::from_str(json_content).map_err(|e| {
            error!("Failed to parse BPMN JSON: {}", e);
            format!("Failed to parse JSON: {}", e)
        })?;

        self.convert(bpmn_workflow)
    }

    /// Convert BPMN JSON workflow to WorkflowFile with Snarl graph
    pub fn convert(&self, bpmn_workflow: BpmnJsonWorkflow) -> Result<WorkflowFile, String> {
        info!("Converting BPMN workflow: {}", bpmn_workflow.bpmn_process.name);

        let mut snarl = Snarl::new();
        let mut node_id_map: HashMap<String, NodeId> = HashMap::new();

        // Build dependency graph to determine layout levels
        let layout_positions = self.calculate_layout(&bpmn_workflow)?;

        // Step 1: Create all nodes
        for step in &bpmn_workflow.workflow_steps {
            debug!("Processing step: {} ({})", step.name, step.step_type);

            let node = self.create_node_from_step(step)?;
            let position = layout_positions.get(&step.id).copied().unwrap_or_else(|| {
                Pos2::new(self.layout_config.start_x, self.layout_config.start_y)
            });

            let node_id = snarl.insert_node(position, node);
            node_id_map.insert(step.id.clone(), node_id);

            debug!("Created node {} at position {:?}", step.id, position);
        }

        // Step 2: Create connections based on sequence flows
        for flow in &bpmn_workflow.sequence_flows {
            let source_id = node_id_map.get(&flow.source_ref).ok_or_else(|| {
                error!("Source node not found: {}", flow.source_ref);
                format!("Source node not found: {}", flow.source_ref)
            })?;

            let target_id = node_id_map.get(&flow.target_ref).ok_or_else(|| {
                error!("Target node not found: {}", flow.target_ref);
                format!("Target node not found: {}", flow.target_ref)
            })?;

            // Create pin IDs for connection
            let out_pin_id = OutPinId {
                node: *source_id,
                output: 0, // Use first output pin
            };

            let in_pin_id = InPinId {
                node: *target_id,
                input: 0, // Use first input pin
            };

            // Create the wire
            snarl.connect(out_pin_id, in_pin_id);

            debug!("Connected {} -> {} (flow: {})", flow.source_ref, flow.target_ref, flow.id);
        }

        info!(
            "Conversion complete: {} nodes, {} connections",
            node_id_map.len(),
            bpmn_workflow.sequence_flows.len()
        );

        Ok(WorkflowFile {
            version: "1.0".to_string(),
            name: bpmn_workflow.bpmn_process.name,
            snarl,
        })
    }

    /// Calculate layout positions for nodes based on workflow structure
    fn calculate_layout(
        &self,
        bpmn_workflow: &BpmnJsonWorkflow,
    ) -> Result<HashMap<String, Pos2>, String> {
        let mut positions = HashMap::new();
        let mut visited = HashMap::new();
        let mut level_counts: HashMap<usize, usize> = HashMap::new();

        // Build adjacency map
        let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
        for flow in &bpmn_workflow.sequence_flows {
            adjacency
                .entry(flow.source_ref.clone())
                .or_default()
                .push(flow.target_ref.clone());
        }

        // Find start event(s)
        let start_events: Vec<&WorkflowStep> = bpmn_workflow
            .workflow_steps
            .iter()
            .filter(|s| s.step_type == "startEvent")
            .collect();

        if start_events.is_empty() {
            warn!("No start event found, using first step");
            if let Some(first_step) = bpmn_workflow.workflow_steps.first() {
                self.layout_node(
                    &first_step.id,
                    0,
                    &adjacency,
                    &mut visited,
                    &mut level_counts,
                    &mut positions,
                );
            }
        } else {
            for start_event in start_events {
                self.layout_node(
                    &start_event.id,
                    0,
                    &adjacency,
                    &mut visited,
                    &mut level_counts,
                    &mut positions,
                );
            }
        }

        Ok(positions)
    }

    /// Recursively layout nodes using BFS-like traversal
    fn layout_node(
        &self,
        node_id: &str,
        level: usize,
        adjacency: &HashMap<String, Vec<String>>,
        visited: &mut HashMap<String, usize>,
        level_counts: &mut HashMap<usize, usize>,
        positions: &mut HashMap<String, Pos2>,
    ) {
        // Skip if already visited at a lower level
        if let Some(&visited_level) = visited.get(node_id) {
            if visited_level <= level {
                return;
            }
        }

        visited.insert(node_id.to_string(), level);

        // Calculate position based on level and count at this level
        let count_at_level = level_counts.entry(level).or_insert(0);
        let x = self.layout_config.start_x + (level as f32) * self.layout_config.horizontal_spacing;
        let y = self.layout_config.start_y
            + (*count_at_level as f32) * self.layout_config.vertical_spacing;

        positions.insert(node_id.to_string(), Pos2::new(x, y));
        *count_at_level += 1;

        // Process children
        if let Some(children) = adjacency.get(node_id) {
            for child_id in children {
                self.layout_node(child_id, level + 1, adjacency, visited, level_counts, positions);
            }
        }
    }

    /// Create an EnhancedBpmnNode from a WorkflowStep
    fn create_node_from_step(&self, step: &WorkflowStep) -> Result<EnhancedBpmnNode, String> {
        let node = match step.step_type.as_str() {
            "startEvent" => EnhancedBpmnNode::start_event(&step.id, &step.name),
            "endEvent" => EnhancedBpmnNode::end_event(&step.id, &step.name),
            "serviceTask" => {
                // Determine specific task type
                match step.task_type.as_deref() {
                    Some("research")
                    | Some("design")
                    | Some("code_generation")
                    | Some("testing")
                    | Some("documentation")
                    | Some("integration")
                    | Some("debugging")
                    | Some("optimization") => EnhancedBpmnNode::service_task(&step.id, &step.name),
                    _ => EnhancedBpmnNode::service_task(&step.id, &step.name),
                }
            },
            "userTask" => EnhancedBpmnNode::user_task(&step.id, &step.name),
            "exclusiveGateway" => EnhancedBpmnNode::exclusive_gateway(&step.id, &step.name),
            "parallelGateway" => {
                // Create parallel gateway
                EnhancedBpmnNode::new(
                    &step.id,
                    BpmnNodeType::Gateway(GatewayNode {
                        name: step.name.clone(),
                        documentation: None,
                        gateway_type: BpmnGatewayType::Parallel,
                        gateway_direction: GatewayDirection::Unspecified,
                    }),
                )
            },
            "inclusiveGateway" => {
                // Create inclusive gateway
                EnhancedBpmnNode::new(
                    &step.id,
                    BpmnNodeType::Gateway(GatewayNode {
                        name: step.name.clone(),
                        documentation: None,
                        gateway_type: BpmnGatewayType::Inclusive,
                        gateway_direction: GatewayDirection::Unspecified,
                    }),
                )
            },
            "task" => {
                // Generic task (abstract)
                EnhancedBpmnNode::new(
                    &step.id,
                    BpmnNodeType::Task(TaskNode {
                        name: step.name.clone(),
                        documentation: None,
                        task_type: BpmnTaskType::Abstract,
                        loop_characteristics: None,
                        is_for_compensation: false,
                    }),
                )
            },
            "scriptTask" => {
                // Script task
                EnhancedBpmnNode::new(
                    &step.id,
                    BpmnNodeType::Task(TaskNode {
                        name: step.name.clone(),
                        documentation: None,
                        task_type: BpmnTaskType::Script {
                            script_format: step
                                .implementation
                                .clone()
                                .unwrap_or_else(|| "application/x-rust".to_string()),
                            script: String::new(), // Empty script by default
                        },
                        loop_characteristics: None,
                        is_for_compensation: false,
                    }),
                )
            },
            "manualTask" => {
                // Manual task
                EnhancedBpmnNode::new(
                    &step.id,
                    BpmnNodeType::Task(TaskNode {
                        name: step.name.clone(),
                        documentation: None,
                        task_type: BpmnTaskType::Manual,
                        loop_characteristics: None,
                        is_for_compensation: false,
                    }),
                )
            },
            "businessRuleTask" => {
                // Business rule task
                EnhancedBpmnNode::new(
                    &step.id,
                    BpmnNodeType::Task(TaskNode {
                        name: step.name.clone(),
                        documentation: None,
                        task_type: BpmnTaskType::BusinessRule {
                            implementation: step.implementation.clone(),
                            rule_ref: None,
                        },
                        loop_characteristics: None,
                        is_for_compensation: false,
                    }),
                )
            },
            "intermediateCatchEvent" | "intermediateThrowEvent" => {
                // Intermediate event
                use crate::ui::enhanced_nodes::{BpmnNodeType, IntermediateEventNode};
                EnhancedBpmnNode::new(
                    &step.id,
                    BpmnNodeType::IntermediateEvent(IntermediateEventNode {
                        name: step.name.clone(),
                        documentation: None,
                        event_definition: None,
                        is_catching: step.step_type == "intermediateCatchEvent",
                        is_interrupting: true,
                        is_boundary: false,
                        attached_to_activity_id: None,
                    }),
                )
            },
            unknown => {
                warn!("Unknown step type '{}', creating as generic task", unknown);
                EnhancedBpmnNode::new(
                    &step.id,
                    BpmnNodeType::Task(TaskNode {
                        name: step.name.clone(),
                        documentation: Some(format!("Unknown type: {}", unknown)),
                        task_type: BpmnTaskType::Abstract,
                        loop_characteristics: None,
                        is_for_compensation: false,
                    }),
                )
            },
        };

        Ok(node)
    }
}

impl Default for BpmnJsonConverter {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to load a workflow file
pub fn load_workflow_file<P: AsRef<Path>>(path: P) -> Result<WorkflowFile, String> {
    let converter = BpmnJsonConverter::new();
    converter.load_from_file(path)
}

/// Convenience function to convert BPMN JSON string
pub fn convert_bpmn_json(json_content: &str) -> Result<WorkflowFile, String> {
    let converter = BpmnJsonConverter::new();
    converter.load_from_string(json_content)
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

        let result = convert_bpmn_json(json);
        assert!(result.is_ok(), "Failed to convert: {:?}", result.err());

        let workflow_file = result.unwrap();
        assert_eq!(workflow_file.name, "Test Process");
        assert_eq!(workflow_file.version, "1.0");

        // Check that we have 3 nodes
        let node_count = workflow_file.snarl.node_ids().count();
        assert_eq!(node_count, 3, "Expected 3 nodes, got {}", node_count);
    }

    #[test]
    fn test_gateway_creation() {
        let json = r#"{
            "bpmn_process": {
                "id": "gateway_test",
                "name": "Gateway Test",
                "version": "1.0.0",
                "isExecutable": true
            },
            "workflow_steps": [
                {
                    "id": "start",
                    "name": "Start",
                    "type": "startEvent"
                },
                {
                    "id": "gateway",
                    "name": "Decision",
                    "type": "exclusiveGateway"
                },
                {
                    "id": "parallel",
                    "name": "Parallel Split",
                    "type": "parallelGateway"
                }
            ],
            "sequence_flows": []
        }"#;

        let result = convert_bpmn_json(json);
        assert!(result.is_ok());

        let workflow_file = result.unwrap();
        assert_eq!(workflow_file.snarl.node_ids().count(), 3);
    }
}
