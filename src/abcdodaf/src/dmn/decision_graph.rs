//! Decision Graph and Requirement Diagram structures
//!
//! This module implements DMN decision graphs that show dependencies
//! between decisions and data inputs.

use crate::dmn::errors::{DmnError, DmnResult};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// A node in the decision graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionNode {
    /// A decision that depends on inputs and other decisions
    Decision { id: String, name: String, description: Option<String> },
    /// An input data source
    InputData { id: String, name: String, description: Option<String> },
    /// A business knowledge model
    BusinessKnowledge { id: String, name: String, description: Option<String> },
}

impl DecisionNode {
    /// Get the node ID
    pub fn id(&self) -> &str {
        match self {
            DecisionNode::Decision { id, .. } => id,
            DecisionNode::InputData { id, .. } => id,
            DecisionNode::BusinessKnowledge { id, .. } => id,
        }
    }

    /// Get the node name
    pub fn name(&self) -> &str {
        match self {
            DecisionNode::Decision { name, .. } => name,
            DecisionNode::InputData { name, .. } => name,
            DecisionNode::BusinessKnowledge { name, .. } => name,
        }
    }

    /// Get the node type
    pub fn node_type(&self) -> &str {
        match self {
            DecisionNode::Decision { .. } => "decision",
            DecisionNode::InputData { .. } => "input_data",
            DecisionNode::BusinessKnowledge { .. } => "business_knowledge",
        }
    }
}

/// A requirement in the decision graph (edge)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Requirement {
    /// ID of the source (required) node
    pub source_id: String,
    /// ID of the target (dependent) node
    pub target_id: String,
}

/// Decision graph showing dependencies between decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionGraph {
    /// ID for this graph
    pub id: String,
    /// All nodes in the graph
    nodes: HashMap<String, DecisionNode>,
    /// All requirements (edges) in the graph
    requirements: Vec<Requirement>,
}

impl DecisionGraph {
    /// Create a new decision graph
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into(), nodes: HashMap::new(), requirements: Vec::new() }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node: DecisionNode) -> DmnResult<()> {
        let node_id = node.id().to_string();
        if self.nodes.contains_key(&node_id) {
            return Err(DmnError::InvalidDecisionGraph(format!("Node {} already exists", node_id)));
        }
        self.nodes.insert(node_id, node);
        Ok(())
    }

    /// Get a node by ID
    pub fn get_node(&self, id: &str) -> Option<&DecisionNode> {
        self.nodes.get(id)
    }

    /// Get all nodes
    pub fn nodes(&self) -> impl Iterator<Item = &DecisionNode> {
        self.nodes.values()
    }

    /// Add a requirement (dependency)
    pub fn add_requirement(&mut self, source_id: String, target_id: String) -> DmnResult<()> {
        // Verify both nodes exist
        if !self.nodes.contains_key(&source_id) {
            return Err(DmnError::InvalidDecisionGraph(format!(
                "Source node {} not found",
                source_id
            )));
        }
        if !self.nodes.contains_key(&target_id) {
            return Err(DmnError::InvalidDecisionGraph(format!(
                "Target node {} not found",
                target_id
            )));
        }

        // Check for circular dependencies
        if self.would_create_cycle(&source_id, &target_id) {
            return Err(DmnError::CircularDependency(format!(
                "Adding requirement {} -> {} would create a cycle",
                source_id, target_id
            )));
        }

        let req = Requirement { source_id, target_id };

        if !self.requirements.contains(&req) {
            self.requirements.push(req);
        }

        Ok(())
    }

    /// Get all requirements
    pub fn requirements(&self) -> &[Requirement] {
        &self.requirements
    }

    /// Get all requirements for a specific target node
    pub fn requirements_for(&self, target_id: &str) -> Vec<&Requirement> {
        self.requirements.iter().filter(|r| r.target_id == target_id).collect()
    }

    /// Get all nodes that depend on a given node
    pub fn dependents(&self, node_id: &str) -> Vec<&DecisionNode> {
        self.requirements
            .iter()
            .filter(|r| r.source_id == node_id)
            .filter_map(|r| self.nodes.get(&r.target_id))
            .collect()
    }

    /// Get all nodes that a given node depends on
    pub fn dependencies(&self, node_id: &str) -> Vec<&DecisionNode> {
        self.requirements
            .iter()
            .filter(|r| r.target_id == node_id)
            .filter_map(|r| self.nodes.get(&r.source_id))
            .collect()
    }

    /// Perform topological sort of nodes based on dependencies
    pub fn topological_sort(&self) -> DmnResult<Vec<String>> {
        let mut visited = HashSet::new();
        let mut sorted = Vec::new();
        let mut visiting = HashSet::new();

        for node_id in self.nodes.keys() {
            if !visited.contains(node_id) {
                self.visit(node_id, &mut visited, &mut visiting, &mut sorted)?;
            }
        }

        Ok(sorted)
    }

    fn visit(
        &self,
        node_id: &str,
        visited: &mut HashSet<String>,
        visiting: &mut HashSet<String>,
        sorted: &mut Vec<String>,
    ) -> DmnResult<()> {
        if visited.contains(node_id) {
            return Ok(());
        }

        if visiting.contains(node_id) {
            return Err(DmnError::CircularDependency(format!(
                "Circular dependency detected at {}",
                node_id
            )));
        }

        visiting.insert(node_id.to_string());

        // Visit dependencies first
        for dep in self.dependencies(node_id) {
            self.visit(dep.id(), visited, visiting, sorted)?;
        }

        visiting.remove(node_id);
        visited.insert(node_id.to_string());
        sorted.push(node_id.to_string());

        Ok(())
    }

    /// Check if adding a requirement would create a cycle
    fn would_create_cycle(&self, source_id: &str, target_id: &str) -> bool {
        // Use BFS to check if target can reach source
        let mut queue = vec![target_id];
        let mut visited = HashSet::new();

        while let Some(node_id) = queue.pop() {
            if visited.contains(node_id) {
                continue;
            }
            visited.insert(node_id);

            if node_id == source_id {
                return true;
            }

            // Add dependents to queue
            for req in &self.requirements {
                if req.source_id == node_id {
                    queue.push(&req.target_id);
                }
            }
        }

        false
    }

    /// Validate the entire graph
    pub fn validate(&self) -> DmnResult<()> {
        // Check that all requirements reference existing nodes
        for req in &self.requirements {
            if !self.nodes.contains_key(&req.source_id) {
                return Err(DmnError::InvalidDecisionGraph(format!(
                    "Source node {} not found",
                    req.source_id
                )));
            }
            if !self.nodes.contains_key(&req.target_id) {
                return Err(DmnError::InvalidDecisionGraph(format!(
                    "Target node {} not found",
                    req.target_id
                )));
            }
        }

        // Check for cycles
        self.topological_sort()?;

        Ok(())
    }
}

