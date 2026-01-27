//! BPMN Editor using egui-snarl
//!
//! Professional node-graph based BPMN editor with full support for:
//! - All BPMN 2.0 node types (Tasks, Gateways, Events)
//! - Visual connections (sequence flows)
//! - Context menus
//! - Node properties

use crate::bpmn::process::{GatewayType, TaskType};
use egui::{Color32, Ui};
use egui_snarl::{
    ui::{PinInfo, SnarlStyle, SnarlViewer},
    InPin, NodeId, OutPin, Snarl,
};

// BPMN standard colors
const TASK_COLOR: Color32 = Color32::from_rgb(173, 216, 230); // Light blue
const SERVICE_COLOR: Color32 = Color32::from_rgb(255, 228, 181); // Moccasin
const GATEWAY_COLOR: Color32 = Color32::from_rgb(255, 255, 153); // Light yellow
const START_EVENT_COLOR: Color32 = Color32::from_rgb(144, 238, 144); // Light green
const END_EVENT_COLOR: Color32 = Color32::from_rgb(255, 160, 160); // Light red
const FLOW_COLOR: Color32 = Color32::from_rgb(100, 100, 100); // Gray

/// BPMN Node types for the node graph
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum BpmnNode {
    /// Start Event - process begins
    StartEvent { name: String },

    /// End Event - process ends
    EndEvent { name: String },

    /// Task node with specific type
    Task { name: String, task_type: TaskType, description: Option<String> },

    /// Gateway for flow control
    Gateway { name: String, gateway_type: GatewayType },

    /// Intermediate Event
    IntermediateEvent { name: String, event_type: String },
}

impl BpmnNode {
    /// Get the display name of the node
    pub fn name(&self) -> &str {
        match self {
            BpmnNode::StartEvent { name } => name,
            BpmnNode::EndEvent { name } => name,
            BpmnNode::Task { name, .. } => name,
            BpmnNode::Gateway { name, .. } => name,
            BpmnNode::IntermediateEvent { name, .. } => name,
        }
    }

    /// Set the name of the node
    pub fn set_name(&mut self, new_name: String) {
        match self {
            BpmnNode::StartEvent { name } => *name = new_name,
            BpmnNode::EndEvent { name } => *name = new_name,
            BpmnNode::Task { name, .. } => *name = new_name,
            BpmnNode::Gateway { name, .. } => *name = new_name,
            BpmnNode::IntermediateEvent { name, .. } => *name = new_name,
        }
    }

    /// Get the number of input pins (connections)
    pub fn input_count(&self) -> usize {
        match self {
            BpmnNode::StartEvent { .. } => 0, // Start events have no inputs
            BpmnNode::EndEvent { .. } => 1,
            BpmnNode::Task { .. } => 1,
            BpmnNode::Gateway { .. } => 1, // Can be extended for parallel gateways
            BpmnNode::IntermediateEvent { .. } => 1,
        }
    }

    /// Get the number of output pins (connections)
    pub fn output_count(&self) -> usize {
        match self {
            BpmnNode::StartEvent { .. } => 1,
            BpmnNode::EndEvent { .. } => 0, // End events have no outputs
            BpmnNode::Task { .. } => 1,
            BpmnNode::Gateway { gateway_type, .. } => {
                match gateway_type {
                    GatewayType::Exclusive => 2, // XOR - choose one path
                    GatewayType::Parallel => 2,  // AND - all paths
                    GatewayType::Inclusive => 2, // OR - one or more
                    GatewayType::EventBased => 2,
                }
            },
            BpmnNode::IntermediateEvent { .. } => 1,
        }
    }

