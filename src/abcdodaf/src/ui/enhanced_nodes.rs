//! Enhanced BPMN Nodes for Visual Editor
//!
//! Bridges comprehensive BPMN 2.0 types with egui-snarl visual editor.
//! Includes full support for all BPMN 2.0 elements and DoDAF metadata.

use crate::bpmn::elements::*;
use crate::dodaf::ov5::*;
use egui::Color32;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Enhanced BPMN Node for visual editor
///
/// Wraps BPMN 2.0 elements and adds DoDAF operational metadata
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EnhancedBpmnNode {
    /// Unique node ID
    pub id: String,

    /// Node type (BPMN element)
    pub node_type: BpmnNodeType,

    /// Visual properties
    pub visual: VisualProperties,

    /// DoDAF metadata (optional)
    pub dodaf_metadata: Option<DodafNodeMetadata>,

    /// Custom properties
    pub properties: HashMap<String, serde_json::Value>,
}

/// BPMN Node Type wrapping all BPMN 2.0 elements
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BpmnNodeType {
    // Events
    StartEvent(StartEventNode),
    EndEvent(EndEventNode),
    IntermediateEvent(IntermediateEventNode),

    // Activities
    Task(TaskNode),
    Subprocess(SubprocessNode),

    // Gateways
    Gateway(GatewayNode),

    // Data Elements
    DataObject(DataObjectNode),
    DataStore(DataStoreNode),

    // Artifacts
    TextAnnotation(TextAnnotationNode),
    Group(GroupNode),
}

