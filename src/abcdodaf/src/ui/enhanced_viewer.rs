//! Enhanced BPMN Viewer for egui-snarl
//!
//! Implements SnarlViewer for EnhancedBpmnNode with full BPMN 2.0 support

use super::bpmn_shapes;
use super::enhanced_nodes::*;
use crate::bpmn::elements::*;
use egui::{Color32, FontId, RichText, Ui};
use egui_snarl::{
    ui::{PinInfo, SnarlStyle, SnarlViewer},
    InPin, NodeId, OutPin, Snarl,
};

const FLOW_COLOR: Color32 = Color32::from_rgb(100, 100, 100);

/// Enhanced BPMN Viewer with full BPMN 2.0 support
pub struct EnhancedBpmnViewer {
    /// Currently selected nodes
    pub selected_nodes: Vec<NodeId>,

    /// Property panel state
    pub show_properties: bool,
    pub selected_node_for_properties: Option<NodeId>,

    /// Validation result for displaying errors/warnings
    pub validation_result: Option<crate::ui::validation::ValidationResult>,
}

impl EnhancedBpmnViewer {
    pub fn new() -> Self {
        Self {
            selected_nodes: Vec::new(),
            show_properties: false,
            selected_node_for_properties: None,
            validation_result: None,
        }
    }

    /// Set validation result for error highlighting
    pub fn set_validation_result(&mut self, result: crate::ui::validation::ValidationResult) {
        self.validation_result = Some(result);
    }

    /// Clear validation result
    pub fn clear_validation_result(&mut self) {
        self.validation_result = None;
    }
}

impl Default for EnhancedBpmnViewer {
    fn default() -> Self {
        Self::new()
    }
}

impl SnarlViewer<EnhancedBpmnNode> for EnhancedBpmnViewer {
    fn title(&mut self, node: &EnhancedBpmnNode) -> String {
        node.name().to_string()
    }

    fn inputs(&mut self, node: &EnhancedBpmnNode) -> usize {
        node.input_count()
    }

    fn outputs(&mut self, node: &EnhancedBpmnNode) -> usize {
        node.output_count()
    }

    fn show_input(
        &mut self,
        pin: &InPin,
        ui: &mut Ui,
        snarl: &mut Snarl<EnhancedBpmnNode>,
    ) -> impl egui_snarl::ui::SnarlPin + 'static {
        let node = &snarl[pin.id.node];

        // Show input pin label based on node type
        match &node.node_type {
            BpmnNodeType::EndEvent(_) => {
                ui.label("in");
            },
            BpmnNodeType::Task(_) => {
                ui.label("in");
            },
            BpmnNodeType::Gateway(_) => {
                ui.label("in");
            },
            BpmnNodeType::IntermediateEvent(e) => {
                if e.is_catching {
                    ui.label("trigger");
                } else {
                    ui.label("in");
                }
            },
            BpmnNodeType::Subprocess(_) => {
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
        snarl: &mut Snarl<EnhancedBpmnNode>,
    ) -> impl egui_snarl::ui::SnarlPin + 'static {
        let node = &snarl[pin.id.node];

        // Show output pin label based on node type
        match &node.node_type {
            BpmnNodeType::StartEvent(_) => {
                ui.label("start");
            },
            BpmnNodeType::Task(_) => {
                ui.label("out");
            },
            BpmnNodeType::Gateway(g) => match &g.gateway_type {
                BpmnGatewayType::Exclusive => {
                    if pin.id.output == 0 {
                        ui.label("yes");
                    } else {
                        ui.label("no");
                    }
                },
                BpmnGatewayType::Parallel => {
                    ui.label(format!("branch {}", pin.id.output + 1));
                },
                BpmnGatewayType::Inclusive => {
                    ui.label(format!("option {}", pin.id.output + 1));
                },
                _ => {
                    ui.label("out");
                },
            },
            BpmnNodeType::IntermediateEvent(_) => {
                ui.label("out");
            },
            BpmnNodeType::Subprocess(_) => {
                ui.label("out");
            },
            _ => {},
        }

        PinInfo::circle().with_fill(FLOW_COLOR)
    }

