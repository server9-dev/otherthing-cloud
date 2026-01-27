//! DMN 1.3 Decision Model and Notation support
//!
//! This module implements the DMN 1.3 specification including:
//! - Decision tables with hit policies (UNIQUE, FIRST, PRIORITY, ANY, COLLECT)
//! - FEEL expression language parser and evaluator
//! - Decision graphs and requirement diagrams
//! - XML import/export for DMN files
//! - Integration with BPMN business rule tasks
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │          DMN 1.3 Engine                      │
//! ├─────────────────────────────────────────────┤
//! │                                             │
//! │  ┌──────────────┐    ┌──────────────┐     │
//! │  │  FEEL Parser │◄───┤  Expressions │     │
//! │  └──────────────┘    └──────────────┘     │
//! │         │                                  │
//! │         ▼                                  │
//! │  ┌──────────────┐    ┌──────────────┐     │
//! │  │Decision Table│◄───┤  Hit Policies│     │
//! │  └──────────────┘    └──────────────┘     │
//! │         │                                  │
//! │         ▼                                  │
//! │  ┌──────────────┐    ┌──────────────┐     │
//! │  │Decision Graph│◄───┤  Execution   │     │
//! │  └──────────────┘    └──────────────┘     │
//! │                                             │
//! └─────────────────────────────────────────────┘
//! ```

pub mod feel;
pub mod expression;
pub mod decision_table;
pub mod decision_graph;
pub mod executor;
pub mod xml;
pub mod errors;

pub use feel::{FeelExpression, FeelValue, FeelEvaluator};
pub use expression::{Expression, ExpressionEvaluator};
pub use decision_table::{
    DecisionTable, HitPolicy, DecisionTableInput,
    DecisionTableOutput, RuleEntry,
};
pub use decision_graph::{DecisionGraph, DecisionNode, RequirementDiagram};
pub use executor::DecisionExecutor;
pub use errors::{DmnError, DmnResult};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// DMN Decision definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    /// Unique identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// Decision table or expression
    pub definition: DecisionDefinition,
    /// Requirement diagram reference
    pub requirement_diagram: Option<String>,
}

/// Decision definition variants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionDefinition {
    /// Decision table definition
    Table(DecisionTable),
    /// FEEL expression definition
    Expression(Expression),
    /// Invocation of another decision
    Invocation {
        decision_id: String,
        parameters: HashMap<String, String>,
    },
}

/// Decision service for reusable decision logic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionService {
    /// Service identifier
    pub id: String,
    /// Service name
    pub name: String,
    /// Output decisions
    pub output_decisions: Vec<String>,
    /// Encapsulated decisions
    pub encapsulated_decisions: Vec<String>,
    /// Input decisions
    pub input_decisions: Vec<String>,
    /// Input data
    pub input_data: Vec<String>,
}

/// DMN model version
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DmnVersion {
    /// DMN 1.1
    V1_1,
    /// DMN 1.2
    V1_2,
    /// DMN 1.3 (current)
    V1_3,
}

impl Default for DmnVersion {
    fn default() -> Self {
        DmnVersion::V1_3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_version() {
        assert_eq!(DmnVersion::default(), DmnVersion::V1_3);
    }

    #[test]
    fn test_decision_creation() {
        let decision = Decision {
            id: "test_decision".to_string(),
            name: "Test Decision".to_string(),
            description: Some("A test decision".to_string()),
            definition: DecisionDefinition::Expression(Expression::Literal(
                FeelValue::String("test".to_string()),
            )),
            requirement_diagram: None,
        };

        assert_eq!(decision.id, "test_decision");
        assert_eq!(decision.name, "Test Decision");
    }
}