/// DMN Requirement Diagram representing the complete decision logic structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementDiagram {
    /// Diagram ID
    pub id: String,
    /// Diagram name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// The decision graph
    pub graph: DecisionGraph,
}

impl RequirementDiagram {
    /// Create a new requirement diagram
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            graph: DecisionGraph::new("default"),
        }
    }

    /// Validate the diagram
    pub fn validate(&self) -> DmnResult<()> {
        self.graph.validate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_node_creation() {
        let node = DecisionNode::Decision {
            id: "test".to_string(),
            name: "Test Decision".to_string(),
            description: None,
        };

        assert_eq!(node.id(), "test");
        assert_eq!(node.name(), "Test Decision");
        assert_eq!(node.node_type(), "decision");
    }

    #[test]
    fn test_input_node_creation() {
        let node = DecisionNode::InputData {
            id: "age".to_string(),
            name: "Age".to_string(),
            description: Some("Customer age".to_string()),
        };

        assert_eq!(node.id(), "age");
        assert_eq!(node.node_type(), "input_data");
    }

    #[test]
    fn test_graph_creation() {
        let graph = DecisionGraph::new("test_graph");
        assert_eq!(graph.id, "test_graph");
        assert!(graph.nodes().count() == 0);
    }

    #[test]
    fn test_add_node() {
        let mut graph = DecisionGraph::new("test");
        let node = DecisionNode::Decision {
            id: "d1".to_string(),
            name: "Decision 1".to_string(),
            description: None,
        };

        graph.add_node(node).unwrap();
        assert!(graph.get_node("d1").is_some());
    }

    #[test]
    fn test_cannot_add_duplicate_node() {
        let mut graph = DecisionGraph::new("test");
        let node = DecisionNode::Decision {
            id: "d1".to_string(),
            name: "Decision 1".to_string(),
            description: None,
        };

        graph.add_node(node.clone()).unwrap();
        assert!(graph.add_node(node).is_err());
    }

    #[test]
    fn test_add_requirement() {
        let mut graph = DecisionGraph::new("test");

        let input = DecisionNode::InputData {
            id: "age".to_string(),
            name: "Age".to_string(),
            description: None,
        };
        graph.add_node(input).unwrap();

        let decision = DecisionNode::Decision {
            id: "category".to_string(),
            name: "Category".to_string(),
            description: None,
        };
        graph.add_node(decision).unwrap();

        graph.add_requirement("age".to_string(), "category".to_string()).unwrap();
        assert_eq!(graph.requirements().len(), 1);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut graph = DecisionGraph::new("test");

        let d1 = DecisionNode::Decision {
            id: "d1".to_string(),
            name: "D1".to_string(),
            description: None,
        };
        let d2 = DecisionNode::Decision {
            id: "d2".to_string(),
            name: "D2".to_string(),
            description: None,
        };

        graph.add_node(d1).unwrap();
        graph.add_node(d2).unwrap();

        // Create: d1 -> d2
        graph.add_requirement("d1".to_string(), "d2".to_string()).unwrap();

        // Try to create: d2 -> d1 (would be circular)
        assert!(graph.add_requirement("d2".to_string(), "d1".to_string()).is_err());
    }

    #[test]
    fn test_topological_sort() {
        let mut graph = DecisionGraph::new("test");

        let input = DecisionNode::InputData {
            id: "age".to_string(),
            name: "Age".to_string(),
            description: None,
        };
        let d1 = DecisionNode::Decision {
            id: "d1".to_string(),
            name: "D1".to_string(),
            description: None,
        };
        let d2 = DecisionNode::Decision {
            id: "d2".to_string(),
            name: "D2".to_string(),
            description: None,
        };

        graph.add_node(input).unwrap();
        graph.add_node(d1).unwrap();
        graph.add_node(d2).unwrap();

        graph.add_requirement("age".to_string(), "d1".to_string()).unwrap();
        graph.add_requirement("d1".to_string(), "d2".to_string()).unwrap();

        let sorted = graph.topological_sort().unwrap();

        // age should come before d1, d1 before d2
        let age_pos = sorted.iter().position(|id| id == "age").unwrap();
        let d1_pos = sorted.iter().position(|id| id == "d1").unwrap();
        let d2_pos = sorted.iter().position(|id| id == "d2").unwrap();

        assert!(age_pos < d1_pos);
        assert!(d1_pos < d2_pos);
    }
}
