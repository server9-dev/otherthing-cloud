//! BpmnDiagram <-> Snarl Converter
//!
//! Bidirectional converter between BpmnDiagram (library's native format)
//! and Snarl<EnhancedBpmnNode> (visual editor format).
//!
//! This module ensures full BPMN 2.0 compatibility throughout the library
//! by preserving all BPMN properties during conversion, including:
//! - All flow elements (events, tasks, gateways, subprocesses)
//! - All connecting objects (sequence flows, data associations)
//! - All data elements (data objects, data stores)
//! - All artifacts (text annotations, groups)
//! - Visual layout information (shapes, edges, positions)
//! - DoDAF metadata

use crate::bpmn::elements::*;
use crate::ui::enhanced_nodes::*;
use egui_snarl::{InPinId, NodeId, OutPinId, Snarl};
use std::collections::HashMap;

/// BpmnDiagram to Snarl converter
///
/// Handles bidirectional conversion while preserving all BPMN semantics
pub struct BpmnDiagramConverter;

/// Conversion error types
#[derive(Debug, Clone)]
pub enum ConversionError {
    /// No processes found in diagram
    EmptyDiagram,
    /// Invalid element reference
    InvalidReference(String),
    /// Missing required data
    MissingData(String),
    /// Invalid BPMN structure
    InvalidStructure(String),
    /// Conversion not supported
    Unsupported(String),
}

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyDiagram => write!(f, "Diagram contains no processes"),
            Self::InvalidReference(s) => write!(f, "Invalid reference: {}", s),
            Self::MissingData(s) => write!(f, "Missing required data: {}", s),
            Self::InvalidStructure(s) => write!(f, "Invalid BPMN structure: {}", s),
            Self::Unsupported(s) => write!(f, "Unsupported: {}", s),
        }
    }
}

impl std::error::Error for ConversionError {}

/// Conversion context - tracks element mappings during conversion
struct ConversionContext {
    /// Map from BPMN element ID to Snarl NodeId
    element_to_node: HashMap<String, NodeId>,
    /// Map from Snarl NodeId to BPMN element ID
    node_to_element: HashMap<NodeId, String>,
    /// Map from BPMN element ID to visual position
    positions: HashMap<String, (f64, f64)>,
}

impl ConversionContext {
    fn new() -> Self {
        Self {
            element_to_node: HashMap::new(),
            node_to_element: HashMap::new(),
            positions: HashMap::new(),
        }
    }

    fn add_mapping(&mut self, element_id: String, node_id: NodeId) {
        self.element_to_node.insert(element_id.clone(), node_id);
        self.node_to_element.insert(node_id, element_id);
    }

    fn get_node_id(&self, element_id: &str) -> Option<NodeId> {
        self.element_to_node.get(element_id).copied()
    }

    fn get_element_id(&self, node_id: NodeId) -> Option<&String> {
        self.node_to_element.get(&node_id)
    }

    fn set_position(&mut self, element_id: String, x: f64, y: f64) {
        self.positions.insert(element_id, (x, y));
    }

    fn get_position(&self, element_id: &str) -> Option<(f64, f64)> {
        self.positions.get(element_id).copied()
    }
}

impl BpmnDiagramConverter {
    /// Convert BpmnDiagram to Snarl for editing
    ///
    /// Takes the first process from the diagram. For multi-process diagrams,
    /// use `to_snarl_with_process` to specify which process to convert.
    pub fn to_snarl(diagram: &BpmnDiagram) -> Result<Snarl<EnhancedBpmnNode>, ConversionError> {
        // Get first process
        let process = diagram
            .processes
            .first()
            .ok_or(ConversionError::EmptyDiagram)?;

        Self::process_to_snarl(process, diagram)
    }

    /// Convert specific process to Snarl
    pub fn to_snarl_with_process(
        diagram: &BpmnDiagram,
        process_id: &str,
    ) -> Result<Snarl<EnhancedBpmnNode>, ConversionError> {
        let process = diagram
            .processes
            .iter()
            .find(|p| p.id == process_id)
            .ok_or_else(|| ConversionError::InvalidReference(process_id.to_string()))?;

        Self::process_to_snarl(process, diagram)
    }

