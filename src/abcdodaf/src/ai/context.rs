//! Context window management
//!
//! Manages conversation context, token limits, and context compression strategies.

use crate::ai::ollama::ChatMessage;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Context window manager
pub struct ContextWindow {
    /// Maximum context size in tokens
    max_tokens: usize,
    /// Current messages
    messages: VecDeque<ChatMessage>,
    /// Token count (estimated)
    estimated_tokens: usize,
    /// Retention strategy
    strategy: ContextStrategy,
}

/// Context management strategy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContextStrategy {
    /// Keep most recent messages (FIFO)
    SlidingWindow,
    /// Keep important messages with summarization
    Summarization,
    /// Keep first and last messages
    HeadTail { head_count: usize, tail_count: usize },
    /// Custom priority-based retention
    Priority,
}

/// Context manager for multiple conversations
pub struct ContextManager {
    /// Contexts by conversation ID
    contexts: std::collections::HashMap<String, ContextWindow>,
    /// Default max tokens
    default_max_tokens: usize,
    /// Default strategy
    default_strategy: ContextStrategy,
}

impl ContextWindow {
    /// Create a new context window
    pub fn new(max_tokens: usize, strategy: ContextStrategy) -> Self {
        Self {
            max_tokens,
            messages: VecDeque::new(),
            estimated_tokens: 0,
            strategy,
        }
    }

    /// Add a message to the context
    pub fn add_message(&mut self, message: ChatMessage) -> Result<()> {
        let msg_tokens = Self::estimate_tokens(&message.content);

        // Add message
        self.messages.push_back(message);
        self.estimated_tokens += msg_tokens;

        // Apply retention strategy if over limit
        if self.estimated_tokens > self.max_tokens {
            self.apply_strategy()?;
        }

        Ok(())
    }

    /// Get all messages
    pub fn messages(&self) -> Vec<ChatMessage> {
        self.messages.iter().cloned().collect()
    }

    /// Get message count
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Get estimated token count
    pub fn token_count(&self) -> usize {
        self.estimated_tokens
    }

    /// Clear all messages
    pub fn clear(&mut self) {
        self.messages.clear();
        self.estimated_tokens = 0;
    }

    /// Get remaining tokens
    pub fn remaining_tokens(&self) -> usize {
        self.max_tokens.saturating_sub(self.estimated_tokens)
    }

    /// Check if context is full
    pub fn is_full(&self) -> bool {
        self.estimated_tokens >= self.max_tokens
    }

    /// Apply retention strategy
    fn apply_strategy(&mut self) -> Result<()> {
        match &self.strategy {
            ContextStrategy::SlidingWindow => {
                // Remove oldest messages until under limit
                while self.estimated_tokens > self.max_tokens && !self.messages.is_empty() {
                    if let Some(msg) = self.messages.pop_front() {
                        let tokens = Self::estimate_tokens(&msg.content);
                        self.estimated_tokens = self.estimated_tokens.saturating_sub(tokens);
                    }
                }
            }
            ContextStrategy::Summarization => {
                // Summarize older messages (simplified implementation)
                if self.messages.len() > 2 {
                    // Keep system message and recent message, summarize middle
                    let system_msg = self.messages.front().cloned();
                    let recent_msg = self.messages.back().cloned();

                    // Create summary message
                    let summary = ChatMessage::system(
                        "[Previous conversation summarized for context efficiency]"
                    );

                    self.messages.clear();
                    if let Some(sys) = system_msg {
                        self.messages.push_back(sys);
                    }
                    self.messages.push_back(summary);
                    if let Some(recent) = recent_msg {
                        self.messages.push_back(recent);
                    }

                    // Recalculate tokens
                    self.recalculate_tokens();
                }
            }
            ContextStrategy::HeadTail { head_count, tail_count } => {
                if self.messages.len() > head_count + tail_count {
                    let mut new_messages = VecDeque::new();

                    // Keep first N messages
                    for i in 0..*head_count {
                        if let Some(msg) = self.messages.get(i) {
                            new_messages.push_back(msg.clone());
                        }
                    }

                    // Add marker
                    new_messages.push_back(ChatMessage::system(
                        "[Middle messages truncated]"
                    ));

                    // Keep last N messages
                    let start = self.messages.len() - tail_count;
                    for i in start..self.messages.len() {
                        if let Some(msg) = self.messages.get(i) {
                            new_messages.push_back(msg.clone());
                        }
                    }

                    self.messages = new_messages;
                    self.recalculate_tokens();
                }
            }
            ContextStrategy::Priority => {
                // Keep system messages and recent messages
                let system_messages: Vec<_> = self
                    .messages
                    .iter()
                    .filter(|m| m.role == "system")
                    .cloned()
                    .collect();

                let recent_count = (self.max_tokens / 4) / 100; // Rough estimate
                let recent_messages: Vec<_> = self
                    .messages
                    .iter()
                    .rev()
                    .take(recent_count)
                    .cloned()
                    .collect();

                self.messages.clear();
                for msg in system_messages {
                    self.messages.push_back(msg);
                }
                for msg in recent_messages.into_iter().rev() {
                    self.messages.push_back(msg);
                }

                self.recalculate_tokens();
            }
        }

        Ok(())
    }

