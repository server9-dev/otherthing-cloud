//! Model selection and fallback strategies
//!
//! Intelligent model selection based on task requirements and fallback handling.

use crate::ai::{EnhancedAgentType, EnhancedCapability, ModelInfo};
use crate::error::{AbcdodafError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Model selector for choosing appropriate models
pub struct ModelSelector {
    /// Available models
    models: Vec<ModelInfo>,
    /// Model capabilities mapping
    capabilities: HashMap<String, Vec<EnhancedCapability>>,
    /// Model performance scores
    performance: HashMap<String, ModelPerformance>,
    /// Selection strategy
    strategy: ModelStrategy,
}

/// Model selection strategy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelStrategy {
    /// Always use fastest model
    Fastest,
    /// Best quality regardless of speed
    BestQuality,
    /// Balance speed and quality
    Balanced,
    /// Cheapest option (for API models)
    Cheapest,
    /// Custom scoring function
    Custom,
}

/// Fallback strategy when primary model fails
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FallbackStrategy {
    /// Try next best model
    NextBest,
    /// Use specific fallback model
    Specific(String),
    /// Try all available models in order
    TryAll,
    /// Fail immediately
    None,
}

/// Model performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformance {
    /// Average tokens per second
    pub tokens_per_second: f32,
    /// Quality score (0.0 - 1.0)
    pub quality_score: f32,
    /// Success rate (0.0 - 1.0)
    pub success_rate: f32,
    /// Average latency in ms
    pub avg_latency_ms: u64,
    /// Cost per 1K tokens (for API models)
    pub cost_per_1k_tokens: Option<f32>,
}

/// Model selection criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionCriteria {
    /// Required capabilities
    pub required_capabilities: Vec<EnhancedCapability>,
    /// Agent type
    pub agent_type: Option<EnhancedAgentType>,
    /// Maximum latency in ms
    pub max_latency_ms: Option<u64>,
    /// Minimum quality score
    pub min_quality_score: Option<f32>,
    /// Maximum cost per 1K tokens
    pub max_cost_per_1k: Option<f32>,
}

impl ModelSelector {
    /// Create a new model selector
    pub fn new(strategy: ModelStrategy) -> Self {
        Self {
            models: vec![],
            capabilities: HashMap::new(),
            performance: HashMap::new(),
            strategy,
        }
    }

    /// Register a model
    pub fn register_model(&mut self, model: ModelInfo, capabilities: Vec<EnhancedCapability>) {
        self.capabilities.insert(model.name.clone(), capabilities);
        self.models.push(model);
    }

    /// Set model performance metrics
    pub fn set_performance(&mut self, model_name: &str, performance: ModelPerformance) {
        self.performance.insert(model_name.to_string(), performance);
    }

    /// Select best model based on criteria
    pub fn select(&self, criteria: &SelectionCriteria) -> Result<String> {
        let candidates = self.filter_by_criteria(criteria);

        if candidates.is_empty() {
            return Err(AbcdodafError::Other(anyhow::anyhow!(
                "No models match the selection criteria"
            )));
        }

        let best = match self.strategy {
            ModelStrategy::Fastest => self.select_fastest(&candidates),
            ModelStrategy::BestQuality => self.select_best_quality(&candidates),
            ModelStrategy::Balanced => self.select_balanced(&candidates),
            ModelStrategy::Cheapest => self.select_cheapest(&candidates),
            ModelStrategy::Custom => self.select_custom(&candidates),
        };

        best.ok_or_else(|| AbcdodafError::Other(anyhow::anyhow!("Model selection failed")))
    }