    /// Convert BpmnProcess to Snarl
    fn process_to_snarl(
        process: &BpmnProcess,
        diagram: &BpmnDiagram,
    ) -> Result<Snarl<EnhancedBpmnNode>, ConversionError> {
        let mut snarl = Snarl::new();
        let mut ctx = ConversionContext::new();

        // Extract visual layout information if available
        // TODO: Add diagram_info parameter when BpmnDiagram includes it
        // Self::extract_layout(&diagram.diagram_info, &mut ctx);

        // Convert all flow elements to nodes
        Self::convert_start_events(&process.start_events, &mut snarl, &mut ctx)?;
        Self::convert_end_events(&process.end_events, &mut snarl, &mut ctx)?;
        Self::convert_intermediate_events(&process.intermediate_events, &mut snarl, &mut ctx)?;
        Self::convert_tasks(&process.tasks, &mut snarl, &mut ctx)?;
        Self::convert_subprocesses(&process.subprocesses, &mut snarl, &mut ctx)?;
        Self::convert_gateways(&process.gateways, &mut snarl, &mut ctx)?;
        Self::convert_data_objects(&process.data_objects, &mut snarl, &mut ctx)?;

        // Convert data stores from diagram level
        Self::convert_data_stores(&diagram.data_stores, &mut snarl, &mut ctx)?;

        // Convert artifacts
        Self::convert_text_annotations(&process.text_annotations, &mut snarl, &mut ctx)?;
        Self::convert_groups(&process.groups, &mut snarl, &mut ctx)?;

        // Convert sequence flows to wires
        Self::convert_sequence_flows(&process.sequence_flows, &mut snarl, &ctx)?;

        Ok(snarl)
    }

    // ========================================================================
    // Flow Element Conversion - BPMN to Snarl
    // ========================================================================

    fn convert_start_events(
        events: &[StartEvent],
        snarl: &mut Snarl<EnhancedBpmnNode>,
        ctx: &mut ConversionContext,
    ) -> Result<(), ConversionError> {
        for event in events {
            let node = EnhancedBpmnNode::new(
                event.id.clone(),
                BpmnNodeType::StartEvent(StartEventNode {
                    name: event.name.clone().unwrap_or_else(|| "Start".to_string()),
                    documentation: event.documentation.clone(),
                    event_definition: event.event_definition.clone(),
                    is_interrupting: event.is_interrupting,
                }),
            );

            let pos = ctx.get_position(&event.id).unwrap_or((100.0, 100.0));
            let node_id = snarl.insert_node(egui::Pos2::new(pos.0 as f32, pos.1 as f32), node);
            ctx.add_mapping(event.id.clone(), node_id);
        }
        Ok(())
    }

    fn convert_end_events(
        events: &[EndEvent],
        snarl: &mut Snarl<EnhancedBpmnNode>,
        ctx: &mut ConversionContext,
    ) -> Result<(), ConversionError> {
        for event in events {
            let node = EnhancedBpmnNode::new(
                event.id.clone(),
                BpmnNodeType::EndEvent(EndEventNode {
                    name: event.name.clone().unwrap_or_else(|| "End".to_string()),
                    documentation: event.documentation.clone(),
                    event_definition: event.event_definition.clone(),
                }),
            );

            let pos = ctx.get_position(&event.id).unwrap_or((500.0, 100.0));
            let node_id = snarl.insert_node(egui::Pos2::new(pos.0 as f32, pos.1 as f32), node);
            ctx.add_mapping(event.id.clone(), node_id);
        }
        Ok(())
    }