    /// Get the color for this node type
    pub fn color(&self) -> Color32 {
        match self {
            BpmnNode::StartEvent { .. } => START_EVENT_COLOR,
            BpmnNode::EndEvent { .. } => END_EVENT_COLOR,
            BpmnNode::Task { task_type, .. } => match task_type {
                TaskType::User => TASK_COLOR,
                TaskType::Service => SERVICE_COLOR,
                TaskType::Script => Color32::from_rgb(216, 191, 216),
                TaskType::Manual => Color32::from_rgb(255, 250, 205),
                TaskType::Send => Color32::from_rgb(176, 224, 230),
                TaskType::Receive => Color32::from_rgb(221, 160, 221),
            },
            BpmnNode::Gateway { .. } => GATEWAY_COLOR,
            BpmnNode::IntermediateEvent { .. } => Color32::from_rgb(255, 255, 200),
        }
    }

    /// Get node type name for display
    pub fn type_name(&self) -> &str {
        match self {
            BpmnNode::StartEvent { .. } => "Start Event",
            BpmnNode::EndEvent { .. } => "End Event",
            BpmnNode::Task { task_type, .. } => match task_type {
                TaskType::User => "User Task",
                TaskType::Service => "Service Task",
                TaskType::Script => "Script Task",
                TaskType::Manual => "Manual Task",
                TaskType::Send => "Send Task",
                TaskType::Receive => "Receive Task",
            },
            BpmnNode::Gateway { gateway_type, .. } => match gateway_type {
                GatewayType::Exclusive => "Exclusive Gateway",
                GatewayType::Parallel => "Parallel Gateway",
                GatewayType::Inclusive => "Inclusive Gateway",
                GatewayType::EventBased => "Event-Based Gateway",
            },
            BpmnNode::IntermediateEvent { .. } => "Intermediate Event",
        }
    }
}

/// BPMN Viewer implementation for egui-snarl
pub struct BpmnViewer {
    /// Currently selected nodes for multi-selection
    pub selected_nodes: Vec<NodeId>,
}

impl BpmnViewer {
    pub fn new() -> Self {
        Self { selected_nodes: Vec::new() }
    }
}

impl Default for BpmnViewer {
    fn default() -> Self {
        Self::new()
    }
}

impl SnarlViewer<BpmnNode> for BpmnViewer {
    fn title(&mut self, node: &BpmnNode) -> String {
        format!("{}", node.name())
    }

    fn inputs(&mut self, node: &BpmnNode) -> usize {
        node.input_count()
    }

    fn outputs(&mut self, node: &BpmnNode) -> usize {
        node.output_count()
    }

    fn show_input(
        &mut self,
        pin: &InPin,
        ui: &mut Ui,
        snarl: &mut Snarl<BpmnNode>,
    ) -> impl egui_snarl::ui::SnarlPin + 'static {
        let node = &snarl[pin.id.node];

        // Show input pin label
        match node {
            BpmnNode::EndEvent { .. } => {
                ui.label("incoming");
            },
            BpmnNode::Task { .. } => {
                ui.label("in");
            },
            BpmnNode::Gateway { .. } => {
                ui.label("in");
            },
            BpmnNode::IntermediateEvent { .. } => {
                ui.label("in");
            },
            _ => {},
        }