    fn connect(&mut self, from: &OutPin, to: &InPin, snarl: &mut Snarl<EnhancedBpmnNode>) {
        let from_node = &snarl[from.id.node];
        let to_node = &snarl[to.id.node];

        // Validate BPMN connection rules
        match (&from_node.node_type, &to_node.node_type) {
            // Can't connect from end event
            (BpmnNodeType::EndEvent(_), _) => return,
            // Can't connect to start event
            (_, BpmnNodeType::StartEvent(_)) => return,
            // Can't connect data objects to control flow
            (BpmnNodeType::DataObject(_), _) | (_, BpmnNodeType::DataObject(_)) => return,
            (BpmnNodeType::DataStore(_), _) | (_, BpmnNodeType::DataStore(_)) => return,
            // All other connections are valid
            _ => {},
        }

        // Disconnect existing connections to this input
        for &remote in &to.remotes {
            snarl.disconnect(remote, to.id);
        }

        // Create the connection
        snarl.connect(from.id, to.id);
    }

    fn has_body(&mut self, _node: &EnhancedBpmnNode) -> bool {
        true
    }

    fn show_body(
        &mut self,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<EnhancedBpmnNode>,
    ) {
        let enhanced_node = &snarl[node];

        // Determine appropriate size for each node type
        let (width, height) = match &enhanced_node.node_type {
            BpmnNodeType::StartEvent(_)
            | BpmnNodeType::EndEvent(_)
            | BpmnNodeType::IntermediateEvent(_) => {
                (60.0, 60.0) // Circular events
            },
            BpmnNodeType::Gateway(_) => {
                (60.0, 60.0) // Diamond gateways
            },
            BpmnNodeType::Task(_) | BpmnNodeType::Subprocess(_) => {
                (100.0, 60.0) // Rectangular tasks
            },
            BpmnNodeType::DataObject(_) => {
                (50.0, 70.0) // Tall data objects
            },
            BpmnNodeType::DataStore(_) => {
                (60.0, 50.0) // Wide data stores
            },
            _ => (80.0, 60.0), // Default
        };

        // Allocate space for the shape
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());

        // Show validation error tooltips
        if let Some(ref validation) = self.validation_result {
            if response.hovered() {
                let errors = crate::ui::validation::Validator::node_errors(validation, node);
                let warnings = crate::ui::validation::Validator::node_warnings(validation, node);

                if !errors.is_empty() || !warnings.is_empty() {
                    response.on_hover_ui(|ui| {
                        if !errors.is_empty() {
                            ui.heading("❌ Errors:");
                            for error in &errors {
                                ui.label(
                                    RichText::new(crate::ui::validation::Validator::error_message(
                                        error,
                                    ))
                                    .color(Color32::from_rgb(255, 0, 0)),
                                );
                            }
                        }
                        if !warnings.is_empty() {
                            if !errors.is_empty() {
                                ui.separator();
                            }
                            ui.heading("⚠ Warnings:");
                            for warning in warnings {
                                ui.label(
                                    RichText::new(
                                        crate::ui::validation::Validator::warning_message(warning),
                                    )
                                    .color(Color32::from_rgb(255, 200, 0)),
                                );
                            }
                        }
                    });
                }
            }
        }

        let painter = ui.painter();

        // Check for validation errors/warnings
        let has_error = if let Some(ref validation) = self.validation_result {
            crate::ui::validation::Validator::node_has_error(validation, node)
        } else {
            false
        };

        let has_warning = if let Some(ref validation) = self.validation_result {
            crate::ui::validation::Validator::node_has_warning(validation, node)
        } else {
            false
        };

        // Draw error border if node has validation errors
        if has_error {
            // Red border for errors
            painter.rect_stroke(
                rect.expand(3.0),
                4.0,
                egui::Stroke::new(2.5, Color32::from_rgb(255, 0, 0)),
                egui::epaint::StrokeKind::Outside,
            );
        } else if has_warning {
            // Yellow border for warnings
            painter.rect_stroke(
                rect.expand(3.0),
                4.0,
                egui::Stroke::new(2.0, Color32::from_rgb(255, 200, 0)),
                egui::epaint::StrokeKind::Outside,
            );
        }

