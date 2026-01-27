//! Execution visualizer - overlays execution state on BPMN diagram

use crate::bpmn::runtime::{ExecutionContext, ExecutionToken, TokenState};
use egui::{Color32, Painter, Rect, Stroke, Vec2};
use std::collections::HashMap;

/// Execution visualization overlay
#[derive(Debug, Clone)]
pub struct ExecutionVisualizer {
    /// Current execution context
    context: Option<ExecutionContext>,
    /// Element positions (from BPMN diagram)
    element_positions: HashMap<String, Rect>,
    /// Animation state
    animation_time: f32,
    /// Show token flows
    show_tokens: bool,
    /// Show performance heatmap
    show_heatmap: bool,
}

impl Default for ExecutionVisualizer {
    fn default() -> Self {
        Self {
            context: None,
            element_positions: HashMap::new(),
            animation_time: 0.0,
            show_tokens: true,
            show_heatmap: false,
        }
    }
}

impl ExecutionVisualizer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update execution context
    pub fn update_context(&mut self, context: ExecutionContext) {
        self.context = Some(context);
    }

    /// Update element positions from BPMN diagram
    pub fn update_positions(&mut self, positions: HashMap<String, Rect>) {
        self.element_positions = positions;
    }

    /// Update animation
    pub fn update_animation(&mut self, delta: f32) {
        self.animation_time += delta;
        if self.animation_time > 2.0 * std::f32::consts::PI {
            self.animation_time -= 2.0 * std::f32::consts::PI;
        }
    }

    /// Render execution overlay
    pub fn render(&self, painter: &Painter, _viewport: Rect) {
        if let Some(context) = &self.context {
            // Highlight active elements
            if let Some(current) = &context.current_element {
                if let Some(rect) = self.element_positions.get(current) {
                    self.draw_active_highlight(painter, *rect);
                }
            }

            // Draw tokens
            if self.show_tokens {
                for token in &context.tokens {
                    if let Some(rect) = self.element_positions.get(&token.current_element) {
                        self.draw_token(painter, *rect, token);
                    }
                }
            }

            // Draw performance heatmap
            if self.show_heatmap {
                self.draw_performance_heatmap(painter, &context);
            }
        }
    }

    /// Draw active element highlight (pulsing border)
    fn draw_active_highlight(&self, painter: &Painter, rect: Rect) {
        // Pulsing animation
        let pulse = (self.animation_time.sin() + 1.0) / 2.0; // 0 to 1
        let width = 3.0 + pulse * 2.0;
        let alpha = (150.0 + pulse * 100.0) as u8;

        // Outer glow
        let glow_color = Color32::from_rgba_premultiplied(0, 200, 255, alpha);
        painter.rect_stroke(
            rect.expand(width * 2.0),
            4.0,
            Stroke::new(width, glow_color),
            egui::epaint::StrokeKind::Outside,
        );

        // Inner highlight
        let highlight_color = Color32::from_rgba_premultiplied(100, 220, 255, 200);
        painter.rect_stroke(rect.expand(2.0), 2.0, Stroke::new(2.0, highlight_color), egui::epaint::StrokeKind::Outside);
    }

    /// Draw execution token
    fn draw_token(&self, painter: &Painter, rect: Rect, token: &ExecutionToken) {
        let center = rect.center();

        // Offset for multiple tokens at same position
        let offset = Vec2::new(
            (token.id.as_u128() % 20) as f32 - 10.0,
            (token.id.as_u128() / 20 % 20) as f32 - 10.0,
        );
        let pos = center + offset;

        // Token color based on state
        let (color, size) = match token.state {
            TokenState::Active => (Color32::from_rgb(0, 255, 0), 8.0),
            TokenState::Waiting => (Color32::from_rgb(255, 255, 0), 6.0),
            TokenState::Blocked => (Color32::from_rgb(255, 0, 0), 8.0),
            TokenState::Consumed => (Color32::from_rgb(128, 128, 128), 4.0),
        };

        // Draw token with glow effect
        painter.circle_filled(pos, size + 2.0, Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 50));
        painter.circle_filled(pos, size, color);
        painter.circle_stroke(pos, size, Stroke::new(1.0, Color32::WHITE));

        // Pulsing effect for active tokens
        if token.state == TokenState::Active {
            let pulse_size = size + (self.animation_time.sin() + 1.0) * 3.0;
            let pulse_alpha = ((1.0 - (self.animation_time.sin() + 1.0) / 2.0) * 100.0) as u8;
            painter.circle_stroke(
                pos,
                pulse_size,
                Stroke::new(2.0, Color32::from_rgba_premultiplied(0, 255, 0, pulse_alpha)),
            );
        }
    }

    /// Draw performance heatmap overlay
    fn draw_performance_heatmap(&self, painter: &Painter, context: &ExecutionContext) {
        if context.performance.is_empty() {
            return;
        }

        // Find max duration for normalization
        let max_duration = context
            .performance
            .values()
            .map(|p| p.avg_duration_ms)
            .max()
            .unwrap_or(1);

        for (element_id, perf) in &context.performance {
            if let Some(rect) = self.element_positions.get(element_id) {
                // Normalize duration to 0-1
                let intensity = (perf.avg_duration_ms as f32 / max_duration as f32).min(1.0);

                // Color gradient: green (fast) -> yellow -> red (slow)
                let color = if intensity < 0.5 {
                    // Green to yellow
                    let t = intensity * 2.0;
                    Color32::from_rgba_premultiplied(
                        (255.0 * t) as u8,
                        255,
                        0,
                        100,
                    )
                } else {
                    // Yellow to red
                    let t = (intensity - 0.5) * 2.0;
                    Color32::from_rgba_premultiplied(
                        255,
                        (255.0 * (1.0 - t)) as u8,
                        0,
                        100,
                    )
                };

                // Draw semi-transparent overlay
                painter.rect_filled(*rect, 2.0, color);

                // Draw duration text
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("{}ms", perf.avg_duration_ms),
                    egui::FontId::monospace(10.0),
                    Color32::WHITE,
                );
            }
        }
    }

    /// Get visual state for an element (for external rendering)
    pub fn get_element_state(&self, element_id: &str) -> ElementVisualState {
        if let Some(context) = &self.context {
            // Check if element is currently active
            if let Some(current) = &context.current_element {
                if current == element_id {
                    return ElementVisualState::Active;
                }
            }

            // Check if any token is at this element
            for token in &context.tokens {
                if token.current_element == element_id {
                    return match token.state {
                        TokenState::Active => ElementVisualState::HasActiveToken,
                        TokenState::Waiting => ElementVisualState::HasWaitingToken,
                        TokenState::Blocked => ElementVisualState::Blocked,
                        TokenState::Consumed => ElementVisualState::Idle,
                    };
                }
            }

            // Check if element has completed
            if context.performance.contains_key(element_id) {
                return ElementVisualState::Completed;
            }
        }

        ElementVisualState::Idle
    }

    /// Get performance data for an element
    pub fn get_element_performance(&self, element_id: &str) -> Option<ElementPerformanceView> {
        self.context.as_ref().and_then(|ctx| {
            ctx.performance.get(element_id).map(|perf| {
                ElementPerformanceView {
                    avg_duration_ms: perf.avg_duration_ms,
                    execution_count: perf.execution_count,
                    is_bottleneck: perf.avg_duration_ms > 500, // Simple threshold
                }
            })
        })
    }

    /// Toggle token visualization
    pub fn toggle_tokens(&mut self) {
        self.show_tokens = !self.show_tokens;
    }

    /// Toggle performance heatmap
    pub fn toggle_heatmap(&mut self) {
        self.show_heatmap = !self.show_heatmap;
    }

    /// Clear current execution
    pub fn clear(&mut self) {
        self.context = None;
    }
}

