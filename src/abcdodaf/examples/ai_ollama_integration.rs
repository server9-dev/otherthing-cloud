//! AI/LLM Integration with Ollama Example
//!
//! Demonstrates the comprehensive AI agent system with Ollama integration,
//! including prompt templates, context management, memory, streaming,
//! model selection, and multi-agent collaboration.

use abcdodaf::prelude::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("=== ABCDODAF AI/LLM Integration Demo ===\n");

    // 1. Ollama Client Setup
    demo_ollama_client().await?;

    // 2. Prompt Template Management
    demo_prompt_templates()?;

    // 3. Context Window Management
    demo_context_management()?;

    // 4. Agent Memory
    demo_agent_memory()?;

    // 5. Model Selection
    demo_model_selection()?;

    // 6. Multi-Agent Collaboration
    demo_multi_agent_collaboration().await?;

    // 7. Tool Registry
    demo_tool_registry().await?;

    // 8. Performance Metrics
    demo_performance_metrics()?;

    // 9. Response Caching
    demo_response_caching()?;

    println!("\n=== Demo Complete ===");
    Ok(())
}

async fn demo_ollama_client() -> Result<(), Box<dyn std::error::Error>> {
    println!("1. Ollama Client Integration");
    println!("-----------------------------");

    // Create Ollama client
    let client = OllamaClient::default()?;
    println!("✓ Connected to Ollama at {}", client.base_url());

    // List available models
    match client.list_models().await {
        Ok(models) => {
            println!("✓ Found {} available models:", models.len());
            for model in models.iter().take(3) {
                println!("  - {} ({:.2} GB)", model.name, model.size as f64 / 1e9);
            }
        }
        Err(e) => {
            println!("✗ Could not list models: {}", e);
            println!("  (Make sure Ollama is running: ollama serve)");
        }
    }

    // Simple chat completion
    let request = abcdodaf::ai::ollama::ChatRequest::new("llama3.2:3b")
        .add_message(abcdodaf::ai::ollama::ChatMessage::system(
            "You are a helpful assistant specialized in Rust programming."
        ))
        .add_message(abcdodaf::ai::ollama::ChatMessage::user(
            "What is the difference between String and &str in Rust?"
        ))
        .with_temperature(0.7)
        .with_max_tokens(500);

    println!("\n✓ Sending chat request to llama3.2:3b...");
    match client.chat(request).await {
        Ok(response) => {
            println!("✓ Response received:");
            let preview = if response.message.content.len() > 200 {
                format!("{}...", &response.message.content[..200])
            } else {
                response.message.content.clone()
            };
            println!("  {}", preview);

            if let Some(tokens) = response.eval_count {
                println!("  Tokens generated: {}", tokens);
            }
        }
        Err(e) => {
            println!("✗ Chat request failed: {}", e);
        }
    }

    println!();
    Ok(())
}

fn demo_prompt_templates() -> Result<(), Box<dyn std::error::Error>> {
    println!("2. Prompt Template Management");
    println!("-----------------------------");

    // Create prompt manager with defaults
    let manager = PromptManager::with_defaults();
    println!("✓ Created prompt manager with {} default templates", manager.list().len());

    // List templates by category
    let code_templates = manager.list_by_category("code");
    println!("✓ Found {} code-related templates", code_templates.len());

    // Use code generation template
    let mut values = HashMap::new();
    values.insert("language".to_string(), "Rust".to_string());
    values.insert("task".to_string(), "Create a binary search tree".to_string());
    values.insert("requirements".to_string(), "Include insert, search, and delete operations".to_string());

    match manager.render("code_generation", &values) {
        Ok(rendered) => {
            println!("✓ Rendered code generation prompt:");
            let preview = if rendered.len() > 150 {
                format!("{}...", &rendered[..150])
            } else {
                rendered
            };
            println!("  {}", preview);
        }
        Err(e) => println!("✗ Template rendering failed: {}", e),
    }

    // Create custom template
    let custom_template = PromptTemplate::new(
        "rust_expert",
        "Rust Expert Consultation",
        "As a Rust expert with {{years}} years of experience, provide advice on: {{topic}}\n\nFocus on: {{focus_areas}}"
    )
    .add_variable(abcdodaf::ai::PromptVariable::new("years", "Years of experience", abcdodaf::ai::prompt::VariableType::Integer))
    .add_variable(abcdodaf::ai::PromptVariable::new("topic", "Topic to discuss", abcdodaf::ai::prompt::VariableType::String))
    .add_variable(
        abcdodaf::ai::PromptVariable::new("focus_areas", "Areas to focus on", abcdodaf::ai::prompt::VariableType::String)
            .optional("best practices and performance")
    )
    .with_category("consulting");

    println!("✓ Created custom template '{}' with {} variables",
        custom_template.name, custom_template.variables.len());

    println!();
    Ok(())
}

