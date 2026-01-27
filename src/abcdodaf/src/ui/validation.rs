use crate::ui::enhanced_nodes::{BpmnNodeType, EnhancedBpmnNode};
use egui_snarl::{InPinId, NodeId, OutPinId, Snarl};
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};

/// Validation error types for BPMN workflows
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValidationError {
    /// VE-001: No start event in workflow
    MissingStartNode,

    /// VE-002: More than one start event
    MultipleStartNodes { start_nodes: Vec<NodeId> },

    /// VE-003: No end event in workflow
    MissingEndNode,

    /// VE-004: Node is unreachable from start
    DisconnectedNode { node_id: NodeId, node_name: String },

    /// VE-005: Node requires input but has none
    MissingRequiredInput { node_id: NodeId, node_name: String },

    /// VE-006: Invalid connection between incompatible node types
    InvalidConnection {
        from_node: NodeId,
        to_node: NodeId,
        reason: String,
    },
}

/// Validation warnings (non-critical issues)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValidationWarning {
    /// VW-001: Non-end node has no outgoing connections
    UnconnectedOutput { node_id: NodeId, node_name: String },

    /// VW-002: Task node missing DoDAF metadata
    MissingDodafMetadata { node_id: NodeId, node_name: String },
}

/// Complete validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    pub fn has_issues(&self) -> bool {
        !self.errors.is_empty() || !self.warnings.is_empty()
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// BPMN workflow validator
pub struct Validator;

impl Validator {
    /// Validate a complete workflow
    pub fn validate_workflow(snarl: &Snarl<EnhancedBpmnNode>) -> ValidationResult {
        let mut result = ValidationResult::new();

        // If workflow is empty, no validation needed
        if snarl.node_ids().count() == 0 {
            return result;
        }

        // Step 1: Find all start and end nodes
        let start_nodes: Vec<NodeId> = snarl
            .node_ids()
            .filter(|&(_, node)| matches!(node.node_type, BpmnNodeType::StartEvent(_)))
            .map(|(node_id, _)| node_id)
            .collect();
        let end_nodes: Vec<NodeId> = snarl
            .node_ids()
            .filter(|&(_, node)| matches!(node.node_type, BpmnNodeType::EndEvent(_)))
            .map(|(node_id, _)| node_id)
            .collect();

        // VE-001: Check for missing start node
        if start_nodes.is_empty() {
            result.errors.push(ValidationError::MissingStartNode);
        }

        // VE-002: Check for multiple start nodes
        if start_nodes.len() > 1 {
            result.errors.push(ValidationError::MultipleStartNodes {
                start_nodes: start_nodes.clone(),
            });
        }

        // VE-003: Check for missing end node
        if end_nodes.is_empty() {
            result.errors.push(ValidationError::MissingEndNode);
        }

        // If we have exactly one start node, perform reachability analysis
        if start_nodes.len() == 1 {
            let start_node = start_nodes[0];
            let reachable = Self::find_reachable_nodes(snarl, start_node);

            // VE-004: Check for disconnected nodes
            for (node_id, node) in snarl.node_ids() {
                if !reachable.contains(&node_id) {
                    result.errors.push(ValidationError::DisconnectedNode {
                        node_id,
                        node_name: node.name().to_string(),
                    });
                }
            }
        }

        // VE-005: Check for nodes missing required inputs
        for (node_id, node) in snarl.node_ids() {

            // Start events don't need inputs
            if matches!(node.node_type, BpmnNodeType::StartEvent(_)) {
                continue;
            }

            // Check if node has any incoming connections
            let input_count = node.input_count();
            let has_input = (0..input_count).any(|pin_idx| {
                let in_pin_id = InPinId { node: node_id, input: pin_idx };
                !snarl.in_pin(in_pin_id).remotes.is_empty()
            });

            if !has_input {
                result.errors.push(ValidationError::MissingRequiredInput {
                    node_id,
                    node_name: node.name().to_string(),
                });
            }
        }

        // VW-001: Check for unconnected outputs (warnings)
        for (node_id, node) in snarl.node_ids() {

            // End events are expected to have no outputs
            if matches!(node.node_type, BpmnNodeType::EndEvent(_)) {
                continue;
            }

            // Check if node has any outgoing connections
            let output_count = node.output_count();
            let has_output = (0..output_count).any(|pin_idx| {
                let out_pin_id = OutPinId { node: node_id, output: pin_idx };
                !snarl.out_pin(out_pin_id).remotes.is_empty()
            });

            if !has_output {
                result.warnings.push(ValidationWarning::UnconnectedOutput {
                    node_id,
                    node_name: node.name().to_string(),
                });
            }
        }

        // VW-002: Check for missing DoDAF metadata on tasks
        for (node_id, node) in snarl.node_ids() {

            if matches!(node.node_type, BpmnNodeType::Task(_)) {
                // Check if any DoDAF metadata is present
                let has_metadata = node.dodaf_metadata.is_some();

                if !has_metadata {
                    result.warnings.push(ValidationWarning::MissingDodafMetadata {
                        node_id,
                        node_name: node.name().to_string(),
                    });
                }
            }
        }

        result
    }

