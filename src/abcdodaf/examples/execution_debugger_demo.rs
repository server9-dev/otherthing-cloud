//! Execution Debugger Demo
//!
//! Demonstrates the enhanced execution engine with debugging and visualization capabilities:
//! - Real-time process instance visualization
//! - Breakpoint support for debugging
//! - Step-through execution mode
//! - Process instance management (pause, resume, cancel)
//! - Event-driven execution with event queue
//! - Token-based flow control visualization
//! - Variable inspector during execution
//! - Execution history and audit trail
//! - Performance profiling (task duration, bottlenecks)

use abcdodaf::bpmn::{
    EnhancedRuntime, ExecutionEvent, ExecutionMode, Process, ProcessBuilder, TaskHandler,
    TaskType,
};
use abcdodaf::error::Result;
use abcdodaf::ui::{
    DebuggerPanel, ExecutionVisualizer, InstanceManager,
};
use async_trait::async_trait;
use eframe::egui;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Demo application state
struct DebuggerDemoApp {
    runtime: Arc<EnhancedRuntime>,
    debugger_panel: DebuggerPanel,
    execution_visualizer: ExecutionVisualizer,
    instance_manager: InstanceManager,
    event_receiver: Option<tokio::sync::broadcast::Receiver<ExecutionEvent>>,
    demo_process: Option<Process>,
    selected_view: DemoView,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DemoView {
    Debugger,
    Visualizer,
    InstanceManager,
}

impl DebuggerDemoApp {
    fn new(_cc: &eframe::CreationContext) -> Self {
        // Create runtime with demo handlers
        let runtime = Arc::new(
            EnhancedRuntime::new()
                .register_handler("user", Arc::new(DemoUserTaskHandler))
                .register_handler("service", Arc::new(DemoServiceTaskHandler))
                .register_handler("script", Arc::new(DemoScriptTaskHandler)),
        );

        // Subscribe to events
        let event_receiver = Some(runtime.subscribe_events());

        // Create demo process
        let demo_process = create_demo_process();

        Self {
            runtime,
            debugger_panel: DebuggerPanel::new(),
            execution_visualizer: ExecutionVisualizer::new(),
            instance_manager: InstanceManager::new(),
            event_receiver,
            demo_process: Some(demo_process),
            selected_view: DemoView::Debugger,
        }
    }

    fn handle_events(&mut self) {
        if let Some(receiver) = &mut self.event_receiver {
            // Process all available events
            while let Ok(event) = receiver.try_recv() {
                // Update debugger panel
                self.debugger_panel.add_event(event.clone());

                // Update visualizer on specific events
                match event {
                    ExecutionEvent::ProcessStarted { instance_id, .. } => {
                        // Load context when process starts
                        let runtime = self.runtime.clone();
                        tokio::spawn(async move {
                            if let Some(context) = runtime.get_context(instance_id).await {
                                // Would update visualizer here
                                println!("Process started: {}", instance_id);
                            }
                        });
                    }
                    _ => {}
                }
            }
        }
    }
}

impl eframe::App for DebuggerDemoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request continuous repaints for animations
        ctx.request_repaint();

        // Handle events
        self.handle_events();

        // Top panel - controls
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🐛 Execution Debugger Demo");

                ui.separator();

                // View selection
                ui.selectable_value(&mut self.selected_view, DemoView::Debugger, "🔍 Debugger");
                ui.selectable_value(
                    &mut self.selected_view,
                    DemoView::Visualizer,
                    "🎨 Visualizer",
                );
                ui.selectable_value(
                    &mut self.selected_view,
                    DemoView::InstanceManager,
                    "📋 Instances",
                );

                ui.separator();

                // Demo controls
                if ui.button("▶️ Start Demo Process").clicked() {
                    if let Some(process) = &self.demo_process {
                        let runtime = self.runtime.clone();
                        let process = process.clone();
                        tokio::spawn(async move {
                            let variables = HashMap::new();
                            match runtime
                                .execute_process(&process, variables, ExecutionMode::Continuous)
                                .await
                            {
                                Ok(instance) => {
                                    println!("Process completed: {}", instance.id);
                                }
                                Err(e) => {
                                    eprintln!("Process failed: {}", e);
                                }
                            }
                        });
                    }
                }

                if ui.button("🔴 Add Breakpoint").clicked() {
                    let runtime = self.runtime.clone();
                    tokio::spawn(async move {
                        runtime.add_breakpoint("task_analyze".to_string()).await;
                    });
                }

