//! Enhanced BPMN Visual Editor with Full BPMN 2.0 Support
//!
//! Demonstrates the enhanced UI with comprehensive BPMN 2.0 types and DoDAF metadata.
//!
//! Run with: `cargo run --example enhanced_ui_editor --features ui`

#[cfg(feature = "ui")]
fn main() -> Result<(), eframe::Error> {
    use abcdodaf::ui::{
        EnhancedBpmnNode, EnhancedBpmnViewer, enhanced_bpmn_style, Snarl,
        BpmnNodeType, TaskNode, GatewayNode, IntermediateEventNode,
        DodafNodeMetadata, PropertyEditor,
    };
    use abcdodaf::dodaf::ov5::{PerformerRef, Cost, CostType, Duration, TimeUnit};
    use abcdodaf::bpmn::elements::{BpmnTaskType, BpmnGatewayType, GatewayDirection, EventDefinition};
    use eframe::App;
    use std::collections::HashMap;

    // Initialize tracing
    tracing_subscriber::fmt::init();

    /// Enhanced BPMN Editor Application
    struct EnhancedBpmnEditorApp {
        snarl: Snarl<EnhancedBpmnNode>,
        viewer: EnhancedBpmnViewer,
        property_editor: PropertyEditor,
        style: egui_snarl::ui::SnarlStyle,
    }

    impl EnhancedBpmnEditorApp {
        fn new(_cc: &eframe::CreationContext) -> Self {
            let mut snarl = Snarl::new();

            // Create a sample BPMN process with full features

            // Start Event
            let start_pos = egui::pos2(100.0, 200.0);
            let start_node = EnhancedBpmnNode::start_event("start_1", "Process Started");
            snarl.insert_node(start_pos, start_node);

            // User Task with DoDAF metadata
            let user_task_pos = egui::pos2(300.0, 200.0);
            let mut user_task = EnhancedBpmnNode::user_task("task_1", "Review Application");
            user_task.dodaf_metadata = Some(DodafNodeMetadata {
                activity_ref: Some("OA-001".to_string()),
                performer: Some(PerformerRef {
                    performer_id: "reviewer".to_string(),
                    role: Some("Application Reviewer".to_string()),
                }),
                cost: Some(Cost {
                    amount: 50.0,
                    currency: "USD".to_string(),
                    cost_type: CostType::Estimated,
                }),
                duration: Some(Duration {
                    value: 2.0,
                    unit: TimeUnit::Hours,
                    is_estimated: true,
                }),
                security_domain: None,
                dodaf_properties: HashMap::new(),
            });
            snarl.insert_node(user_task_pos, user_task);

            // Exclusive Gateway (Decision Point)
            let gateway_pos = egui::pos2(500.0, 200.0);
            let gateway_node = EnhancedBpmnNode::exclusive_gateway("gateway_1", "Approved?");
            snarl.insert_node(gateway_pos, gateway_node);

            // Service Task (Approval Path)
            let service_task_pos = egui::pos2(700.0, 150.0);
            let service_task = EnhancedBpmnNode::service_task("task_2", "Process Approval");
            snarl.insert_node(service_task_pos, service_task);

            // Script Task (Rejection Path)
            let script_task_pos = egui::pos2(700.0, 250.0);
            let script_task = EnhancedBpmnNode::new(
                "task_3",
                BpmnNodeType::Task(TaskNode {
                    name: "Send Rejection Notice".to_string(),
                    documentation: Some("Send automated rejection email".to_string()),
                    task_type: BpmnTaskType::Script {
                        script_format: "javascript".to_string(),
                        script: "sendEmail(applicant, 'rejected')".to_string(),
                    },
                    loop_characteristics: None,
                    is_for_compensation: false,
                }),
            );
            snarl.insert_node(script_task_pos, script_task);

            // Parallel Gateway (Merge)
            let merge_gateway_pos = egui::pos2(900.0, 200.0);
            let merge_gateway = EnhancedBpmnNode::new(
                "gateway_2",
                BpmnNodeType::Gateway(GatewayNode {
                    name: "Merge".to_string(),
                    documentation: None,
                    gateway_type: BpmnGatewayType::Parallel,
                    gateway_direction: GatewayDirection::Converging,
                }),
            );
            snarl.insert_node(merge_gateway_pos, merge_gateway);

            // Intermediate Timer Event
            let timer_pos = egui::pos2(1100.0, 200.0);
            let timer_event = EnhancedBpmnNode::new(
                "event_1",
                BpmnNodeType::IntermediateEvent(IntermediateEventNode {
                    name: "Wait 24h".to_string(),
                    documentation: Some("Wait for confirmation period".to_string()),
                    event_definition: Some(EventDefinition::Timer {
                        time_expression: "PT24H".to_string(),
                    }),
                    is_catching: true,
                    is_interrupting: false,
                    is_boundary: false,
                }),
            );
            snarl.insert_node(timer_pos, timer_event);

            // End Event
            let end_pos = egui::pos2(1300.0, 200.0);
            let end_node = EnhancedBpmnNode::end_event("end_1", "Process Complete");
            snarl.insert_node(end_pos, end_node);

            Self {
                snarl,
                viewer: EnhancedBpmnViewer::new(),
                property_editor: PropertyEditor::new(),
                style: enhanced_bpmn_style(),
            }
        }
    }

    impl App for EnhancedBpmnEditorApp {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            // Top menu bar
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("New").clicked() {
                            self.snarl = Snarl::new();
                            ui.close();
                        }
                        if ui.button("Save as JSON...").clicked() {
                            // Serialize nodes
                            let nodes: Vec<_> = self.snarl.node_ids()
                                .map(|(_, node)| node)
                                .collect();

                            if let Ok(json) = serde_json::to_string_pretty(&nodes) {
                                println!("BPMN Process JSON:\n{}", json);
                            }
                            ui.close();
                        }
                        ui.separator();
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });

                    ui.menu_button("Edit", |ui| {
                        if ui.button("Clear All").clicked() {
                            self.snarl = Snarl::new();
                            ui.close();
                        }
                    });

                    ui.menu_button("View", |ui| {
                        if ui.button("Reset Zoom").clicked() {
                            ui.close();
                        }
                    });

                    ui.menu_button("Help", |ui| {
                        ui.label("Enhanced BPMN Editor v0.2.0");
                        ui.separator();
                        ui.label("✅ Full BPMN 2.0 Support");
                        ui.label("✅ All Event Types (12)");
                        ui.label("✅ All Task Types (8)");
                        ui.label("✅ All Gateway Types (6)");
                        ui.label("✅ Data Objects & Stores");
                        ui.label("✅ DoDAF Metadata");
                        ui.separator();
                        ui.label("Right-click canvas: Add nodes");
                        ui.label("Right-click node: Node menu");
                        ui.label("Drag from pins: Connect");
                        ui.label("Ctrl+Drag: Pan view");
                        ui.label("Scroll: Zoom");
                    });
                });
            });

            // Bottom status bar
            egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Count nodes
                    let node_count = self.snarl.node_ids().count();
                    ui.label(format!("📦 Nodes: {}", node_count));

                    ui.separator();

                    // Count connections
                    let wire_count = self.snarl.wires().count();
                    ui.label(format!("🔗 Connections: {}", wire_count));

                    ui.separator();
                    ui.label("💡 Right-click to add BPMN elements");

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label("✨ BPMN 2.0 + DoDAF 2.02 Compliant");
                    });
                });
            });

            // Main editor area
            egui::CentralPanel::default().show(ctx, |ui| {
                // Draw the BPMN graph
                self.snarl.show(
                    &mut self.viewer,
                    &self.style,
                    egui::Id::new("enhanced_bpmn_snarl"),
                    ui,
                );
            });

            // Editable Property panel (if enabled)
            if self.viewer.show_properties {
                let mut close_panel = false;

                egui::Window::new("✏ Properties")
                    .resizable(true)
                    .default_width(350.0)
                    .max_width(500.0)
                    .show(ctx, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            if let Some(node_id) = self.viewer.selected_node_for_properties {
                                if let Some(node) = self.snarl.get_node_mut(node_id) {
                                    // Show editable properties
                                    let _changed = self.property_editor.show_properties(ui, node);
                                } else {
                                    ui.label("Node not found");
                                }
                            } else {
                                ui.label("No node selected");
                                ui.label("Right-click a node and select 'Edit Properties' to edit it.");
                            }

                            ui.separator();
                            if ui.button("✖ Close").clicked() {
                                close_panel = true;
                            }
                        });
                    });

                if close_panel {
                    self.viewer.show_properties = false;
                }
            }
        }
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 800.0])
            .with_min_inner_size([1000.0, 600.0])
            .with_title("Enhanced ABCDODAF BPMN Editor - BPMN 2.0 + DoDAF 2.02"),
        ..Default::default()
    };

    eframe::run_native(
        "Enhanced ABCDODAF BPMN Editor",
        options,
        Box::new(|cc| Ok(Box::new(EnhancedBpmnEditorApp::new(cc)))),
    )
}

#[cfg(not(feature = "ui"))]
fn main() {
    eprintln!("❌ UI feature not enabled.");
    eprintln!("Build with: cargo run --example enhanced_ui_editor --features ui");
    std::process::exit(1);
}