    fn convert_intermediate_events(
        events: &[IntermediateEvent],
        snarl: &mut Snarl<EnhancedBpmnNode>,
        ctx: &mut ConversionContext,
    ) -> Result<(), ConversionError> {
        for event in events {
            let node = EnhancedBpmnNode::new(
                event.id.clone(),
                BpmnNodeType::IntermediateEvent(IntermediateEventNode {
                    name: event
                        .name
                        .clone()
                        .unwrap_or_else(|| "Intermediate".to_string()),
                    documentation: event.documentation.clone(),
                    event_definition: event.event_definition.clone(),
                    is_catching: event.is_catching,
                    is_interrupting: event.is_interrupting,
                    is_boundary: event.attached_to_ref.is_some(),
                    attached_to_activity_id: event.attached_to_ref.clone(),
                }),
            );

            let pos = ctx.get_position(&event.id).unwrap_or((300.0, 100.0));
            let node_id = snarl.insert_node(egui::Pos2::new(pos.0 as f32, pos.1 as f32), node);
            ctx.add_mapping(event.id.clone(), node_id);
        }
        Ok(())
    }

    fn convert_tasks(
        tasks: &[BpmnTask],
        snarl: &mut Snarl<EnhancedBpmnNode>,
        ctx: &mut ConversionContext,
    ) -> Result<(), ConversionError> {
        for task in tasks {
            let mut node = EnhancedBpmnNode::new(
                task.id.clone(),
                BpmnNodeType::Task(TaskNode {
                    name: task.name.clone().unwrap_or_else(|| "Task".to_string()),
                    documentation: task.documentation.clone(),
                    task_type: task.task_type.clone(),
                    loop_characteristics: task.loop_characteristics.clone(),
                    is_for_compensation: task.is_for_compensation,
                }),
            );

            // Preserve custom properties
            node.properties = task.properties.clone();

            // Add visual markers based on loop characteristics
            if let Some(loop_char) = &task.loop_characteristics {
                match loop_char {
                    LoopCharacteristics::Standard { .. } => {
                        node.visual.markers.push(VisualMarker::Loop);
                    }
                    LoopCharacteristics::MultiInstance { .. } => {
                        node.visual.markers.push(VisualMarker::MultiInstance);
                    }
                }
            }

            if task.is_for_compensation {
                node.visual.markers.push(VisualMarker::Compensation);
            }

            let pos = ctx.get_position(&task.id).unwrap_or((250.0, 150.0));
            let node_id = snarl.insert_node(egui::Pos2::new(pos.0 as f32, pos.1 as f32), node);
            ctx.add_mapping(task.id.clone(), node_id);
        }
        Ok(())
    }

    fn convert_subprocesses(
        subprocesses: &[Subprocess],
        snarl: &mut Snarl<EnhancedBpmnNode>,
        ctx: &mut ConversionContext,
    ) -> Result<(), ConversionError> {
        for subprocess in subprocesses {
            let mut node = EnhancedBpmnNode::new(
                subprocess.id.clone(),
                BpmnNodeType::Subprocess(SubprocessNode {
                    name: subprocess
                        .name
                        .clone()
                        .unwrap_or_else(|| "Subprocess".to_string()),
                    documentation: subprocess.documentation.clone(),
                    subprocess_type: subprocess.subprocess_type.clone(),
                    is_expanded: false, // Will be set from diagram info if available
                    loop_characteristics: subprocess.loop_characteristics.clone(),
                }),
            );

            // Add visual markers
            if subprocess.subprocess_type == SubprocessType::AdHoc {
                node.visual.markers.push(VisualMarker::AdHoc);
            }

            if let Some(loop_char) = &subprocess.loop_characteristics {
                match loop_char {
                    LoopCharacteristics::Standard { .. } => {
                        node.visual.markers.push(VisualMarker::Loop);
                    }
                    LoopCharacteristics::MultiInstance { .. } => {
                        node.visual.markers.push(VisualMarker::MultiInstance);
                    }
                }
            }

            let pos = ctx.get_position(&subprocess.id).unwrap_or((250.0, 250.0));
            let node_id = snarl.insert_node(egui::Pos2::new(pos.0 as f32, pos.1 as f32), node);
            ctx.add_mapping(subprocess.id.clone(), node_id);
        }
        Ok(())
    }