        PinInfo::circle().with_fill(FLOW_COLOR)
    }

    fn show_output(
        &mut self,
        pin: &OutPin,
        ui: &mut Ui,
        snarl: &mut Snarl<BpmnNode>,
    ) -> impl egui_snarl::ui::SnarlPin + 'static {
        let node = &snarl[pin.id.node];

        // Show output pin label
        match node {
            BpmnNode::StartEvent { .. } => {
                ui.label("outgoing");
            },
            BpmnNode::Task { .. } => {
                ui.label("out");
            },
            BpmnNode::Gateway { gateway_type, .. } => {
                match pin.id.output {
                    0 => match gateway_type {
                        GatewayType::Exclusive => ui.label("option A"),
                        GatewayType::Parallel => ui.label("parallel 1"),
                        _ => ui.label("out 1"),
                    },
                    1 => match gateway_type {
                        GatewayType::Exclusive => ui.label("option B"),
                        GatewayType::Parallel => ui.label("parallel 2"),
                        _ => ui.label("out 2"),
                    },
                    _ => ui.label("out"),
                };
            },
            BpmnNode::IntermediateEvent { .. } => {
                ui.label("out");
            },
            _ => {},
        }

        PinInfo::circle().with_fill(FLOW_COLOR)
    }

    fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<BpmnNode>) {
        // Validate BPMN connection rules
        let from_node = &snarl[from.id.node];
        let to_node = &snarl[to.id.node];

        // Check if connection is valid
        match (from_node, to_node) {
            // Can't connect from end event (no outputs)
            (BpmnNode::EndEvent { .. }, _) => return,
            // Can't connect to start event (no inputs)
            (_, BpmnNode::StartEvent { .. }) => return,
            // All other connections are valid
            _ => {},
        }

        // Disconnect existing connections to this input if needed
        // (BPMN typically allows only one incoming connection per node)
        for &remote in &to.remotes {
            snarl.disconnect(remote, to.id);
        }

        // Create the connection
        snarl.connect(from.id, to.id);
    }

    fn has_body(&mut self, _node: &BpmnNode) -> bool {
        true
    }

    fn show_body(
        &mut self,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<BpmnNode>,
    ) {
        let bpmn_node = &snarl[node];

        // Show node-specific content
        match bpmn_node {
            BpmnNode::Task { description, task_type, .. } => {
                ui.horizontal(|ui| {
                    ui.label("Type:");
                    ui.label(format!("{:?}", task_type));
                });

                if let Some(desc) = description {
                    if !desc.is_empty() {
                        ui.label(desc);
                    }
                }
            },
            BpmnNode::Gateway { gateway_type, .. } => {
                ui.label(format!("{:?}", gateway_type));
            },
            BpmnNode::IntermediateEvent { event_type, .. } => {
                ui.label(format!("Event: {}", event_type));
            },
            _ => {
                // Start and End events just show their type
                ui.label(bpmn_node.type_name());
            },
        }
    }

    fn has_graph_menu(&mut self, _pos: egui::Pos2, _snarl: &mut Snarl<BpmnNode>) -> bool {
        true
    }

    fn show_graph_menu(&mut self, pos: egui::Pos2, ui: &mut Ui, snarl: &mut Snarl<BpmnNode>) {
        ui.label("Add Node:");
        ui.separator();

        if ui.button("▶ Start Event").clicked() {
            let node = BpmnNode::StartEvent { name: "Start".to_string() };
            snarl.insert_node(pos, node);
            ui.close();
        }

        if ui.button("⏹ End Event").clicked() {
            let node = BpmnNode::EndEvent { name: "End".to_string() };
            snarl.insert_node(pos, node);
            ui.close();
        }

        ui.separator();
        ui.label("Tasks:");

        if ui.button("👤 User Task").clicked() {
            let node = BpmnNode::Task {
                name: "User Task".to_string(),
                task_type: TaskType::User,
                description: None,
            };
            snarl.insert_node(pos, node);
            ui.close();
        }

        if ui.button("⚙ Service Task").clicked() {
            let node = BpmnNode::Task {
                name: "Service Task".to_string(),
                task_type: TaskType::Service,
                description: None,
            };
            snarl.insert_node(pos, node);
            ui.close();
        }

        if ui.button("📜 Script Task").clicked() {
            let node = BpmnNode::Task {
                name: "Script Task".to_string(),
                task_type: TaskType::Script,
                description: None,
            };
            snarl.insert_node(pos, node);
            ui.close();
        }

        if ui.button("✋ Manual Task").clicked() {
            let node = BpmnNode::Task {
                name: "Manual Task".to_string(),
                task_type: TaskType::Manual,
                description: None,
            };
            snarl.insert_node(pos, node);
            ui.close();
        }

        if ui.button("📤 Send Task").clicked() {
            let node = BpmnNode::Task {
                name: "Send Task".to_string(),
                task_type: TaskType::Send,
                description: None,
            };
            snarl.insert_node(pos, node);
            ui.close();
        }

        if ui.button("📥 Receive Task").clicked() {
            let node = BpmnNode::Task {
                name: "Receive Task".to_string(),
                task_type: TaskType::Receive,
                description: None,
            };
            snarl.insert_node(pos, node);
            ui.close();
        }

        ui.separator();
        ui.label("Gateways:");

        if ui.button("✕ Exclusive (XOR)").clicked() {
            let node = BpmnNode::Gateway {
                name: "XOR Gateway".to_string(),
                gateway_type: GatewayType::Exclusive,
            };
            snarl.insert_node(pos, node);
            ui.close();
        }

        if ui.button("+ Parallel (AND)").clicked() {
            let node = BpmnNode::Gateway {
                name: "AND Gateway".to_string(),
                gateway_type: GatewayType::Parallel,
            };
            snarl.insert_node(pos, node);
            ui.close();
        }

        if ui.button("○ Inclusive (OR)").clicked() {
            let node = BpmnNode::Gateway {
                name: "OR Gateway".to_string(),
                gateway_type: GatewayType::Inclusive,
            };
            snarl.insert_node(pos, node);
            ui.close();
        }

        if ui.button("⬡ Event-Based").clicked() {
            let node = BpmnNode::Gateway {
                name: "Event Gateway".to_string(),
                gateway_type: GatewayType::EventBased,
            };
            snarl.insert_node(pos, node);
            ui.close();
        }
    }

    fn has_node_menu(&mut self, _node: &BpmnNode) -> bool {
        true
    }

    fn show_node_menu(
        &mut self,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<BpmnNode>,
    ) {
        let bpmn_node = &snarl[node];

        ui.label(format!("Node: {}", bpmn_node.name()));
        ui.separator();

        if ui.button("✏ Rename").clicked() {
            // This would open a rename dialog
            ui.close();
        }

        // Show type-specific options
        match bpmn_node {
            BpmnNode::Task { task_type: _, .. } => {
                ui.menu_button("Change Type", |ui| {
                    if ui.button("User Task").clicked() {
                        if let BpmnNode::Task { name, description, .. } = snarl[node].clone() {
                            snarl[node] =
                                BpmnNode::Task { name, task_type: TaskType::User, description };
                        }
                        ui.close();
                    }
                    if ui.button("Service Task").clicked() {
                        if let BpmnNode::Task { name, description, .. } = snarl[node].clone() {
                            snarl[node] =
                                BpmnNode::Task { name, task_type: TaskType::Service, description };
                        }
                        ui.close();
                    }
                    if ui.button("Script Task").clicked() {
                        if let BpmnNode::Task { name, description, .. } = snarl[node].clone() {
                            snarl[node] =
                                BpmnNode::Task { name, task_type: TaskType::Script, description };
                        }
                        ui.close();
                    }
                });
            },
            BpmnNode::Gateway { .. } => {
                ui.menu_button("Change Type", |ui| {
                    if ui.button("Exclusive (XOR)").clicked() {
                        if let BpmnNode::Gateway { name, .. } = snarl[node].clone() {
                            snarl[node] =
                                BpmnNode::Gateway { name, gateway_type: GatewayType::Exclusive };
                        }
                        ui.close();
                    }
                    if ui.button("Parallel (AND)").clicked() {
                        if let BpmnNode::Gateway { name, .. } = snarl[node].clone() {
                            snarl[node] =
                                BpmnNode::Gateway { name, gateway_type: GatewayType::Parallel };
                        }
                        ui.close();
                    }
                });
            },
            _ => {},
        }

        ui.separator();

        if ui.button("📋 Properties").clicked() {
            // This would open properties panel
            ui.close();
        }

        if ui.button("📄 Duplicate").clicked() {
            let new_node = snarl[node].clone();
            let current_pos = snarl.get_node_info(node).expect("Node exists").pos;
            let new_pos = current_pos + egui::vec2(50.0, 50.0);
            snarl.insert_node(new_pos, new_node);
            ui.close();
        }

        ui.separator();

        if ui.button("🗑 Delete").clicked() {
            snarl.remove_node(node);
            ui.close();
        }
    }
}

/// Get default BPMN style for the Snarl widget
pub fn bpmn_style() -> SnarlStyle {
    SnarlStyle::default()
}
