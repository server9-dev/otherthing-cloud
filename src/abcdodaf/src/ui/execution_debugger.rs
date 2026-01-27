//! Execution debugger UI component

use crate::bpmn::runtime::{
    Breakpoint, ExecutionContext, ExecutionEvent, ExecutionMode, TaskPerformance,
};
use egui::{Color32, RichText, ScrollArea, Ui};
use std::collections::HashMap;
use uuid::Uuid;

/// Debugger panel UI state
#[derive(Debug, Clone)]
pub struct DebuggerPanel {
    /// Current instance being debugged
    pub current_instance: Option<Uuid>,
    /// Execution context
    pub context: Option<ExecutionContext>,
    /// Recent events
    pub recent_events: Vec<ExecutionEvent>,
    /// Breakpoints
    pub breakpoints: Vec<Breakpoint>,
    /// Performance data
    pub performance: HashMap<String, TaskPerformance>,
    /// Selected tab
    pub selected_tab: DebuggerTab,
    /// Show only errors in event log
    pub filter_errors_only: bool,
    /// Event log scroll to bottom
    pub auto_scroll_events: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebuggerTab {
    Overview,
    Breakpoints,
    Variables,
    Events,
    Performance,
}

impl Default for DebuggerPanel {
    fn default() -> Self {
        Self {
            current_instance: None,
            context: None,
            recent_events: Vec::new(),
            breakpoints: Vec::new(),
            performance: HashMap::new(),
            selected_tab: DebuggerTab::Overview,
            filter_errors_only: false,
            auto_scroll_events: true,
        }
    }
}

impl DebuggerPanel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update with new context
    pub fn update_context(&mut self, context: ExecutionContext) {
        self.current_instance = Some(context.instance.id);
        self.context = Some(context);
    }

    /// Add event to log
    pub fn add_event(&mut self, event: ExecutionEvent) {
        self.recent_events.push(event);
        // Keep only last 1000 events
        if self.recent_events.len() > 1000 {
            self.recent_events.remove(0);
        }
    }

    /// Update breakpoints
    pub fn update_breakpoints(&mut self, breakpoints: Vec<Breakpoint>) {
        self.breakpoints = breakpoints;
    }

    /// Update performance data
    pub fn update_performance(&mut self, performance: HashMap<String, TaskPerformance>) {
        self.performance = performance;
    }

