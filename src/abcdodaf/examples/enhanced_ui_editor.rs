//! IDE-like BPMN Visual Editor with Multi-Workflow Support
//!
//! Full-featured BPMN 2.0 editor with workspace management, validation,
//! file browser, and DoDAF aggregation.
//!
//! Run with: `cargo run --example enhanced_ui_editor --features ui`

#[cfg(feature = "ui")]
fn main() -> Result<(), eframe::Error> {
    use abcdodaf::ui::{
        EnhancedBpmnViewer, enhanced_bpmn_style, PropertyEditor,
        Workspace, FileBrowser, FileAction, TabBar, TabAction,
        DodafAggregator, Validator, NotificationManager,
    };
    use eframe::App;

    // Initialize tracing
    tracing_subscriber::fmt::init();

    /// IDE-like BPMN Workspace Editor
    struct WorkspaceEditorApp {
        workspace: Workspace,
        viewer: EnhancedBpmnViewer,
        property_editor: PropertyEditor,
        file_browser: FileBrowser,
        tab_bar: TabBar,
        show_dodaf_panel: bool,
        dodaf_aggregator: DodafAggregator,
        style: egui_snarl::ui::SnarlStyle,
        notifications: NotificationManager,

        // UI state
        show_file_browser: bool,
        show_properties: bool,
        pending_save_as: Option<usize>, // Workflow ID waiting for save path
    }

    impl WorkspaceEditorApp {
        fn new(_cc: &eframe::CreationContext) -> Self {
            let mut workspace = Workspace::new();

            // Create initial empty workflow
            workspace.create_new_workflow();

            Self {
                workspace,
                viewer: EnhancedBpmnViewer::new(),
                property_editor: PropertyEditor::new(),
                file_browser: FileBrowser::new(),
                tab_bar: TabBar::new(),
                show_dodaf_panel: false,
                dodaf_aggregator: DodafAggregator::new(),
                style: enhanced_bpmn_style(),
                notifications: NotificationManager::new(),
                show_file_browser: true,
                show_properties: true,
                pending_save_as: None,
            }
        }

        fn render_menu_bar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
            egui::MenuBar::new().ui(ui, |ui| {
                // File menu
                ui.menu_button("File", |ui| {
                    if ui.button("📄 New Workflow").clicked() {
                        self.workspace.create_new_workflow();
                        ui.close();
                    }

                    if ui.button("📂 Open Workflow...").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("JSON", &["json"])
                            .pick_file()
                        {
                            if let Err(e) = self.workspace.open_workflow(path) {
                                self.notifications.error(format!("Failed to open workflow: {}", e));
                            } else {
                                self.notifications.success("Workflow opened");
                            }
                        }
                        ui.close();
                    }

                    ui.separator();

                    let has_active = self.workspace.active_workflow_id().is_some();

                    if ui.add_enabled(has_active, egui::Button::new("💾 Save")).clicked() {
                        if let Some(id) = self.workspace.active_workflow_id() {
                            // Sync diagram from snarl before checking file path
                            if let Some(doc) = self.workspace.get_workflow_mut(id) {
                                if let Err(e) = doc.sync_from_snarl() {
                                    self.notifications.error(format!("Failed to sync diagram: {}", e));
                                    ui.close();
                                    return;
                                }
                            }

                            if let Some(doc) = self.workspace.get_workflow(id) {
                                if doc.file_path.is_some() {
                                    if let Err(e) = self.workspace.save_workflow(id) {
                                        self.notifications.error(format!("Failed to save: {}", e));
                                    } else {
                                        self.file_browser.mark_dirty();
                                        self.notifications.success("Workflow saved successfully");
                                    }
                                } else {
                                    self.pending_save_as = Some(id);
                                }
                            }
                        }
                        ui.close();
                    }

                    if ui.add_enabled(has_active, egui::Button::new("💾 Save As...")).clicked() {
                        if let Some(id) = self.workspace.active_workflow_id() {
                            self.pending_save_as = Some(id);
                        }
                        ui.close();
                    }

                    ui.separator();

                    if ui.add_enabled(has_active, egui::Button::new("✖ Close Workflow")).clicked() {
                        if let Some(id) = self.workspace.active_workflow_id() {
                            self.workspace.close_workflow(id);
                        }
                        ui.close();
                    }

                    if ui.button("✖ Close All").clicked() {
                        let ids: Vec<_> = self.workspace.workflow_ids();
                        for id in ids {
                            self.workspace.close_workflow(id);
                        }
                        ui.close();
                    }

                    ui.separator();

                    if ui.button("🚪 Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                // Edit menu
                ui.menu_button("Edit", |ui| {
                    ui.label("Undo/Redo (Coming Soon)");
                    ui.separator();
                    ui.label("Cut/Copy/Paste (Coming Soon)");
                });

                // View menu
                ui.menu_button("View", |ui| {
                    if ui.checkbox(&mut self.show_file_browser, "File Browser").clicked() {
                        ui.close();
                    }
                    if ui.checkbox(&mut self.show_properties, "Properties Panel").clicked() {
                        ui.close();
                    }
                    if ui.checkbox(&mut self.show_dodaf_panel, "DoDAF Panel").clicked() {
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Reset Layout").clicked() {
                        self.show_file_browser = true;
                        self.show_properties = true;
                        self.show_dodaf_panel = false;
                        ui.close();
                    }
                });

                // Validate menu
                ui.menu_button("Validate", |ui| {
                    if ui.button("✓ Validate Current Workflow").clicked() {
                        if let Some(doc) = self.workspace.get_active_workflow_mut() {
                            let result = Validator::validate_workflow(&doc.snarl);
                            self.viewer.set_validation_result(result.clone());
                            doc.validation_errors = result.errors;
                        }
                        ui.close();
                    }

                    if ui.button("✓ Validate All Workflows").clicked() {
                        for id in self.workspace.workflow_ids() {
                            if let Some(doc) = self.workspace.get_workflow_mut(id) {
                                let result = Validator::validate_workflow(&doc.snarl);
                                doc.validation_errors = result.errors;
                            }
                        }
                        ui.close();
                    }
                });

                // Help menu
                ui.menu_button("Help", |ui| {
                    ui.heading("ABCDODAF BPMN IDE");
                    ui.label("Version 0.3.0");
                    ui.separator();
                    ui.label("✅ Multi-workflow IDE");
                    ui.label("✅ Full BPMN 2.0 Support");
                    ui.label("✅ DoDAF 2.02 Metadata");
                    ui.label("✅ Validation Engine");
                    ui.label("✅ File Management");
                    ui.separator();
                    ui.heading("Keyboard Shortcuts");
                    ui.label("Ctrl+N: New Workflow");
                    ui.label("Ctrl+S: Save");
                    ui.label("Ctrl+W: Close Tab");
                    ui.label("F5: Validate");
                });
            });
        }

        fn render_status_bar(&self, ui: &mut egui::Ui) {
            ui.horizontal(|ui| {
                if let Some(doc) = self.workspace.get_active_workflow() {
                    // Validation status
                    if doc.validation_errors.is_empty() {
                        ui.label(egui::RichText::new("✅ Valid").color(egui::Color32::from_rgb(0, 128, 0)));
                    } else {
                        let error_count = doc.validation_errors.len();
                        ui.label(
                            egui::RichText::new(format!("❌ {} Error{}", error_count, if error_count == 1 { "" } else { "s" }))
                                .color(egui::Color32::from_rgb(255, 0, 0))
                        );
                    }

                    ui.separator();

                    // Node count
                    let node_count = doc.snarl.node_ids().count();
                    ui.label(format!("📦 {} Node{}", node_count, if node_count == 1 { "" } else { "s" }));

                    ui.separator();

                    // Connection count
                    let conn_count = doc.snarl.wires().count();
                    ui.label(format!("🔗 {} Connection{}", conn_count, if conn_count == 1 { "" } else { "s" }));

                    ui.separator();

                    // Modified indicator
                    if doc.is_modified {
                        ui.label(egui::RichText::new("● Modified").color(egui::Color32::from_rgb(255, 200, 0)));
                    }
                } else {
                    ui.label("No workflow open");
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("✨ BPMN 2.0 + DoDAF 2.02");
                });
            });
        }

        fn render_welcome_screen(&self, ui: &mut egui::Ui) {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.heading("Welcome to ABCDODAF BPMN IDE");
                ui.add_space(20.0);
                ui.label("No workflow open");
                ui.add_space(10.0);
                ui.label("Create a new workflow or open an existing one from the file browser");
                ui.add_space(20.0);

                ui.horizontal(|ui| {
                    if ui.button("📄 New Workflow").clicked() {
                        // Will be handled by workspace
                    }
                    if ui.button("📂 Open Workflow").clicked() {
                        // Will be handled by file dialog
                    }
                });
            });
        }

        fn render_properties(&mut self, ui: &mut egui::Ui) {
            ui.heading("⚙ Properties");
            ui.separator();

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if let Some(doc) = self.workspace.get_active_workflow_mut() {
                        let doc_id = doc.id;
                        if let Some(node_id) = self.viewer.selected_node_for_properties {
                            if let Some(node) = doc.snarl.get_node_mut(node_id) {
                                let changed = self.property_editor.show_properties(ui, node);
                                if changed {
                                    self.workspace.mark_modified(doc_id);
                                }
                            } else {
                                ui.label("Node not found");
                            }
                        } else {
                            ui.heading("Workflow Properties");
                            ui.separator();
                            ui.label(format!("Name: {}", doc.name));
                            if let Some(path) = &doc.file_path {
                                ui.label(format!("File: {}", path.display()));
                            } else {
                                ui.label("File: Unsaved");
                            }
                            ui.separator();
                            ui.label("Select a node to edit its properties");
                        }
                    } else {
                        ui.label("No workflow open");
                    }
                });
        }

        fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
            // Ctrl+N: New workflow
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::N)) {
                self.workspace.create_new_workflow();
            }

            // Ctrl+S: Save
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::S)) {
                if let Some(id) = self.workspace.active_workflow_id() {
                    // Sync diagram from snarl before saving
                    if let Some(doc) = self.workspace.get_workflow_mut(id) {
                        if let Err(e) = doc.sync_from_snarl() {
                            self.notifications.error(format!("Failed to sync diagram: {}", e));
                            return;
                        }
                    }

                    if let Some(doc) = self.workspace.get_workflow(id) {
                        if doc.file_path.is_some() {
                            if let Err(e) = self.workspace.save_workflow(id) {
                                self.notifications.error(format!("Failed to save: {}", e));
                            } else {
                                self.file_browser.mark_dirty();
                                self.notifications.success("Workflow saved successfully");
                            }
                        } else {
                            self.pending_save_as = Some(id);
                        }
                    }
                }
            }

            // Ctrl+W: Close tab
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::W)) {
                if let Some(id) = self.workspace.active_workflow_id() {
                    self.workspace.close_workflow(id);
                }
            }

            // F5: Validate
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F5)) {
                if let Some(doc) = self.workspace.get_active_workflow_mut() {
                    let result = Validator::validate_workflow(&doc.snarl);
                    doc.validation_errors = result.errors.clone();
                    self.viewer.set_validation_result(result);
                }
            }
        }
    }

    impl App for WorkspaceEditorApp {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            // Handle keyboard shortcuts
            self.handle_keyboard_shortcuts(ctx);

            // Handle pending save as dialog
            if let Some(workflow_id) = self.pending_save_as {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("JSON", &["json"])
                    .set_file_name("workflow.json")
                    .save_file()
                {
                    if let Err(e) = self.workspace.save_workflow_as(workflow_id, path) {
                        self.notifications.error(format!("Failed to save: {}", e));
                    } else {
                        self.file_browser.mark_dirty();
                        self.notifications.success("Workflow saved successfully");
                    }
                }
                self.pending_save_as = None;
            }

            // Top menu bar
            egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
                self.render_menu_bar(ui, ctx);
            });

            // Left file browser
            if self.show_file_browser {
                egui::SidePanel::left("file_browser")
                    .default_width(200.0)
                    .resizable(true)
                    .show(ctx, |ui| {
                        if let Some(action) = self.file_browser.render(ui, &mut self.workspace) {
                            match action {
                                FileAction::NewFile => {
                                    self.workspace.create_new_workflow();
                                }
                                FileAction::OpenFile(path) => {
                                    if let Err(e) = self.workspace.open_workflow(path) {
                                        self.notifications.error(format!("Failed to open: {}", e));
                                    } else {
                                        self.notifications.success("Workflow opened");
                                    }
                                }
                                FileAction::DeleteFile(path) => {
                                    if let Err(e) = std::fs::remove_file(&path) {
                                        self.notifications.error(format!("Failed to delete: {}", e));
                                    } else {
                                        self.file_browser.mark_dirty();
                                        self.notifications.success("File deleted");
                                    }
                                }
                                FileAction::RenameFile(_path) => {
                                    // TODO: Implement rename dialog
                                }
                            }
                        }
                    });
            }

            // Right properties panel
            if self.show_properties {
                egui::SidePanel::right("properties")
                    .default_width(350.0)
                    .resizable(true)
                    .show(ctx, |ui| {
                        self.render_properties(ui);
                    });
            }

            // Bottom DoDAF panel
            if self.show_dodaf_panel {
                egui::TopBottomPanel::bottom("dodaf_panel")
                    .default_height(200.0)
                    .resizable(true)
                    .show(ctx, |ui| {
                        self.dodaf_aggregator.render(ui, &self.workspace);
                    });
            }

            // Bottom status bar
            egui::TopBottomPanel::bottom("status_bar")
                .min_height(24.0)
                .show(ctx, |ui| {
                    self.render_status_bar(ui);
                });

            // Center panel with tabs + editor
            egui::CentralPanel::default().show(ctx, |ui| {
                // Tab bar
                egui::TopBottomPanel::top("tabs")
                    .min_height(32.0)
                    .show_inside(ui, |ui| {
                        if let Some(action) = self.tab_bar.render(ui, &mut self.workspace) {
                            match action {
                                TabAction::NewTab => {
                                    self.workspace.create_new_workflow();
                                }
                                TabAction::SwitchTab(id) => {
                                    self.workspace.set_active_workflow(id);
                                }
                                TabAction::CloseTab(id) => {
                                    self.workspace.close_workflow(id);
                                }
                            }
                        }
                    });

                // Active workflow editor
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    if let Some(doc) = self.workspace.get_active_workflow_mut() {
                        // Update validation in viewer
                        let result = Validator::validate_workflow(&doc.snarl);
                        self.viewer.set_validation_result(result.clone());
                        doc.validation_errors = result.errors;

                        // Show the graph editor
                        doc.snarl.show(
                            &mut self.viewer,
                            &self.style,
                            egui::Id::new(format!("workflow_{}", doc.id)),
                            ui,
                        );

                        // Mark as modified if graph changed
                        // Note: egui-snarl doesn't provide change detection, so we'd need to track this manually
                        // For now, we'll mark modified on property changes
                    } else {
                        self.render_welcome_screen(ui);
                    }
                });
            });

            // Render notifications at the end (on top of everything)
            self.notifications.render(ctx);
        }
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1600.0, 900.0])
            .with_min_inner_size([1200.0, 700.0])
            .with_title("ABCDODAF BPMN IDE - Multi-Workflow Editor"),
        ..Default::default()
    };

    eframe::run_native(
        "ABCDODAF BPMN IDE",
        options,
        Box::new(|cc| Ok(Box::new(WorkspaceEditorApp::new(cc)))),
    )
}

#[cfg(not(feature = "ui"))]
fn main() {
    eprintln!("❌ UI feature not enabled.");
    eprintln!("Build with: cargo run --example enhanced_ui_editor --features ui");
    std::process::exit(1);
}