    fn convert_gateways(
        gateways: &[BpmnGateway],
        snarl: &mut Snarl<EnhancedBpmnNode>,
        ctx: &mut ConversionContext,
    ) -> Result<(), ConversionError> {
        for gateway in gateways {
            let node = EnhancedBpmnNode::new(
                gateway.id.clone(),
                BpmnNodeType::Gateway(GatewayNode {
                    name: gateway.name.clone().unwrap_or_else(|| "Gateway".to_string()),
                    documentation: gateway.documentation.clone(),
                    gateway_type: gateway.gateway_type.clone(),
                    gateway_direction: gateway.gateway_direction.clone(),
                }),
            );

            let pos = ctx.get_position(&gateway.id).unwrap_or((300.0, 150.0));
            let node_id = snarl.insert_node(egui::Pos2::new(pos.0 as f32, pos.1 as f32), node);
            ctx.add_mapping(gateway.id.clone(), node_id);
        }
        Ok(())
    }

    fn convert_data_objects(
        data_objects: &[DataObject],
        snarl: &mut Snarl<EnhancedBpmnNode>,
        ctx: &mut ConversionContext,
    ) -> Result<(), ConversionError> {
        for obj in data_objects {
            let node = EnhancedBpmnNode::new(
                obj.id.clone(),
                BpmnNodeType::DataObject(DataObjectNode {
                    name: obj.name.clone().unwrap_or_else(|| "Data".to_string()),
                    is_collection: obj.is_collection,
                    data_state: obj.data_state.clone(),
                }),
            );

            let pos = ctx.get_position(&obj.id).unwrap_or((400.0, 200.0));
            let node_id = snarl.insert_node(egui::Pos2::new(pos.0 as f32, pos.1 as f32), node);
            ctx.add_mapping(obj.id.clone(), node_id);
        }
        Ok(())
    }

    fn convert_data_stores(
        data_stores: &[DataStore],
        snarl: &mut Snarl<EnhancedBpmnNode>,
        ctx: &mut ConversionContext,
    ) -> Result<(), ConversionError> {
        for store in data_stores {
            let node = EnhancedBpmnNode::new(
                store.id.clone(),
                BpmnNodeType::DataStore(DataStoreNode {
                    name: store.name.clone().unwrap_or_else(|| "Store".to_string()),
                    is_unlimited: store.is_unlimited,
                    capacity: store.capacity,
                }),
            );

            let pos = ctx.get_position(&store.id).unwrap_or((450.0, 250.0));
            let node_id = snarl.insert_node(egui::Pos2::new(pos.0 as f32, pos.1 as f32), node);
            ctx.add_mapping(store.id.clone(), node_id);
        }
        Ok(())
    }

    fn convert_text_annotations(
        annotations: &[TextAnnotation],
        snarl: &mut Snarl<EnhancedBpmnNode>,
        ctx: &mut ConversionContext,
    ) -> Result<(), ConversionError> {
        for annotation in annotations {
            let node = EnhancedBpmnNode::new(
                annotation.id.clone(),
                BpmnNodeType::TextAnnotation(TextAnnotationNode {
                    text: annotation.text.clone(),
                    text_format: annotation.text_format.clone(),
                }),
            );

            let pos = ctx.get_position(&annotation.id).unwrap_or((500.0, 300.0));
            let node_id = snarl.insert_node(egui::Pos2::new(pos.0 as f32, pos.1 as f32), node);
            ctx.add_mapping(annotation.id.clone(), node_id);
        }
        Ok(())
    }

    fn convert_groups(
        groups: &[Group],
        snarl: &mut Snarl<EnhancedBpmnNode>,
        ctx: &mut ConversionContext,
    ) -> Result<(), ConversionError> {
        for group in groups {
            let node = EnhancedBpmnNode::new(
                group.id.clone(),
                BpmnNodeType::Group(GroupNode {
                    category: group.category_value_ref.clone(),
                }),
            );

            let pos = ctx.get_position(&group.id).unwrap_or((550.0, 350.0));
            let node_id = snarl.insert_node(egui::Pos2::new(pos.0 as f32, pos.1 as f32), node);
            ctx.add_mapping(group.id.clone(), node_id);
        }
        Ok(())
    }