    /// Filter models by criteria
    fn filter_by_criteria(&self, criteria: &SelectionCriteria) -> Vec<&str> {
        self.models
            .iter()
            .filter(|model| {
                // Check capabilities
                if !criteria.required_capabilities.is_empty() {
                    if let Some(model_caps) = self.capabilities.get(&model.name) {
                        if !criteria
                            .required_capabilities
                            .iter()
                            .all(|cap| model_caps.contains(cap))
                        {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }

                // Check performance constraints
                if let Some(perf) = self.performance.get(&model.name) {
                    if let Some(max_latency) = criteria.max_latency_ms {
                        if perf.avg_latency_ms > max_latency {
                            return false;
                        }
                    }

                    if let Some(min_quality) = criteria.min_quality_score {
                        if perf.quality_score < min_quality {
                            return false;
                        }
                    }

                    if let Some(max_cost) = criteria.max_cost_per_1k {
                        if let Some(cost) = perf.cost_per_1k_tokens {
                            if cost > max_cost {
                                return false;
                            }
                        }
                    }
                }

                true
            })
            .map(|m| m.name.as_str())
            .collect()
    }

    /// Select fastest model
    fn select_fastest(&self, candidates: &[&str]) -> Option<String> {
        candidates
            .iter()
            .filter_map(|name| {
                self.performance
                    .get(*name)
                    .map(|perf| (*name, perf.tokens_per_second))
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(name, _)| name.to_string())
    }

    /// Select best quality model
    fn select_best_quality(&self, candidates: &[&str]) -> Option<String> {
        candidates
            .iter()
            .filter_map(|name| {
                self.performance
                    .get(*name)
                    .map(|perf| (*name, perf.quality_score))
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(name, _)| name.to_string())
    }

    /// Select balanced model
    fn select_balanced(&self, candidates: &[&str]) -> Option<String> {
        candidates
            .iter()
            .filter_map(|name| {
                self.performance.get(*name).map(|perf| {
                    let score = (perf.quality_score + perf.tokens_per_second / 100.0) / 2.0;
                    (*name, score)
                })
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(name, _)| name.to_string())
    }

    /// Select cheapest model
    fn select_cheapest(&self, candidates: &[&str]) -> Option<String> {
        candidates
            .iter()
            .filter_map(|name| {
                self.performance
                    .get(*name)
                    .and_then(|perf| perf.cost_per_1k_tokens.map(|cost| (*name, cost)))
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(name, _)| name.to_string())
            .or_else(|| candidates.first().map(|s| s.to_string()))
    }

    /// Custom selection (placeholder)
    fn select_custom(&self, candidates: &[&str]) -> Option<String> {
        // Default to balanced for custom
        self.select_balanced(candidates)
    }

    /// Get fallback models
    pub fn get_fallbacks(
        &self,
        primary: &str,
        strategy: &FallbackStrategy,
        criteria: &SelectionCriteria,
    ) -> Vec<String> {
        match strategy {
            FallbackStrategy::NextBest => {
                let candidates = self.filter_by_criteria(criteria);
                candidates
                    .into_iter()
                    .filter(|name| *name != primary)
                    .take(1)
                    .map(String::from)
                    .collect()
            }
            FallbackStrategy::Specific(model) => vec![model.clone()],
            FallbackStrategy::TryAll => self
                .models
                .iter()
                .filter(|m| m.name != primary)
                .map(|m| m.name.clone())
                .collect(),
            FallbackStrategy::None => vec![],
        }
    }

    /// Get model capabilities
    pub fn get_capabilities(&self, model_name: &str) -> Option<&Vec<EnhancedCapability>> {
        self.capabilities.get(model_name)
    }

    /// List all models
    pub fn list_models(&self) -> &[ModelInfo] {
        &self.models
    }

    /// Create selector with Ollama defaults
    pub fn with_ollama_defaults() -> Self {
        let mut selector = Self::new(ModelStrategy::Balanced);

        // Register common Ollama models with capabilities
        selector.register_model(
            ModelInfo {
                name: "llama3.2:3b".to_string(),
                size: 2_000_000_000,
                family: Some("llama".to_string()),
                parameter_size: Some("3B".to_string()),
                quantization: Some("Q4_0".to_string()),
            },
            vec![
                EnhancedCapability::NaturalLanguageProcessing,
                EnhancedCapability::ChainOfThought,
                EnhancedCapability::ToolUsage,
            ],
        );

        selector.set_performance(
            "llama3.2:3b",
            ModelPerformance {
                tokens_per_second: 50.0,
                quality_score: 0.75,
                success_rate: 0.95,
                avg_latency_ms: 100,
                cost_per_1k_tokens: None,
            },
        );

        selector.register_model(
            ModelInfo {
                name: "llama3.1:8b".to_string(),
                size: 8_000_000_000,
                family: Some("llama".to_string()),
                parameter_size: Some("8B".to_string()),
                quantization: Some("Q4_0".to_string()),
            },
            vec![
                EnhancedCapability::NaturalLanguageProcessing,
                EnhancedCapability::CodeGeneration,
                EnhancedCapability::ChainOfThought,
                EnhancedCapability::MathematicalReasoning,
            ],
        );

        selector.set_performance(
            "llama3.1:8b",
            ModelPerformance {
                tokens_per_second: 35.0,
                quality_score: 0.85,
                success_rate: 0.97,
                avg_latency_ms: 150,
                cost_per_1k_tokens: None,
            },
        );

        selector
    }
}

impl Default for ModelStrategy {
    fn default() -> Self {
        Self::Balanced
    }
}

impl Default for FallbackStrategy {
    fn default() -> Self {
        Self::NextBest
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_selector_creation() {
        let selector = ModelSelector::new(ModelStrategy::Balanced);
        assert_eq!(selector.strategy, ModelStrategy::Balanced);
    }

    #[test]
    fn test_register_model() {
        let mut selector = ModelSelector::new(ModelStrategy::Balanced);
        selector.register_model(
            ModelInfo {
                name: "test-model".to_string(),
                size: 1000,
                family: None,
                parameter_size: None,
                quantization: None,
            },
            vec![EnhancedCapability::NaturalLanguageProcessing],
        );

        assert_eq!(selector.list_models().len(), 1);
    }

    #[test]
    fn test_ollama_defaults() {
        let selector = ModelSelector::with_ollama_defaults();
        assert!(!selector.list_models().is_empty());
        assert!(selector.get_capabilities("llama3.2:3b").is_some());
    }

    #[test]
    fn test_selection_criteria() {
        let criteria = SelectionCriteria {
            required_capabilities: vec![EnhancedCapability::CodeGeneration],
            agent_type: None,
            max_latency_ms: Some(200),
            min_quality_score: Some(0.8),
            max_cost_per_1k: None,
        };

        let selector = ModelSelector::with_ollama_defaults();
        let result = selector.select(&criteria);
        assert!(result.is_ok());
    }
}