fn demo_context_management() -> Result<(), Box<dyn std::error::Error>> {
    println!("3. Context Window Management");
    println!("---------------------------");

    // Create context manager
    let mut manager = ContextManager::new(4096, ContextStrategy::SlidingWindow);
    println!("✓ Created context manager with 4096 token limit");

    // Create a conversation
    let context = manager.get_or_create("conversation_1");
    context.add_message(abcdodaf::ai::ollama::ChatMessage::system(
        "You are a helpful AI assistant."
    ))?;
    context.add_message(abcdodaf::ai::ollama::ChatMessage::user(
        "Tell me about Rust's ownership system."
    ))?;
    context.add_message(abcdodaf::ai::ollama::ChatMessage::assistant(
        "Rust's ownership system is a set of rules that the compiler checks at compile time..."
    ))?;

    println!("✓ Added 3 messages to conversation");
    println!("  Messages: {}", context.message_count());
    println!("  Estimated tokens: {}", context.token_count());
    println!("  Remaining capacity: {} tokens", context.remaining_tokens());

    // Test different strategies
    let mut window = ContextWindow::new(1000, ContextStrategy::HeadTail { head_count: 2, tail_count: 2 });
    for i in 0..10 {
        window.add_message(abcdodaf::ai::ollama::ChatMessage::user(format!("Message {}", i)))?;
    }
    println!("✓ Tested HeadTail strategy: {} messages retained", window.message_count());

    println!();
    Ok(())
}

fn demo_agent_memory() -> Result<(), Box<dyn std::error::Error>> {
    println!("4. Agent Memory System");
    println!("---------------------");

    // Create agent memory
    let mut memory = AgentMemory::new(50);
    println!("✓ Created agent memory with capacity for 50 short-term entries");

    // Add memories
    use abcdodaf::ai::memory::{MemoryEntry, MemoryType};

    memory.add_short_term(
        MemoryEntry::new("mem1", MemoryType::Fact, "User prefers concise explanations")
            .with_importance(0.8)
    );

    memory.add_short_term(
        MemoryEntry::new("mem2", MemoryType::Preference, "Interested in Rust systems programming")
            .with_importance(0.9)
    );

    memory.add_long_term(
        MemoryEntry::new("mem3", MemoryType::TaskContext, "Working on a distributed systems project")
            .with_importance(0.95)
    );

    // Add conversation
    memory.add_message("conv1", abcdodaf::ai::ollama::ChatMessage::user("Hello"));
    memory.add_message("conv1", abcdodaf::ai::ollama::ChatMessage::assistant("Hi! How can I help?"));

    let stats = memory.stats();
    println!("✓ Memory statistics:");
    println!("  Short-term: {} entries", stats.short_term_count);
    println!("  Long-term: {} entries", stats.long_term_count);
    println!("  Conversations: {}", stats.conversation_count);
    println!("  Total messages: {}", stats.total_messages);

    // Search memory
    let results = memory.search("rust");
    println!("✓ Search for 'rust' found {} results", results.len());

    // Consolidate important memories
    memory.consolidate(0.85);
    let new_stats = memory.stats();
    println!("✓ After consolidation: {} long-term entries", new_stats.long_term_count);

    println!();
    Ok(())
}