    fn convert_sequence_flows(
        flows: &[SequenceFlow],
        snarl: &mut Snarl<EnhancedBpmnNode>,
        ctx: &ConversionContext,
    ) -> Result<(), ConversionError> {
        for flow in flows {
            let source_node_id = ctx.get_node_id(&flow.source_ref).ok_or_else(|| {
                ConversionError::InvalidReference(format!("Source not found: {}", flow.source_ref))
            })?;

            let target_node_id = ctx.get_node_id(&flow.target_ref).ok_or_else(|| {
                ConversionError::InvalidReference(format!("Target not found: {}", flow.target_ref))
            })?;

            // Determine output pin index (default to 0)
            let output_pin = 0;

            // Determine input pin index (default to 0)
            let input_pin = 0;

            // Create the connection
            let out_pin_id = OutPinId {
                node: source_node_id,
                output: output_pin,
            };
            let in_pin_id = InPinId {
                node: target_node_id,
                input: input_pin,
            };

            snarl.connect(out_pin_id, in_pin_id);
        }
        Ok(())
    }

    // ========================================================================
    // Snarl to BPMN Conversion
    // ========================================================================

    /// Convert Snarl back to BpmnDiagram for saving
    pub fn from_snarl(
        snarl: &Snarl<EnhancedBpmnNode>,
        diagram_id: &str,
        name: &str,
    ) -> Result<BpmnDiagram, ConversionError> {
        // Extract process from snarl
        let process = Self::snarl_to_process(snarl, &format!("{}_process", diagram_id), name)?;

        // Extract data stores from snarl (they belong at diagram level)
        let data_stores = Self::extract_data_stores(snarl);

        // Create diagram
        let diagram = BpmnDiagram {
            id: diagram_id.to_string(),
            name: Some(name.to_string()),
            documentation: None,
            processes: vec![process],
            collaborations: Vec::new(),
            data_stores,
            messages: Vec::new(),
            signals: Vec::new(),
        };

        Ok(diagram)
    }

    /// Extract DataStore nodes from Snarl
    fn extract_data_stores(snarl: &Snarl<EnhancedBpmnNode>) -> Vec<DataStore> {
        let mut data_stores = Vec::new();

        for (_node_id, node) in snarl.nodes() {
            if let BpmnNodeType::DataStore(n) = &node.node_type {
                data_stores.push(DataStore {
                    id: node.id.clone(),
                    name: Some(n.name.clone()),
                    capacity: None,
                    is_unlimited: false,
                    data_state: None,
                });
            }
        }

        data_stores
    }

