//! Simple notification/toast system for UI feedback

use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub notification_type: NotificationType,
    pub created_at: Instant,
    pub duration: Duration,
}

impl Notification {
    pub fn new(message: impl Into<String>, notification_type: NotificationType) -> Self {
        Self {
            message: message.into(),
            notification_type,
            created_at: Instant::now(),
            duration: Duration::from_secs(5),
        }
    }

    pub fn info(message: impl Into<String>) -> Self {
        Self::new(message, NotificationType::Info)
    }

    pub fn success(message: impl Into<String>) -> Self {
        Self::new(message, NotificationType::Success)
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(message, NotificationType::Warning)
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::new(message, NotificationType::Error)
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.duration
    }

    pub fn remaining_progress(&self) -> f32 {
        let elapsed = self.created_at.elapsed().as_secs_f32();
        let total = self.duration.as_secs_f32();
        1.0 - (elapsed / total).min(1.0)
    }
}

/// Manages notifications/toasts for the UI
#[derive(Default)]
pub struct NotificationManager {
    notifications: Vec<Notification>,
}

impl NotificationManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, notification: Notification) {
        self.notifications.push(notification);
    }

    pub fn info(&mut self, message: impl Into<String>) {
        self.add(Notification::info(message));
    }

    pub fn success(&mut self, message: impl Into<String>) {
        self.add(Notification::success(message));
    }

    pub fn warning(&mut self, message: impl Into<String>) {
        self.add(Notification::warning(message));
    }

    pub fn error(&mut self, message: impl Into<String>) {
        self.add(Notification::error(message));
    }

    /// Remove expired notifications
    pub fn cleanup(&mut self) {
        self.notifications.retain(|n| !n.is_expired());
    }

    /// Render all active notifications
    pub fn render(&mut self, ctx: &egui::Context) {
        self.cleanup();

        if self.notifications.is_empty() {
            return;
        }

        let screen_rect = ctx.content_rect();
        let toast_width = 400.0;
        let toast_height = 60.0;
        let margin = 10.0;
        let spacing = 10.0;

        for (i, notification) in self.notifications.iter().enumerate() {
            let y_offset = margin + (toast_height + spacing) * i as f32;

            let window_rect = egui::Rect::from_min_size(
                egui::pos2(
                    screen_rect.right() - toast_width - margin,
                    screen_rect.top() + y_offset,
                ),
                egui::vec2(toast_width, toast_height),
            );

            let (icon, color) = match notification.notification_type {
                NotificationType::Info => ("ℹ", egui::Color32::from_rgb(100, 149, 237)),
                NotificationType::Success => ("✓", egui::Color32::from_rgb(50, 205, 50)),
                NotificationType::Warning => ("⚠", egui::Color32::from_rgb(255, 165, 0)),
                NotificationType::Error => ("✗", egui::Color32::from_rgb(220, 20, 60)),
            };

            egui::Window::new(format!("notification_{}", i))
                .fixed_rect(window_rect)
                .title_bar(false)
                .resizable(false)
                .collapsible(false)
                .frame(egui::Frame::window(&ctx.style()).fill(color.linear_multiply(0.15)))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(icon).size(24.0).color(color));
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new(&notification.message)
                                    .color(egui::Color32::WHITE)
                            );

                            // Progress bar showing time remaining
                            let progress = notification.remaining_progress();
                            ui.add(
                                egui::ProgressBar::new(progress)
                                    .fill(color)
                                    .show_percentage()
                            );
                        });
                    });
                });
        }
    }
}