/// Visual state of a BPMN element
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementVisualState {
    /// Element is idle/not executed
    Idle,
    /// Element is currently active/executing
    Active,
    /// Element has an active token
    HasActiveToken,
    /// Element has a waiting token
    HasWaitingToken,
    /// Element is blocked (breakpoint)
    Blocked,
    /// Element has completed execution
    Completed,
}

impl ElementVisualState {
    /// Get color for this state
    pub fn color(&self) -> Color32 {
        match self {
            Self::Idle => Color32::GRAY,
            Self::Active => Color32::from_rgb(0, 200, 255),
            Self::HasActiveToken => Color32::from_rgb(0, 255, 0),
            Self::HasWaitingToken => Color32::from_rgb(255, 255, 0),
            Self::Blocked => Color32::from_rgb(255, 0, 0),
            Self::Completed => Color32::from_rgb(100, 255, 100),
        }
    }

    /// Get stroke width for this state
    pub fn stroke_width(&self) -> f32 {
        match self {
            Self::Idle => 1.0,
            Self::Active => 3.0,
            Self::HasActiveToken => 2.5,
            Self::HasWaitingToken => 2.0,
            Self::Blocked => 3.0,
            Self::Completed => 1.5,
        }
    }
}

/// Performance view for an element
#[derive(Debug, Clone)]
pub struct ElementPerformanceView {
    pub avg_duration_ms: u64,
    pub execution_count: u64,
    pub is_bottleneck: bool,
}

/// Control panel for execution visualization
pub struct ExecutionControlPanel {
    show_tokens: bool,
    show_heatmap: bool,
    show_bottlenecks: bool,
}

impl Default for ExecutionControlPanel {
    fn default() -> Self {
        Self {
            show_tokens: true,
            show_heatmap: false,
            show_bottlenecks: true,
        }
    }
}

impl ExecutionControlPanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> ExecutionVisualizerAction {
        let mut action = ExecutionVisualizerAction::None;

        ui.heading("Visualization Options");

        ui.checkbox(&mut self.show_tokens, "🎯 Show Tokens");
        if ui.checkbox(&mut self.show_heatmap, "🔥 Performance Heatmap").changed() {
            action = ExecutionVisualizerAction::ToggleHeatmap(self.show_heatmap);
        }
        ui.checkbox(&mut self.show_bottlenecks, "🐌 Highlight Bottlenecks");

        ui.separator();

        ui.label("Legend:");
        ui.horizontal(|ui| {
            let mut color = Color32::from_rgb(0, 200, 255);
            ui.color_edit_button_srgba(&mut color);
            ui.label("Active");
        });
        ui.horizontal(|ui| {
            let mut color = Color32::from_rgb(0, 255, 0);
            ui.color_edit_button_srgba(&mut color);
            ui.label("Token");
        });
        ui.horizontal(|ui| {
            let mut color = Color32::from_rgb(255, 255, 0);
            ui.color_edit_button_srgba(&mut color);
            ui.label("Waiting");
        });
        ui.horizontal(|ui| {
            let mut color = Color32::from_rgb(255, 0, 0);
            ui.color_edit_button_srgba(&mut color);
            ui.label("Blocked");
        });

        action
    }
}

#[derive(Debug, Clone)]
pub enum ExecutionVisualizerAction {
    None,
    ToggleHeatmap(bool),
}
