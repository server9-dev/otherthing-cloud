//! Toolbar Component
//!
//! Provides a comprehensive toolbar with common actions and tools.

use super::shortcuts::EditorAction;
use egui::{Response, RichText, Ui, Vec2};

/// Toolbar button style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolbarButtonStyle {
    Normal,
    Toggle,
    Separator,
}

/// Toolbar button
#[derive(Debug, Clone)]
pub struct ToolbarButton {
    pub action: Option<EditorAction>,
    pub icon: String,
    pub label: Option<String>,
    pub tooltip: String,
    pub style: ToolbarButtonStyle,
    pub enabled: bool,
    pub active: bool,
}

impl ToolbarButton {
    pub fn new(action: EditorAction, icon: impl Into<String>, tooltip: impl Into<String>) -> Self {
        Self {
            action: Some(action),
            icon: icon.into(),
            label: None,
            tooltip: tooltip.into(),
            style: ToolbarButtonStyle::Normal,
            enabled: true,
            active: false,
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn toggle(mut self) -> Self {
        self.style = ToolbarButtonStyle::Toggle;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn separator() -> Self {
        Self {
            action: None,
            icon: String::new(),
            label: None,
            tooltip: String::new(),
            style: ToolbarButtonStyle::Separator,
            enabled: false,
            active: false,
        }
    }
}

/// Toolbar component
pub struct Toolbar {
    buttons: Vec<ToolbarButton>,
    show_labels: bool,
    icon_size: f32,
}

impl Toolbar {
    pub fn new() -> Self {
        Self { buttons: Vec::new(), show_labels: false, icon_size: 20.0 }
    }

    pub fn with_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }

    pub fn with_icon_size(mut self, size: f32) -> Self {
        self.icon_size = size;
        self
    }

    pub fn add_button(&mut self, button: ToolbarButton) {
        self.buttons.push(button);
    }

    pub fn add_separator(&mut self) {
        self.buttons.push(ToolbarButton::separator());
    }

    /// Show the toolbar and return the action that was clicked (if any)
    pub fn show(&mut self, ui: &mut Ui) -> Option<EditorAction> {
        let mut clicked_action = None;

        ui.horizontal(|ui| {
            ui.style_mut().spacing.item_spacing = Vec2::new(4.0, 4.0);

            for button in &self.buttons {
                match button.style {
                    ToolbarButtonStyle::Separator => {
                        ui.separator();
                    },
                    ToolbarButtonStyle::Normal | ToolbarButtonStyle::Toggle => {
                        let btn_response = self.show_button(ui, button);
                        if btn_response.clicked() {
                            if let Some(action) = button.action {
                                clicked_action = Some(action);
                            }
                        }
                    },
                }
            }
        });

        clicked_action
    }

    fn show_button(&self, ui: &mut Ui, button: &ToolbarButton) -> Response {
        let text = if self.show_labels {
            if let Some(label) = &button.label {
                format!("{} {}", button.icon, label)
            } else {
                button.icon.clone()
            }
        } else {
            button.icon.clone()
        };

        let button_text = RichText::new(text).size(self.icon_size);

        let mut btn = if button.style == ToolbarButtonStyle::Toggle {
            ui.selectable_label(button.active, button_text)
        } else {
            ui.button(button_text)
        };

        if !button.enabled {
            btn = btn.on_disabled_hover_text(&button.tooltip);
        } else {
            btn = btn.on_hover_text(&button.tooltip);
        }

        btn
    }
}

impl Default for Toolbar {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a standard toolbar with common actions
pub fn create_standard_toolbar(
    can_undo: bool,
    can_redo: bool,
    has_selection: bool,
    has_clipboard: bool,
    show_grid: bool,
    snap_to_grid: bool,
    show_minimap: bool,
) -> Toolbar {
    let mut toolbar = Toolbar::new();

    // File operations
    toolbar.add_button(ToolbarButton::new(EditorAction::NewFile, "📄", "New workflow (Ctrl+N)"));
    toolbar.add_button(ToolbarButton::new(EditorAction::OpenFile, "📁", "Open workflow (Ctrl+O)"));
    toolbar.add_button(ToolbarButton::new(EditorAction::Save, "💾", "Save workflow (Ctrl+S)"));
    toolbar.add_separator();

    // Edit operations
    toolbar
        .add_button(ToolbarButton::new(EditorAction::Undo, "↶", "Undo (Ctrl+Z)").enabled(can_undo));
    toolbar
        .add_button(ToolbarButton::new(EditorAction::Redo, "↷", "Redo (Ctrl+Y)").enabled(can_redo));
    toolbar.add_separator();

    toolbar.add_button(
        ToolbarButton::new(EditorAction::Cut, "✂", "Cut (Ctrl+X)").enabled(has_selection),
    );
    toolbar.add_button(
        ToolbarButton::new(EditorAction::Copy, "📋", "Copy (Ctrl+C)").enabled(has_selection),
    );
    toolbar.add_button(
        ToolbarButton::new(EditorAction::Paste, "📄", "Paste (Ctrl+V)").enabled(has_clipboard),
    );
    toolbar.add_button(
        ToolbarButton::new(EditorAction::Delete, "🗑", "Delete (Del)").enabled(has_selection),
    );
    toolbar.add_separator();

    // View operations
    toolbar.add_button(ToolbarButton::new(EditorAction::ZoomIn, "🔍+", "Zoom in (Ctrl+=)"));
    toolbar.add_button(ToolbarButton::new(EditorAction::ZoomOut, "🔍-", "Zoom out (Ctrl+-)"));
    toolbar.add_button(ToolbarButton::new(
        EditorAction::ZoomFit,
        "⊡",
        "Fit to screen (Ctrl+Shift+F)",
    ));
    toolbar.add_separator();

    // Grid and snapping
    toolbar.add_button(
        ToolbarButton::new(EditorAction::ToggleGrid, "⊞", "Toggle grid (Ctrl+G)")
            .toggle()
            .active(show_grid),
    );
    toolbar.add_button(
        ToolbarButton::new(EditorAction::ToggleSnapping, "⊟", "Toggle snapping (Ctrl+Shift+G)")
            .toggle()
            .active(snap_to_grid),
    );
    toolbar.add_separator();

    // Minimap
    toolbar.add_button(
        ToolbarButton::new(EditorAction::ToggleMinimap, "🗺", "Toggle minimap (Ctrl+M)")
            .toggle()
            .active(show_minimap),
    );
    toolbar.add_separator();

    // Validation
    toolbar.add_button(ToolbarButton::new(
        EditorAction::Validate,
        "✓",
        "Validate workflow (Ctrl+Shift+V)",
    ));

    // Help
    toolbar.add_button(ToolbarButton::new(EditorAction::ShowHelp, "❓", "Show help (F1)"));

    toolbar
}

/// Create a node creation toolbar
pub fn create_node_toolbar() -> Toolbar {
    let mut toolbar = Toolbar::new().with_labels(true);

    // Events
    toolbar.add_button(
        ToolbarButton::new(EditorAction::AddStartEvent, "▶", "Add Start Event").with_label("Start"),
    );
    toolbar.add_button(
        ToolbarButton::new(EditorAction::AddEndEvent, "⏹", "Add End Event").with_label("End"),
    );
    toolbar.add_separator();

    // Tasks
    toolbar
        .add_button(ToolbarButton::new(EditorAction::AddTask, "⚙", "Add Task").with_label("Task"));
    toolbar.add_separator();

    // Gateways
    toolbar.add_button(
        ToolbarButton::new(EditorAction::AddGateway, "◇", "Add Gateway").with_label("Gateway"),
    );

    toolbar
}

/// Alignment toolbar
pub fn create_alignment_toolbar(has_selection: bool) -> Toolbar {
    let mut toolbar = Toolbar::new();

    toolbar.add_button(
        ToolbarButton::new(EditorAction::AlignLeft, "⊣", "Align left").enabled(has_selection),
    );
    toolbar.add_button(
        ToolbarButton::new(EditorAction::AlignCenterHorizontal, "⊢", "Align center horizontal")
            .enabled(has_selection),
    );
    toolbar.add_button(
        ToolbarButton::new(EditorAction::AlignRight, "⊢", "Align right").enabled(has_selection),
    );
    toolbar.add_separator();

    toolbar.add_button(
        ToolbarButton::new(EditorAction::AlignTop, "⊤", "Align top").enabled(has_selection),
    );
    toolbar.add_button(
        ToolbarButton::new(EditorAction::AlignCenterVertical, "⊥", "Align center vertical")
            .enabled(has_selection),
    );
    toolbar.add_button(
        ToolbarButton::new(EditorAction::AlignBottom, "⊥", "Align bottom").enabled(has_selection),
    );
    toolbar.add_separator();

    toolbar.add_button(
        ToolbarButton::new(EditorAction::DistributeHorizontal, "⟷", "Distribute horizontal")
            .enabled(has_selection),
    );
    toolbar.add_button(
        ToolbarButton::new(EditorAction::DistributeVertical, "⟥", "Distribute vertical")
            .enabled(has_selection),
    );

    toolbar
}
