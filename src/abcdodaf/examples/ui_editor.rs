//! BPMN Visual Editor using egui-snarl
//!
//! Professional node-graph based BPMN editor with:
//! - All BPMN 2.0 node types
//! - Right-click context menus
//! - Drag-and-drop connections
//! - Serialization support
//!
//! Run with: `cargo run --example ui_editor --features ui`

#[cfg(feature = "ui")]
fn main() -> Result<(), eframe::Error> {
    use abcdodaf::ui::{BpmnNode, BpmnViewer, Snarl, bpmn_style};
    use abcdodaf::bpmn::process::TaskType;
    use eframe::App;

    // Initialize tracing for debugging
    tracing_subscriber::fmt::init();

    /// Main BPMN Editor Application
    struct BpmnEditorApp {
        snarl: Snarl<BpmnNode>,
        viewer: BpmnViewer,
        style: egui_snarl::ui::SnarlStyle,
    }

    impl BpmnEditorApp {
        fn new(_cc: &eframe::CreationContext) -> Self {
            let mut snarl = Snarl::new();

            // Add a default start node
            let start_pos = egui::pos2(100.0, 200.0);
            snarl.insert_node(
                start_pos,
                BpmnNode::StartEvent {
                    name: "Start".to_string(),
                },
            );

            // Add a sample task
            let task_pos = egui::pos2(300.0, 200.0);
            snarl.insert_node(
                task_pos,
                BpmnNode::Task {
                    name: "Process Request".to_string(),
                    task_type: TaskType::User,
                    description: Some("Handle incoming request".to_string()),
                },
            );

            // Add end node
            let end_pos = egui::pos2(500.0, 200.0);
            snarl.insert_node(
                end_pos,
                BpmnNode::EndEvent {
                    name: "End".to_string(),
                },
            );

            Self {
                snarl,
                viewer: BpmnViewer::new(),
                style: bpmn_style(),
            }
        }
    }

    impl App for BpmnEditorApp {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            // Top menu bar
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("New").clicked() {
                            self.snarl = Snarl::new();
                            ui.close();
                        }
                        if ui.button("Save...").clicked() {
                            // Serialize the snarl
                            if let Ok(json) = serde_json::to_string_pretty(&self.snarl) {
                                println!("BPMN Process:\n{}", json);
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
                            // Zoom reset would be handled by snarl
                            ui.close();
                        }
                    });

                    ui.menu_button("Help", |ui| {
                        ui.label("BPMN Editor v0.1.0");
                        ui.separator();
                        ui.label("Right-click: Add nodes");
                        ui.label("Drag: Connect nodes");
                        ui.label("Ctrl+Drag: Pan view");
                        ui.label("Scroll: Zoom");
                    });
                });
            });

            // Bottom status bar
            egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("Nodes: {}", self.snarl.nodes().count()));
                    ui.separator();
                    // Count wires manually
                    let wire_count = self.snarl.wires().count();
                    ui.label(format!("Connections: {}", wire_count));
                    ui.separator();
                    ui.label("Right-click to add nodes | Drag from pins to connect");
                });
            });

            // Main editor area
            egui::CentralPanel::default().show(ctx, |ui| {
                // Draw the BPMN graph using egui-snarl
                self.snarl.show(
                    &mut self.viewer,
                    &self.style,
                    egui::Id::new("bpmn_snarl"),
                    ui,
                );
            });
        }
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("ABCDODAF BPMN Editor"),
        ..Default::default()
    };

    eframe::run_native(
        "ABCDODAF BPMN Editor",
        options,
        Box::new(|cc| Ok(Box::new(BpmnEditorApp::new(cc)))),
    )
}

#[cfg(not(feature = "ui"))]
fn main() {
    eprintln!("❌ UI feature not enabled.");
    eprintln!("Build with: cargo run --example ui_editor --features ui");
    std::process::exit(1);
}
