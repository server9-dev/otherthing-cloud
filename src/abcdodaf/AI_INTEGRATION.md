# AI/LLM Integration for ABCDODAF

## Overview

The ABCDODAF library now includes comprehensive AI/LLM integration with Ollama support, enabling advanced agentic capabilities for workflow orchestration. This integration provides:

- **Local LLM Inference**: Native Ollama client for running models locally
- **Prompt Engineering**: Template management with variable substitution
- **Context Management**: Intelligent context window handling with multiple strategies
- **Agent Memory**: Short-term and long-term memory with conversation history
- **Streaming Responses**: Real-time token streaming with progress tracking
- **Model Selection**: Automatic model selection with fallback strategies
- **Multi-Agent Collaboration**: Coordinated execution patterns (sequential, parallel, consensus, etc.)
- **Tool Use**: Function calling and external tool integration
- **Performance Metrics**: Comprehensive tracking of latency, tokens, and costs
- **Response Caching**: LRU/LFU/TTL-based caching for efficiency

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                  AI Integration Layer                    │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌──────────────┐   ┌──────────────┐   ┌────────────┐ │
│  │    Ollama    │   │   Prompt     │   │  Context   │ │
│  │    Client    │◄──┤  Templates   │◄──┤  Manager   │ │
│  └──────────────┘   └──────────────┘   └────────────┘ │
│         │                   │                  │        │
│         └───────────────────┴──────────────────┘        │
│                          │                               │
│         ┌────────────────┴────────────────┐            │
│         │                                  │            │
│  ┌──────▼─────┐   ┌──────────┐   ┌───────▼──────┐    │
│  │   Memory   │   │  Tools   │   │   Metrics    │    │
│  │   System   │   │ Registry │   │   & Cache    │    │
│  └────────────┘   └──────────┘   └──────────────┘    │
│                                                          │
│  ┌─────────────────────────────────────────────────┐   │
│  │      Multi-Agent Collaboration                   │   │
│  │   Sequential • Parallel • Hierarchical • P2P     │   │
│  └─────────────────────────────────────────────────┘   │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

## Quick Start

### 1. Basic Ollama Integration

```rust
use abcdodaf::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create Ollama client
    let client = OllamaClient::default()?;

    // List available models
    let models = client.list_models().await?;
    println!("Available models: {}", models.len());

    // Simple generation
    let response = client.generate(
        "llama3.2:3b",
        "Explain Rust ownership in one sentence",
        Some(0.7)
    ).await?;

    println!("Response: {}", response);
    Ok(())
}
```

### 2. Prompt Template Management

```rust
// Create prompt manager with default templates
let mut manager = PromptManager::with_defaults();

// Use a template
let mut values = HashMap::new();
values.insert("language".to_string(), "Rust".to_string());
values.insert("task".to_string(), "Build a web server".to_string());

let prompt = manager.render("code_generation", &values)?;

// Create custom template
let template = PromptTemplate::new(
    "custom_assistant",
    "Custom Assistant",
    "You are an expert in {{domain}}. Help with: {{task}}"
)
.add_variable(PromptVariable::new("domain", "Area of expertise", VariableType::String))
.add_variable(PromptVariable::new("task", "Task description", VariableType::String));

manager.register(template)?;
```

### 3. Context Window Management

```rust
// Create context manager
let mut manager = ContextManager::new(4096, ContextStrategy::SlidingWindow);

// Get or create conversation context
let context = manager.get_or_create("user_123");

// Add messages
context.add_message(ChatMessage::system("You are helpful"))?;
context.add_message(ChatMessage::user("Hello"))?;
context.add_message(ChatMessage::assistant("Hi! How can I help?"))?;

// Check capacity
println!("Tokens used: {}/{}", context.token_count(), context.max_tokens);
println!("Messages: {}", context.message_count());

// Different strategies
let window = ContextWindow::new(
    2000,
    ContextStrategy::HeadTail { head_count: 3, tail_count: 3 }
);
```

### 4. Agent Memory System

```rust
use abcdodaf::ai::memory::{AgentMemory, MemoryEntry, MemoryType};

// Create agent memory
let mut memory = AgentMemory::new(100);

// Store important facts
memory.add_long_term(
    MemoryEntry::new("fact1", MemoryType::Fact, "User prefers Rust")
        .with_importance(0.9)
);

// Store conversation
memory.add_message("conv1", ChatMessage::user("Tell me about async"));
memory.add_message("conv1", ChatMessage::assistant("Async in Rust..."));

// Search memory
let results = memory.search("rust");
println!("Found {} relevant memories", results.len());

// Get statistics
let stats = memory.stats();
println!("Short-term: {}, Long-term: {}",
    stats.short_term_count,
    stats.long_term_count
);

// Consolidate important short-term to long-term
memory.consolidate(0.85); // Threshold: 0.85
```

### 5. Streaming Responses