                if ui.button("⏭️ Step Mode Demo").clicked() {
                    if let Some(process) = &self.demo_process {
                        let runtime = self.runtime.clone();
                        let process = process.clone();
                        tokio::spawn(async move {
                            let variables = HashMap::new();
                            match runtime
                                .execute_process(&process, variables, ExecutionMode::StepByStep)
                                .await
                            {
                                Ok(instance) => {
                                    println!("Process completed: {}", instance.id);
                                }
                                Err(e) => {
                                    eprintln!("Process failed: {}", e);
                                }
                            }
                        });
                    }
                }
            });
        });

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.selected_view {
                DemoView::Debugger => {
                    let action = self.debugger_panel.ui(ui);
                    // Handle debugger actions
                    use abcdodaf::ui::DebuggerAction;
                    match action {
                        DebuggerAction::Pause => {
                            println!("Pause requested");
                        }
                        DebuggerAction::Resume => {
                            println!("Resume requested");
                        }
                        DebuggerAction::Step => {
                            println!("Step requested");
                        }
                        DebuggerAction::Stop => {
                            println!("Stop requested");
                        }
                        _ => {}
                    }
                }
                DemoView::Visualizer => {
                    ui.heading("Execution Visualizer");
                    ui.label("Real-time visualization of process execution with token flow");
                    ui.separator();

                    // Animation update
                    self.execution_visualizer
                        .update_animation(ctx.input(|i| i.stable_dt));

                    // Placeholder for diagram
                    egui::Frame::canvas(ui.style()).show(ui, |ui| {
                        let (response, painter) =
                            ui.allocate_painter(ui.available_size(), egui::Sense::hover());

                        // Draw example BPMN elements
                        let rect = response.rect;
                        painter.rect_filled(
                            rect,
                            0.0,
                            egui::Color32::from_rgb(30, 30, 40),
                        );

                        // Draw some example nodes
                        let node1 = egui::Rect::from_center_size(
                            rect.center() - egui::Vec2::new(200.0, 0.0),
                            egui::Vec2::new(100.0, 60.0),
                        );
                        let node2 = egui::Rect::from_center_size(
                            rect.center(),
                            egui::Vec2::new(100.0, 60.0),
                        );
                        let node3 = egui::Rect::from_center_size(
                            rect.center() + egui::Vec2::new(200.0, 0.0),
                            egui::Vec2::new(100.0, 60.0),
                        );

                        // Draw connections
                        painter.line_segment(
                            [node1.right_center(), node2.left_center()],
                            egui::Stroke::new(2.0, egui::Color32::GRAY),
                        );
                        painter.line_segment(
                            [node2.right_center(), node3.left_center()],
                            egui::Stroke::new(2.0, egui::Color32::GRAY),
                        );

                        // Draw nodes
                        painter.rect_stroke(
                            node1,
                            5.0,
                            egui::Stroke::new(2.0, egui::Color32::WHITE),
                        );
                        painter.text(
                            node1.center(),
                            egui::Align2::CENTER_CENTER,
                            "Start",
                            egui::FontId::proportional(14.0),
                            egui::Color32::WHITE,
                        );

                        painter.rect_stroke(
                            node2,
                            5.0,
                            egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 200, 255)),
                        );
                        painter.text(
                            node2.center(),
                            egui::Align2::CENTER_CENTER,
                            "Analyze",
                            egui::FontId::proportional(14.0),
                            egui::Color32::WHITE,
                        );

                        painter.rect_stroke(
                            node3,
                            5.0,
                            egui::Stroke::new(2.0, egui::Color32::WHITE),
                        );
                        painter.text(
                            node3.center(),
                            egui::Align2::CENTER_CENTER,
                            "End",
                            egui::FontId::proportional(14.0),
                            egui::Color32::WHITE,
                        );

                        // Render execution overlay
                        self.execution_visualizer.render(&painter, rect);
                    });

                    ui.separator();

                    // Visualization controls
                    ui.horizontal(|ui| {
                        if ui.button("🎯 Toggle Tokens").clicked() {
                            self.execution_visualizer.toggle_tokens();
                        }
                        if ui.button("🔥 Toggle Heatmap").clicked() {
                            self.execution_visualizer.toggle_heatmap();
                        }
                    });
                }
                DemoView::InstanceManager => {
                    let action = self.instance_manager.ui(ui);
                    // Handle instance manager actions
                    use abcdodaf::ui::InstanceManagerAction;
                    match action {
                        InstanceManagerAction::Debug(id) => {
                            println!("Debug instance: {}", id);
                            self.selected_view = DemoView::Debugger;
                        }
                        InstanceManagerAction::Pause(id) => {
                            let runtime = self.runtime.clone();
                            tokio::spawn(async move {
                                if let Err(e) = runtime.pause(id).await {
                                    eprintln!("Failed to pause: {}", e);
                                }
                            });
                        }
                        InstanceManagerAction::Cancel(id) => {
                            let runtime = self.runtime.clone();
                            tokio::spawn(async move {
                                if let Err(e) = runtime.cancel(id).await {
                                    eprintln!("Failed to cancel: {}", e);
                                }
                            });
                        }
                        InstanceManagerAction::Refresh => {
                            // Refresh instances
                            let runtime = self.runtime.clone();
                            tokio::spawn(async move {
                                let instances = runtime.get_active_instances().await;
                                println!("Active instances: {}", instances.len());
                            });
                        }
                        _ => {}
                    }
                }
            }
        });

        // Bottom panel - status
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Status: Ready");
                ui.separator();
                ui.label(format!("FPS: {:.1}", ctx.input(|i| 1.0 / i.stable_dt.max(0.001))));
            });
        });
    }
}