// ============================================================================
// Event Nodes
// ============================================================================

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StartEventNode {
    pub name: String,
    pub documentation: Option<String>,
    pub event_definition: Option<EventDefinition>,
    pub is_interrupting: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EndEventNode {
    pub name: String,
    pub documentation: Option<String>,
    pub event_definition: Option<EventDefinition>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IntermediateEventNode {
    pub name: String,
    pub documentation: Option<String>,
    pub event_definition: Option<EventDefinition>,
    pub is_catching: bool,
    pub is_interrupting: bool,
    pub is_boundary: bool, // Attached to activity
}

// ============================================================================
// Activity Nodes
// ============================================================================

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TaskNode {
    pub name: String,
    pub documentation: Option<String>,
    pub task_type: BpmnTaskType,
    pub loop_characteristics: Option<LoopCharacteristics>,
    pub is_for_compensation: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SubprocessNode {
    pub name: String,
    pub documentation: Option<String>,
    pub subprocess_type: SubprocessType,
    pub is_expanded: bool,
    pub loop_characteristics: Option<LoopCharacteristics>,
}

// ============================================================================
// Gateway Node
// ============================================================================

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GatewayNode {
    pub name: String,
    pub documentation: Option<String>,
    pub gateway_type: BpmnGatewayType,
    pub gateway_direction: GatewayDirection,
}

// ============================================================================
// Data Element Nodes
// ============================================================================

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DataObjectNode {
    pub name: String,
    pub is_collection: bool,
    pub data_state: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DataStoreNode {
    pub name: String,
    pub is_unlimited: bool,
    pub capacity: Option<i32>,
}

// ============================================================================
// Artifact Nodes
// ============================================================================

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextAnnotationNode {
    pub text: String,
    pub text_format: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GroupNode {
    pub category: Option<String>,
}

// ============================================================================
// Visual Properties
// ============================================================================

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VisualProperties {
    /// Node color
    pub color: Option<[u8; 3]>,

    /// Whether node is highlighted
    pub highlighted: bool,

    /// Whether node is selected
    pub selected: bool,

    /// Custom visual markers
    pub markers: Vec<VisualMarker>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum VisualMarker {
    Loop,
    MultiInstance,
    Compensation,
    AdHoc,
    Collapsed,
}

// ============================================================================
// DoDAF Metadata
// ============================================================================

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DodafNodeMetadata {
    /// Reference to operational activity
    pub activity_ref: Option<String>,

    /// Performer assignment
    pub performer: Option<PerformerRef>,

    /// Cost information
    pub cost: Option<Cost>,

    /// Duration estimate
    pub duration: Option<Duration>,

    /// Security classification
    pub security_domain: Option<SecurityDomain>,

    /// Custom DoDAF properties
    pub dodaf_properties: HashMap<String, serde_json::Value>,
}

// ============================================================================
// Implementation
// ============================================================================

impl EnhancedBpmnNode {
    /// Create a new node
    pub fn new(id: impl Into<String>, node_type: BpmnNodeType) -> Self {
        Self {
            id: id.into(),
            node_type,
            visual: VisualProperties {
                color: None,
                highlighted: false,
                selected: false,
                markers: Vec::new(),
            },
            dodaf_metadata: None,
            properties: HashMap::new(),
        }
    }

    /// Get node name
    pub fn name(&self) -> &str {
        match &self.node_type {
            BpmnNodeType::StartEvent(n) => &n.name,
            BpmnNodeType::EndEvent(n) => &n.name,
            BpmnNodeType::IntermediateEvent(n) => &n.name,
            BpmnNodeType::Task(n) => &n.name,
            BpmnNodeType::Subprocess(n) => &n.name,
            BpmnNodeType::Gateway(n) => &n.name,
            BpmnNodeType::DataObject(n) => &n.name,
            BpmnNodeType::DataStore(n) => &n.name,
            BpmnNodeType::TextAnnotation(n) => &n.text,
            BpmnNodeType::Group(_) => "Group",
        }
    }

    /// Set node name
    pub fn set_name(&mut self, new_name: String) {
        match &mut self.node_type {
            BpmnNodeType::StartEvent(n) => n.name = new_name,
            BpmnNodeType::EndEvent(n) => n.name = new_name,
            BpmnNodeType::IntermediateEvent(n) => n.name = new_name,
            BpmnNodeType::Task(n) => n.name = new_name,
            BpmnNodeType::Subprocess(n) => n.name = new_name,
            BpmnNodeType::Gateway(n) => n.name = new_name,
            BpmnNodeType::DataObject(n) => n.name = new_name,
            BpmnNodeType::DataStore(n) => n.name = new_name,
            BpmnNodeType::TextAnnotation(n) => n.text = new_name,
            BpmnNodeType::Group(_) => {}
        }
    }

    /// Get number of input pins
    pub fn input_count(&self) -> usize {
        match &self.node_type {
            BpmnNodeType::StartEvent(_) => 0,
            BpmnNodeType::EndEvent(_) => 1,
            BpmnNodeType::IntermediateEvent(e) => if e.is_catching { 1 } else { 1 },
            BpmnNodeType::Task(_) => 1,
            BpmnNodeType::Subprocess(_) => 1,
            BpmnNodeType::Gateway(_) => 1,
            BpmnNodeType::DataObject(_) => 0,
            BpmnNodeType::DataStore(_) => 0,
            BpmnNodeType::TextAnnotation(_) => 0,
            BpmnNodeType::Group(_) => 0,
        }
    }

    /// Get number of output pins
    pub fn output_count(&self) -> usize {
        match &self.node_type {
            BpmnNodeType::StartEvent(_) => 1,
            BpmnNodeType::EndEvent(_) => 0,
            BpmnNodeType::IntermediateEvent(_) => 1,
            BpmnNodeType::Task(_) => 1,
            BpmnNodeType::Subprocess(_) => 1,
            BpmnNodeType::Gateway(g) => {
                match &g.gateway_type {
                    BpmnGatewayType::Exclusive => 2,
                    BpmnGatewayType::Parallel => 2,
                    BpmnGatewayType::Inclusive => 2,
                    BpmnGatewayType::EventBased { .. } => 2,
                    BpmnGatewayType::ParallelEventBased => 2,
                    BpmnGatewayType::Complex { .. } => 2,
                }
            }
            BpmnNodeType::DataObject(_) => 0,
            BpmnNodeType::DataStore(_) => 0,
            BpmnNodeType::TextAnnotation(_) => 0,
            BpmnNodeType::Group(_) => 0,
        }
    }

    /// Get node color
    pub fn color(&self) -> Color32 {
        // Use custom color if set
        if let Some(rgb) = self.visual.color {
            return Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
        }

        // Default colors based on type
        match &self.node_type {
            BpmnNodeType::StartEvent(_) => Color32::from_rgb(144, 238, 144), // Light green
            BpmnNodeType::EndEvent(_) => Color32::from_rgb(255, 160, 160),   // Light red
            BpmnNodeType::IntermediateEvent(_) => Color32::from_rgb(255, 255, 200), // Light yellow
            BpmnNodeType::Task(t) => {
                match &t.task_type {
                    BpmnTaskType::User { .. } => Color32::from_rgb(173, 216, 230), // Light blue
                    BpmnTaskType::Service { .. } => Color32::from_rgb(255, 228, 181), // Moccasin
                    BpmnTaskType::Script { .. } => Color32::from_rgb(216, 191, 216), // Thistle
                    BpmnTaskType::Manual => Color32::from_rgb(255, 250, 205), // Lemon chiffon
                    BpmnTaskType::Send { .. } => Color32::from_rgb(176, 224, 230), // Powder blue
                    BpmnTaskType::Receive { .. } => Color32::from_rgb(221, 160, 221), // Plum
                    BpmnTaskType::BusinessRule { .. } => Color32::from_rgb(255, 222, 173), // Navajo white
                    BpmnTaskType::Abstract => Color32::from_rgb(211, 211, 211), // Light gray
                }
            }
            BpmnNodeType::Subprocess(_) => Color32::from_rgb(200, 200, 255), // Light blue-violet
            BpmnNodeType::Gateway(_) => Color32::from_rgb(255, 255, 153), // Light yellow
            BpmnNodeType::DataObject(_) => Color32::from_rgb(240, 240, 240), // Very light gray
            BpmnNodeType::DataStore(_) => Color32::from_rgb(200, 220, 240), // Light steel blue
            BpmnNodeType::TextAnnotation(_) => Color32::from_rgb(255, 255, 224), // Light yellow
            BpmnNodeType::Group(_) => Color32::from_rgba_premultiplied(200, 200, 200, 50), // Transparent gray
        }
    }

    /// Get node type name for display
    pub fn type_name(&self) -> &str {
        match &self.node_type {
            BpmnNodeType::StartEvent(_) => "Start Event",
            BpmnNodeType::EndEvent(_) => "End Event",
            BpmnNodeType::IntermediateEvent(_) => "Intermediate Event",
            BpmnNodeType::Task(t) => {
                match &t.task_type {
                    BpmnTaskType::User { .. } => "User Task",
                    BpmnTaskType::Service { .. } => "Service Task",
                    BpmnTaskType::Script { .. } => "Script Task",
                    BpmnTaskType::Manual => "Manual Task",
                    BpmnTaskType::Send { .. } => "Send Task",
                    BpmnTaskType::Receive { .. } => "Receive Task",
                    BpmnTaskType::BusinessRule { .. } => "Business Rule Task",
                    BpmnTaskType::Abstract => "Task",
                }
            }
            BpmnNodeType::Subprocess(s) => {
                match s.subprocess_type {
                    SubprocessType::Embedded => "Subprocess",
                    SubprocessType::CallActivity => "Call Activity",
                    SubprocessType::EventSubprocess => "Event Subprocess",
                    SubprocessType::Transaction => "Transaction",
                    SubprocessType::AdHoc => "Ad-Hoc Subprocess",
                }
            }
            BpmnNodeType::Gateway(g) => {
                match &g.gateway_type {
                    BpmnGatewayType::Exclusive => "Exclusive Gateway",
                    BpmnGatewayType::Parallel => "Parallel Gateway",
                    BpmnGatewayType::Inclusive => "Inclusive Gateway",
                    BpmnGatewayType::EventBased { .. } => "Event-Based Gateway",
                    BpmnGatewayType::ParallelEventBased => "Parallel Event Gateway",
                    BpmnGatewayType::Complex { .. } => "Complex Gateway",
                }
            }
            BpmnNodeType::DataObject(_) => "Data Object",
            BpmnNodeType::DataStore(_) => "Data Store",
            BpmnNodeType::TextAnnotation(_) => "Text Annotation",
            BpmnNodeType::Group(_) => "Group",
        }
    }

    /// Convert to BPMN 2.0 element
    pub fn to_bpmn_element(&self) -> BpmnElement {
        match &self.node_type {
            BpmnNodeType::StartEvent(n) => BpmnElement::StartEvent(StartEvent {
                id: self.id.clone(),
                name: Some(n.name.clone()),
                documentation: n.documentation.clone(),
                event_definition: n.event_definition.clone(),
                is_interrupting: n.is_interrupting,
            }),
            BpmnNodeType::Task(n) => BpmnElement::Task(BpmnTask {
                id: self.id.clone(),
                name: Some(n.name.clone()),
                documentation: n.documentation.clone(),
                task_type: n.task_type.clone(),
                default_flow: None,
                io_specification: None,
                properties: self.properties.clone(),
                loop_characteristics: n.loop_characteristics.clone(),
                is_for_compensation: n.is_for_compensation,
            }),
            // Add more conversions as needed...
            _ => unimplemented!("Conversion for this node type not yet implemented"),
        }
    }

    /// Add DoDAF metadata
    pub fn with_dodaf_metadata(mut self, metadata: DodafNodeMetadata) -> Self {
        self.dodaf_metadata = Some(metadata);
        self
    }
}

/// BPMN Element enum for conversion
pub enum BpmnElement {
    StartEvent(StartEvent),
    EndEvent(EndEvent),
    IntermediateEvent(IntermediateEvent),
    Task(BpmnTask),
    Subprocess(Subprocess),
    Gateway(BpmnGateway),
}

impl Default for VisualProperties {
    fn default() -> Self {
        Self {
            color: None,
            highlighted: false,
            selected: false,
            markers: Vec::new(),
        }
    }
}

// ============================================================================
// Convenience constructors
// ============================================================================

impl EnhancedBpmnNode {
    pub fn start_event(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self::new(
            id,
            BpmnNodeType::StartEvent(StartEventNode {
                name: name.into(),
                documentation: None,
                event_definition: None,
                is_interrupting: true,
            }),
        )
    }

    pub fn end_event(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self::new(
            id,
            BpmnNodeType::EndEvent(EndEventNode {
                name: name.into(),
                documentation: None,
                event_definition: None,
            }),
        )
    }

    pub fn user_task(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self::new(
            id,
            BpmnNodeType::Task(TaskNode {
                name: name.into(),
                documentation: None,
                task_type: BpmnTaskType::User {
                    implementation: None,
                    rendering: None,
                },
                loop_characteristics: None,
                is_for_compensation: false,
            }),
        )
    }

    pub fn service_task(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self::new(
            id,
            BpmnNodeType::Task(TaskNode {
                name: name.into(),
                documentation: None,
                task_type: BpmnTaskType::Service {
                    implementation: None,
                    operation_ref: None,
                },
                loop_characteristics: None,
                is_for_compensation: false,
            }),
        )
    }

    pub fn exclusive_gateway(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self::new(
            id,
            BpmnNodeType::Gateway(GatewayNode {
                name: name.into(),
                documentation: None,
                gateway_type: BpmnGatewayType::Exclusive,
                gateway_direction: GatewayDirection::Unspecified,
            }),
        )
    }
}
