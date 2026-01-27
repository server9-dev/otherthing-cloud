//! DoDAF 2.02 SV-4: Systems Functionality Description
//!
//! Specifies functionality of systems, functional hierarchies,
//! and data flows between functions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// SV-4 Systems Functionality Description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemsFunctionalityDescription {
    /// Unique identifier
    pub id: String,
    /// Name of the description
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Version number
    pub version: String,

    /// System functions
    pub functions: Vec<SystemFunction>,
    /// Function-to-function flows
    pub function_flows: Vec<FunctionFlow>,
    /// Data flows between functions
    pub data_flows: Vec<FunctionDataFlow>,
    /// Functional decomposition hierarchies
    pub functional_hierarchies: Vec<FunctionalHierarchy>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// System function - atomic computational behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemFunction {
    /// Function ID
    pub id: String,
    /// Function name
    pub name: String,
    /// Function description
    pub description: Option<String>,
    /// Type of function
    pub function_type: FunctionType,
    /// System that provides this function
    pub belongs_to_system: String,

    /// Input data/resources
    pub inputs: Vec<String>,
    /// Output data/resources
    pub outputs: Vec<String>,
    /// Systems that implement/perform this function
    pub allocated_to: Vec<String>,

    // Hierarchical decomposition
    /// Parent function (for decomposed functions)
    pub parent_function: Option<String>,
    /// Child functions (decomposition)
    pub child_functions: Vec<String>,

    /// Additional properties
    pub properties: HashMap<String, serde_json::Value>,
}

/// Type of system function
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FunctionType {
    /// Primary operational function
    Primary,
    /// Supporting function
    Supporting,
    /// Enabling function
    Enabling,
    /// Operational function
    Operational,
    /// Maintenance function
    Maintenance,
    /// Management function
    Management,
}

/// Function-to-function flow (control flow)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionFlow {
    /// Flow ID
    pub id: String,
    /// Source function ID
    pub source_function: String,
    /// Target function ID
    pub target_function: String,
    /// Type of flow
    pub flow_type: String,
}

/// Data flow between functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDataFlow {
    /// Flow ID
    pub id: String,
    /// Flow name
    pub name: Option<String>,
    /// Source function ID
    pub source_function: String,
    /// Target function ID
    pub target_function: String,
    /// Data elements flowing
    pub data_elements: Vec<String>,
    /// Timing of the flow
    pub timing: Option<String>,
}

/// Functional decomposition hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionalHierarchy {
    /// Root function ID
    pub root_function: String,
    /// Hierarchy levels
    pub levels: Vec<HierarchyLevel>,
}

/// Single level in functional hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyLevel {
    /// Level number (0 = root)
    pub level: i32,
    /// Function IDs at this level
    pub functions: Vec<String>,
    /// Parent-child relationships
    pub parent_functions: HashMap<String, Vec<String>>,
}

// ============================================================================
// Helper implementations
// ============================================================================

impl SystemsFunctionalityDescription {
    /// Create a new systems functionality description
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            version: "1.0".to_string(),
            functions: Vec::new(),
            function_flows: Vec::new(),
            data_flows: Vec::new(),
            functional_hierarchies: Vec::new(),
            created_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add a function
    pub fn add_function(mut self, function: SystemFunction) -> Self {
        self.functions.push(function);
        self
    }

    /// Add a function flow
    pub fn add_function_flow(mut self, flow: FunctionFlow) -> Self {
        self.function_flows.push(flow);
        self
    }

    /// Add a data flow
    pub fn add_data_flow(mut self, flow: FunctionDataFlow) -> Self {
        self.data_flows.push(flow);
        self
    }

    /// Add a hierarchy
    pub fn add_hierarchy(mut self, hierarchy: FunctionalHierarchy) -> Self {
        self.functional_hierarchies.push(hierarchy);
        self
    }

    /// Get functions for a system
    pub fn get_system_functions(&self, system_id: &str) -> Vec<&SystemFunction> {
        self.functions.iter().filter(|f| f.belongs_to_system == system_id).collect()
    }

    /// Get outbound data flows from a function
    pub fn get_function_outputs(&self, function_id: &str) -> Vec<&FunctionDataFlow> {
        self.data_flows.iter().filter(|f| f.source_function == function_id).collect()
    }

    /// Get inbound data flows to a function
    pub fn get_function_inputs(&self, function_id: &str) -> Vec<&FunctionDataFlow> {
        self.data_flows.iter().filter(|f| f.target_function == function_id).collect()
    }
}

