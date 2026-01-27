//! # ABCDODAF - BPMN + DoDAF 2.02 + DMN Process Modeling for AI Workforce
//!
//! This library combines BPMN process execution, DoDAF 2.02 architectural framework,
//! and DMN 1.3 decision modeling to handle both agentic and non-agentic operations
//! within an AI assistant workforce context.
//!
//! ## Core Concepts
//!
//! - **BPMN Integration**: Leverage BPMN 2.0 for process modeling and execution
//! - **DoDAF 2.02**: Department of Defense Architecture Framework for operational modeling
//! - **DMN 1.3**: Decision Model and Notation for business rule execution
//! - **AI Workforce**: Model agentic (AI-driven) and non-agentic (human/system) operations
//! - **BPM+ Principles**: Extensible framework for business process management best practices
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────┐
//! │                    ABCDODAF Library                          │
//! ├──────────────────────────────────────────────────────────────┤
//! │                                                              │
//! │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
//! │  │     BPMN     │  │    DoDAF     │  │     DMN      │      │
//! │  │   Engine     │◄─┤    2.02      │◄─┤  1.3 Rules   │      │
//! │  └──────────────┘  └──────────────┘  └──────────────┘      │
//! │         │                 │                   │             │
//! │         └─────────────────┴───────────────────┘             │
//! │                          │                                  │
//! │              ┌──────────────────────┐                       │
//! │              │   Workforce Modeling │                       │
//! │              │      + BPM+ Princ.   │                       │
//! │              └──────────────────────┘                       │
//! │                                                              │
//! └──────────────────────────────────────────────────────────────┘
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
pub mod dmn;
pub mod dev_logger;
pub mod analytics;
pub mod documentation;
pub mod testing;
pub mod security;
pub mod ai;
pub mod validation;

#[cfg(feature = "ui")]
pub mod ui;

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
    pub use crate::dmn::{
        Decision, DecisionTable, DecisionGraph, DecisionExecutor,
        FeelValue, FeelExpression, FeelEvaluator, Expression,
        HitPolicy, DmnError, DmnResult,
    };
    pub use crate::analytics::{
        AnalyticsDashboard, MetricsCollector, AnalyticsEngine,
        AlertingSystem, ExportFormat, ExportManager,
        Alert, AlertLevel, SlaPolicy, SlaViolation,
        ProcessMetrics, PerformanceMetrics, CostMetrics,
        VisualizationData, ChartData, CustomMetricDefinition,
    };
    pub use crate::security::{
        SecurityContext, RoleManager, Role, Subject, SubjectType,
        PermissionModel, Permission, ResourceType,
        AuditLogger, AuditEvent, AuditLevel,
        ComplianceValidator, ComplianceRule, ComplianceReport,
        SecretManager, SecretValue,
        SessionManager, Session,
        SecurityPolicyEngine,
        AuditExporter,
    };
    pub use crate::security::audit::EventCategory;
    pub use crate::security::session::SessionState;
    pub use crate::security::security_policy::{SecurityPolicy, PolicyType};
    pub use crate::security::audit_export::ExportFormat as AuditExportFormat;

    // AI/LLM exports
    pub use crate::ai::{
        OllamaClient, OllamaConfig, ModelInfo,
        PromptTemplate, PromptManager, PromptVariable,
        ContextWindow, ContextManager, ContextStrategy,
        AgentMemory, MemoryStore, ConversationHistory,
        StreamingResponse, StreamHandler,
        ModelSelector, ModelStrategy, FallbackStrategy,
        MultiAgentCoordinator, AgentCollaboration, CollaborationPattern,
        ToolRegistry, ToolDefinition, ToolCall, ToolExecutor,
        AgentMetrics, PerformanceTracker, CostTracker,
        ResponseCache, CacheStrategy, CacheKey,
        EnhancedAgentType, EnhancedCapability,
        AgentConfig, AgentRequest, AgentResponse,
    };
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

        #[error("YAML serialization error: {0}")]
        YamlSerializationError(String),

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
