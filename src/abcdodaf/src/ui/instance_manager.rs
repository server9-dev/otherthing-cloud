//! Process instance manager UI

use crate::bpmn::{ProcessInstance, ProcessState};
use egui::{Color32, RichText, ScrollArea, Ui};
use uuid::Uuid;

/// Instance manager UI state
#[derive(Debug, Clone)]
pub struct InstanceManager {
    /// All instances
    instances: Vec<ProcessInstance>,
    /// Selected instance
    selected_instance: Option<Uuid>,
    /// Filter state
    filter: InstanceFilter,
    /// Sort order
    sort_by: SortBy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceFilter {
    All,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    StartedNewest,
    StartedOldest,
    ProcessId,
    State,
}

impl Default for InstanceManager {
    fn default() -> Self {
        Self {
            instances: Vec::new(),
            selected_instance: None,
            filter: InstanceFilter::All,
            sort_by: SortBy::StartedNewest,
        }
    }
}

impl InstanceManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update instances list
    pub fn update_instances(&mut self, instances: Vec<ProcessInstance>) {
        self.instances = instances;
        self.apply_sort();
    }

    /// Add or update an instance
    pub fn upsert_instance(&mut self, instance: ProcessInstance) {
        if let Some(existing) = self.instances.iter_mut().find(|i| i.id == instance.id) {
            *existing = instance;
        } else {
            self.instances.push(instance);
        }
        self.apply_sort();
    }

    /// Get selected instance
    pub fn selected_instance(&self) -> Option<&ProcessInstance> {
        self.selected_instance
            .and_then(|id| self.instances.iter().find(|i| i.id == id))
    }

    /// Apply current sort
    fn apply_sort(&mut self) {
        match self.sort_by {
            SortBy::StartedNewest => {
                self.instances.sort_by(|a, b| b.started_at.cmp(&a.started_at));
            }
            SortBy::StartedOldest => {
                self.instances.sort_by(|a, b| a.started_at.cmp(&b.started_at));
            }
            SortBy::ProcessId => {
                self.instances.sort_by(|a, b| a.process_id.cmp(&b.process_id));
            }
            SortBy::State => {
                self.instances.sort_by_key(|i| format!("{:?}", i.state));
            }
        }
    }

    /// Render the instance manager
    pub fn ui(&mut self, ui: &mut Ui) -> InstanceManagerAction {
        let mut action = InstanceManagerAction::None;

        // Header with controls
        ui.horizontal(|ui| {
            ui.heading("Process Instances");

            ui.separator();

            // Filter
            ui.label("Filter:");
            egui::ComboBox::from_id_source("instance_filter")
                .selected_text(format!("{:?}", self.filter))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.filter, InstanceFilter::All, "All");
                    ui.selectable_value(&mut self.filter, InstanceFilter::Running, "Running");
                    ui.selectable_value(&mut self.filter, InstanceFilter::Completed, "Completed");
                    ui.selectable_value(&mut self.filter, InstanceFilter::Failed, "Failed");
                    ui.selectable_value(&mut self.filter, InstanceFilter::Cancelled, "Cancelled");
                });

