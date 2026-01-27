//! Keyboard Shortcuts System
//!
//! Provides comprehensive keyboard shortcuts for all editor operations,
//! with customizable key bindings and conflict detection.

use egui::{Context, Key, KeyboardShortcut, Modifiers};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// All available actions in the editor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EditorAction {
    // File operations
    NewFile,
    OpenFile,
    Save,
    SaveAs,
    CloseFile,
    CloseAllFiles,

    // Edit operations
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    Delete,
    SelectAll,
    DeselectAll,
    DuplicateSelection,

    // View operations
    ZoomIn,
    ZoomOut,
    ZoomReset,
    ZoomFit,
    ToggleMinimap,
    ToggleGrid,
    ToggleSnapping,

    // Node operations
    AddStartEvent,
    AddEndEvent,
    AddTask,
    AddGateway,
    AlignLeft,
    AlignRight,
    AlignTop,
    AlignBottom,
    AlignCenterHorizontal,
    AlignCenterVertical,
    DistributeHorizontal,
    DistributeVertical,

    // Validation
    Validate,
    ToggleValidationPanel,

    // Search
    Find,
    FindNext,
    FindPrevious,
    Replace,

    // Misc
    ToggleProperties,
    ToggleToolbar,
    ToggleStatusBar,
    ToggleFileBrowser,
    ShowHelp,
}

impl EditorAction {
    /// Get a human-readable name for this action
    pub fn name(&self) -> &'static str {
        match self {
            Self::NewFile => "New File",
            Self::OpenFile => "Open File",
            Self::Save => "Save",
            Self::SaveAs => "Save As",
            Self::CloseFile => "Close File",
            Self::CloseAllFiles => "Close All Files",
            Self::Undo => "Undo",
            Self::Redo => "Redo",
            Self::Cut => "Cut",
            Self::Copy => "Copy",
            Self::Paste => "Paste",
            Self::Delete => "Delete",
            Self::SelectAll => "Select All",
            Self::DeselectAll => "Deselect All",
            Self::DuplicateSelection => "Duplicate Selection",
            Self::ZoomIn => "Zoom In",
            Self::ZoomOut => "Zoom Out",
            Self::ZoomReset => "Reset Zoom",
            Self::ZoomFit => "Fit to Screen",
            Self::ToggleMinimap => "Toggle Minimap",
            Self::ToggleGrid => "Toggle Grid",
            Self::ToggleSnapping => "Toggle Snapping",
            Self::AddStartEvent => "Add Start Event",
            Self::AddEndEvent => "Add End Event",
            Self::AddTask => "Add Task",
            Self::AddGateway => "Add Gateway",
            Self::AlignLeft => "Align Left",
            Self::AlignRight => "Align Right",
            Self::AlignTop => "Align Top",
            Self::AlignBottom => "Align Bottom",
            Self::AlignCenterHorizontal => "Align Center Horizontal",
            Self::AlignCenterVertical => "Align Center Vertical",
            Self::DistributeHorizontal => "Distribute Horizontal",
            Self::DistributeVertical => "Distribute Vertical",
            Self::Validate => "Validate Workflow",
            Self::ToggleValidationPanel => "Toggle Validation Panel",
            Self::Find => "Find",
            Self::FindNext => "Find Next",
            Self::FindPrevious => "Find Previous",
            Self::Replace => "Replace",
            Self::ToggleProperties => "Toggle Properties",
            Self::ToggleToolbar => "Toggle Toolbar",
            Self::ToggleStatusBar => "Toggle Status Bar",
            Self::ToggleFileBrowser => "Toggle File Browser",
            Self::ShowHelp => "Show Help",
        }
    }

    /// Get a description for this action
    pub fn description(&self) -> &'static str {
        match self {
            Self::NewFile => "Create a new workflow",
            Self::OpenFile => "Open an existing workflow",
            Self::Save => "Save the current workflow",
            Self::SaveAs => "Save the current workflow with a new name",
            Self::CloseFile => "Close the current workflow",
            Self::CloseAllFiles => "Close all open workflows",
            Self::Undo => "Undo the last action",
            Self::Redo => "Redo the last undone action",
            Self::Cut => "Cut selected nodes to clipboard",
            Self::Copy => "Copy selected nodes to clipboard",
            Self::Paste => "Paste nodes from clipboard",
            Self::Delete => "Delete selected nodes",
            Self::SelectAll => "Select all nodes",
            Self::DeselectAll => "Deselect all nodes",
            Self::DuplicateSelection => "Duplicate selected nodes",
            Self::ZoomIn => "Increase zoom level",
            Self::ZoomOut => "Decrease zoom level",
            Self::ZoomReset => "Reset zoom to 100%",
            Self::ZoomFit => "Fit entire workflow to screen",
            Self::ToggleMinimap => "Show/hide minimap",
            Self::ToggleGrid => "Show/hide grid",
            Self::ToggleSnapping => "Enable/disable grid snapping",
            Self::AddStartEvent => "Add a start event",
            Self::AddEndEvent => "Add an end event",
            Self::AddTask => "Add a task",
            Self::AddGateway => "Add a gateway",
            Self::AlignLeft => "Align selected nodes to the left",
            Self::AlignRight => "Align selected nodes to the right",
            Self::AlignTop => "Align selected nodes to the top",
            Self::AlignBottom => "Align selected nodes to the bottom",
            Self::AlignCenterHorizontal => "Align selected nodes horizontally",
            Self::AlignCenterVertical => "Align selected nodes vertically",
            Self::DistributeHorizontal => "Distribute selected nodes horizontally",
            Self::DistributeVertical => "Distribute selected nodes vertically",
            Self::Validate => "Validate the current workflow",
            Self::ToggleValidationPanel => "Show/hide validation panel",
            Self::Find => "Find nodes by name or properties",
            Self::FindNext => "Find next occurrence",
            Self::FindPrevious => "Find previous occurrence",
            Self::Replace => "Find and replace",
            Self::ToggleProperties => "Show/hide properties panel",
            Self::ToggleToolbar => "Show/hide toolbar",
            Self::ToggleStatusBar => "Show/hide status bar",
            Self::ToggleFileBrowser => "Show/hide file browser",
            Self::ShowHelp => "Show keyboard shortcuts help",
        }
    }
}

