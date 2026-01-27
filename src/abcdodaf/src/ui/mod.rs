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
pub mod workspace;

#[cfg(feature = "ui")]
pub mod validation;

#[cfg(feature = "ui")]
pub mod file_browser;

#[cfg(feature = "ui")]
pub mod tab_bar;

#[cfg(feature = "ui")]
pub mod dodaf_aggregator;

#[cfg(feature = "ui")]
pub mod execution_debugger;

#[cfg(feature = "ui")]
pub mod execution_visualizer;

#[cfg(feature = "ui")]
pub mod instance_manager;

#[cfg(feature = "ui")]
pub mod commands;

#[cfg(feature = "ui")]
pub mod shortcuts;

#[cfg(feature = "ui")]
pub mod toolbar;

#[cfg(feature = "ui")]
pub mod status_bar;

// TODO: Re-enable when egui-snarl exposes required APIs
// #[cfg(feature = "ui")]
// pub mod minimap;

// #[cfg(feature = "ui")]
// pub mod alignment;

// #[cfg(feature = "ui")]
// pub mod find_replace;

#[cfg(feature = "ui")]
pub use bpmn_snarl::{BpmnNode, BpmnViewer, bpmn_style};

#[cfg(feature = "ui")]
pub use enhanced_nodes::*;

#[cfg(feature = "ui")]
pub use enhanced_viewer::*;

#[cfg(feature = "ui")]
pub use property_editor::*;

#[cfg(feature = "ui")]
pub use workspace::*;

#[cfg(feature = "ui")]
pub use validation::*;

#[cfg(feature = "ui")]
pub use file_browser::*;

#[cfg(feature = "ui")]
pub use tab_bar::*;

#[cfg(feature = "ui")]
pub use dodaf_aggregator::*;

#[cfg(feature = "ui")]
pub use execution_debugger::*;

#[cfg(feature = "ui")]
pub use execution_visualizer::*;

#[cfg(feature = "ui")]
pub use instance_manager::*;

#[cfg(feature = "ui")]
pub use commands::*;

#[cfg(feature = "ui")]
pub use shortcuts::*;

#[cfg(feature = "ui")]
pub use toolbar::*;

#[cfg(feature = "ui")]
pub use status_bar::*;

// TODO: Re-enable when egui-snarl exposes required APIs
// #[cfg(feature = "ui")]
// pub use minimap::*;

// #[cfg(feature = "ui")]
// pub use alignment::*;

// #[cfg(feature = "ui")]
// pub use find_replace::*;

#[cfg(feature = "ui")]
pub use egui_snarl::Snarl;
