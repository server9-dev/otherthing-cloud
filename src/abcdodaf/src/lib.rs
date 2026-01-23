//! # ABCDODAF - BPMN + DoDAF 2.02 Process Modeling for AI Workforce
//!
//! This library combines BPMN process execution with DoDAF 2.02 architectural framework
//! to model both agentic and non-agentic operations within an AI assistant workforce context.
//!
//! ## Core Concepts
//!
//! - **BPMN Integration**: Leverage BPMN 2.0 for process modeling and execution
//! - **DoDAF 2.02**: Department of Defense Architecture Framework for operational modeling
//! - **AI Workforce**: Model agentic (AI-driven) and non-agentic (human/system) operations
//! - **BPM+ Principles**: Extensible framework for business process management best practices
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     ABCDODAF Library                        │
//! ├─────────────────────────────────────────────────────────────┤
//! │                                                             │
//! │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
//! │  │     BPMN     │  │    DoDAF     │  │  Workforce   │     │
//! │  │   Engine     │◄─┤    2.02      │◄─┤  Modeling    │     │
//! │  └──────────────┘  └──────────────┘  └──────────────┘     │
//! │         │                 │                   │            │
//! │         └─────────────────┴───────────────────┘            │
//! │                          │                                 │
//! │                  ┌──────────────┐                          │
//! │                  │   BPM+       │                          │
//! │                  │  Principles  │                          │
//! │                  └──────────────┘                          │
//! │                                                             │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,no_run
//! use abcdodaf::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Define a workflow combining agentic and non-agentic tasks
//!     let workflow = WorkflowBuilder::new("ai_assistant_workflow")
//!         .add_agent_task(
//!             "analyze_request",
//!             AgentTask::new("analyze_request", "Analyze", AgentCapability::NaturalLanguageProcessing),
//!         )
//!         .add_human_task(
//!             "review_output",
//!             HumanTask::new("review_output", "Review", HumanRole::QualityAssurance),
//!         )
//!         .add_system_task(
//!             "store_results",
//!             SystemTask::new("store_results", "Store", SystemOperation::DatabaseWrite),
//!         )
//!         .with_context(
//!             OperationalContext::new()
//!                 .with_mission_area("AI Workforce Operations")
//!                 .with_capability("Intelligent Task Processing")
//!         )
//!         .build()?;
//!
//!     // Execute workflow
//!     workflow.execute().await?;
//!
//!     Ok(())
//! }
//! ```

pub mod bpmn;
pub mod dodaf;
pub mod workforce;
pub mod bpm_plus;
pub mod integration;

pub mod prelude {
    //! Convenience re-exports for common types and traits

    pub use crate::bpmn::{Process, ProcessBuilder, ProcessExecutor};
    pub use crate::bpmn::executor::TaskHandler;
    pub use crate::dodaf::{
        OperationalActivity, OperationalContext, CapabilityView, ServiceView,
    };
    pub use crate::workforce::{
        AgentTask, HumanTask, SystemTask, WorkflowBuilder,
        AgentCapability, HumanRole, SystemOperation, AgentType,
    };
    pub use crate::bpm_plus::BpmPlusModel;
    pub use crate::integration::McpIntegration;
}

pub use prelude::*;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Error types for the library
pub mod error {
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum AbcdodafError {
        #[error("BPMN process error: {0}")]
        BpmnError(String),

        #[error("DoDAF framework error: {0}")]
        DodafError(String),

        #[error("Workflow execution error: {0}")]
        WorkflowError(String),

        #[error("Integration error: {0}")]
        IntegrationError(String),

        #[error("Serialization error: {0}")]
        SerializationError(#[from] serde_json::Error),

        #[error("IO error: {0}")]
        IoError(#[from] std::io::Error),

        #[error(transparent)]
        Other(#[from] anyhow::Error),
    }

    pub type Result<T> = std::result::Result<T, AbcdodafError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
