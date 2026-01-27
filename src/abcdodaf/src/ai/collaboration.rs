//! Multi-agent collaboration patterns
//!
//! Enables multiple AI agents to work together on complex tasks.

use crate::ai::{AgentConfig, AgentResponse};
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Multi-agent coordinator
pub struct MultiAgentCoordinator {
    /// Registered agents
    agents: HashMap<String, AgentConfig>,
    /// Collaboration patterns
    patterns: HashMap<String, CollaborationPattern>,
    /// Execution history
    history: Vec<CollaborationExecution>,
}

/// Agent collaboration definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCollaboration {
    /// Collaboration ID
    pub id: String,
    /// Participating agent IDs
    pub agent_ids: Vec<String>,
    /// Collaboration pattern
    pub pattern: CollaborationPattern,
    /// Shared context
    pub shared_context: HashMap<String, serde_json::Value>,
}

/// Collaboration pattern
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollaborationPattern {
    /// Agents work in sequence (pipeline)
    Sequential,
    /// Agents work in parallel and results are aggregated
    Parallel,
    /// One agent delegates to others
    Delegation,
    /// Agents debate and reach consensus
    Consensus,
    /// Hierarchical with supervisor
    Hierarchical,
    /// Peer-to-peer collaboration
    PeerToPeer,
}

/// Collaboration execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationExecution {
    /// Execution ID
    pub execution_id: String,
    /// Collaboration ID
    pub collaboration_id: String,
    /// Agent responses
    pub responses: Vec<AgentResponse>,
    /// Final result
    pub final_result: String,
    /// Execution time in ms
    pub duration_ms: u64,
}

impl MultiAgentCoordinator {
    /// Create a new coordinator
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            patterns: HashMap::new(),
            history: vec![],
        }
    }

    /// Register an agent
    pub fn register_agent(&mut self, agent: AgentConfig) {
        self.agents.insert(agent.agent_id.clone(), agent);
    }

    /// Create a collaboration
    pub fn create_collaboration(
        &mut self,
        id: impl Into<String>,
        agent_ids: Vec<String>,
        pattern: CollaborationPattern,
    ) -> Result<AgentCollaboration> {
        let collaboration = AgentCollaboration {
            id: id.into(),
            agent_ids,
            pattern: pattern.clone(),
            shared_context: HashMap::new(),
        };

        self.patterns.insert(collaboration.id.clone(), pattern);
        Ok(collaboration)
    }

    /// Execute sequential collaboration
    pub async fn execute_sequential(
        &self,
        collaboration: &AgentCollaboration,
        initial_input: &str,
    ) -> Result<CollaborationExecution> {
        let start = std::time::Instant::now();
        let mut responses = vec![];
        let mut current_input = initial_input.to_string();

        for agent_id in &collaboration.agent_ids {
            // Simulate agent execution
            let response = AgentResponse {
                request_id: uuid::Uuid::new_v4().to_string(),
                content: format!("Agent {} processed: {}", agent_id, current_input),
                confidence: 0.85,
                tokens_used: crate::ai::TokenUsage {
                    prompt_tokens: 100,
                    completion_tokens: 50,
                    total_tokens: 150,
                },
                duration_ms: 100,
                model_used: "llama3.2:3b".to_string(),
                tool_calls: vec![],
                metadata: HashMap::new(),
            };

            current_input = response.content.clone();
            responses.push(response);
        }

        Ok(CollaborationExecution {
            execution_id: uuid::Uuid::new_v4().to_string(),
            collaboration_id: collaboration.id.clone(),
            responses: responses.clone(),
            final_result: responses.last().map(|r| r.content.clone()).unwrap_or_default(),
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    /// Execute parallel collaboration
    pub async fn execute_parallel(
        &self,
        collaboration: &AgentCollaboration,
        input: &str,
    ) -> Result<CollaborationExecution> {
        let start = std::time::Instant::now();
        let mut responses = vec![];

        // Simulate parallel execution
        for agent_id in &collaboration.agent_ids {
            let response = AgentResponse {
                request_id: uuid::Uuid::new_v4().to_string(),
                content: format!("Agent {} analyzed: {}", agent_id, input),
                confidence: 0.80,
                tokens_used: crate::ai::TokenUsage {
                    prompt_tokens: 100,
                    completion_tokens: 50,
                    total_tokens: 150,
                },
                duration_ms: 100,
                model_used: "llama3.2:3b".to_string(),
                tool_calls: vec![],
                metadata: HashMap::new(),
            };

            responses.push(response);
        }

        // Aggregate results
        let final_result = responses
            .iter()
            .map(|r| r.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        Ok(CollaborationExecution {
            execution_id: uuid::Uuid::new_v4().to_string(),
            collaboration_id: collaboration.id.clone(),
            responses,
            final_result,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    /// Execute collaboration based on pattern
    pub async fn execute(
        &mut self,
        collaboration: &AgentCollaboration,
        input: &str,
    ) -> Result<CollaborationExecution> {
        let result = match collaboration.pattern {
            CollaborationPattern::Sequential => {
                self.execute_sequential(collaboration, input).await?
            }
            CollaborationPattern::Parallel => self.execute_parallel(collaboration, input).await?,
            _ => {
                // Default to sequential for other patterns
                self.execute_sequential(collaboration, input).await?
            }
        };

        self.history.push(result.clone());
        Ok(result)
    }

    /// Get execution history
    pub fn history(&self) -> &[CollaborationExecution] {
        &self.history
    }

    /// Get agent count
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }
}

impl Default for MultiAgentCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinator_creation() {
        let coordinator = MultiAgentCoordinator::new();
        assert_eq!(coordinator.agent_count(), 0);
    }

    #[test]
    fn test_create_collaboration() {
        let mut coordinator = MultiAgentCoordinator::new();
        let collaboration = coordinator
            .create_collaboration(
                "collab1",
                vec!["agent1".to_string(), "agent2".to_string()],
                CollaborationPattern::Sequential,
            )
            .unwrap();

        assert_eq!(collaboration.id, "collab1");
        assert_eq!(collaboration.agent_ids.len(), 2);
    }

    #[tokio::test]
    async fn test_sequential_execution() {
        let coordinator = MultiAgentCoordinator::new();
        let collaboration = AgentCollaboration {
            id: "test".to_string(),
            agent_ids: vec!["a1".to_string(), "a2".to_string()],
            pattern: CollaborationPattern::Sequential,
            shared_context: HashMap::new(),
        };

        let result = coordinator
            .execute_sequential(&collaboration, "test input")
            .await
            .unwrap();

        assert_eq!(result.responses.len(), 2);
    }
}