        // Draw custom BPMN shape based on node type
        match &enhanced_node.node_type {
            BpmnNodeType::StartEvent(_) => {
                bpmn_shapes::draw_start_event(painter, rect, Color32::from_rgb(0, 128, 0));
            },
            BpmnNodeType::EndEvent(_) => {
                bpmn_shapes::draw_end_event(painter, rect, Color32::from_rgb(200, 0, 0));
            },
            BpmnNodeType::IntermediateEvent(_) => {
                bpmn_shapes::draw_intermediate_event(painter, rect, Color32::from_rgb(255, 140, 0));
            },
            BpmnNodeType::Gateway(g) => {
                let fill = Color32::from_rgb(255, 255, 200);
                let stroke = Color32::from_rgb(200, 150, 0);
                bpmn_shapes::draw_gateway(painter, rect, stroke, fill);

                // Draw gateway symbol
                match g.gateway_type {
                    BpmnGatewayType::Exclusive => {
                        bpmn_shapes::draw_exclusive_gateway_symbol(painter, rect, stroke);
                    },
                    BpmnGatewayType::Parallel => {
                        bpmn_shapes::draw_parallel_gateway_symbol(painter, rect, stroke);
                    },
                    BpmnGatewayType::Inclusive => {
                        bpmn_shapes::draw_inclusive_gateway_symbol(painter, rect, stroke);
                    },
                    BpmnGatewayType::EventBased { .. } => {
                        bpmn_shapes::draw_event_based_gateway_symbol(painter, rect, stroke);
                    },
                    BpmnGatewayType::Complex { .. } => {
                        bpmn_shapes::draw_complex_gateway_symbol(painter, rect, stroke);
                    },
                    _ => {},
                }
            },
            BpmnNodeType::Task(_) => {
                let fill = Color32::from_rgb(255, 250, 240);
                let stroke = Color32::from_rgb(100, 100, 200);
                bpmn_shapes::draw_task(painter, rect, stroke, fill);
            },
            BpmnNodeType::Subprocess(_) => {
                let fill = Color32::from_rgb(255, 250, 240);
                let stroke = Color32::from_rgb(100, 100, 200);
                bpmn_shapes::draw_task(painter, rect, stroke, fill);
                bpmn_shapes::draw_subprocess_indicator(painter, rect, stroke);
            },
            BpmnNodeType::DataObject(_) => {
                let fill = Color32::from_rgb(240, 248, 255);
                let stroke = Color32::from_rgb(100, 149, 237);
                bpmn_shapes::draw_data_object(painter, rect, stroke, fill);
            },
            BpmnNodeType::DataStore(_) => {
                let fill = Color32::from_rgb(224, 255, 255);
                let stroke = Color32::from_rgb(0, 139, 139);
                bpmn_shapes::draw_data_store(painter, rect, stroke, fill);
            },
            _ => {},
        }

        // Show node type label below shape
        ui.label(
            RichText::new(enhanced_node.type_name())
                .font(FontId::proportional(10.0))
                .color(Color32::DARK_GRAY),
        );

        // Show visual markers
        if !enhanced_node.visual.markers.is_empty() {
            ui.horizontal(|ui| {
                for marker in &enhanced_node.visual.markers {
                    let marker_text = match marker {
                        VisualMarker::Loop => "↻",
                        VisualMarker::MultiInstance => "|||",
                        VisualMarker::Compensation => "⏪",
                        VisualMarker::AdHoc => "~",
                        VisualMarker::Collapsed => "+",
                    };
                    ui.label(RichText::new(marker_text).strong());
                }
            });
        }

