feat: Add comprehensive AI/LLM integration with Ollama support

Implements Task #10 - Enhance AI/LLM integration with Ollama and advanced agent capabilities

## Major Features

### 1. Ollama Integration
- Native Rust client for local LLM inference
- Full async/await support with Tokio
- Chat completion and streaming APIs
- Model listing, pulling, and management
- Configurable temperature, sampling, and token limits

### 2. Prompt Template System
- Template management with variable substitution
- Type-safe parameters (String, Integer, Float, Boolean, List, JSON)
- Required/optional variables with defaults
- Built-in templates for common tasks (code gen, review, QA, summarization)
- Template validation and versioning

### 3. Context Window Management
- Multiple retention strategies (SlidingWindow, Summarization, HeadTail, Priority)
- Token counting and tracking
- Per-conversation context isolation
- Automatic pruning when over limit
- Configurable max token limits

### 4. Agent Memory System
- Dual-layer architecture (short-term/long-term)
- Multiple memory types (Conversation, Fact, Preference, TaskContext, Error)
- Importance-based scoring (0.0 - 1.0)
- Conversation history tracking
- Content search and consolidation

### 5. Streaming Support
- Real-time response streaming
- Progress tracking with metrics
- Stream handlers for custom processing
- Chunk collection and aggregation
- Cancellation support

### 6. Model Selection & Fallback
- Intelligent selection strategies (Fastest, BestQuality, Balanced, Cheapest, Custom)
- Capability-based filtering
- Performance constraints (latency, quality, cost)
- Multiple fallback strategies (NextBest, Specific, TryAll)
- Pre-configured Ollama models

### 7. Multi-Agent Collaboration
- 6 collaboration patterns (Sequential, Parallel, Delegation, Consensus, Hierarchical, PeerToPeer)
- Agent registration and coordination
- Shared context between agents
- Result aggregation
- Execution history tracking

### 8. Tool Use & Function Calling
- Tool definition framework with parameters
- Async tool execution
- Type-safe parameter validation
- Built-in tools (calculator, etc.)
- Category organization

### 9. Performance Metrics & Cost Tracking
- Per-agent metrics (requests, tokens, latency, confidence, success rates)
- Cost tracking by agent and model
- Performance summaries and statistics
- Historical data collection
- Real-time monitoring

### 10. Response Caching
- Multiple cache strategies (LRU, LFU, TTL, Adaptive)
- Automatic eviction policies
- Hit/miss tracking
- Cost savings calculation
- Configurable size limits

## Enhanced Capabilities

### Agent Types (20+)
- Expanded from 8 to 20+ specialized types
- New: Researcher, ContentWriter, CodeReviewer, Architect, QualityAssurance, DevOps, SecurityAnalyst, DataScientist, ProjectManager, CustomerSupport, Translator, Summarizer, QuestionAnswering, SentimentAnalyzer, EntityRecognizer, TextClassifier, Conversational, CreativeWriter, TechnicalWriter

### Capabilities (25+)
- Expanded from 12 to 25+ capabilities
- New: ChainOfThought, SelfReflection, GoalPlanning, ContextManagement, TaskChaining, AgentCollaboration, FunctionCalling, StreamingGeneration, MultiLanguage, SemanticSearch, EmbeddingGeneration, DocumentUnderstanding, DataExtraction, WorkflowOrchestration, ErrorRecovery, PerformanceOptimization

## Files Added

```
src/ai/
├── mod.rs                      # 10.8 KB - Module exports and core types
├── ollama.rs                   # 15.1 KB - Ollama client implementation
├── prompt.rs                   # 13.8 KB - Prompt template management
├── context.rs                  # 10.7 KB - Context window management
├── memory.rs                   # 10.1 KB - Agent memory system
├── streaming.rs                # 7.7 KB  - Streaming support
├── selection.rs                # 12.5 KB - Model selection & fallback
├── collaboration.rs            # 8.0 KB  - Multi-agent coordination
├── tools.rs                    # 7.7 KB  - Tool use & function calling
├── metrics.rs                  # 10.6 KB - Performance metrics
└── cache.rs                    # 10.2 KB - Response caching

examples/
└── ai_ollama_integration.rs    # 18.3 KB - Comprehensive demo

tests/
└── ai_integration_test.rs      # 8.5 KB  - Integration tests

docs/
├── AI_INTEGRATION.md           # 15.2 KB - Usage guide
└── TASK_10_SUMMARY.md          # 14.8 KB - Implementation summary

Total: ~164 KB of new code + documentation
```

## Dependencies Added

```toml
reqwest = { version = "0.11", features = ["json", "stream"] }
tokio-util = { version = "0.7", features = ["io"] }
regex = "1"
async-stream = "0.3"
```

## Integration

- Exported via `prelude` module for easy access
- Compatible with existing BPMN workflow system
- Works with DoDAF operational modeling
- Maintains async/await patterns throughout
- Comprehensive error handling with AbcdodafError

## Testing

- Unit tests for all modules
- Integration tests for end-to-end workflows
- Example demonstrating all features
- Tests pass with Ollama running

## Documentation

- Comprehensive AI_INTEGRATION.md guide with examples
- Inline documentation for all public APIs
- Task summary with implementation details
- Example code with working demonstrations

## Breaking Changes

None. All additions are new modules that don't affect existing APIs.

## Migration Guide

No migration needed. Simply use the new AI features via:

```rust
use abcdodaf::prelude::*;

let client = OllamaClient::default()?;
let manager = PromptManager::with_defaults();
let memory = AgentMemory::new(100);
// ... etc
```

## Future Work

- Vector database integration for RAG
- External API support (OpenAI, Anthropic, etc.)
- Advanced reasoning patterns (tree-of-thought)
- Multi-modal support (images, audio)
- Distributed agent execution
- Fine-tuning support

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