```rust
use abcdodaf::ai::streaming::{StreamCollector, ProgressTracker};

// Request streaming
let mut receiver = client.chat_stream(request).await?;

// Option 1: Collect all chunks
while let Some(result) = receiver.recv().await {
    match result {
        Ok(chunk) => {
            print!("{}", chunk.content);
            if chunk.done {
                break;
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

// Option 2: Use handlers
let mut collector = StreamCollector::new();
let mut progress = ProgressTracker::new();

// Process with multiple handlers
let response = StreamingResponse::new(receiver);
let final_content = response.process_with(&mut collector).await?;

println!("\n\nFinal: {}", final_content);
println!("Stats: {} chunks, {:.2} chars/sec",
    progress.total_chunks,
    progress.chars_per_second()
);
```

### 6. Model Selection & Fallback

```rust
use abcdodaf::ai::selection::{ModelSelector, SelectionCriteria, FallbackStrategy};

// Create selector with Ollama models
let selector = ModelSelector::with_ollama_defaults();

// Define criteria
let criteria = SelectionCriteria {
    required_capabilities: vec![
        EnhancedCapability::CodeGeneration,
        EnhancedCapability::ChainOfThought,
    ],
    agent_type: Some(EnhancedAgentType::CodeAgent),
    max_latency_ms: Some(200),
    min_quality_score: Some(0.8),
    max_cost_per_1k: None,
};

// Select best model
let model = selector.select(&criteria)?;
println!("Selected: {}", model);

// Get fallback models
let fallbacks = selector.get_fallbacks(
    &model,
    &FallbackStrategy::TryAll,
    &criteria
);
println!("Fallbacks: {:?}", fallbacks);
```

### 7. Multi-Agent Collaboration

```rust
// Create coordinator
let mut coordinator = MultiAgentCoordinator::new();

// Register agents
let researcher = AgentConfig {
    agent_id: "researcher".to_string(),
    agent_type: EnhancedAgentType::Researcher,
    capabilities: vec![
        EnhancedCapability::InformationRetrieval,
        EnhancedCapability::KnowledgeSynthesis,
    ],
    model_config: ModelConfig::default(),
    // ... other config
};

let coder = AgentConfig {
    agent_id: "coder".to_string(),
    agent_type: EnhancedAgentType::CodeAgent,
    capabilities: vec![EnhancedCapability::CodeGeneration],
    // ... other config
};

coordinator.register_agent(researcher);
coordinator.register_agent(coder);

// Create collaboration
let collaboration = coordinator.create_collaboration(
    "research_and_code",
    vec!["researcher".to_string(), "coder".to_string()],
    CollaborationPattern::Sequential,
)?;

// Execute
let result = coordinator.execute(&collaboration, "Build a web scraper").await?;
println!("Result: {}", result.final_result);
println!("Duration: {}ms", result.duration_ms);
```

### 8. Tool Use & Function Calling

```rust
use abcdodaf::ai::tools::{ToolRegistry, ToolDefinition, ToolExecutor};

// Create registry
let mut registry = ToolRegistry::new();

// Define tool
let tool = ToolDefinition::new(
    "web_search",
    "Search the web for information"
)
.add_parameter(ParameterDefinition::new(
    "query",
    "string",
    "Search query"
))
.with_returns("array")
.with_category("search");

// Implement executor
struct WebSearchExecutor;

#[async_trait]
impl ToolExecutor for WebSearchExecutor {
    async fn execute(&self, arguments: HashMap<String, Value>) -> Result<Value> {
        // Implementation
        Ok(json!({"results": []}))
    }
}

registry.register(tool, Box::new(WebSearchExecutor))?;

// Execute tool
let call = ToolCall {
    id: "call1".to_string(),
    tool_name: "web_search".to_string(),
    arguments: {
        let mut args = HashMap::new();
        args.insert("query".to_string(), json!("Rust async"));
        args
    },
};

let result = registry.execute(&call).await;
```

### 9. Performance Metrics & Cost Tracking

```rust
use abcdodaf::ai::metrics::{PerformanceTracker, RequestMetrics};

// Create tracker
let mut tracker = PerformanceTracker::new();

// Record requests
tracker.record_request(RequestMetrics {
    request_id: "req1".to_string(),
    agent_id: "agent1".to_string(),
    latency_ms: 150,
    tokens: 250,
    success: true,
    confidence: 0.92,
    timestamp: Utc::now(),
});

// Record costs
tracker.record_cost("agent1", "llama3.2:3b", 1000, 0.0); // Free
tracker.record_cost("agent1", "gpt-4", 2000, 0.03); // $0.03/1k tokens

// Get summary
let summary = tracker.summary();
println!("Requests: {}", summary.total_requests);
println!("Success rate: {:.1}%", summary.success_rate * 100.0);
println!("Avg latency: {:.1}ms", summary.avg_latency_ms);
println!("Total cost: ${:.4}", summary.total_cost);

// Agent-specific metrics
let metrics = tracker.get_metrics("agent1").unwrap();
println!("Agent success rate: {:.1}%", metrics.success_rate() * 100.0);
println!("Avg confidence: {:.2}", metrics.avg_confidence);
```

