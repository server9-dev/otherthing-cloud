//! Simplified Command Pattern for Undo/Redo
//!
//! Uses state snapshots instead of granular operations to work around
//! egui-snarl API limitations.

use crate::ui::enhanced_nodes::EnhancedBpmnNode;
use egui_snarl::Snarl;
use serde::{Deserialize, Serialize};

/// Maximum number of undo history entries
const MAX_UNDO_HISTORY: usize = 100;

/// Snapshot of Snarl state for undo/redo
#[derive(Clone, Serialize, Deserialize)]
pub struct SnarlSnapshot {
    snarl: Snarl<EnhancedBpmnNode>,
    description: String,
}

/// Command history using state snapshots
#[derive(Default)]
pub struct CommandHistory {
    /// Stack of previous states (for undo)
    undo_stack: Vec<SnarlSnapshot>,

    /// Stack of undone states (for redo)
    redo_stack: Vec<SnarlSnapshot>,

    /// Current state description
    current_description: String,
}

impl CommandHistory {
    pub fn new() -> Self {
        Self::default()
    }

    /// Save current state before making changes
    pub fn save_state(&mut self, snarl: &Snarl<EnhancedBpmnNode>, description: impl Into<String>) {
        // Clear redo stack when new action is performed
        self.redo_stack.clear();

        // Save current state
        let snapshot =
            SnarlSnapshot { snarl: snarl.clone(), description: self.current_description.clone() };

        self.undo_stack.push(snapshot);

        // Update description
        self.current_description = description.into();

        // Limit history size
        if self.undo_stack.len() > MAX_UNDO_HISTORY {
            self.undo_stack.remove(0);
        }
    }

    /// Undo to previous state
    pub fn undo(&mut self, snarl: &mut Snarl<EnhancedBpmnNode>) -> bool {
        if let Some(snapshot) = self.undo_stack.pop() {
            // Save current state to redo stack
            let current = SnarlSnapshot {
                snarl: snarl.clone(),
                description: self.current_description.clone(),
            };
            self.redo_stack.push(current);

            // Restore previous state
            *snarl = snapshot.snarl;
            self.current_description = snapshot.description;

            true
        } else {
            false
        }
    }

    /// Redo previously undone action
    pub fn redo(&mut self, snarl: &mut Snarl<EnhancedBpmnNode>) -> bool {
        if let Some(snapshot) = self.redo_stack.pop() {
            // Save current state to undo stack
            let current = SnarlSnapshot {
                snarl: snarl.clone(),
                description: self.current_description.clone(),
            };
            self.undo_stack.push(current);

            // Restore future state
            *snarl = snapshot.snarl;
            self.current_description = snapshot.description;

            true
        } else {
            false
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get description of next undo action
    pub fn undo_description(&self) -> Option<&str> {
        self.undo_stack.last().map(|s| s.description.as_str())
    }

    /// Get description of next redo action
    pub fn redo_description(&self) -> Option<&str> {
        self.redo_stack.last().map(|s| s.description.as_str())
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.current_description.clear();
    }

    /// Get undo stack size
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get redo stack size
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }
}

// ============================================================================
// Clipboard Support
// ============================================================================

/// Clipboard data for copy/paste operations
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ClipboardData {
    /// Serialized snarl state containing only selected nodes
    pub data: Option<Vec<u8>>,
}

impl ClipboardData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_none()
    }

    pub fn clear(&mut self) {
        self.data = None;
    }

    /// Set clipboard from serialized data
    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = Some(data);
    }

    /// Get clipboard data
    pub fn get_data(&self) -> Option<&[u8]> {
        self.data.as_deref()
    }
}
