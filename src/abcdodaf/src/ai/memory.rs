//! Agent memory and state management
//!
//! Provides short-term and long-term memory for AI agents with conversation history.

use crate::ai::ollama::ChatMessage;
use crate::error::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// Agent memory store
pub struct AgentMemory {
    /// Short-term memory (recent interactions)
    short_term: VecDeque<MemoryEntry>,
    /// Long-term memory (important facts/context)
    long_term: HashMap<String, MemoryEntry>,
    /// Conversation history
    conversations: HashMap<String, ConversationHistory>,
    /// Maximum short-term entries
    max_short_term: usize,
}

/// Memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Entry ID
    pub id: String,
    /// Entry type
    pub entry_type: MemoryType,
    /// Content
    pub content: String,
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Importance score (0.0 - 1.0)
    pub importance: f32,
}

/// Memory type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryType {
    /// Conversation exchange
    Conversation,
    /// Important fact or insight
    Fact,
    /// User preference
    Preference,
    /// Task context
    TaskContext,
    /// Error or issue
    Error,
    /// Custom type
    Custom(String),
}

/// Conversation history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationHistory {
    /// Conversation ID
    pub conversation_id: String,
    /// Messages in order
    pub messages: Vec<ChatMessage>,
    /// Conversation metadata
    pub metadata: HashMap<String, String>,
    /// Started timestamp
    pub started_at: DateTime<Utc>,
    /// Last updated
    pub updated_at: DateTime<Utc>,
}

/// Memory store interface
pub trait MemoryStore: Send + Sync {
    /// Store a memory entry
    fn store(&mut self, entry: MemoryEntry) -> Result<()>;

    /// Retrieve memory entries by type
    fn retrieve_by_type(&self, memory_type: MemoryType) -> Vec<&MemoryEntry>;

    /// Search memory by content
    fn search(&self, query: &str) -> Vec<&MemoryEntry>;

    /// Get recent memories
    fn recent(&self, count: usize) -> Vec<&MemoryEntry>;
}

impl AgentMemory {
    /// Create a new agent memory
    pub fn new(max_short_term: usize) -> Self {
        Self {
            short_term: VecDeque::new(),
            long_term: HashMap::new(),
            conversations: HashMap::new(),
            max_short_term,
        }
    }

    /// Add to short-term memory
    pub fn add_short_term(&mut self, entry: MemoryEntry) {
        if self.short_term.len() >= self.max_short_term {
            self.short_term.pop_front();
        }
        self.short_term.push_back(entry);
    }

    /// Add to long-term memory
    pub fn add_long_term(&mut self, entry: MemoryEntry) {
        self.long_term.insert(entry.id.clone(), entry);
    }

    /// Get short-term memories
    pub fn short_term_memories(&self) -> Vec<&MemoryEntry> {
        self.short_term.iter().collect()
    }

    /// Get long-term memories
    pub fn long_term_memories(&self) -> Vec<&MemoryEntry> {
        self.long_term.values().collect()
    }

    /// Search memories
    pub fn search(&self, query: &str) -> Vec<&MemoryEntry> {
        let query_lower = query.to_lowercase();

        let mut results: Vec<&MemoryEntry> = self
            .short_term
            .iter()
            .chain(self.long_term.values())
            .filter(|entry| entry.content.to_lowercase().contains(&query_lower))
            .collect();

        // Sort by importance and recency
        results.sort_by(|a, b| {
            b.importance
                .partial_cmp(&a.importance)
                .unwrap()
                .then_with(|| b.timestamp.cmp(&a.timestamp))
        });

        results
    }

    /// Get or create conversation history
    pub fn get_or_create_conversation(&mut self, conversation_id: &str) -> &mut ConversationHistory {
        self.conversations
            .entry(conversation_id.to_string())
            .or_insert_with(|| ConversationHistory {
                conversation_id: conversation_id.to_string(),
                messages: vec![],
                metadata: HashMap::new(),
                started_at: Utc::now(),
                updated_at: Utc::now(),
            })
    }

    /// Add message to conversation
    pub fn add_message(&mut self, conversation_id: &str, message: ChatMessage) {
        let conversation = self.get_or_create_conversation(conversation_id);
        conversation.messages.push(message);
        conversation.updated_at = Utc::now();
    }

    /// Get conversation history
    pub fn get_conversation(&self, conversation_id: &str) -> Option<&ConversationHistory> {
        self.conversations.get(conversation_id)
    }

    /// Clear short-term memory
    pub fn clear_short_term(&mut self) {
        self.short_term.clear();
    }

