use crate::ui::workspace::{Workspace, WorkflowId};

pub struct TabBar {
    tabs: Vec<TabInfo>,
    active_tab: Option<usize>,
}

#[derive(Clone)]
struct TabInfo {
    workflow_id: WorkflowId,
    title: String,
    is_modified: bool,
}

impl TabBar {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab: None,
        }
    }

    /// Update tabs from workspace state
    pub fn update_from_workspace(&mut self, workspace: &Workspace) {
        // Clear and rebuild tabs
        self.tabs.clear();

        let workflow_ids = workspace.workflow_ids();
        for id in workflow_ids {
            if let Some(doc) = workspace.get_workflow(id) {
                self.tabs.push(TabInfo {
                    workflow_id: id,
                    title: doc.display_name(),
                    is_modified: doc.is_modified,
                });
            }
        }

        // Update active tab
        if let Some(active_id) = workspace.active_workflow_id() {
            self.active_tab = self.tabs.iter().position(|t| t.workflow_id == active_id);
        } else {
            self.active_tab = None;
        }
    }

    /// Render the tab bar
    pub fn render(&mut self, ui: &mut egui::Ui, workspace: &mut Workspace) -> Option<TabAction> {
        let mut action = None;

        // Update tabs from workspace
        self.update_from_workspace(workspace);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;

            // Render each tab
            for (idx, tab) in self.tabs.iter().enumerate() {
                let is_active = self.active_tab == Some(idx);

                let mut title = tab.title.clone();
                if tab.is_modified {
                    title = format!("{}*", title);
                }

                // Tab button
                let tab_response = ui.selectable_label(is_active, &title);

                if tab_response.clicked() && !is_active {
                    action = Some(TabAction::SwitchTab(tab.workflow_id));
                }

                // Close button (×)
                let close_response = ui.small_button("×");
                if close_response.clicked() {
                    action = Some(TabAction::CloseTab(tab.workflow_id));
                }

                // Add spacing between tabs
                ui.add_space(4.0);
            }

            // New tab button
            if ui.button("➕").clicked() {
                action = Some(TabAction::NewTab);
            }
        });

        action
    }

    /// Get the number of tabs
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Navigate to next tab
    pub fn next_tab(&mut self, _workspace: &Workspace) -> Option<WorkflowId> {
        if self.tabs.is_empty() {
            return None;
        }

        if let Some(current) = self.active_tab {
            let next_idx = (current + 1) % self.tabs.len();
            self.active_tab = Some(next_idx);
            Some(self.tabs[next_idx].workflow_id)
        } else {
            self.active_tab = Some(0);
            Some(self.tabs[0].workflow_id)
        }
    }

    /// Navigate to previous tab
    pub fn prev_tab(&mut self, _workspace: &Workspace) -> Option<WorkflowId> {
        if self.tabs.is_empty() {
            return None;
        }

        if let Some(current) = self.active_tab {
            let prev_idx = if current == 0 {
                self.tabs.len() - 1
            } else {
                current - 1
            };
            self.active_tab = Some(prev_idx);
            Some(self.tabs[prev_idx].workflow_id)
        } else {
            self.active_tab = Some(0);
            Some(self.tabs[0].workflow_id)
        }
    }
}

impl Default for TabBar {
    fn default() -> Self {
        Self::new()
    }
}

/// Actions that can be triggered from the tab bar
#[derive(Debug, Clone, Copy)]
pub enum TabAction {
    NewTab,
    SwitchTab(WorkflowId),
    CloseTab(WorkflowId),
}