        // Show type-specific content
        match &enhanced_node.node_type {
            BpmnNodeType::Task(t) => {
                if let Some(doc) = &t.documentation {
                    if !doc.is_empty() {
                        ui.label(RichText::new(doc).font(FontId::proportional(9.0)).italics());
                    }
                }
            },
            BpmnNodeType::Gateway(g) => {
                ui.label(format!("{:?}", g.gateway_direction));
            },
            BpmnNodeType::IntermediateEvent(e) => {
                if let Some(def) = &e.event_definition {
                    let event_type = match def {
                        EventDefinition::Message { .. } => "✉ Message",
                        EventDefinition::Timer { .. } => "⏰ Timer",
                        EventDefinition::Signal { .. } => "📡 Signal",
                        EventDefinition::Error { .. } => "⚡ Error",
                        EventDefinition::Conditional { .. } => "? Conditional",
                        EventDefinition::Link { .. } => "🔗 Link",
                        EventDefinition::Terminate => "⏹ Terminate",
                        _ => "Event",
                    };
                    ui.label(event_type);
                }
            },
            BpmnNodeType::DataObject(d) => {
                // Data objects with light blue background
                egui::Frame::new()
                    .fill(Color32::from_rgb(240, 248, 255))
                    .stroke(egui::Stroke::new(1.5, Color32::from_rgb(100, 149, 237)))
                    .inner_margin(egui::Margin::same(6))
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            if d.is_collection {
                                ui.label(
                                    RichText::new("📚 Collection")
                                        .strong()
                                        .color(Color32::from_rgb(70, 130, 180)),
                                );
                            }
                            if let Some(state) = &d.data_state {
                                ui.label(
                                    RichText::new(format!("State: {}", state))
                                        .font(FontId::proportional(9.0))
                                        .color(Color32::DARK_GRAY),
                                );
                            }
                        });
                    });
            },
            BpmnNodeType::DataStore(d) => {
                // Data stores with cyan background
                egui::Frame::new()
                    .fill(Color32::from_rgb(224, 255, 255))
                    .stroke(egui::Stroke::new(2.0, Color32::from_rgb(0, 139, 139)))
                    .inner_margin(egui::Margin::same(6))
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            if d.is_unlimited {
                                ui.label(
                                    RichText::new("∞ Unlimited")
                                        .color(Color32::from_rgb(0, 128, 128)),
                                );
                            } else if let Some(cap) = d.capacity {
                                ui.label(
                                    RichText::new(format!("Capacity: {} items", cap))
                                        .color(Color32::from_rgb(0, 128, 128)),
                                );
                            }
                        });
                    });
            },
            BpmnNodeType::TextAnnotation(a) => {
                // Text annotations displayed in italic with light background
                egui::Frame::new()
                    .fill(Color32::from_rgb(255, 255, 230))
                    .stroke(egui::Stroke::new(1.0, Color32::from_rgb(200, 200, 200)))
                    .inner_margin(egui::Margin::same(4))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(&a.text)
                                .italics()
                                .font(FontId::proportional(9.0))
                                .color(Color32::from_rgb(100, 100, 100)),
                        );
                    });
            },
            BpmnNodeType::Group(g) => {
                // Groups displayed as containers
                egui::Frame::new()
                    .fill(Color32::from_rgba_premultiplied(200, 220, 255, 30))
                    .stroke(egui::Stroke::new(2.0, Color32::from_rgb(100, 150, 255)))
                    .corner_radius(egui::CornerRadius::same(5))
                    .inner_margin(egui::Margin::same(8))
                    .show(ui, |ui| {
                        if let Some(category) = &g.category {
                            ui.label(
                                RichText::new(category)
                                    .strong()
                                    .color(Color32::from_rgb(50, 100, 200)),
                            );
                        } else {
                            ui.label(
                                RichText::new("Group").color(Color32::from_rgb(100, 150, 200)),
                            );
                        }
                    });
            },
            _ => {},
        }

        // Show DoDAF metadata if present
        if let Some(dodaf) = &enhanced_node.dodaf_metadata {
            ui.separator();
            ui.label(RichText::new("DoDAF").strong().color(Color32::BLUE));

            if let Some(performer) = &dodaf.performer {
                ui.label(format!("👤 {}", performer.performer_id));
            }

            if let Some(cost) = &dodaf.cost {
                ui.label(format!("💰 {} {}", cost.amount, cost.currency));
            }

            if let Some(duration) = &dodaf.duration {
                ui.label(format!("⏱ {} {:?}", duration.value, duration.unit));
            }
        }
    }

    fn has_graph_menu(&mut self, _pos: egui::Pos2, _snarl: &mut Snarl<EnhancedBpmnNode>) -> bool {
        true
    }

    fn show_graph_menu(
        &mut self,
        pos: egui::Pos2,
        ui: &mut Ui,
        snarl: &mut Snarl<EnhancedBpmnNode>,
    ) {
        ui.label(RichText::new("Add BPMN Element").strong());
        ui.separator();

        // Events
        if ui.button("▶ Start Event").clicked() {
            let id = format!("start_{}", ui.next_auto_id().value());
            let node = EnhancedBpmnNode::start_event(id, "Start");
            snarl.insert_node(pos, node);
            ui.close();
        }

        if ui.button("⏹ End Event").clicked() {
            let id = format!("end_{}", ui.next_auto_id().value());
            let node = EnhancedBpmnNode::end_event(id, "End");
            snarl.insert_node(pos, node);
            ui.close();
        }

        ui.menu_button("⭕ Intermediate Event", |ui| {
            if ui.button("Message").clicked() {
                let id = format!("intermediate_{}", ui.next_auto_id().value());
                let node = EnhancedBpmnNode::new(
                    id,
                    BpmnNodeType::IntermediateEvent(IntermediateEventNode {
                        name: "Message Event".to_string(),
                        documentation: None,
                        event_definition: Some(EventDefinition::Message { message_ref: None }),
                        is_catching: true,
                        is_interrupting: true,
                        is_boundary: false,
                        attached_to_activity_id: None,
                    }),
                );
                snarl.insert_node(pos, node);
                ui.close();
            }
            if ui.button("Timer").clicked() {
                let id = format!("intermediate_{}", ui.next_auto_id().value());
                let node = EnhancedBpmnNode::new(
                    id,
                    BpmnNodeType::IntermediateEvent(IntermediateEventNode {
                        name: "Timer Event".to_string(),
                        documentation: None,
                        event_definition: Some(EventDefinition::Timer {
                            time_expression: "PT1H".to_string(),
                        }),
                        is_catching: true,
                        is_interrupting: true,
                        is_boundary: false,
                        attached_to_activity_id: None,
                    }),
                );
                snarl.insert_node(pos, node);
                ui.close();
            }
        });

        ui.separator();
        ui.label("Tasks:");

        // Tasks
        if ui.button("👤 User Task").clicked() {
            let id = format!("task_{}", ui.next_auto_id().value());
            let node = EnhancedBpmnNode::user_task(id, "User Task");
            snarl.insert_node(pos, node);
            ui.close();
        }

        if ui.button("⚙ Service Task").clicked() {
            let id = format!("task_{}", ui.next_auto_id().value());
            let node = EnhancedBpmnNode::service_task(id, "Service Task");
            snarl.insert_node(pos, node);
            ui.close();
        }

        if ui.button("📜 Script Task").clicked() {
            let id = format!("task_{}", ui.next_auto_id().value());
            let node = EnhancedBpmnNode::new(
                id,
                BpmnNodeType::Task(TaskNode {
                    name: "Script Task".to_string(),
                    documentation: None,
                    task_type: BpmnTaskType::Script {
                        script_format: "javascript".to_string(),
                        script: String::new(),
                    },
                    loop_characteristics: None,
                    is_for_compensation: false,
                }),
            );
            snarl.insert_node(pos, node);
            ui.close();
        }

        if ui.button("📋 Business Rule Task").clicked() {
            let id = format!("task_{}", ui.next_auto_id().value());
            let node = EnhancedBpmnNode::new(
                id,
                BpmnNodeType::Task(TaskNode {
                    name: "Business Rule Task".to_string(),
                    documentation: None,
                    task_type: BpmnTaskType::BusinessRule { implementation: None, rule_ref: None },
                    loop_characteristics: None,
                    is_for_compensation: false,
                }),
            );
            snarl.insert_node(pos, node);
            ui.close();
        }

        ui.separator();
        ui.label("Gateways:");

        // Gateways
        if ui.button("✕ Exclusive (XOR)").clicked() {
            let id = format!("gateway_{}", ui.next_auto_id().value());
            let node = EnhancedBpmnNode::exclusive_gateway(id, "XOR");
            snarl.insert_node(pos, node);
            ui.close();
        }

        if ui.button("+ Parallel (AND)").clicked() {
            let id = format!("gateway_{}", ui.next_auto_id().value());
            let node = EnhancedBpmnNode::new(
                id,
                BpmnNodeType::Gateway(GatewayNode {
                    name: "AND".to_string(),
                    documentation: None,
                    gateway_type: BpmnGatewayType::Parallel,
                    gateway_direction: GatewayDirection::Unspecified,
                }),
            );
            snarl.insert_node(pos, node);
            ui.close();
        }

        if ui.button("○ Inclusive (OR)").clicked() {
            let id = format!("gateway_{}", ui.next_auto_id().value());
            let node = EnhancedBpmnNode::new(
                id,
                BpmnNodeType::Gateway(GatewayNode {
                    name: "OR".to_string(),
                    documentation: None,
                    gateway_type: BpmnGatewayType::Inclusive,
                    gateway_direction: GatewayDirection::Unspecified,
                }),
            );
            snarl.insert_node(pos, node);
            ui.close();
        }

        ui.separator();
        ui.label("Data:");

        // Data Elements
        if ui.button("📄 Data Object").clicked() {
            let id = format!("data_{}", ui.next_auto_id().value());
            let node = EnhancedBpmnNode::new(
                id,
                BpmnNodeType::DataObject(DataObjectNode {
                    name: "Data Object".to_string(),
                    is_collection: false,
                    data_state: None,
                }),
            );
            snarl.insert_node(pos, node);
            ui.close();
        }

        if ui.button("🗄 Data Store").clicked() {
            let id = format!("datastore_{}", ui.next_auto_id().value());
            let node = EnhancedBpmnNode::new(
                id,
                BpmnNodeType::DataStore(DataStoreNode {
                    name: "Data Store".to_string(),
                    is_unlimited: true,
                    capacity: None,
                }),
            );
            snarl.insert_node(pos, node);
            ui.close();
        }

        ui.separator();
        ui.label("Artifacts:");

        // Artifacts
        if ui.button("📝 Text Annotation").clicked() {
            let id = format!("annotation_{}", ui.next_auto_id().value());
            let node = EnhancedBpmnNode::new(
                id,
                BpmnNodeType::TextAnnotation(TextAnnotationNode {
                    text: "Add annotation text here...".to_string(),
                    text_format: "text/plain".to_string(),
                }),
            );
            snarl.insert_node(pos, node);
            ui.close();
        }

        if ui.button("📦 Group").clicked() {
            let id = format!("group_{}", ui.next_auto_id().value());
            let node = EnhancedBpmnNode::new(
                id,
                BpmnNodeType::Group(GroupNode { category: Some("Group".to_string()) }),
            );
            snarl.insert_node(pos, node);
            ui.close();
        }
    }

    fn has_node_menu(&mut self, _node: &EnhancedBpmnNode) -> bool {
        true
    }

    fn show_node_menu(
        &mut self,
        node: NodeId,
        _inputs: &[InPin],
        _outputs: &[OutPin],
        ui: &mut Ui,
        snarl: &mut Snarl<EnhancedBpmnNode>,
    ) {
        let enhanced_node = &snarl[node];

        ui.label(RichText::new(format!("Node: {}", enhanced_node.name())).strong());
        ui.separator();

        if ui.button("✏ Edit Properties").clicked() {
            self.show_properties = true;
            self.selected_node_for_properties = Some(node);
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

/// Get enhanced BPMN style
pub fn enhanced_bpmn_style() -> SnarlStyle {
    SnarlStyle::default()
}