    /// Clear all memory
    pub fn clear_all(&mut self) {
        self.short_term.clear();
        self.long_term.clear();
        self.conversations.clear();
    }

    /// Consolidate important short-term memories to long-term
    pub fn consolidate(&mut self, importance_threshold: f32) {
        let to_promote: Vec<_> = self
            .short_term
            .iter()
            .filter(|entry| entry.importance >= importance_threshold)
            .cloned()
            .collect();

        for entry in to_promote {
            self.long_term.insert(entry.id.clone(), entry);
        }
    }

    /// Get memory statistics
    pub fn stats(&self) -> MemoryStats {
        MemoryStats {
            short_term_count: self.short_term.len(),
            long_term_count: self.long_term.len(),
            conversation_count: self.conversations.len(),
            total_messages: self
                .conversations
                .values()
                .map(|c| c.messages.len())
                .sum(),
        }
    }
}

/// Memory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Short-term memory count
    pub short_term_count: usize,
    /// Long-term memory count
    pub long_term_count: usize,
    /// Active conversation count
    pub conversation_count: usize,
    /// Total message count across conversations
    pub total_messages: usize,
}

impl MemoryEntry {
    /// Create a new memory entry
    pub fn new(
        id: impl Into<String>,
        entry_type: MemoryType,
        content: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            entry_type,
            content: content.into(),
            metadata: HashMap::new(),
            timestamp: Utc::now(),
            importance: 0.5,
        }
    }

    /// Set importance
    pub fn with_importance(mut self, importance: f32) -> Self {
        self.importance = importance.clamp(0.0, 1.0);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

impl Default for AgentMemory {
    fn default() -> Self {
        Self::new(100)
    }
}

impl MemoryStore for AgentMemory {
    fn store(&mut self, entry: MemoryEntry) -> Result<()> {
        if entry.importance >= 0.7 {
            self.add_long_term(entry);
        } else {
            self.add_short_term(entry);
        }
        Ok(())
    }

    fn retrieve_by_type(&self, memory_type: MemoryType) -> Vec<&MemoryEntry> {
        self.short_term
            .iter()
            .chain(self.long_term.values())
            .filter(|entry| entry.entry_type == memory_type)
            .collect()
    }

    fn search(&self, query: &str) -> Vec<&MemoryEntry> {
        self.search(query)
    }

    fn recent(&self, count: usize) -> Vec<&MemoryEntry> {
        self.short_term.iter().rev().take(count).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_memory_creation() {
        let memory = AgentMemory::new(10);
        assert_eq!(memory.short_term_memories().len(), 0);
    }

    #[test]
    fn test_add_short_term() {
        let mut memory = AgentMemory::new(2);
        memory.add_short_term(MemoryEntry::new("1", MemoryType::Conversation, "Test 1"));
        memory.add_short_term(MemoryEntry::new("2", MemoryType::Conversation, "Test 2"));
        memory.add_short_term(MemoryEntry::new("3", MemoryType::Conversation, "Test 3"));

        // Should have only 2 entries (oldest removed)
        assert_eq!(memory.short_term_memories().len(), 2);
    }

    #[test]
    fn test_add_long_term() {
        let mut memory = AgentMemory::default();
        memory.add_long_term(MemoryEntry::new("fact1", MemoryType::Fact, "Important fact"));
        assert_eq!(memory.long_term_memories().len(), 1);
    }

    #[test]
    fn test_memory_search() {
        let mut memory = AgentMemory::default();
        memory.add_short_term(MemoryEntry::new("1", MemoryType::Conversation, "Rust programming"));
        memory.add_short_term(MemoryEntry::new("2", MemoryType::Conversation, "Python code"));

        let results = memory.search("rust");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_conversation_history() {
        let mut memory = AgentMemory::default();
        memory.add_message("conv1", ChatMessage::user("Hello"));
        memory.add_message("conv1", ChatMessage::assistant("Hi there"));

        let conv = memory.get_conversation("conv1").unwrap();
        assert_eq!(conv.messages.len(), 2);
    }

    #[test]
    fn test_consolidate() {
        let mut memory = AgentMemory::default();
        memory.add_short_term(
            MemoryEntry::new("1", MemoryType::Fact, "Important").with_importance(0.9),
        );
        memory.add_short_term(
            MemoryEntry::new("2", MemoryType::Conversation, "Normal").with_importance(0.3),
        );

        memory.consolidate(0.8);
        assert_eq!(memory.long_term_memories().len(), 1);
    }
}
