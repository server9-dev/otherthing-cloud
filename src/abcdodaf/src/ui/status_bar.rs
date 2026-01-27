//! Status Bar Component
//!
//! Displays editor status, cursor position, zoom level, and other information.

use egui::{Align, Color32, Layout, RichText, Ui};

/// Status bar information
#[derive(Debug, Clone, Default)]
pub struct StatusBarInfo {
    /// Current cursor position
    pub cursor_pos: Option<(f32, f32)>,

    /// Current zoom level (1.0 = 100%)
    pub zoom: f32,

    /// Number of selected nodes
    pub selected_count: usize,

    /// Total number of nodes
    pub total_nodes: usize,

    /// Number of connections
    pub total_connections: usize,

    /// Validation status
    pub validation_status: ValidationStatus,

    /// Current file path
    pub file_path: Option<String>,

    /// Is file modified
    pub is_modified: bool,

    /// Custom status message
    pub status_message: Option<String>,

    /// Grid settings
    pub grid_enabled: bool,
    pub snap_enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationStatus {
    NotValidated,
    Valid,
    Warning(usize),
    Error(usize),
}

impl Default for ValidationStatus {
    fn default() -> Self {
        Self::NotValidated
    }
}

impl StatusBarInfo {
    pub fn new() -> Self {
        Self {
            zoom: 1.0,
            ..Default::default()
        }
    }
}

/// Status bar component
pub struct StatusBar;

impl StatusBar {
    /// Show the status bar at the bottom of the UI
    pub fn show(ui: &mut Ui, info: &StatusBarInfo) {
        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(24.0)
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    // Left side - status message and validation
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        // Validation status
                        Self::show_validation_status(ui, &info.validation_status);

                        ui.separator();

                        // Status message or default info
                        if let Some(msg) = &info.status_message {
                            ui.label(msg);
                        } else {
                            ui.label(format!(
                                "{} nodes, {} connections",
                                info.total_nodes, info.total_connections
                            ));
                        }

                        // Selection info
                        if info.selected_count > 0 {
                            ui.separator();
                            ui.label(
                                RichText::new(format!("{} selected", info.selected_count))
                                    .color(Color32::from_rgb(100, 150, 255))
                            );
                        }
                    });

                    // Right side - cursor, zoom, grid settings
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        // Grid and snap indicators
                        if info.snap_enabled {
                            ui.label(
                                RichText::new("⊟ Snap")
                                    .color(Color32::from_rgb(0, 200, 0))
                            );
                            ui.separator();
                        }

                        if info.grid_enabled {
                            ui.label(
                                RichText::new("⊞ Grid")
                                    .color(Color32::from_rgb(0, 200, 0))
                            );
                            ui.separator();
                        }

                        // Zoom level
                        ui.label(format!("{}%", (info.zoom * 100.0) as i32));
                        ui.separator();

                        // Cursor position
                        if let Some((x, y)) = info.cursor_pos {
                            ui.label(format!("X: {:.0}, Y: {:.0}", x, y));
                            ui.separator();
                        }

                        // File modified indicator
                        if info.is_modified {
                            ui.label(
                                RichText::new("●")
                                    .color(Color32::from_rgb(255, 165, 0))
                                    .strong()
                            )
                            .on_hover_text("File has unsaved changes");
                            ui.separator();
                        }

                        // File name
                        if let Some(path) = &info.file_path {
                            ui.label(
                                RichText::new(path)
                                    .color(Color32::GRAY)
                            );
                        }
                    });
                });
            });
    }

    fn show_validation_status(ui: &mut Ui, status: &ValidationStatus) {
        match status {
            ValidationStatus::NotValidated => {
                ui.label(
                    RichText::new("○")
                        .color(Color32::GRAY)
                )
                .on_hover_text("Not validated");
            }
            ValidationStatus::Valid => {
                ui.label(
                    RichText::new("✓")
                        .color(Color32::from_rgb(0, 200, 0))
                        .strong()
                )
                .on_hover_text("Validation passed");
            }
            ValidationStatus::Warning(count) => {
                ui.label(
                    RichText::new(format!("⚠ {}", count))
                        .color(Color32::from_rgb(255, 200, 0))
                        .strong()
                )
                .on_hover_text(format!("{} validation warnings", count));
            }
            ValidationStatus::Error(count) => {
                ui.label(
                    RichText::new(format!("✗ {}", count))
                        .color(Color32::from_rgb(255, 0, 0))
                        .strong()
                )
                .on_hover_text(format!("{} validation errors", count));
            }
        }
    }
}

/// Zoom presets for quick zoom levels
pub const ZOOM_PRESETS: &[f32] = &[0.25, 0.5, 0.75, 1.0, 1.5, 2.0, 3.0, 4.0];

/// Get the next zoom level up from the current zoom
pub fn next_zoom_level(current: f32) -> f32 {
    for &preset in ZOOM_PRESETS {
        if preset > current {
            return preset;
        }
    }
    ZOOM_PRESETS.last().copied().unwrap_or(4.0)
}

/// Get the next zoom level down from the current zoom
pub fn previous_zoom_level(current: f32) -> f32 {
    for &preset in ZOOM_PRESETS.iter().rev() {
        if preset < current {
            return preset;
        }
    }
    ZOOM_PRESETS.first().copied().unwrap_or(0.25)
}

/// Clamp zoom to reasonable bounds
pub fn clamp_zoom(zoom: f32) -> f32 {
    zoom.clamp(0.1, 10.0)
}
