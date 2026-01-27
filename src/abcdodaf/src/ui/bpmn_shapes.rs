//! BPMN Shape Rendering
//!
//! Custom shape rendering for BPMN 2.0 symbols using egui painting primitives.

use egui::epaint::{CornerRadius, RectShape, StrokeKind};
use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke};
use std::f32::consts::PI;

/// Draw a BPMN start event (thin circle)
pub fn draw_start_event(painter: &Painter, rect: Rect, color: Color32) {
    let center = rect.center();
    let radius = rect.width().min(rect.height()) / 2.0 - 2.0;
    painter.circle_stroke(center, radius, Stroke::new(2.0, color));
}

/// Draw a BPMN end event (thick circle)
pub fn draw_end_event(painter: &Painter, rect: Rect, color: Color32) {
    let center = rect.center();
    let radius = rect.width().min(rect.height()) / 2.0 - 3.0;
    painter.circle_stroke(center, radius, Stroke::new(4.0, color));
}

/// Draw a BPMN intermediate event (double circle)
pub fn draw_intermediate_event(painter: &Painter, rect: Rect, color: Color32) {
    let center = rect.center();
    let outer_radius = rect.width().min(rect.height()) / 2.0 - 2.0;
    let inner_radius = outer_radius - 3.0;

    painter.circle_stroke(center, outer_radius, Stroke::new(2.0, color));
    painter.circle_stroke(center, inner_radius, Stroke::new(2.0, color));
}

/// Draw a BPMN task (rounded rectangle)
pub fn draw_task(painter: &Painter, rect: Rect, color: Color32, fill: Color32) {
    painter.add(Shape::Rect(RectShape {
        rect,
        corner_radius: CornerRadius::same(5),
        fill,
        stroke: Stroke::new(2.0, color),
        stroke_kind: StrokeKind::Middle,
        blur_width: 0.0,
        brush: Default::default(),
        round_to_pixels: Some(true),
    }));
}

/// Draw a BPMN gateway (diamond)
pub fn draw_gateway(painter: &Painter, rect: Rect, color: Color32, fill: Color32) {
    let center = rect.center();
    let half_width = rect.width() / 2.0 - 2.0;
    let half_height = rect.height() / 2.0 - 2.0;

    let points = vec![
        Pos2::new(center.x, center.y - half_height), // top
        Pos2::new(center.x + half_width, center.y),  // right
        Pos2::new(center.x, center.y + half_height), // bottom
        Pos2::new(center.x - half_width, center.y),  // left
    ];

    painter.add(Shape::convex_polygon(points.clone(), fill, Stroke::new(2.0, color)));
}

/// Draw exclusive gateway symbol (X inside diamond)
pub fn draw_exclusive_gateway_symbol(painter: &Painter, rect: Rect, color: Color32) {
    let center = rect.center();
    let size = rect.width().min(rect.height()) / 3.0;

    // Draw X
    painter.line_segment(
        [
            Pos2::new(center.x - size / 2.0, center.y - size / 2.0),
            Pos2::new(center.x + size / 2.0, center.y + size / 2.0),
        ],
        Stroke::new(2.5, color),
    );
    painter.line_segment(
        [
            Pos2::new(center.x + size / 2.0, center.y - size / 2.0),
            Pos2::new(center.x - size / 2.0, center.y + size / 2.0),
        ],
        Stroke::new(2.5, color),
    );
}

/// Draw parallel gateway symbol (+ inside diamond)
pub fn draw_parallel_gateway_symbol(painter: &Painter, rect: Rect, color: Color32) {
    let center = rect.center();
    let size = rect.width().min(rect.height()) / 2.5;

    // Draw +
    painter.line_segment(
        [
            Pos2::new(center.x, center.y - size / 2.0),
            Pos2::new(center.x, center.y + size / 2.0),
        ],
        Stroke::new(2.5, color),
    );
    painter.line_segment(
        [
            Pos2::new(center.x - size / 2.0, center.y),
            Pos2::new(center.x + size / 2.0, center.y),
        ],
        Stroke::new(2.5, color),
    );
}

/// Draw inclusive gateway symbol (O inside diamond)
pub fn draw_inclusive_gateway_symbol(painter: &Painter, rect: Rect, color: Color32) {
    let center = rect.center();
    let radius = rect.width().min(rect.height()) / 4.0;

    painter.circle_stroke(center, radius, Stroke::new(2.5, color));
}

