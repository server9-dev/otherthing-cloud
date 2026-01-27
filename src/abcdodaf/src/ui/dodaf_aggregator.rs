use crate::ui::enhanced_nodes::BpmnNodeType;
use crate::ui::workspace::Workspace;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct CapabilityView {
    pub id: String,
    pub name: String,
    pub workflows: Vec<String>,
    pub node_count: usize,
}

#[derive(Clone, Debug)]
pub struct PerformerView {
    pub id: String,
    pub name: String,
    pub role: String,
    pub task_count: usize,
    pub workflows: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct ActivityView {
    pub id: String,
    pub name: String,
    pub workflow: String,
    pub node_label: String,
}

#[derive(Clone, Debug, Default)]
pub struct CostSummary {
    pub total_cost: f64,
    pub by_workflow: HashMap<String, f64>,
    pub by_capability: HashMap<String, f64>,
}

#[derive(Clone, Debug, Default)]
pub struct DurationSummary {
    pub total_duration_minutes: f64,
    pub by_workflow: HashMap<String, f64>,
}

pub struct DodafAggregator {
    capabilities: HashMap<String, CapabilityView>,
    performers: HashMap<String, PerformerView>,
    activities: Vec<ActivityView>,
    cost_summary: CostSummary,
    duration_summary: DurationSummary,
    selected_tab: DodafTab,
}

#[derive(Clone, Copy, PartialEq)]
enum DodafTab {
    Capabilities,
    Performers,
    Activities,
    Costs,
}

impl DodafAggregator {
    pub fn new() -> Self {
        Self {
            capabilities: HashMap::new(),
            performers: HashMap::new(),
            activities: Vec::new(),
            cost_summary: CostSummary::default(),
            duration_summary: DurationSummary::default(),
            selected_tab: DodafTab::Capabilities,
        }
    }

    /// Aggregate DoDAF metadata from all workflows in the workspace
    pub fn aggregate_from_workspace(&mut self, workspace: &Workspace) {
        // Clear previous aggregation
        self.capabilities.clear();
        self.performers.clear();
        self.activities.clear();
        self.cost_summary = CostSummary::default();
        self.duration_summary = DurationSummary::default();

        // Process each workflow
        for doc in workspace.workflows() {
            let workflow_name = doc.display_name();
            let mut workflow_cost = 0.0;
            let mut workflow_duration = 0.0;

            // Process each node in the workflow
            for node_id in doc.snarl.node_ids() {
                let node = &doc.snarl[node_id];

                // Only process nodes with DoDAF metadata
                if let Some(ref dodaf_meta) = node.dodaf_metadata {
                    // Aggregate performers
                    if let Some(ref performer) = dodaf_meta.performer {
                        self.performers
                            .entry(performer.performer_id.clone())
                            .and_modify(|p| {
                                p.task_count += 1;
                                if !p.workflows.contains(&workflow_name) {
                                    p.workflows.push(workflow_name.clone());
                                }
                            })
                            .or_insert_with(|| PerformerView {
                                id: performer.performer_id.clone(),
                                name: performer.performer_id.clone(), // Use ID as name if no name
                                role: performer.role.clone().unwrap_or_default(),
                                task_count: 1,
                                workflows: vec![workflow_name.clone()],
                            });
                    }

                    // Aggregate activities
                    if let Some(ref activity_ref) = dodaf_meta.activity_ref {
                        self.activities.push(ActivityView {
                            id: activity_ref.clone(),
                            name: activity_ref.clone(),
                            workflow: workflow_name.clone(),
                            node_label: node.name().to_string(),
                        });
                    }

                    // Aggregate costs
                    if let Some(ref cost) = dodaf_meta.cost {
                        workflow_cost += cost.amount;
                        self.cost_summary.total_cost += cost.amount;
                    }

                    // Aggregate durations
                    if let Some(ref duration) = dodaf_meta.duration {
                        let minutes = match duration.unit {
                            crate::dodaf::ov5::TimeUnit::Seconds => duration.value / 60.0,
                            crate::dodaf::ov5::TimeUnit::Minutes => duration.value,
                            crate::dodaf::ov5::TimeUnit::Hours => duration.value * 60.0,
                            crate::dodaf::ov5::TimeUnit::Days => duration.value * 60.0 * 8.0,
                            crate::dodaf::ov5::TimeUnit::Weeks => duration.value * 60.0 * 8.0 * 5.0,
                            crate::dodaf::ov5::TimeUnit::Months => duration.value * 60.0 * 8.0 * 20.0,
                        };
                        workflow_duration += minutes;
                        self.duration_summary.total_duration_minutes += minutes;
                    }
                }

                // Default duration estimate for tasks without DoDAF metadata
                if matches!(node.node_type, BpmnNodeType::Task(_)) && node.dodaf_metadata.is_none() {
                    let default_duration = 30.0; // 30 minutes
                    workflow_duration += default_duration;
                    self.duration_summary.total_duration_minutes += default_duration;
                }
            }

            // Store workflow totals
            if workflow_cost > 0.0 {
                self.cost_summary
                    .by_workflow
                    .insert(workflow_name.clone(), workflow_cost);
            }
            if workflow_duration > 0.0 {
                self.duration_summary
                    .by_workflow
                    .insert(workflow_name.clone(), workflow_duration);
            }
        }
    }