fn demo_model_selection() -> Result<(), Box<dyn std::error::Error>> {
    println!("5. Model Selection & Fallback");
    println!("----------------------------");

    // Create model selector with Ollama defaults
    let selector = ModelSelector::with_ollama_defaults();
    println!("✓ Created model selector with {} models", selector.list_models().len());

    // Define selection criteria
    use abcdodaf::ai::selection::SelectionCriteria;

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

    match selector.select(&criteria) {
        Ok(model) => {
            println!("✓ Selected model: {}", model);

            // Get fallbacks
            let fallbacks = selector.get_fallbacks(
                &model,
                &FallbackStrategy::NextBest,
                &criteria
            );
            println!("✓ Fallback models: {:?}", fallbacks);
        }
        Err(e) => println!("✗ Model selection failed: {}", e),
    }

    println!();
    Ok(())
}

async fn demo_multi_agent_collaboration() -> Result<(), Box<dyn std::error::Error>> {
    println!("6. Multi-Agent Collaboration");
    println!("---------------------------");

    // Create coordinator
    let mut coordinator = MultiAgentCoordinator::new();
    println!("✓ Created multi-agent coordinator");

    // Register agents
    let researcher = AgentConfig {
        agent_id: "researcher".to_string(),
        agent_type: EnhancedAgentType::Researcher,
        capabilities: vec![
            EnhancedCapability::InformationRetrieval,
            EnhancedCapability::KnowledgeSynthesis,
        ],
        model_config: abcdodaf::ai::ModelConfig::default(),
        prompt_config: abcdodaf::ai::PromptConfig::default(),
        context_config: abcdodaf::ai::ContextConfig::default(),
        memory_config: abcdodaf::ai::MemoryConfig::default(),
        performance_config: abcdodaf::ai::PerformanceConfig::default(),
    };

    let coder = AgentConfig {
        agent_id: "coder".to_string(),
        agent_type: EnhancedAgentType::CodeAgent,
        capabilities: vec![
            EnhancedCapability::CodeGeneration,
            EnhancedCapability::LogicalReasoning,
        ],
        model_config: abcdodaf::ai::ModelConfig::default(),
        prompt_config: abcdodaf::ai::PromptConfig::default(),
        context_config: abcdodaf::ai::ContextConfig::default(),
        memory_config: abcdodaf::ai::MemoryConfig::default(),
        performance_config: abcdodaf::ai::PerformanceConfig::default(),
    };

    coordinator.register_agent(researcher);
    coordinator.register_agent(coder);
    println!("✓ Registered {} agents", coordinator.agent_count());

    // Create sequential collaboration
    let collaboration = coordinator.create_collaboration(
        "research_then_code",
        vec!["researcher".to_string(), "coder".to_string()],
        CollaborationPattern::Sequential,
    )?;

    println!("✓ Created sequential collaboration: {}", collaboration.id);

    // Execute collaboration
    let result = coordinator.execute(&collaboration, "Create a Rust web server").await?;
    println!("✓ Collaboration executed:");
    println!("  Duration: {}ms", result.duration_ms);
    println!("  Responses: {}", result.responses.len());
    println!("  Final result: {}", &result.final_result[..result.final_result.len().min(100)]);

    println!();
    Ok(())
}

async fn demo_tool_registry() -> Result<(), Box<dyn std::error::Error>> {
    println!("7. Tool Registry & Function Calling");
    println!("----------------------------------");

    // Create tool registry with defaults
    let registry = ToolRegistry::with_defaults();
    println!("✓ Created tool registry with {} tools", registry.list_tools().len());

    // List available tools
    for tool in registry.list_tools() {
        println!("  - {}: {}", tool.name, tool.description);
    }

    // Execute a tool
    let tool_call = ToolCall {
        id: "call1".to_string(),
        tool_name: "calculator".to_string(),
        arguments: {
            let mut args = HashMap::new();
            args.insert("expression".to_string(), serde_json::json!("2 + 2 * 3"));
            args
        },
    };

    let result = registry.execute(&tool_call).await;
    println!("✓ Tool execution:");
    println!("  Success: {}", result.success);
    println!("  Result: {:?}", result.result);
    println!("  Duration: {}ms", result.duration_ms);

    println!();
    Ok(())
}

