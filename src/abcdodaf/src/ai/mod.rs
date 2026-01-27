//! AI/LLM Integration Module
//!
//! Provides comprehensive AI agent capabilities with Ollama integration,
//! prompt management, context handling, and advanced agent features.

pub mod cache;
pub mod collaboration;
pub mod context;
pub mod memory;
pub mod metrics;
pub mod ollama;
pub mod prompt;
pub mod selection;
pub mod streaming;
pub mod tools;

pub use cache::{CacheKey, CacheStrategy, ResponseCache};
pub use collaboration::{AgentCollaboration, CollaborationPattern, MultiAgentCoordinator};
pub use context::{ContextManager, ContextStrategy, ContextWindow};
pub use memory::{AgentMemory, ConversationHistory, MemoryStore};
pub use metrics::{AgentMetrics, CostTracker, PerformanceTracker};
pub use ollama::{ModelInfo, OllamaClient, OllamaConfig};
pub use prompt::{PromptManager, PromptTemplate, PromptVariable};
pub use selection::{FallbackStrategy, ModelSelector, ModelStrategy};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
pub use streaming::{StreamHandler, StreamingResponse};
pub use tools::{ToolCall, ToolDefinition, ToolExecutor, ToolRegistry};

/// Enhanced agent type with AI/LLM capabilities
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnhancedAgentType {
    // Original types
    LanguageModel,
    VisionModel,
    CodeAgent,
    DataAnalyst,
    Planner,
    Reasoner,
    ToolUser,
    MultiModal,

    // New enhanced types
    /// Research and information gathering agent
    Researcher,
    /// Content creation and writing agent
    ContentWriter,
    /// Code review and analysis specialist
    CodeReviewer,
    /// System architecture and design agent
    Architect,
    /// Testing and QA specialist
    QualityAssurance,
    /// DevOps and deployment specialist
    DevOps,
    /// Security analysis specialist
    SecurityAnalyst,
    /// Data science and ML specialist
    DataScientist,
    /// Project management agent
    ProjectManager,
    /// Customer support agent
    CustomerSupport,
    /// Translation specialist
    Translator,
    /// Summarization specialist
    Summarizer,
    /// Question answering specialist
    QuestionAnswering,
    /// Sentiment analysis specialist
    SentimentAnalyzer,
    /// Named entity recognition specialist
    EntityRecognizer,
    /// Text classification specialist
    TextClassifier,
    /// Conversational agent
    Conversational,
    /// Creative writing specialist
    CreativeWriter,
    /// Technical documentation writer
    TechnicalWriter,
    /// Custom agent type
    Custom(String),
}

/// Enhanced agent capability
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum EnhancedCapability {
    // Original capabilities
    NaturalLanguageProcessing,
    ComputerVision,
    CodeGeneration,
    MathematicalReasoning,
    LogicalReasoning,
    InformationRetrieval,
    ToolUsage,
    KnowledgeSynthesis,
    CreativeGeneration,
    DecisionMaking,
    PatternRecognition,

    // New enhanced capabilities
    /// Long-form reasoning and chain-of-thought
    ChainOfThought,
    /// Self-reflection and improvement
    SelfReflection,
    /// Planning and goal decomposition
    GoalPlanning,
    /// Memory and context management
    ContextManagement,
    /// Multi-step task execution
    TaskChaining,
    /// Collaboration with other agents
    AgentCollaboration,
    /// Function calling and tool use
    FunctionCalling,
    /// Streaming response generation
    StreamingGeneration,
    /// Multi-language support
    MultiLanguage,
    /// Semantic search
    SemanticSearch,
    /// Text embedding generation
    EmbeddingGeneration,
    /// Document understanding
    DocumentUnderstanding,
    /// Data extraction and transformation
    DataExtraction,
    /// Workflow orchestration
    WorkflowOrchestration,
    /// Error handling and recovery
    ErrorRecovery,
    /// Performance optimization
    PerformanceOptimization,
    /// Custom capability
    Custom(String),
}

/// AI agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent identifier
    pub agent_id: String,
    /// Agent type
    pub agent_type: EnhancedAgentType,
    /// Agent capabilities
    pub capabilities: Vec<EnhancedCapability>,
    /// Model configuration
    pub model_config: ModelConfig,
    /// Prompt configuration
    pub prompt_config: PromptConfig,
    /// Context configuration
    pub context_config: ContextConfig,
    /// Memory configuration
    pub memory_config: MemoryConfig,
    /// Performance configuration
    pub performance_config: PerformanceConfig,
}

