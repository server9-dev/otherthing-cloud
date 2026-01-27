//! UI Module for ABCDODAF
//!
//! Provides visual editors for BPMN processes using egui-snarl.

pub mod bpmn_snarl;
pub mod enhanced_nodes;
pub mod enhanced_viewer;
pub mod property_editor;
pub mod bpmn_shapes;
pub mod workspace;
pub mod validation;
pub mod file_browser;
pub mod tab_bar;
pub mod dodaf_aggregator;
pub mod execution_debugger;
pub mod execution_visualizer;
pub mod instance_manager;
pub mod commands;
pub mod shortcuts;
pub mod toolbar;
pub mod status_bar;
pub mod notifications;
pub mod bpmn_json_loader;
pub mod diagram_converter;

// TODO: Re-enable when egui-snarl exposes required APIs
// pub mod minimap;
// pub mod alignment;
// pub mod find_replace;

pub use bpmn_snarl::{BpmnNode, BpmnViewer, bpmn_style};
pub use enhanced_nodes::*;
pub use enhanced_viewer::*;
pub use property_editor::*;
pub use workspace::*;
pub use validation::*;
pub use file_browser::*;
pub use tab_bar::*;
pub use dodaf_aggregator::*;
pub use execution_debugger::*;
pub use execution_visualizer::*;
pub use instance_manager::*;
pub use commands::*;
pub use shortcuts::*;
pub use toolbar::*;
pub use status_bar::*;
pub use notifications::*;
pub use bpmn_json_loader::*;
pub use diagram_converter::*;

// TODO: Re-enable when egui-snarl exposes required APIs
// pub use minimap::*;
// pub use alignment::*;
// pub use find_replace::*;

pub use egui_snarl::Snarl;