/// Create a demo process for testing
fn create_demo_process() -> Process {
    ProcessBuilder::new("demo_process", "Demo Process")
        .description("Demonstration process for debugging features")
        .add_user_task("task_input", "User Input")
        .add_service_task("task_analyze", "Analyze Data")
        .add_script_task("task_transform", "Transform Results")
        .add_service_task("task_notify", "Send Notification")
        .add_flow("flow1", "task_input", "task_analyze")
        .add_flow("flow2", "task_analyze", "task_transform")
        .add_flow("flow3", "task_transform", "task_notify")
        .build()
        .expect("Failed to build demo process")
}

/// Demo user task handler
struct DemoUserTaskHandler;

#[async_trait]
impl TaskHandler for DemoUserTaskHandler {
    async fn execute(
        &self,
        task: &abcdodaf::bpmn::Task,
        variables: &HashMap<String, serde_json::Value>,
    ) -> Result<HashMap<String, serde_json::Value>> {
        println!("Executing user task: {}", task.name);
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        let mut output = variables.clone();
        output.insert("user_input".to_string(), serde_json::json!("demo_data"));
        Ok(output)
    }

    fn name(&self) -> &str {
        "demo_user_handler"
    }
}

/// Demo service task handler
struct DemoServiceTaskHandler;

#[async_trait]
impl TaskHandler for DemoServiceTaskHandler {
    async fn execute(
        &self,
        task: &abcdodaf::bpmn::Task,
        variables: &HashMap<String, serde_json::Value>,
    ) -> Result<HashMap<String, serde_json::Value>> {
        println!("Executing service task: {}", task.name);
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        let mut output = variables.clone();
        output.insert("service_result".to_string(), serde_json::json!({"status": "success"}));
        Ok(output)
    }

    fn name(&self) -> &str {
        "demo_service_handler"
    }
}

/// Demo script task handler
struct DemoScriptTaskHandler;

#[async_trait]
impl TaskHandler for DemoScriptTaskHandler {
    async fn execute(
        &self,
        task: &abcdodaf::bpmn::Task,
        variables: &HashMap<String, serde_json::Value>,
    ) -> Result<HashMap<String, serde_json::Value>> {
        println!("Executing script task: {}", task.name);
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

        let mut output = variables.clone();
        output.insert("transformed_data".to_string(), serde_json::json!(["item1", "item2"]));
        Ok(output)
    }

    fn name(&self) -> &str {
        "demo_script_handler"
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🐛 ABCDODAF Execution Debugger Demo");
    println!("=====================================");
    println!();
    println!("Features demonstrated:");
    println!("  ✓ Real-time process instance visualization");
    println!("  ✓ Breakpoint support for debugging");
    println!("  ✓ Step-through execution mode");
    println!("  ✓ Process instance management (pause, resume, cancel)");
    println!("  ✓ Event-driven execution with event queue");
    println!("  ✓ Token-based flow control visualization");
    println!("  ✓ Variable inspector during execution");
    println!("  ✓ Execution history and audit trail");
    println!("  ✓ Performance profiling (task duration, bottlenecks)");
    println!();

    // Run the UI
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("ABCDODAF - Execution Debugger Demo")
            .with_inner_size([1400.0, 900.0]),
        ..Default::default()
    };

    eframe::run_native(
        "ABCDODAF Execution Debugger",
        native_options,
        Box::new(|cc| Ok(Box::new(DebuggerDemoApp::new(cc)))),
    )?;

    Ok(())
}