    /// Recalculate token count
    fn recalculate_tokens(&mut self) {
        self.estimated_tokens = self
            .messages
            .iter()
            .map(|m| Self::estimate_tokens(&m.content))
            .sum();
    }

    /// Estimate tokens in text (rough approximation)
    fn estimate_tokens(text: &str) -> usize {
        // Rough approximation: ~4 characters per token
        (text.len() + 3) / 4
    }

    /// Set max tokens
    pub fn set_max_tokens(&mut self, max_tokens: usize) {
        self.max_tokens = max_tokens;
        if self.estimated_tokens > max_tokens {
            let _ = self.apply_strategy();
        }
    }

    /// Set strategy
    pub fn set_strategy(&mut self, strategy: ContextStrategy) {
        self.strategy = strategy;
    }
}

impl ContextManager {
    /// Create a new context manager
    pub fn new(default_max_tokens: usize, default_strategy: ContextStrategy) -> Self {
        Self {
            contexts: std::collections::HashMap::new(),
            default_max_tokens,
            default_strategy,
        }
    }

    /// Get or create a context for a conversation
    pub fn get_or_create(&mut self, conversation_id: &str) -> &mut ContextWindow {
        self.contexts
            .entry(conversation_id.to_string())
            .or_insert_with(|| {
                ContextWindow::new(self.default_max_tokens, self.default_strategy.clone())
            })
    }

    /// Get a context (read-only)
    pub fn get(&self, conversation_id: &str) -> Option<&ContextWindow> {
        self.contexts.get(conversation_id)
    }

    /// Remove a context
    pub fn remove(&mut self, conversation_id: &str) -> Option<ContextWindow> {
        self.contexts.remove(conversation_id)
    }

    /// Clear all contexts
    pub fn clear_all(&mut self) {
        self.contexts.clear();
    }

    /// Get active conversation count
    pub fn conversation_count(&self) -> usize {
        self.contexts.len()
    }

    /// List all conversation IDs
    pub fn list_conversations(&self) -> Vec<String> {
        self.contexts.keys().cloned().collect()
    }
}

impl Default for ContextStrategy {
    fn default() -> Self {
        Self::SlidingWindow
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new(4096, ContextStrategy::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_window_creation() {
        let window = ContextWindow::new(1000, ContextStrategy::SlidingWindow);
        assert_eq!(window.max_tokens, 1000);
        assert_eq!(window.message_count(), 0);
    }

    #[test]
    fn test_add_message() {
        let mut window = ContextWindow::new(1000, ContextStrategy::SlidingWindow);
        window.add_message(ChatMessage::user("Hello")).unwrap();
        assert_eq!(window.message_count(), 1);
        assert!(window.token_count() > 0);
    }

    #[test]
    fn test_sliding_window_strategy() {
        let mut window = ContextWindow::new(100, ContextStrategy::SlidingWindow);

        // Add many messages
        for i in 0..10 {
            window.add_message(ChatMessage::user(format!("Message {}", i))).unwrap();
        }

        // Should have removed old messages
        assert!(window.token_count() <= 100);
    }

    #[test]
    fn test_clear() {
        let mut window = ContextWindow::new(1000, ContextStrategy::SlidingWindow);
        window.add_message(ChatMessage::user("Hello")).unwrap();
        window.clear();
        assert_eq!(window.message_count(), 0);
        assert_eq!(window.token_count(), 0);
    }

    #[test]
    fn test_context_manager() {
        let mut manager = ContextManager::default();

        let ctx1 = manager.get_or_create("conv1");
        ctx1.add_message(ChatMessage::user("Hello")).unwrap();

        let ctx2 = manager.get_or_create("conv2");
        ctx2.add_message(ChatMessage::user("Hi")).unwrap();

        assert_eq!(manager.conversation_count(), 2);
    }

    #[test]
    fn test_token_estimation() {
        let text = "Hello world";
        let tokens = ContextWindow::estimate_tokens(text);
        assert!(tokens > 0);
        assert!(tokens < text.len());
    }
}