    /// Find all nodes of a specific type
    fn find_nodes_by_type(
        snarl: &Snarl<EnhancedBpmnNode>,
        node_type: BpmnNodeType,
    ) -> Vec<NodeId> {
        snarl
            .node_ids()
            .filter(|&(_, node)| node.node_type == node_type)
            .map(|(node_id, _)| node_id)
            .collect()
    }

    /// Find all nodes reachable from a start node using BFS
    fn find_reachable_nodes(
        snarl: &Snarl<EnhancedBpmnNode>,
        start_node: NodeId,
    ) -> HashSet<NodeId> {
        let mut reachable = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(start_node);
        reachable.insert(start_node);

        while let Some(current) = queue.pop_front() {
            // Follow all outgoing connections
            let current_node = &snarl[current];
            let output_count = current_node.output_count();
            for pin_idx in 0..output_count {
                let out_pin_id = OutPinId { node: current, output: pin_idx };
                for &remote_pin in snarl.out_pin(out_pin_id).remotes.iter() {
                    let remote_node = snarl.in_pin(remote_pin).id.node;

                    if reachable.insert(remote_node) {
                        queue.push_back(remote_node);
                    }
                }
            }
        }

        reachable
    }

    /// Get error message for a validation error
    pub fn error_message(error: &ValidationError) -> String {
        match error {
            ValidationError::MissingStartNode => {
                "Workflow must have exactly one start event".to_string()
            }
            ValidationError::MultipleStartNodes { start_nodes } => {
                format!(
                    "Workflow cannot have multiple start events (found {})",
                    start_nodes.len()
                )
            }
            ValidationError::MissingEndNode => {
                "Workflow must have at least one end event".to_string()
            }
            ValidationError::DisconnectedNode { node_name, .. } => {
                format!("Node '{}' is disconnected from workflow", node_name)
            }
            ValidationError::MissingRequiredInput { node_name, .. } => {
                format!("Node '{}' requires an input connection", node_name)
            }
            ValidationError::InvalidConnection { reason, .. } => {
                format!("Invalid connection: {}", reason)
            }
        }
    }

    /// Get warning message for a validation warning
    pub fn warning_message(warning: &ValidationWarning) -> String {
        match warning {
            ValidationWarning::UnconnectedOutput { node_name, .. } => {
                format!("Node '{}' has no outgoing connections", node_name)
            }
            ValidationWarning::MissingDodafMetadata { node_name, .. } => {
                format!("Task '{}' has no DoDAF metadata", node_name)
            }
        }
    }

    /// Check if a specific node has validation errors
    pub fn node_has_error(result: &ValidationResult, node_id: NodeId) -> bool {
        result.errors.iter().any(|error| match error {
            ValidationError::DisconnectedNode { node_id: id, .. } => *id == node_id,
            ValidationError::MissingRequiredInput { node_id: id, .. } => *id == node_id,
            ValidationError::InvalidConnection { from_node, to_node, .. } => {
                *from_node == node_id || *to_node == node_id
            }
            ValidationError::MultipleStartNodes { start_nodes } => start_nodes.contains(&node_id),
            _ => false,
        })
    }

    /// Check if a specific node has validation warnings
    pub fn node_has_warning(result: &ValidationResult, node_id: NodeId) -> bool {
        result.warnings.iter().any(|warning| match warning {
            ValidationWarning::UnconnectedOutput { node_id: id, .. } => *id == node_id,
            ValidationWarning::MissingDodafMetadata { node_id: id, .. } => *id == node_id,
        })
    }

    /// Get all errors for a specific node
    pub fn node_errors(result: &ValidationResult, node_id: NodeId) -> Vec<&ValidationError> {
        result
            .errors
            .iter()
            .filter(|error| match error {
                ValidationError::DisconnectedNode { node_id: id, .. } => *id == node_id,
                ValidationError::MissingRequiredInput { node_id: id, .. } => *id == node_id,
                ValidationError::InvalidConnection { from_node, to_node, .. } => {
                    *from_node == node_id || *to_node == node_id
                }
                ValidationError::MultipleStartNodes { start_nodes } => {
                    start_nodes.contains(&node_id)
                }
                _ => false,
            })
            .collect()
    }

    /// Get all warnings for a specific node
    pub fn node_warnings(result: &ValidationResult, node_id: NodeId) -> Vec<&ValidationWarning> {
        result
            .warnings
            .iter()
            .filter(|warning| match warning {
                ValidationWarning::UnconnectedOutput { node_id: id, .. } => *id == node_id,
                ValidationWarning::MissingDodafMetadata { node_id: id, .. } => *id == node_id,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_workflow() {
        let snarl = Snarl::<EnhancedBpmnNode>::new();
        let result = Validator::validate_workflow(&snarl);
        assert!(result.is_valid());
        assert_eq!(result.error_count(), 0);
    }

    #[test]
    fn test_missing_start_node() {
        let mut snarl = Snarl::<EnhancedBpmnNode>::new();
        snarl.insert_node(
            egui::Pos2::ZERO,
            EnhancedBpmnNode::new(BpmnNodeType::Task, "Task 1".to_string()),
        );

        let result = Validator::validate_workflow(&snarl);
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| matches!(e, ValidationError::MissingStartNode)));
    }
}
