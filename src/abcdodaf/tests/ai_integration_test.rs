//! Integration tests for AI/LLM functionality

use abcdodaf::prelude::*;
use std::collections::HashMap;

#[tokio::test]
async fn test_ollama_client_creation() {
    // Test that we can create an Ollama client
    let result = OllamaClient::default();
    assert!(result.is_ok(), "Should create Ollama client");

    let client = result.unwrap();
    assert_eq!(client.base_url(), "http://localhost:11434");
    assert_eq!(client.default_model(), "llama3.2:3b");
}

#[tokio::test]
async fn test_ollama_list_models() {
    // This test requires Ollama to be running
    let client = OllamaClient::default().unwrap();

    match client.list_models().await {
        Ok(models) => {
            println!("Found {} models", models.len());
            assert!(!models.is_empty(), "Should have at least one model");
        },
        Err(e) => {
            println!("Note: Ollama not running or not accessible: {}", e);
            // Don't fail the test if Ollama isn't running
        },
    }
}

#[test]
fn test_prompt_template_creation() {
    let template = PromptTemplate::new("test", "Test Template", "Hello {{name}}!").add_variable(
        abcdodaf::ai::PromptVariable::new(
            "name",
            "User name",
            abcdodaf::ai::prompt::VariableType::String,
        ),
    );

    assert_eq!(template.id, "test");
    assert_eq!(template.variables.len(), 1);
}

#[test]
fn test_prompt_template_rendering() {
    let template =
        PromptTemplate::new("greeting", "Greeting", "Hello {{name}}, you are {{age}} years old!")
            .add_variable(abcdodaf::ai::PromptVariable::new(
                "name",
                "Name",
                abcdodaf::ai::prompt::VariableType::String,
            ))
            .add_variable(abcdodaf::ai::PromptVariable::new(
                "age",
                "Age",
                abcdodaf::ai::prompt::VariableType::Integer,
            ));

    let mut values = HashMap::new();
    values.insert("name".to_string(), "Alice".to_string());
    values.insert("age".to_string(), "30".to_string());

    let rendered = template.render(&values).unwrap();
    assert_eq!(rendered, "Hello Alice, you are 30 years old!");
}

#[test]
fn test_prompt_manager() {
    let manager = PromptManager::with_defaults();

    // Check default templates exist
    assert!(!manager.list().is_empty());
    assert!(manager.get("code_generation").is_some());
    assert!(manager.get("code_review").is_some());
    assert!(manager.get("question_answering").is_some());

    // Check categories
    let code_templates = manager.list_by_category("code");
    assert!(!code_templates.is_empty());
}

#[test]
fn test_context_window() {
    use abcdodaf::ai::ollama::ChatMessage;

    let mut window = ContextWindow::new(4096, ContextStrategy::SlidingWindow);

    window.add_message(ChatMessage::system("You are helpful")).unwrap();
    window.add_message(ChatMessage::user("Hello")).unwrap();
    window.add_message(ChatMessage::assistant("Hi there!")).unwrap();

    assert_eq!(window.message_count(), 3);
    assert!(window.token_count() > 0);
    assert!(window.remaining_tokens() < 4096);
}

#[test]
fn test_context_manager() {
    let mut manager = ContextManager::new(4096, ContextStrategy::SlidingWindow);

    let ctx1 = manager.get_or_create("user1");
    ctx1.add_message(abcdodaf::ai::ollama::ChatMessage::user("Test")).unwrap();

    let ctx2 = manager.get_or_create("user2");
    ctx2.add_message(abcdodaf::ai::ollama::ChatMessage::user("Test 2")).unwrap();

    assert_eq!(manager.conversation_count(), 2);
}

#[test]
fn test_agent_memory() {
    use abcdodaf::ai::memory::{AgentMemory, MemoryEntry, MemoryType};

    let mut memory = AgentMemory::new(50);

    memory
        .add_short_term(MemoryEntry::new("m1", MemoryType::Fact, "Test fact").with_importance(0.5));

    memory.add_long_term(
        MemoryEntry::new("m2", MemoryType::Preference, "User prefers Rust").with_importance(0.9),
    );

    assert_eq!(memory.short_term_memories().len(), 1);
    assert_eq!(memory.long_term_memories().len(), 1);

    // Test search
    let results = memory.search("rust");
    assert_eq!(results.len(), 1);
}

