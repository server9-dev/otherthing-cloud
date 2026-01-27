//! UI Module for ABCDODAF
//!
//! Provides visual editors for BPMN processes using egui-snarl.

pub mod bpmn_json_loader;
pub mod bpmn_shapes;
pub mod bpmn_snarl;
pub mod commands;
pub mod diagram_converter;
pub mod dodaf_aggregator;
pub mod enhanced_nodes;
pub mod enhanced_viewer;
pub mod execution_debugger;
pub mod execution_visualizer;
pub mod file_browser;
pub mod instance_manager;
pub mod notifications;
pub mod property_editor;
pub mod shortcuts;
pub mod status_bar;
pub mod tab_bar;
pub mod toolbar;
pub mod validation;
pub mod workspace;

// TODO: Re-enable when egui-snarl exposes required APIs
// pub mod minimap;
// pub mod alignment;
// pub mod find_replace;

pub use bpmn_json_loader::*;
pub use bpmn_snarl::{bpmn_style, BpmnNode, BpmnViewer};
pub use commands::*;
pub use diagram_converter::*;
pub use dodaf_aggregator::*;
pub use enhanced_nodes::*;
pub use enhanced_viewer::*;
pub use execution_debugger::*;
pub use execution_visualizer::*;
pub use file_browser::*;
pub use instance_manager::*;
pub use notifications::*;
pub use property_editor::*;
pub use shortcuts::*;
pub use status_bar::*;
pub use tab_bar::*;
pub use toolbar::*;
pub use validation::*;
pub use workspace::*;

// TODO: Re-enable when egui-snarl exposes required APIs
// pub use minimap::*;
// pub use alignment::*;
// pub use find_replace::*;

pub use egui_snarl::Snarl;