### 10. Response Caching

```rust
use abcdodaf::ai::cache::{ResponseCache, CacheStrategy, CacheKey, CacheMetadata};

// Create cache
let mut cache = ResponseCache::new(CacheStrategy::LRU, 1000);

// Create cache key
let key = CacheKey::new("llama3.2:3b", "What is Rust?", 0.7, "");

// Store response
cache.put(
    key.clone(),
    "Rust is a systems programming language...".to_string(),
    CacheMetadata {
        tokens: 150,
        latency_ms: 200,
        cost_saved: 0.01,
    },
);

// Retrieve
if let Some(cached) = cache.get(&key) {
    println!("Cache hit: {}", cached);
}

// Get statistics
let stats = cache.stats();
println!("Hit rate: {:.1}%", stats.hit_rate * 100.0);
println!("Cost saved: ${:.4}", stats.total_cost_saved);
```

## Enhanced Agent Types

The library now supports 20+ specialized agent types:

- **LanguageModel**: General-purpose language understanding
- **VisionModel**: Image and visual processing
- **CodeAgent**: Code generation and analysis
- **Researcher**: Information gathering and synthesis
- **ContentWriter**: Content creation and copywriting
- **CodeReviewer**: Code review and quality analysis
- **Architect**: System architecture and design
- **QualityAssurance**: Testing and QA
- **DevOps**: Deployment and operations
- **SecurityAnalyst**: Security analysis
- **DataScientist**: Data science and ML
- **ProjectManager**: Project coordination
- **CustomerSupport**: Support and assistance
- **Translator**: Language translation
- **Summarizer**: Text summarization
- And more...

## Enhanced Capabilities

Over 25 specialized capabilities:

- NaturalLanguageProcessing
- ComputerVision
- CodeGeneration
- ChainOfThought (long-form reasoning)
- SelfReflection
- GoalPlanning
- TaskChaining
- AgentCollaboration
- FunctionCalling
- StreamingGeneration
- SemanticSearch
- EmbeddingGeneration
- DocumentUnderstanding
- WorkflowOrchestration
- ErrorRecovery
- And more...

## Collaboration Patterns

- **Sequential**: Pipeline processing (A → B → C)
- **Parallel**: Concurrent execution with aggregation
- **Delegation**: One agent delegates to others
- **Consensus**: Agents debate and reach agreement
- **Hierarchical**: Supervisor-subordinate structure
- **PeerToPeer**: Collaborative peer agents

## Running the Example

```bash
# Make sure Ollama is running
ollama serve

# Pull a model if needed
ollama pull llama3.2:3b

# Run the comprehensive example
cargo run --example ai_ollama_integration

# Run specific examples
cargo run --example agent_orchestration
```

## Best Practices

### 1. Context Management
- Use sliding window for long conversations
- Use head-tail for preserving important context
- Monitor token usage to stay within limits

### 2. Memory Management
- Store important facts in long-term memory
- Use importance scores (0.0-1.0) effectively
- Consolidate periodically with appropriate threshold

### 3. Model Selection
- Define clear capability requirements
- Set reasonable latency and quality constraints
- Always configure fallback strategies

### 4. Caching
- Cache for deterministic queries
- Use appropriate TTL for time-sensitive data
- Monitor hit rates and adjust strategy

### 5. Performance
- Track metrics for all agents
- Monitor token usage and costs
- Use streaming for better UX

### 6. Multi-Agent Systems
- Match agents to specialized tasks
- Use sequential for dependent tasks
- Use parallel for independent analysis

## Integration with BPMN Workflows

```rust
use abcdodaf::prelude::*;

// Create workflow with AI agents
let workflow = WorkflowBuilder::new("ai_workflow")
    .add_agent_task(
        "research",
        AgentTask::new("research", "Research", AgentCapability::InformationRetrieval)
            .with_agent_type(EnhancedAgentType::Researcher)
    )
    .add_agent_task(
        "code_gen",
        AgentTask::new("code_gen", "Generate Code", AgentCapability::CodeGeneration)
            .with_agent_type(EnhancedAgentType::CodeAgent)
    )
    .add_agent_task(
        "review",
        AgentTask::new("review", "Review Code", AgentCapability::PatternRecognition)
            .with_agent_type(EnhancedAgentType::CodeReviewer)
    )
    .build()?;

// Execute with AI backend
workflow.execute().await?;
```

## Future Enhancements

- [ ] Vector database integration for semantic search
- [ ] Fine-tuning support for specialized tasks
- [ ] RAG (Retrieval Augmented Generation) pipeline
- [ ] Multi-modal input/output (images, audio)
- [ ] Distributed agent execution across nodes
- [ ] Advanced reasoning (tree-of-thought, etc.)
- [ ] Agent self-improvement and learning
- [ ] Integration with external LLM APIs (OpenAI, Anthropic, etc.)

## Contributing

Contributions are welcome! Please see the main CONTRIBUTING.md for guidelines.

## License

MIT License - see LICENSE file for details.