/// Draw event-based gateway symbol (pentagon/circle inside diamond)
pub fn draw_event_based_gateway_symbol(painter: &Painter, rect: Rect, color: Color32) {
    let center = rect.center();
    let radius = rect.width().min(rect.height()) / 4.5;

    // Outer circle
    painter.circle_stroke(center, radius, Stroke::new(2.0, color));

    // Inner pentagon
    let inner_radius = radius * 0.6;
    let mut points = Vec::new();
    for i in 0..5 {
        let angle = -PI / 2.0 + (i as f32) * 2.0 * PI / 5.0;
        points.push(Pos2::new(
            center.x + inner_radius * angle.cos(),
            center.y + inner_radius * angle.sin(),
        ));
    }
    painter.add(Shape::convex_polygon(points, Color32::TRANSPARENT, Stroke::new(2.0, color)));
}

/// Draw complex gateway symbol (asterisk inside diamond)
pub fn draw_complex_gateway_symbol(painter: &Painter, rect: Rect, color: Color32) {
    let center = rect.center();
    let size = rect.width().min(rect.height()) / 3.0;

    // Draw asterisk (3 lines at 60 degree angles)
    for i in 0..3 {
        let angle = (i as f32) * PI / 3.0;
        let dx = (size / 2.0) * angle.cos();
        let dy = (size / 2.0) * angle.sin();

        painter.line_segment(
            [Pos2::new(center.x - dx, center.y - dy), Pos2::new(center.x + dx, center.y + dy)],
            Stroke::new(2.0, color),
        );
    }
}

/// Draw data object (rectangle with folded corner)
pub fn draw_data_object(painter: &Painter, rect: Rect, color: Color32, fill: Color32) {
    let fold_size = 8.0;

    // Draw main body
    let points = vec![
        rect.min,
        Pos2::new(rect.max.x - fold_size, rect.min.y),
        Pos2::new(rect.max.x, rect.min.y + fold_size),
        Pos2::new(rect.max.x, rect.max.y),
        Pos2::new(rect.min.x, rect.max.y),
    ];

    painter.add(Shape::convex_polygon(points, fill, Stroke::new(2.0, color)));

    // Draw fold
    let fold_points = vec![
        Pos2::new(rect.max.x - fold_size, rect.min.y),
        Pos2::new(rect.max.x - fold_size, rect.min.y + fold_size),
        Pos2::new(rect.max.x, rect.min.y + fold_size),
    ];
    painter.add(Shape::convex_polygon(
        fold_points,
        Color32::from_white_alpha(50),
        Stroke::new(1.5, color),
    ));
}

/// Draw data store (cylinder)
pub fn draw_data_store(painter: &Painter, rect: Rect, color: Color32, fill: Color32) {
    let ellipse_height = rect.height() * 0.15;

    // Top ellipse
    let top_center = Pos2::new(rect.center().x, rect.min.y + ellipse_height / 2.0);
    draw_ellipse(painter, top_center, rect.width() / 2.0, ellipse_height / 2.0, color, fill);

    // Side rectangles
    let side_rect = Rect::from_min_max(
        Pos2::new(rect.min.x, rect.min.y + ellipse_height / 2.0),
        Pos2::new(rect.max.x, rect.max.y - ellipse_height / 2.0),
    );
    painter.rect_filled(side_rect, 0.0, fill);
    painter.line_segment([side_rect.left_top(), side_rect.left_bottom()], Stroke::new(2.0, color));
    painter
        .line_segment([side_rect.right_top(), side_rect.right_bottom()], Stroke::new(2.0, color));

    // Bottom ellipse
    let bottom_center = Pos2::new(rect.center().x, rect.max.y - ellipse_height / 2.0);
    draw_ellipse(painter, bottom_center, rect.width() / 2.0, ellipse_height / 2.0, color, fill);
}

/// Helper to draw an ellipse
fn draw_ellipse(
    painter: &Painter,
    center: Pos2,
    rx: f32,
    ry: f32,
    stroke_color: Color32,
    fill: Color32,
) {
    let num_segments = 32;
    let mut points = Vec::new();

    for i in 0..num_segments {
        let angle = (i as f32) * 2.0 * PI / (num_segments as f32);
        points.push(Pos2::new(center.x + rx * angle.cos(), center.y + ry * angle.sin()));
    }

    painter.add(Shape::convex_polygon(points, fill, Stroke::new(2.0, stroke_color)));
}

/// Draw subprocess indicator (+ symbol at bottom center)
pub fn draw_subprocess_indicator(painter: &Painter, rect: Rect, color: Color32) {
    let center = Pos2::new(rect.center().x, rect.max.y - 8.0);
    let size = 6.0;

    painter.line_segment(
        [
            Pos2::new(center.x, center.y - size / 2.0),
            Pos2::new(center.x, center.y + size / 2.0),
        ],
        Stroke::new(2.0, color),
    );
    painter.line_segment(
        [
            Pos2::new(center.x - size / 2.0, center.y),
            Pos2::new(center.x + size / 2.0, center.y),
        ],
        Stroke::new(2.0, color),
    );
}