    /// Render the DoDAF aggregation panel
    pub fn render(&mut self, ui: &mut egui::Ui, workspace: &Workspace) {
        // Refresh aggregation
        self.aggregate_from_workspace(workspace);

        ui.horizontal(|ui| {
            ui.heading("📊 DoDAF Combined View");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🗙").clicked() {
                    // Signal to close panel (handled by parent)
                }
            });
        });
        ui.separator();

        // Tab selector
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut self.selected_tab,
                DodafTab::Capabilities,
                "🎯 Capabilities",
            );
            ui.selectable_value(&mut self.selected_tab, DodafTab::Performers, "👥 Performers");
            ui.selectable_value(&mut self.selected_tab, DodafTab::Activities, "📋 Activities");
            ui.selectable_value(&mut self.selected_tab, DodafTab::Costs, "💰 Costs");
        });
        ui.separator();

        // Content area
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| match self.selected_tab {
                DodafTab::Capabilities => self.render_capabilities(ui),
                DodafTab::Performers => self.render_performers(ui),
                DodafTab::Activities => self.render_activities(ui),
                DodafTab::Costs => self.render_costs(ui),
            });
    }

    fn render_capabilities(&self, ui: &mut egui::Ui) {
        if self.capabilities.is_empty() {
            ui.label("No capabilities found in workflows");
            return;
        }

        egui::Grid::new("capabilities_grid")
            .striped(true)
            .show(ui, |ui| {
                ui.heading("ID");
                ui.heading("Name");
                ui.heading("Workflows");
                ui.heading("Nodes");
                ui.end_row();

                let mut caps: Vec<_> = self.capabilities.values().collect();
                caps.sort_by(|a, b| a.name.cmp(&b.name));

                for cap in caps {
                    ui.label(&cap.id);
                    ui.label(&cap.name);
                    ui.label(cap.workflows.join(", "));
                    ui.label(cap.node_count.to_string());
                    ui.end_row();
                }
            });
    }

    fn render_performers(&self, ui: &mut egui::Ui) {
        if self.performers.is_empty() {
            ui.label("No performers found in workflows");
            return;
        }

        egui::Grid::new("performers_grid")
            .striped(true)
            .show(ui, |ui| {
                ui.heading("ID");
                ui.heading("Name");
                ui.heading("Role");
                ui.heading("Tasks");
                ui.heading("Workflows");
                ui.end_row();

                let mut perfs: Vec<_> = self.performers.values().collect();
                perfs.sort_by(|a, b| a.name.cmp(&b.name));

                for perf in perfs {
                    ui.label(&perf.id);
                    ui.label(&perf.name);
                    ui.label(&perf.role);
                    ui.label(perf.task_count.to_string());
                    ui.label(perf.workflows.join(", "));
                    ui.end_row();
                }
            });
    }

    fn render_activities(&self, ui: &mut egui::Ui) {
        if self.activities.is_empty() {
            ui.label("No activities found in workflows");
            return;
        }

        egui::Grid::new("activities_grid")
            .striped(true)
            .show(ui, |ui| {
                ui.heading("ID");
                ui.heading("Name");
                ui.heading("Workflow");
                ui.heading("Node");
                ui.end_row();

                for activity in &self.activities {
                    ui.label(&activity.id);
                    ui.label(&activity.name);
                    ui.label(&activity.workflow);
                    ui.label(&activity.node_label);
                    ui.end_row();
                }
            });
    }

    fn render_costs(&self, ui: &mut egui::Ui) {
        ui.heading(format!("Total Cost: ${:.2}", self.cost_summary.total_cost));
        ui.separator();

        if !self.cost_summary.by_workflow.is_empty() {
            ui.heading("By Workflow");
            egui::Grid::new("cost_by_workflow")
                .striped(true)
                .show(ui, |ui| {
                    ui.heading("Workflow");
                    ui.heading("Cost");
                    ui.end_row();

                    let mut workflows: Vec<_> = self.cost_summary.by_workflow.iter().collect();
                    workflows.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

                    for (workflow, cost) in workflows {
                        ui.label(workflow);
                        ui.label(format!("${:.2}", cost));
                        ui.end_row();
                    }
                });
            ui.separator();
        }

        if !self.cost_summary.by_capability.is_empty() {
            ui.heading("By Capability");
            egui::Grid::new("cost_by_capability")
                .striped(true)
                .show(ui, |ui| {
                    ui.heading("Capability");
                    ui.heading("Cost");
                    ui.end_row();

                    let mut caps: Vec<_> = self.cost_summary.by_capability.iter().collect();
                    caps.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

                    for (cap, cost) in caps {
                        ui.label(cap);
                        ui.label(format!("${:.2}", cost));
                        ui.end_row();
                    }
                });
        }

        if self.cost_summary.by_workflow.is_empty() && self.cost_summary.by_capability.is_empty() {
            ui.label("No cost data available");
        }
    }
}

impl Default for DodafAggregator {
    fn default() -> Self {
        Self::new()
    }
}
