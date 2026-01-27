//! Agentic operations modeling
//!
//! Models AI agent tasks and capabilities within the workforce

use crate::dodaf::{ActivityType, OperationalActivity};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// AI agent task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    /// Task ID
    pub id: String,
    /// Task name
    pub name: String,
    /// Task description
    pub description: Option<String>,
    /// Agent type required
    pub agent_type: AgentType,
    /// Required capabilities
    pub capabilities: Vec<AgentCapability>,
    /// Task parameters/configuration
    pub parameters: HashMap<String, serde_json::Value>,
    /// Autonomy level (0.0 = fully supervised, 1.0 = fully autonomous)
    pub autonomy_level: f64,
    /// Expected duration in seconds
    pub expected_duration_secs: Option<u64>,
}

/// Type of AI agent
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentType {
    /// Language model agent (GPT, Claude, etc.)
    LanguageModel,
    /// Vision model agent
    VisionModel,
    /// Code generation/analysis agent
    CodeAgent,
    /// Data analysis agent
    DataAnalyst,
    /// Task planning agent
    Planner,
    /// Reasoning agent
    Reasoner,
    /// Tool-using agent
    ToolUser,
    /// Multi-modal agent
    MultiModal,
    /// Custom agent type
    Custom(String),
}

/// AI agent capability
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentCapability {
    /// Natural language understanding/generation
    NaturalLanguageProcessing,
    /// Image understanding/generation
    ComputerVision,
    /// Code generation/analysis
    CodeGeneration,
    /// Mathematical reasoning
    MathematicalReasoning,
    /// Logical reasoning
    LogicalReasoning,
    /// Information retrieval
    InformationRetrieval,
    /// Tool/API usage
    ToolUsage,
    /// Knowledge synthesis
    KnowledgeSynthesis,
    /// Creative generation
    CreativeGeneration,
    /// Decision making
    DecisionMaking,
    /// Pattern recognition
    PatternRecognition,
    /// Custom capability
    Custom(String),
}

impl AgentTask {
    /// Create a new agent task
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        capability: AgentCapability,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            agent_type: AgentType::LanguageModel,
            capabilities: vec![capability],
            parameters: HashMap::new(),
            autonomy_level: 0.8, // Default to mostly autonomous
            expected_duration_secs: None,
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set agent type
    pub fn with_agent_type(mut self, agent_type: AgentType) -> Self {
        self.agent_type = agent_type;
        self
    }

    /// Add capability
    pub fn add_capability(mut self, capability: AgentCapability) -> Self {
        self.capabilities.push(capability);
        self
    }

    /// Set autonomy level
    pub fn with_autonomy(mut self, level: f64) -> Self {
        self.autonomy_level = level.clamp(0.0, 1.0);
        self
    }

    /// Set expected duration
    pub fn with_duration(mut self, secs: u64) -> Self {
        self.expected_duration_secs = Some(secs);
        self
    }

    /// Add parameter
    pub fn add_parameter(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.parameters.insert(key.into(), value);
        self
    }

    /// Convert to DoDAF operational activity
    pub fn to_operational_activity(&self) -> OperationalActivity {
        OperationalActivity::new(&self.id, &self.name)
            .with_type(ActivityType::Automated)
            .with_description(self.description.clone().unwrap_or_default())
            .add_performer(format!("{:?} Agent", self.agent_type))
    }

    /// Check if task requires specific capability
    pub fn has_capability(&self, capability: &AgentCapability) -> bool {
        self.capabilities.contains(capability)
    }

    /// Check if task is highly autonomous (>= 0.7)
    pub fn is_autonomous(&self) -> bool {
        self.autonomy_level >= 0.7
    }
}

impl std::fmt::Display for AgentCapability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentCapability::NaturalLanguageProcessing => write!(f, "NLP"),
            AgentCapability::ComputerVision => write!(f, "Vision"),
            AgentCapability::CodeGeneration => write!(f, "Code"),
            AgentCapability::MathematicalReasoning => write!(f, "Math"),
            AgentCapability::LogicalReasoning => write!(f, "Logic"),
            AgentCapability::InformationRetrieval => write!(f, "Retrieval"),
            AgentCapability::ToolUsage => write!(f, "Tools"),
            AgentCapability::KnowledgeSynthesis => write!(f, "Synthesis"),
            AgentCapability::CreativeGeneration => write!(f, "Creative"),
            AgentCapability::DecisionMaking => write!(f, "Decision"),
            AgentCapability::PatternRecognition => write!(f, "Pattern"),
            AgentCapability::Custom(s) => write!(f, "{}", s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_task_creation() {
        let task =
            AgentTask::new("agent1", "Process Text", AgentCapability::NaturalLanguageProcessing)
                .with_agent_type(AgentType::LanguageModel)
                .with_autonomy(0.9)
                .add_parameter("temperature", serde_json::json!(0.7));

        assert_eq!(task.id, "agent1");
        assert_eq!(task.agent_type, AgentType::LanguageModel);
        assert_eq!(task.autonomy_level, 0.9);
        assert!(task.is_autonomous());
    }

    #[test]
    fn test_capability_check() {
        let task = AgentTask::new("t1", "Test", AgentCapability::CodeGeneration)
            .add_capability(AgentCapability::LogicalReasoning);

        assert!(task.has_capability(&AgentCapability::CodeGeneration));
        assert!(task.has_capability(&AgentCapability::LogicalReasoning));
        assert!(!task.has_capability(&AgentCapability::ComputerVision));
    }
}