/// Model configuration for AI agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Primary model name
    pub primary_model: String,
    /// Fallback models
    pub fallback_models: Vec<String>,
    /// Temperature (0.0 - 2.0)
    pub temperature: f32,
    /// Top-p sampling
    pub top_p: Option<f32>,
    /// Top-k sampling
    pub top_k: Option<i32>,
    /// Maximum tokens to generate
    pub max_tokens: Option<i32>,
    /// Stop sequences
    pub stop_sequences: Vec<String>,
    /// Enable streaming
    pub streaming: bool,
}

/// Prompt configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptConfig {
    /// System prompt template
    pub system_prompt: Option<String>,
    /// User prompt template
    pub user_prompt_template: String,
    /// Few-shot examples
    pub examples: Vec<PromptExample>,
    /// Prompt variables
    pub variables: HashMap<String, String>,
}

/// Context configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    /// Maximum context window size (tokens)
    pub max_context_tokens: usize,
    /// Context retention strategy
    pub retention_strategy: String,
    /// Enable context compression
    pub compression_enabled: bool,
}

/// Memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Enable short-term memory
    pub short_term_enabled: bool,
    /// Enable long-term memory
    pub long_term_enabled: bool,
    /// Memory persistence strategy
    pub persistence_strategy: String,
    /// Maximum memory entries
    pub max_entries: usize,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable response caching
    pub cache_enabled: bool,
    /// Cache TTL in seconds
    pub cache_ttl_secs: u64,
    /// Enable metrics tracking
    pub metrics_enabled: bool,
    /// Enable cost tracking
    pub cost_tracking_enabled: bool,
    /// Timeout in seconds
    pub timeout_secs: u64,
}

/// Prompt example for few-shot learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptExample {
    /// Input text
    pub input: String,
    /// Expected output
    pub output: String,
}

/// AI agent execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRequest {
    /// Request ID
    pub request_id: String,
    /// Agent ID to use
    pub agent_id: String,
    /// Input data
    pub input: AgentInput,
    /// Request metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Agent input data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInput {
    /// Primary text input
    pub text: String,
    /// Additional context
    pub context: Option<String>,
    /// Conversation history
    pub history: Vec<ConversationMessage>,
    /// Attachments (images, documents, etc.)
    pub attachments: Vec<Attachment>,
}

/// Conversation message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    /// Role (user, assistant, system)
    pub role: String,
    /// Message content
    pub content: String,
    /// Message timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Attachment data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    /// Attachment type
    pub attachment_type: String,
    /// Data or URL
    pub data: String,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// AI agent response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    /// Request ID
    pub request_id: String,
    /// Response content
    pub content: String,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Tokens used
    pub tokens_used: TokenUsage,
    /// Execution time in milliseconds
    pub duration_ms: u64,
    /// Model used
    pub model_used: String,
    /// Tool calls made
    pub tool_calls: Vec<ToolCall>,
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Prompt tokens
    pub prompt_tokens: usize,
    /// Completion tokens
    pub completion_tokens: usize,
    /// Total tokens
    pub total_tokens: usize,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            primary_model: "llama3.2:3b".to_string(),
            fallback_models: vec!["llama3.1:8b".to_string()],
            temperature: 0.7,
            top_p: Some(0.9),
            top_k: Some(40),
            max_tokens: Some(2048),
            stop_sequences: vec![],
            streaming: false,
        }
    }
}

impl Default for PromptConfig {
    fn default() -> Self {
        Self {
            system_prompt: Some("You are a helpful AI assistant.".to_string()),
            user_prompt_template: "{input}".to_string(),
            examples: vec![],
            variables: HashMap::new(),
        }
    }
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_context_tokens: 4096,
            retention_strategy: "sliding_window".to_string(),
            compression_enabled: false,
        }
    }
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            short_term_enabled: true,
            long_term_enabled: false,
            persistence_strategy: "in_memory".to_string(),
            max_entries: 100,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            cache_enabled: true,
            cache_ttl_secs: 3600,
            metrics_enabled: true,
            cost_tracking_enabled: true,
            timeout_secs: 300,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_config_default() {
        let config = ModelConfig::default();
        assert_eq!(config.primary_model, "llama3.2:3b");
        assert_eq!(config.temperature, 0.7);
    }

    #[test]
    fn test_enhanced_agent_types() {
        let agent_type = EnhancedAgentType::Researcher;
        assert_eq!(serde_json::to_string(&agent_type).unwrap(), "\"Researcher\"");
    }
}