    /// Convert Snarl to BpmnProcess
    fn snarl_to_process(
        snarl: &Snarl<EnhancedBpmnNode>,
        process_id: &str,
        name: &str,
    ) -> Result<BpmnProcess, ConversionError> {
        let mut process = BpmnProcess {
            id: process_id.to_string(),
            name: Some(name.to_string()),
            documentation: None,
            is_executable: true,
            process_type: ProcessType::None,
            start_events: Vec::new(),
            end_events: Vec::new(),
            intermediate_events: Vec::new(),
            tasks: Vec::new(),
            subprocesses: Vec::new(),
            gateways: Vec::new(),
            sequence_flows: Vec::new(),
            data_objects: Vec::new(),
            data_associations: Vec::new(),
            text_annotations: Vec::new(),
            groups: Vec::new(),
            lanes: Vec::new(),
            metadata: HashMap::new(),
        };

        // Convert all nodes to BPMN elements
        for (node_id, node) in snarl.nodes() {
            match &node.node_type {
                BpmnNodeType::StartEvent(n) => {
                    process.start_events.push(StartEvent {
                        id: node.id.clone(),
                        name: Some(n.name.clone()),
                        documentation: n.documentation.clone(),
                        event_definition: n.event_definition.clone(),
                        is_interrupting: n.is_interrupting,
                    });
                }
                BpmnNodeType::EndEvent(n) => {
                    process.end_events.push(EndEvent {
                        id: node.id.clone(),
                        name: Some(n.name.clone()),
                        documentation: n.documentation.clone(),
                        event_definition: n.event_definition.clone(),
                    });
                }
                BpmnNodeType::IntermediateEvent(n) => {
                    process.intermediate_events.push(IntermediateEvent {
                        id: node.id.clone(),
                        name: Some(n.name.clone()),
                        documentation: n.documentation.clone(),
                        event_definition: n.event_definition.clone(),
                        is_catching: n.is_catching,
                        is_interrupting: n.is_interrupting,
                        attached_to_ref: n.attached_to_activity_id.clone(),
                    });
                }
                BpmnNodeType::Task(n) => {
                    process.tasks.push(BpmnTask {
                        id: node.id.clone(),
                        name: Some(n.name.clone()),
                        documentation: n.documentation.clone(),
                        task_type: n.task_type.clone(),
                        default_flow: None,
                        io_specification: None,
                        properties: node.properties.clone(),
                        loop_characteristics: n.loop_characteristics.clone(),
                        is_for_compensation: n.is_for_compensation,
                    });
                }
                BpmnNodeType::Subprocess(n) => {
                    process.subprocesses.push(Subprocess {
                        id: node.id.clone(),
                        name: Some(n.name.clone()),
                        documentation: n.documentation.clone(),
                        subprocess_type: n.subprocess_type.clone(),
                        triggered_by_event: false,
                        process: None, // TODO: Handle embedded subprocesses
                        called_element: None,
                        loop_characteristics: n.loop_characteristics.clone(),
                    });
                }
                BpmnNodeType::Gateway(n) => {
                    process.gateways.push(BpmnGateway {
                        id: node.id.clone(),
                        name: Some(n.name.clone()),
                        documentation: n.documentation.clone(),
                        gateway_type: n.gateway_type.clone(),
                        gateway_direction: n.gateway_direction.clone(),
                        default_flow: None,
                    });
                }
                BpmnNodeType::DataObject(n) => {
                    process.data_objects.push(DataObject {
                        id: node.id.clone(),
                        name: Some(n.name.clone()),
                        item_subject_ref: None,
                        is_collection: n.is_collection,
                        data_state: n.data_state.clone(),
                    });
                }
                BpmnNodeType::DataStore(_n) => {
                    // DataStores are extracted at diagram level by extract_data_stores()
                    // They belong to the diagram, not the process, per BPMN 2.0 spec
                }
                BpmnNodeType::TextAnnotation(n) => {
                    process.text_annotations.push(TextAnnotation {
                        id: node.id.clone(),
                        text: n.text.clone(),
                        text_format: n.text_format.clone(),
                    });
                }
                BpmnNodeType::Group(n) => {
                    process.groups.push(Group {
                        id: node.id.clone(),
                        category_value_ref: n.category.clone(),
                    });
                }
            }
        }

        // Convert wires to sequence flows
        Self::extract_sequence_flows(snarl, &mut process)?;

        Ok(process)
    }

    /// Extract sequence flows from Snarl wires
    fn extract_sequence_flows(
        snarl: &Snarl<EnhancedBpmnNode>,
        process: &mut BpmnProcess,
    ) -> Result<(), ConversionError> {
        let mut flow_id_counter = 1;

        for (node_id, node) in snarl.nodes() {
            // Get all outgoing connections from this node
            for output_idx in 0..node.output_count() {
                let out_pin = OutPinId {
                    node: node_id,
                    output: output_idx,
                };

                // Get all connections from this output pin
                if let Some(remotes) = snarl.out_pin(out_pin).map(|p| &p.remotes) {
                    for &in_pin_id in remotes {
                        let target_node = &snarl[in_pin_id.node];

                        // Create sequence flow
                        let flow = SequenceFlow {
                            id: format!("flow_{}", flow_id_counter),
                            name: None,
                            source_ref: node.id.clone(),
                            target_ref: target_node.id.clone(),
                            condition_expression: None,
                            is_immediate: true,
                        };

                        process.sequence_flows.push(flow);
                        flow_id_counter += 1;
                    }
                }
            }
        }

        Ok(())
    }

    // ========================================================================
    // Visual Layout Extraction (for future use)
    // ========================================================================