            // Sort
            ui.label("Sort:");
            egui::ComboBox::from_id_source("instance_sort")
                .selected_text(format!("{:?}", self.sort_by))
                .show_ui(ui, |ui| {
                    if ui.selectable_value(&mut self.sort_by, SortBy::StartedNewest, "Newest First").changed() {
                        self.apply_sort();
                    }
                    if ui.selectable_value(&mut self.sort_by, SortBy::StartedOldest, "Oldest First").changed() {
                        self.apply_sort();
                    }
                    if ui.selectable_value(&mut self.sort_by, SortBy::ProcessId, "Process ID").changed() {
                        self.apply_sort();
                    }
                    if ui.selectable_value(&mut self.sort_by, SortBy::State, "State").changed() {
                        self.apply_sort();
                    }
                });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🔄 Refresh").clicked() {
                    action = InstanceManagerAction::Refresh;
                }
            });
        });

        ui.separator();

        // Statistics
        ui.horizontal(|ui| {
            let running = self.instances.iter().filter(|i| i.state == ProcessState::Running).count();
            let completed = self.instances.iter().filter(|i| i.state == ProcessState::Completed).count();
            let failed = self.instances.iter().filter(|i| i.state == ProcessState::Failed).count();

            ui.label(format!("Total: {}", self.instances.len()));
            ui.colored_label(Color32::GREEN, format!("Running: {}", running));
            ui.colored_label(Color32::BLUE, format!("Completed: {}", completed));
            ui.colored_label(Color32::RED, format!("Failed: {}", failed));
        });

        ui.separator();

        // Instance list
        let filtered_instances: Vec<ProcessInstance> = self
            .instances
            .iter()
            .filter(|i| match self.filter {
                InstanceFilter::All => true,
                InstanceFilter::Running => i.state == ProcessState::Running,
                InstanceFilter::Completed => i.state == ProcessState::Completed,
                InstanceFilter::Failed => i.state == ProcessState::Failed,
                InstanceFilter::Cancelled => i.state == ProcessState::Cancelled,
            })
            .cloned()
            .collect();

        if filtered_instances.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new("No instances found").color(Color32::GRAY));
            });
        } else {
            ScrollArea::vertical().show(ui, |ui| {
                for instance in &filtered_instances {
                    let response = self.render_instance_card(ui, instance);
                    if let Some(a) = response {
                        action = a;
                    }
                }
            });
        }

        action
    }

    fn render_instance_card(&mut self, ui: &mut Ui, instance: &ProcessInstance) -> Option<InstanceManagerAction> {
        let mut action = None;
        let is_selected = self.selected_instance == Some(instance.id);

        let response = ui.group(|ui| {
            let mut frame = egui::Frame::none();
            if is_selected {
                frame = frame.fill(Color32::from_rgb(40, 60, 80));
            }

            frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    // State indicator
                    let (state_icon, state_color) = match instance.state {
                        ProcessState::Pending => ("⏳", Color32::GRAY),
                        ProcessState::Running => ("▶️", Color32::GREEN),
                        ProcessState::Suspended => ("⏸️", Color32::YELLOW),
                        ProcessState::Completed => ("✅", Color32::BLUE),
                        ProcessState::Failed => ("❌", Color32::RED),
                        ProcessState::Cancelled => ("🚫", Color32::YELLOW),
                    };

                    ui.label(state_icon);

                    ui.vertical(|ui| {
                        // Instance ID
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Instance:").strong());
                            ui.label(RichText::new(
                                instance.id.to_string().chars().take(8).collect::<String>()
                            ).monospace());
                        });

                        // Process ID
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Process:").small());
                            ui.label(RichText::new(&instance.process_id).small());
                        });

                        // Time info
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Started:").small());
                            ui.label(RichText::new(
                                instance.started_at.format("%Y-%m-%d %H:%M:%S").to_string()
                            ).small());

                            if let Some(completed) = instance.completed_at {
                                let duration = completed - instance.started_at;
                                ui.label(RichText::new(format!(
                                    "Duration: {:.2}s",
                                    duration.num_milliseconds() as f64 / 1000.0
                                )).small());
                            }
                        });
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // State label
                        ui.colored_label(state_color, format!("{:?}", instance.state));

                        ui.separator();

                        // Action buttons
                        match instance.state {
                            ProcessState::Running | ProcessState::Suspended => {
                                if ui.small_button("🔍 Debug").clicked() {
                                    action = Some(InstanceManagerAction::Debug(instance.id));
                                }
                                if ui.small_button("⏸️ Pause").clicked() {
                                    action = Some(InstanceManagerAction::Pause(instance.id));
                                }
                                if ui.small_button("⏹️ Cancel").clicked() {
                                    action = Some(InstanceManagerAction::Cancel(instance.id));
                                }
                            }
                            ProcessState::Completed | ProcessState::Failed | ProcessState::Cancelled => {
                                if ui.small_button("📊 View").clicked() {
                                    action = Some(InstanceManagerAction::View(instance.id));
                                }
                                if ui.small_button("🗑️ Delete").clicked() {
                                    action = Some(InstanceManagerAction::Delete(instance.id));
                                }
                            }
                            ProcessState::Pending => {
                                if ui.small_button("▶️ Start").clicked() {
                                    action = Some(InstanceManagerAction::Start(instance.id));
                                }
                            }
                        }
                    });
                });

                // Variable count
                if !instance.variables.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("💾 {} variables", instance.variables.len())).small().color(Color32::GRAY));
                    });
                }
            });
        });

        // Selection
        if response.response.clicked() {
            self.selected_instance = Some(instance.id);
        }

        action
    }
}

/// Actions that can be triggered from the instance manager
#[derive(Debug, Clone, Copy)]
pub enum InstanceManagerAction {
    None,
    Refresh,
    Start(Uuid),
    Pause(Uuid),
    Resume(Uuid),
    Cancel(Uuid),
    Debug(Uuid),
    View(Uuid),
    Delete(Uuid),
}