    /// Render the debugger panel
    pub fn ui(&mut self, ui: &mut Ui) -> DebuggerAction {
        let mut action = DebuggerAction::None;

        // Tab bar
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.selected_tab, DebuggerTab::Overview, "📊 Overview");
            ui.selectable_value(&mut self.selected_tab, DebuggerTab::Breakpoints, "🔴 Breakpoints");
            ui.selectable_value(&mut self.selected_tab, DebuggerTab::Variables, "💾 Variables");
            ui.selectable_value(&mut self.selected_tab, DebuggerTab::Events, "📋 Events");
            ui.selectable_value(&mut self.selected_tab, DebuggerTab::Performance, "⚡ Performance");
        });

        ui.separator();

        // Control buttons
        if let Some(context) = &self.context {
            ui.horizontal(|ui| {
                // Execution controls
                match context.mode {
                    ExecutionMode::Paused => {
                        if ui.button("▶️ Resume").clicked() {
                            action = DebuggerAction::Resume;
                        }
                        if ui.button("⏭️ Step").clicked() {
                            action = DebuggerAction::Step;
                        }
                    },
                    ExecutionMode::Continuous | ExecutionMode::StepByStep => {
                        if ui.button("⏸️ Pause").clicked() {
                            action = DebuggerAction::Pause;
                        }
                    },
                }

                if ui.button("⏹️ Stop").clicked() {
                    action = DebuggerAction::Stop;
                }

                ui.separator();

                // Instance info
                ui.label(format!(
                    "Instance: {}",
                    context.instance.id.to_string().chars().take(8).collect::<String>()
                ));
                ui.label(format!("State: {:?}", context.instance.state));
                ui.label(format!("Mode: {:?}", context.mode));
            });

            ui.separator();
        }

        // Tab content
        ScrollArea::vertical().show(ui, |ui| match self.selected_tab {
            DebuggerTab::Overview => self.render_overview(ui),
            DebuggerTab::Breakpoints => action = self.render_breakpoints(ui),
            DebuggerTab::Variables => self.render_variables(ui),
            DebuggerTab::Events => self.render_events(ui),
            DebuggerTab::Performance => self.render_performance(ui),
        });

        action
    }

    fn render_overview(&mut self, ui: &mut Ui) {
        if let Some(context) = &self.context {
            ui.heading("Execution Overview");

            ui.group(|ui| {
                ui.label(RichText::new("Process Information").strong());
                egui::Grid::new("overview_grid").num_columns(2).spacing([40.0, 4.0]).show(
                    ui,
                    |ui| {
                        ui.label("Process ID:");
                        ui.label(&context.instance.process_id);
                        ui.end_row();

                        ui.label("Instance ID:");
                        ui.label(context.instance.id.to_string());
                        ui.end_row();

                        ui.label("State:");
                        let state_color = match context.instance.state {
                            crate::bpmn::ProcessState::Running => Color32::GREEN,
                            crate::bpmn::ProcessState::Completed => Color32::BLUE,
                            crate::bpmn::ProcessState::Failed => Color32::RED,
                            crate::bpmn::ProcessState::Cancelled => Color32::YELLOW,
                            _ => Color32::GRAY,
                        };
                        ui.colored_label(state_color, format!("{:?}", context.instance.state));
                        ui.end_row();

                        ui.label("Started:");
                        ui.label(
                            context.instance.started_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                        );
                        ui.end_row();

                        if let Some(completed) = context.instance.completed_at {
                            ui.label("Completed:");
                            ui.label(completed.format("%Y-%m-%d %H:%M:%S").to_string());
                            ui.end_row();

                            let duration = completed - context.instance.started_at;
                            ui.label("Duration:");
                            ui.label(format!(
                                "{:.2}s",
                                duration.num_milliseconds() as f64 / 1000.0
                            ));
                            ui.end_row();
                        }
                    },
                );
            });

            ui.add_space(10.0);

            // Token visualization
            ui.group(|ui| {
                ui.label(RichText::new("Active Tokens").strong());
                for token in &context.tokens {
                    ui.horizontal(|ui| {
                        let token_color = match token.state {
                            crate::bpmn::runtime::TokenState::Active => Color32::GREEN,
                            crate::bpmn::runtime::TokenState::Waiting => Color32::YELLOW,
                            crate::bpmn::runtime::TokenState::Blocked => Color32::RED,
                            crate::bpmn::runtime::TokenState::Consumed => Color32::GRAY,
                        };

                        ui.colored_label(
                            token_color,
                            format!(
                                "🎯 Token {} at {}",
                                token.id.to_string().chars().take(8).collect::<String>(),
                                token.current_element
                            ),
                        );
                    });
                }
            });

            ui.add_space(10.0);

            // Current element highlight
            if let Some(current) = &context.current_element {
                ui.group(|ui| {
                    ui.label(RichText::new("Current Execution Point").strong());
                    ui.colored_label(Color32::LIGHT_BLUE, format!("📍 {}", current));
                });
            }
        } else {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new("No active execution").color(Color32::GRAY));
            });
        }
    }

    fn render_breakpoints(&mut self, ui: &mut Ui) -> DebuggerAction {
        let mut action = DebuggerAction::None;

        ui.heading("Breakpoints");

        if ui.button("➕ Add Breakpoint").clicked() {
            action = DebuggerAction::AddBreakpoint;
        }

        ui.add_space(10.0);

        if self.breakpoints.is_empty() {
            ui.label(RichText::new("No breakpoints set").color(Color32::GRAY));
        } else {
            for breakpoint in &self.breakpoints {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        let icon = if breakpoint.enabled { "🔴" } else { "⚪" };
                        if ui.button(icon).clicked() {
                            action =
                                DebuggerAction::ToggleBreakpoint(breakpoint.element_id.clone());
                        }

                        ui.label(&breakpoint.element_id);

                        if let Some(condition) = &breakpoint.condition {
                            ui.label(RichText::new(format!("when: {}", condition)).italics());
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("🗑️").clicked() {
                                action =
                                    DebuggerAction::RemoveBreakpoint(breakpoint.element_id.clone());
                            }
                            ui.label(format!("hits: {}", breakpoint.hit_count));
                        });
                    });
                });
            }
        }

        action
    }

    fn render_variables(&mut self, ui: &mut Ui) {
        ui.heading("Process Variables");

        if let Some(context) = &self.context {
            if context.instance.variables.is_empty() {
                ui.label(RichText::new("No variables").color(Color32::GRAY));
            } else {
                egui::Grid::new("variables_grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(RichText::new("Name").strong());
                        ui.label(RichText::new("Value").strong());
                        ui.end_row();

                        for (key, value) in &context.instance.variables {
                            ui.label(RichText::new(key).monospace());
                            ui.label(RichText::new(format!("{}", value)).monospace());
                            ui.end_row();
                        }
                    });
            }
        } else {
            ui.label(RichText::new("No active execution").color(Color32::GRAY));
        }
    }

    fn render_events(&mut self, ui: &mut Ui) {
        ui.heading("Event Log");

        ui.horizontal(|ui| {
            ui.checkbox(&mut self.filter_errors_only, "Errors only");
            ui.checkbox(&mut self.auto_scroll_events, "Auto-scroll");
            if ui.button("🗑️ Clear").clicked() {
                self.recent_events.clear();
            }
        });

        ui.separator();

        let events_to_show: Vec<_> = if self.filter_errors_only {
            self.recent_events
                .iter()
                .filter(|e| {
                    matches!(
                        e,
                        ExecutionEvent::TaskFailed { .. } | ExecutionEvent::ProcessFailed { .. }
                    )
                })
                .collect()
        } else {
            self.recent_events.iter().collect()
        };

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(self.auto_scroll_events)
            .show(ui, |ui| {
                for event in events_to_show.iter().rev() {
                    self.render_event(ui, event);
                }
            });
    }

    fn render_event(&self, ui: &mut Ui, event: &ExecutionEvent) {
        let (icon, text, color) = match event {
            ExecutionEvent::ProcessStarted { timestamp, .. } => (
                "🚀",
                format!("Process started at {}", timestamp.format("%H:%M:%S")),
                Color32::GREEN,
            ),
            ExecutionEvent::TokenCreated { element_id, timestamp, .. } => (
                "🎯",
                format!("Token created at {} ({})", element_id, timestamp.format("%H:%M:%S")),
                Color32::LIGHT_BLUE,
            ),
            ExecutionEvent::TokenMoved { from_element, to_element, timestamp, .. } => (
                "➡️",
                format!(
                    "Token moved {} → {} ({})",
                    from_element,
                    to_element,
                    timestamp.format("%H:%M:%S")
                ),
                Color32::LIGHT_BLUE,
            ),
            ExecutionEvent::TaskStarted { task_id, timestamp, .. } => (
                "▶️",
                format!("Task {} started ({})", task_id, timestamp.format("%H:%M:%S")),
                Color32::YELLOW,
            ),
            ExecutionEvent::TaskCompleted { task_id, duration_ms, timestamp, .. } => (
                "✅",
                format!(
                    "Task {} completed in {}ms ({})",
                    task_id,
                    duration_ms,
                    timestamp.format("%H:%M:%S")
                ),
                Color32::GREEN,
            ),
            ExecutionEvent::TaskFailed { task_id, error, timestamp, .. } => (
                "❌",
                format!("Task {} failed: {} ({})", task_id, error, timestamp.format("%H:%M:%S")),
                Color32::RED,
            ),
            ExecutionEvent::VariableChanged { variable_name, new_value, timestamp, .. } => (
                "💾",
                format!(
                    "Variable {} = {} ({})",
                    variable_name,
                    new_value,
                    timestamp.format("%H:%M:%S")
                ),
                Color32::LIGHT_GREEN,
            ),
            ExecutionEvent::BreakpointHit { element_id, timestamp, .. } => (
                "🔴",
                format!("Breakpoint hit at {} ({})", element_id, timestamp.format("%H:%M:%S")),
                Color32::RED,
            ),
            ExecutionEvent::ProcessCompleted { timestamp, .. } => (
                "🎉",
                format!("Process completed at {}", timestamp.format("%H:%M:%S")),
                Color32::GREEN,
            ),
            ExecutionEvent::ProcessFailed { error, timestamp, .. } => (
                "💥",
                format!("Process failed: {} ({})", error, timestamp.format("%H:%M:%S")),
                Color32::RED,
            ),
            ExecutionEvent::GatewayEvaluated { gateway_id, outgoing_flows, timestamp } => (
                "🔀",
                format!(
                    "Gateway {} evaluated → {} flows ({})",
                    gateway_id,
                    outgoing_flows.len(),
                    timestamp.format("%H:%M:%S")
                ),
                Color32::KHAKI,
            ),
        };

        ui.horizontal(|ui| {
            ui.label(icon);
            ui.colored_label(color, text);
        });
    }

    fn render_performance(&mut self, ui: &mut Ui) {
        ui.heading("Performance Analysis");

        if self.performance.is_empty() {
            ui.label(RichText::new("No performance data").color(Color32::GRAY));
            return;
        }

        // Sort by average duration
        let mut tasks: Vec<_> = self.performance.iter().collect();
        tasks.sort_by(|a, b| b.1.avg_duration_ms.cmp(&a.1.avg_duration_ms));

        ui.group(|ui| {
            ui.label(RichText::new("Task Performance").strong());
            egui::Grid::new("performance_grid")
                .num_columns(6)
                .spacing([20.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label(RichText::new("Task").strong());
                    ui.label(RichText::new("Executions").strong());
                    ui.label(RichText::new("Avg (ms)").strong());
                    ui.label(RichText::new("Min (ms)").strong());
                    ui.label(RichText::new("Max (ms)").strong());
                    ui.label(RichText::new("Total (ms)").strong());
                    ui.end_row();

                    for (task_id, perf) in &tasks {
                        ui.label(*task_id);
                        ui.label(perf.execution_count.to_string());

                        // Highlight slow tasks
                        let avg_color = if perf.avg_duration_ms > 1000 {
                            Color32::RED
                        } else if perf.avg_duration_ms > 500 {
                            Color32::YELLOW
                        } else {
                            Color32::GREEN
                        };

                        ui.colored_label(avg_color, perf.avg_duration_ms.to_string());
                        ui.label(perf.min_duration_ms.to_string());
                        ui.label(perf.max_duration_ms.to_string());
                        ui.label(perf.total_duration_ms.to_string());
                        ui.end_row();
                    }
                });
        });

        ui.add_space(10.0);

        // Bottleneck detection
        ui.group(|ui| {
            ui.label(RichText::new("🐌 Bottleneck Analysis").strong());

            let bottlenecks: Vec<_> =
                tasks.iter().filter(|(_, perf)| perf.avg_duration_ms > 500).take(5).collect();

            if bottlenecks.is_empty() {
                ui.colored_label(Color32::GREEN, "✅ No significant bottlenecks detected");
            } else {
                ui.colored_label(
                    Color32::YELLOW,
                    format!("⚠️ {} potential bottlenecks found:", bottlenecks.len()),
                );

                for (task_id, perf) in bottlenecks {
                    ui.horizontal(|ui| {
                        ui.label("•");
                        ui.label(*task_id);
                        ui.colored_label(
                            Color32::RED,
                            format!(
                                "avg {}ms ({}% of total)",
                                perf.avg_duration_ms,
                                (perf.total_duration_ms as f64
                                    / tasks.iter().map(|(_, p)| p.total_duration_ms).sum::<u64>()
                                        as f64
                                    * 100.0) as u32
                            ),
                        );
                    });
                }
            }
        });
    }
}

/// Actions that can be triggered from the debugger UI
#[derive(Debug, Clone)]
pub enum DebuggerAction {
    None,
    Pause,
    Resume,
    Step,
    Stop,
    AddBreakpoint,
    RemoveBreakpoint(String),
    ToggleBreakpoint(String),
}