#[test]
fn test_model_selector() {
    let selector = ModelSelector::with_ollama_defaults();

    assert!(!selector.list_models().is_empty());
    assert!(selector.get_capabilities("llama3.2:3b").is_some());
}

#[test]
fn test_model_selection() {
    use abcdodaf::ai::selection::SelectionCriteria;

    let selector = ModelSelector::with_ollama_defaults();

    let criteria = SelectionCriteria {
        required_capabilities: vec![EnhancedCapability::NaturalLanguageProcessing],
        agent_type: None,
        max_latency_ms: Some(300),
        min_quality_score: Some(0.7),
        max_cost_per_1k: None,
    };

    let result = selector.select(&criteria);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_multi_agent_coordinator() {
    let mut coordinator = MultiAgentCoordinator::new();

    let agent = abcdodaf::ai::AgentConfig {
        agent_id: "test_agent".to_string(),
        agent_type: EnhancedAgentType::LanguageModel,
        capabilities: vec![EnhancedCapability::NaturalLanguageProcessing],
        model_config: abcdodaf::ai::ModelConfig::default(),
        prompt_config: abcdodaf::ai::PromptConfig::default(),
        context_config: abcdodaf::ai::ContextConfig::default(),
        memory_config: abcdodaf::ai::MemoryConfig::default(),
        performance_config: abcdodaf::ai::PerformanceConfig::default(),
    };

    coordinator.register_agent(agent);
    assert_eq!(coordinator.agent_count(), 1);
}

#[tokio::test]
async fn test_tool_registry() {
    let registry = ToolRegistry::with_defaults();

    assert!(!registry.list_tools().is_empty());
    assert!(registry.get_definition("calculator").is_some());

    // Test tool execution
    let call = abcdodaf::ai::ToolCall {
        id: "test".to_string(),
        tool_name: "calculator".to_string(),
        arguments: {
            let mut args = HashMap::new();
            args.insert("expression".to_string(), serde_json::json!("1 + 1"));
            args
        },
    };

    let result = registry.execute(&call).await;
    assert_eq!(result.call_id, "test");
}

#[test]
fn test_performance_tracker() {
    use abcdodaf::ai::metrics::{PerformanceTracker, RequestMetrics};

    let mut tracker = PerformanceTracker::new();

    tracker.record_request(RequestMetrics {
        request_id: "req1".to_string(),
        agent_id: "agent1".to_string(),
        latency_ms: 100,
        tokens: 150,
        success: true,
        confidence: 0.9,
        timestamp: chrono::Utc::now(),
    });

    let summary = tracker.summary();
    assert_eq!(summary.total_requests, 1);
    assert_eq!(summary.success_rate, 1.0);
}

#[test]
fn test_response_cache() {
    use abcdodaf::ai::cache::{CacheKey, CacheMetadata, ResponseCache};

    let mut cache = ResponseCache::new(CacheStrategy::LRU, 100);

    let key = CacheKey::new("model1", "test prompt", 0.7, "");

    cache.put(
        key.clone(),
        "test response".to_string(),
        CacheMetadata { tokens: 50, latency_ms: 100, cost_saved: 0.01 },
    );

    let retrieved = cache.get(&key);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), "test response");

    let stats = cache.stats();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 0);
}

#[test]
fn test_enhanced_agent_types() {
    // Test serialization
    let agent_type = EnhancedAgentType::Researcher;
    let json = serde_json::to_string(&agent_type).unwrap();
    assert_eq!(json, "\"Researcher\"");

    let deserialized: EnhancedAgentType = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, agent_type);
}

#[test]
fn test_enhanced_capabilities() {
    let cap = EnhancedCapability::ChainOfThought;

    // Test hash
    let mut set = std::collections::HashSet::new();
    set.insert(cap.clone());
    assert!(set.contains(&EnhancedCapability::ChainOfThought));

    // Test serialization
    let json = serde_json::to_string(&cap).unwrap();
    assert_eq!(json, "\"ChainOfThought\"");
}