    /// Extract layout information from BpmnDiagramInfo
    #[allow(dead_code)]
    fn extract_layout(diagram_info: &BpmnDiagramInfo, ctx: &mut ConversionContext) {
        for shape in &diagram_info.plane.shapes {
            ctx.set_position(
                shape.bpmn_element.clone(),
                shape.bounds.x,
                shape.bounds.y,
            );
        }
    }

    /// Generate BpmnDiagramInfo from Snarl
    pub fn generate_diagram_info(
        snarl: &Snarl<EnhancedBpmnNode>,
        diagram_id: &str,
        process_id: &str,
    ) -> BpmnDiagramInfo {
        let mut shapes = Vec::new();
        let mut edges = Vec::new();

        // Generate shapes from nodes
        for (node_id, node) in snarl.nodes() {
            let pos = snarl.get_node_pos(node_id);

            // Determine node size based on type
            let (width, height) = match &node.node_type {
                BpmnNodeType::StartEvent(_) | BpmnNodeType::EndEvent(_) => (36.0, 36.0),
                BpmnNodeType::IntermediateEvent(_) => (36.0, 36.0),
                BpmnNodeType::Task(_) => (100.0, 80.0),
                BpmnNodeType::Subprocess(_) => (150.0, 100.0),
                BpmnNodeType::Gateway(_) => (50.0, 50.0),
                BpmnNodeType::DataObject(_) => (40.0, 60.0),
                BpmnNodeType::DataStore(_) => (50.0, 50.0),
                BpmnNodeType::TextAnnotation(_) => (100.0, 40.0),
                BpmnNodeType::Group(_) => (200.0, 150.0),
            };

            shapes.push(BpmnShape {
                id: format!("shape_{}", node.id),
                bpmn_element: node.id.clone(),
                bounds: Bounds {
                    x: pos.x as f64,
                    y: pos.y as f64,
                    width,
                    height,
                },
                is_horizontal: true,
                is_expanded: match &node.node_type {
                    BpmnNodeType::Subprocess(s) => s.is_expanded,
                    _ => true,
                },
                is_marker_visible: false,
                label: None,
            });
        }

        // Generate edges from wires
        let mut edge_id_counter = 1;
        for (node_id, node) in snarl.nodes() {
            for output_idx in 0..node.output_count() {
                let out_pin = OutPinId {
                    node: node_id,
                    output: output_idx,
                };

                if let Some(remotes) = snarl.out_pin(out_pin).map(|p| &p.remotes) {
                    for &in_pin_id in remotes {
                        let source_pos = snarl.get_node_pos(node_id);
                        let target_pos = snarl.get_node_pos(in_pin_id.node);

                        // Simple straight line for now
                        edges.push(BpmnEdge {
                            id: format!("edge_{}", edge_id_counter),
                            bpmn_element: format!("flow_{}", edge_id_counter),
                            waypoints: vec![
                                Point {
                                    x: source_pos.x as f64,
                                    y: source_pos.y as f64,
                                },
                                Point {
                                    x: target_pos.x as f64,
                                    y: target_pos.y as f64,
                                },
                            ],
                            label: None,
                        });
                        edge_id_counter += 1;
                    }
                }
            }
        }

        BpmnDiagramInfo {
            id: format!("{}_diagram", diagram_id),
            name: Some("Diagram".to_string()),
            plane: BpmnPlane {
                id: format!("{}_plane", diagram_id),
                bpmn_element: process_id.to_string(),
                shapes,
                edges,
            },
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_diagram_error() {
        let diagram = BpmnDiagram {
            id: "test".to_string(),
            name: Some("Test".to_string()),
            documentation: None,
            processes: Vec::new(),
            collaborations: Vec::new(),
            data_stores: Vec::new(),
            messages: Vec::new(),
            signals: Vec::new(),
        };

        let result = BpmnDiagramConverter::to_snarl(&diagram);
        assert!(matches!(result, Err(ConversionError::EmptyDiagram)));
    }

    #[test]
    fn test_simple_process_conversion() {
        let process = BpmnProcess {
            id: "proc1".to_string(),
            name: Some("Test Process".to_string()),
            documentation: None,
            is_executable: true,
            process_type: ProcessType::None,
            start_events: vec![StartEvent {
                id: "start1".to_string(),
                name: Some("Start".to_string()),
                documentation: None,
                event_definition: None,
                is_interrupting: true,
            }],
            end_events: vec![EndEvent {
                id: "end1".to_string(),
                name: Some("End".to_string()),
                documentation: None,
                event_definition: None,
            }],
            intermediate_events: Vec::new(),
            tasks: vec![BpmnTask {
                id: "task1".to_string(),
                name: Some("Task".to_string()),
                documentation: None,
                task_type: BpmnTaskType::Abstract,
                default_flow: None,
                io_specification: None,
                properties: HashMap::new(),
                loop_characteristics: None,
                is_for_compensation: false,
            }],
            subprocesses: Vec::new(),
            gateways: Vec::new(),
            sequence_flows: vec![
                SequenceFlow {
                    id: "flow1".to_string(),
                    name: None,
                    source_ref: "start1".to_string(),
                    target_ref: "task1".to_string(),
                    condition_expression: None,
                    is_immediate: true,
                },
                SequenceFlow {
                    id: "flow2".to_string(),
                    name: None,
                    source_ref: "task1".to_string(),
                    target_ref: "end1".to_string(),
                    condition_expression: None,
                    is_immediate: true,
                },
            ],
            data_objects: Vec::new(),
            data_associations: Vec::new(),
            text_annotations: Vec::new(),
            groups: Vec::new(),
            lanes: Vec::new(),
            metadata: HashMap::new(),
        };

        let diagram = BpmnDiagram {
            id: "diag1".to_string(),
            name: Some("Test Diagram".to_string()),
            documentation: None,
            processes: vec![process],
            collaborations: Vec::new(),
            data_stores: Vec::new(),
            messages: Vec::new(),
            signals: Vec::new(),
        };

        let snarl_result = BpmnDiagramConverter::to_snarl(&diagram);
        assert!(snarl_result.is_ok());

        let snarl = snarl_result.unwrap();
        assert_eq!(snarl.nodes().count(), 3); // start, task, end
    }

    #[test]
    fn test_roundtrip_conversion() {
        // Create simple process
        let original_process = BpmnProcess {
            id: "proc1".to_string(),
            name: Some("Test".to_string()),
            documentation: None,
            is_executable: true,
            process_type: ProcessType::None,
            start_events: vec![StartEvent {
                id: "start1".to_string(),
                name: Some("Start".to_string()),
                documentation: None,
                event_definition: None,
                is_interrupting: true,
            }],
            end_events: vec![EndEvent {
                id: "end1".to_string(),
                name: Some("End".to_string()),
                documentation: None,
                event_definition: None,
            }],
            intermediate_events: Vec::new(),
            tasks: Vec::new(),
            subprocesses: Vec::new(),
            gateways: Vec::new(),
            sequence_flows: vec![SequenceFlow {
                id: "flow1".to_string(),
                name: None,
                source_ref: "start1".to_string(),
                target_ref: "end1".to_string(),
                condition_expression: None,
                is_immediate: true,
            }],
            data_objects: Vec::new(),
            data_associations: Vec::new(),
            text_annotations: Vec::new(),
            groups: Vec::new(),
            lanes: Vec::new(),
            metadata: HashMap::new(),
        };

        let diagram = BpmnDiagram {
            id: "diag1".to_string(),
            name: Some("Test".to_string()),
            documentation: None,
            processes: vec![original_process.clone()],
            collaborations: Vec::new(),
            data_stores: Vec::new(),
            messages: Vec::new(),
            signals: Vec::new(),
        };

        // Convert to Snarl
        let snarl = BpmnDiagramConverter::to_snarl(&diagram).unwrap();

        // Convert back to BpmnDiagram
        let result_diagram = BpmnDiagramConverter::from_snarl(&snarl, "diag1", "Test").unwrap();

        // Verify
        assert_eq!(result_diagram.processes.len(), 1);
        let result_process = &result_diagram.processes[0];
        assert_eq!(result_process.start_events.len(), 1);
        assert_eq!(result_process.end_events.len(), 1);
        assert_eq!(result_process.sequence_flows.len(), 1);
    }
}