fn demo_performance_metrics() -> Result<(), Box<dyn std::error::Error>> {
    println!("8. Performance Metrics & Cost Tracking");
    println!("-------------------------------------");

    // Create performance tracker
    let mut tracker = PerformanceTracker::new();
    println!("✓ Created performance tracker");

    // Record some requests
    use abcdodaf::ai::metrics::RequestMetrics;

    for i in 0..5 {
        tracker.record_request(RequestMetrics {
            request_id: format!("req{}", i),
            agent_id: "agent1".to_string(),
            latency_ms: 100 + i * 10,
            tokens: 150 + i * 20,
            success: i < 4, // One failure
            confidence: 0.8 + (i as f32 * 0.02),
            timestamp: chrono::Utc::now(),
        });
    }

    // Record costs
    tracker.record_cost("agent1", "llama3.2:3b", 1000, 0.0); // Free for local models
    tracker.record_cost("agent1", "gpt-4", 2000, 0.03);

    // Get summary
    let summary = tracker.summary();
    println!("✓ Performance summary:");
    println!("  Total agents: {}", summary.total_agents);
    println!("  Total requests: {}", summary.total_requests);
    println!("  Success rate: {:.1}%", summary.success_rate * 100.0);
    println!("  Avg latency: {:.1}ms", summary.avg_latency_ms);
    println!("  Total tokens: {}", summary.total_tokens);
    println!("  Total cost: ${:.4}", summary.total_cost);

    // Get agent-specific metrics
    if let Some(metrics) = tracker.get_metrics("agent1") {
        println!("✓ Agent 'agent1' metrics:");
        println!("  Requests: {}", metrics.total_requests);
        println!("  Success rate: {:.1}%", metrics.success_rate() * 100.0);
        println!("  Avg confidence: {:.2}", metrics.avg_confidence);
    }

    println!();
    Ok(())
}

fn demo_response_caching() -> Result<(), Box<dyn std::error::Error>> {
    println!("9. Response Caching");
    println!("------------------");

    // Create cache with LRU strategy
    let mut cache = ResponseCache::new(CacheStrategy::LRU, 100);
    println!("✓ Created response cache (LRU, max 100 entries)");

    // Create cache keys and store responses
    use abcdodaf::ai::cache::{CacheKey, CacheMetadata};

    let key1 = CacheKey::new("llama3.2:3b", "What is Rust?", 0.7, "");
    let key2 = CacheKey::new("llama3.2:3b", "Explain ownership", 0.7, "");

    cache.put(
        key1.clone(),
        "Rust is a systems programming language...".to_string(),
        CacheMetadata {
            tokens: 150,
            latency_ms: 200,
            cost_saved: 0.01,
        },
    );

    cache.put(
        key2.clone(),
        "Ownership is Rust's most unique feature...".to_string(),
        CacheMetadata {
            tokens: 200,
            latency_ms: 250,
            cost_saved: 0.015,
        },
    );

    // Test cache hits and misses
    println!("✓ Stored 2 responses");

    if let Some(response) = cache.get(&key1) {
        println!("✓ Cache hit for key1: {}...", &response[..50]);
    }

    let key3 = CacheKey::new("llama3.2:3b", "Unknown query", 0.7, "");
    if cache.get(&key3).is_none() {
        println!("✓ Cache miss for unknown key");
    }

    // Get cache statistics
    let stats = cache.stats();
    println!("✓ Cache statistics:");
    println!("  Size: {}/{}", stats.size, stats.max_size);
    println!("  Hits: {}", stats.hits);
    println!("  Misses: {}", stats.misses);
    println!("  Hit rate: {:.1}%", stats.hit_rate * 100.0);
    println!("  Cost saved: ${:.4}", stats.total_cost_saved);

    println!();
    Ok(())
}