/// Keyboard shortcut manager
#[derive(Debug, Clone)]
pub struct ShortcutManager {
    /// Map from action to keyboard shortcut
    bindings: HashMap<EditorAction, KeyboardShortcut>,

    /// Map from keyboard shortcut to action (for quick lookup)
    reverse_bindings: HashMap<KeyboardShortcut, EditorAction>,
}

impl ShortcutManager {
    pub fn new() -> Self {
        let mut manager = Self { bindings: HashMap::new(), reverse_bindings: HashMap::new() };

        manager.load_default_bindings();
        manager
    }

    /// Load default keyboard shortcuts
    fn load_default_bindings(&mut self) {
        let ctrl = Modifiers::CTRL;
        let shift = Modifiers::SHIFT;
        let alt = Modifiers::ALT;
        let ctrl_shift = Modifiers::CTRL | Modifiers::SHIFT;

        // File operations
        self.bind(EditorAction::NewFile, ctrl, Key::N);
        self.bind(EditorAction::OpenFile, ctrl, Key::O);
        self.bind(EditorAction::Save, ctrl, Key::S);
        self.bind(EditorAction::SaveAs, ctrl_shift, Key::S);
        self.bind(EditorAction::CloseFile, ctrl, Key::W);
        self.bind(EditorAction::CloseAllFiles, ctrl_shift, Key::W);

        // Edit operations
        self.bind(EditorAction::Undo, ctrl, Key::Z);
        self.bind(EditorAction::Redo, ctrl, Key::Y);
        self.bind(EditorAction::Cut, ctrl, Key::X);
        self.bind(EditorAction::Copy, ctrl, Key::C);
        self.bind(EditorAction::Paste, ctrl, Key::V);
        self.bind(EditorAction::Delete, Modifiers::NONE, Key::Delete);
        self.bind(EditorAction::SelectAll, ctrl, Key::A);
        self.bind(EditorAction::DeselectAll, ctrl, Key::D);
        self.bind(EditorAction::DuplicateSelection, ctrl, Key::D);

        // View operations
        self.bind(EditorAction::ZoomIn, ctrl, Key::Equals); // Ctrl + =
        self.bind(EditorAction::ZoomOut, ctrl, Key::Minus);
        self.bind(EditorAction::ZoomReset, ctrl, Key::Num0);
        self.bind(EditorAction::ZoomFit, ctrl_shift, Key::F);
        self.bind(EditorAction::ToggleMinimap, ctrl, Key::M);
        self.bind(EditorAction::ToggleGrid, ctrl, Key::G);
        self.bind(EditorAction::ToggleSnapping, ctrl_shift, Key::G);

        // Validation
        self.bind(EditorAction::Validate, ctrl_shift, Key::V);
        self.bind(EditorAction::ToggleValidationPanel, ctrl, Key::E);

        // Search
        self.bind(EditorAction::Find, ctrl, Key::F);
        self.bind(EditorAction::FindNext, Modifiers::NONE, Key::F3);
        self.bind(EditorAction::FindPrevious, shift, Key::F3);
        self.bind(EditorAction::Replace, ctrl, Key::H);

        // Panels
        self.bind(EditorAction::ToggleProperties, ctrl, Key::P);
        self.bind(EditorAction::ToggleToolbar, alt, Key::T);
        self.bind(EditorAction::ToggleStatusBar, alt, Key::S);
        self.bind(EditorAction::ToggleFileBrowser, ctrl, Key::B);
        self.bind(EditorAction::ShowHelp, Modifiers::NONE, Key::F1);
    }

    /// Bind an action to a keyboard shortcut
    pub fn bind(&mut self, action: EditorAction, modifiers: Modifiers, logical_key: Key) {
        let shortcut = KeyboardShortcut::new(modifiers, logical_key);

        // Remove old binding if it exists
        if let Some(old_shortcut) = self.bindings.remove(&action) {
            self.reverse_bindings.remove(&old_shortcut);
        }

        self.bindings.insert(action, shortcut);
        self.reverse_bindings.insert(shortcut, action);
    }