impl SystemFunction {
    /// Create a new system function
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        function_type: FunctionType,
        system_id: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            function_type,
            belongs_to_system: system_id.into(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            allocated_to: Vec::new(),
            parent_function: None,
            child_functions: Vec::new(),
            properties: HashMap::new(),
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add input data element
    pub fn add_input(mut self, data_element_id: impl Into<String>) -> Self {
        self.inputs.push(data_element_id.into());
        self
    }

    /// Add output data element
    pub fn add_output(mut self, data_element_id: impl Into<String>) -> Self {
        self.outputs.push(data_element_id.into());
        self
    }

    /// Add child function
    pub fn add_child_function(mut self, child_id: impl Into<String>) -> Self {
        self.child_functions.push(child_id.into());
        self
    }

    /// Set parent function
    pub fn with_parent(mut self, parent_id: impl Into<String>) -> Self {
        self.parent_function = Some(parent_id.into());
        self
    }

    /// Add system allocation
    pub fn allocate_to_system(mut self, system_id: impl Into<String>) -> Self {
        self.allocated_to.push(system_id.into());
        self
    }

    /// Get function arity (inputs/outputs count)
    pub fn arity(&self) -> (usize, usize) {
        (self.inputs.len(), self.outputs.len())
    }
}

impl FunctionFlow {
    /// Create a new function flow
    pub fn new(id: impl Into<String>, source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            source_function: source.into(),
            target_function: target.into(),
            flow_type: "control".to_string(),
        }
    }

    /// Set flow type
    pub fn with_flow_type(mut self, flow_type: impl Into<String>) -> Self {
        self.flow_type = flow_type.into();
        self
    }
}

impl FunctionDataFlow {
    /// Create a new data flow
    pub fn new(
        id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: None,
            source_function: source.into(),
            target_function: target.into(),
            data_elements: Vec::new(),
            timing: None,
        }
    }

    /// Set name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Add data element
    pub fn add_data_element(mut self, element_id: impl Into<String>) -> Self {
        self.data_elements.push(element_id.into());
        self
    }

    /// Set timing
    pub fn with_timing(mut self, timing: impl Into<String>) -> Self {
        self.timing = Some(timing.into());
        self
    }
}

impl FunctionalHierarchy {
    /// Create a new functional hierarchy
    pub fn new(root_function: impl Into<String>) -> Self {
        Self {
            root_function: root_function.into(),
            levels: vec![HierarchyLevel {
                level: 0,
                functions: vec![],
                parent_functions: HashMap::new(),
            }],
        }
    }

    /// Add a level to the hierarchy
    pub fn add_level(mut self, level: HierarchyLevel) -> Self {
        self.levels.push(level);
        self
    }

    /// Get maximum hierarchy depth
    pub fn max_depth(&self) -> i32 {
        self.levels.iter().map(|l| l.level).max().unwrap_or(0)
    }
}

impl HierarchyLevel {
    /// Create a new hierarchy level
    pub fn new(level: i32) -> Self {
        Self {
            level,
            functions: Vec::new(),
            parent_functions: HashMap::new(),
        }
    }

    /// Add a function at this level
    pub fn add_function(mut self, function_id: impl Into<String>) -> Self {
        self.functions.push(function_id.into());
        self
    }

    /// Add parent-child relationship
    pub fn add_relationship(mut self, parent: impl Into<String>, child: impl Into<String>) -> Self {
        self.parent_functions
            .entry(parent.into())
            .or_insert_with(Vec::new)
            .push(child.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_functionality_description() {
        let desc = SystemsFunctionalityDescription::new("sv4_1", "Functionality")
            .with_description("System functionality and data flows");

        assert_eq!(desc.name, "Functionality");
        assert_eq!(desc.functions.len(), 0);
        assert_eq!(desc.data_flows.len(), 0);
    }

    #[test]
    fn test_system_functions() {
        let func1 =
            SystemFunction::new("func_1", "Analyze", FunctionType::Primary, "sys_1")
                .add_input("raw_data")
                .add_output("analysis_result");

        let func2 = SystemFunction::new("func_2", "Store", FunctionType::Supporting, "sys_2")
            .add_input("analysis_result");

        assert_eq!(func1.function_type, FunctionType::Primary);
        assert_eq!(func1.arity(), (1, 1));
        assert_eq!(func2.inputs.len(), 1);
    }

    #[test]
    fn test_function_data_flows() {
        let flow = FunctionDataFlow::new("flow_1", "func_1", "func_2")
            .add_data_element("data_elem_1")
            .with_timing("Real-time");

        assert_eq!(flow.data_elements.len(), 1);
        assert!(flow.timing.is_some());
    }

    #[test]
    fn test_functional_hierarchy() {
        let mut level0 = HierarchyLevel::new(0);
        level0.functions.push("func_root".to_string());

        let mut level1 = HierarchyLevel::new(1)
            .add_function("func_1")
            .add_function("func_2");
        level1 = level1.add_relationship("func_root", "func_1");
        level1 = level1.add_relationship("func_root", "func_2");

        let hierarchy = FunctionalHierarchy::new("func_root")
            .add_level(level1);

        assert_eq!(hierarchy.max_depth(), 1);
    }

    #[test]
    fn test_function_types() {
        assert_eq!(FunctionType::Primary, FunctionType::Primary);
        assert_ne!(FunctionType::Primary, FunctionType::Supporting);
    }
}
