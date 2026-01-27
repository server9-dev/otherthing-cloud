//! UI Module for ABCDODAF
//!
//! Provides visual editors for BPMN processes using egui-snarl.

#[cfg(feature = "ui")]
pub mod bpmn_snarl;

#[cfg(feature = "ui")]
pub mod enhanced_nodes;

#[cfg(feature = "ui")]
pub mod enhanced_viewer;

#[cfg(feature = "ui")]
pub mod property_editor;

#[cfg(feature = "ui")]
pub mod bpmn_shapes;

#[cfg(feature = "ui")]
pub use bpmn_snarl::{BpmnNode, BpmnViewer, bpmn_style};

#[cfg(feature = "ui")]
pub use enhanced_nodes::*;

#[cfg(feature = "ui")]
pub use enhanced_viewer::*;

#[cfg(feature = "ui")]
pub use property_editor::*;

#[cfg(feature = "ui")]
pub use egui_snarl::Snarl;