    /// Unbind an action
    pub fn unbind(&mut self, action: EditorAction) {
        if let Some(shortcut) = self.bindings.remove(&action) {
            self.reverse_bindings.remove(&shortcut);
        }
    }

    /// Get the shortcut for an action
    pub fn get_shortcut(&self, action: EditorAction) -> Option<KeyboardShortcut> {
        self.bindings.get(&action).copied()
    }

    /// Get the action for a shortcut
    pub fn get_action(&self, shortcut: KeyboardShortcut) -> Option<EditorAction> {
        self.reverse_bindings.get(&shortcut).copied()
    }

    /// Check if a shortcut is pressed in the current frame
    pub fn is_pressed(&self, ctx: &Context, action: EditorAction) -> bool {
        if let Some(shortcut) = self.get_shortcut(action) {
            ctx.input_mut(|i| i.consume_shortcut(&shortcut))
        } else {
            false
        }
    }

    /// Check if a shortcut is pressed without consuming it
    pub fn check_pressed(&self, ctx: &Context, action: EditorAction) -> bool {
        if let Some(shortcut) = self.get_shortcut(action) {
            ctx.input(|i| {
                i.modifiers.matches_exact(shortcut.modifiers) && i.key_pressed(shortcut.logical_key)
            })
        } else {
            false
        }
    }

    /// Format a shortcut for display
    pub fn format_shortcut(&self, action: EditorAction) -> Option<String> {
        self.get_shortcut(action).map(|s| format_keyboard_shortcut(&s))
    }

    /// Get all bindings
    pub fn all_bindings(&self) -> Vec<(EditorAction, KeyboardShortcut)> {
        self.bindings.iter().map(|(&a, &s)| (a, s)).collect()
    }
}

impl Default for ShortcutManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Format a keyboard shortcut for display
pub fn format_keyboard_shortcut(shortcut: &KeyboardShortcut) -> String {
    let mut parts = Vec::new();

    if shortcut.modifiers.ctrl {
        parts.push("Ctrl".to_string());
    }
    if shortcut.modifiers.shift {
        parts.push("Shift".to_string());
    }
    if shortcut.modifiers.alt {
        parts.push("Alt".to_string());
    }
    #[cfg(target_os = "macos")]
    if shortcut.modifiers.mac_cmd {
        parts.push("Cmd".to_string());
    }

    parts.push(format!("{:?}", shortcut.logical_key));

    parts.join("+")
}

/// Helper to show keyboard shortcuts in the UI
pub fn show_shortcut_help(ui: &mut egui::Ui, manager: &ShortcutManager) {
    ui.heading("Keyboard Shortcuts");
    ui.separator();

    // Group actions by category
    let categories = vec![
        (
            "File Operations",
            vec![
                EditorAction::NewFile,
                EditorAction::OpenFile,
                EditorAction::Save,
                EditorAction::SaveAs,
                EditorAction::CloseFile,
            ],
        ),
        (
            "Edit Operations",
            vec![
                EditorAction::Undo,
                EditorAction::Redo,
                EditorAction::Cut,
                EditorAction::Copy,
                EditorAction::Paste,
                EditorAction::Delete,
                EditorAction::SelectAll,
                EditorAction::DuplicateSelection,
            ],
        ),
        (
            "View Operations",
            vec![
                EditorAction::ZoomIn,
                EditorAction::ZoomOut,
                EditorAction::ZoomReset,
                EditorAction::ZoomFit,
                EditorAction::ToggleMinimap,
                EditorAction::ToggleGrid,
                EditorAction::ToggleSnapping,
            ],
        ),
        (
            "Search & Validation",
            vec![
                EditorAction::Find,
                EditorAction::FindNext,
                EditorAction::Replace,
                EditorAction::Validate,
            ],
        ),
        (
            "Panels",
            vec![
                EditorAction::ToggleProperties,
                EditorAction::ToggleFileBrowser,
                EditorAction::ToggleValidationPanel,
            ],
        ),
    ];

    for (category, actions) in categories {
        ui.collapsing(category, |ui| {
            egui::Grid::new(category).num_columns(2).striped(true).show(ui, |ui| {
                for action in actions {
                    ui.label(action.name());
                    if let Some(shortcut_str) = manager.format_shortcut(action) {
                        ui.label(egui::RichText::new(shortcut_str).monospace());
                    } else {
                        ui.label("-");
                    }
                    ui.end_row();
                }
            });
        });
    }
}

// ============================================================================
// Action Handler Trait
// ============================================================================

/// Trait for handling editor actions
pub trait ActionHandler {
    fn handle_action(&mut self, action: EditorAction, ctx: &Context);
}

/// Helper function to poll for keyboard shortcuts and execute actions
pub fn poll_shortcuts<H: ActionHandler>(ctx: &Context, manager: &ShortcutManager, handler: &mut H) {
    // Check all registered shortcuts
    for (action, shortcut) in manager.all_bindings() {
        if ctx.input_mut(|i| i.consume_shortcut(&shortcut)) {
            handler.handle_action(action, ctx);
        }
    }
}
