//! Streaming response support
//!
//! Handles streaming LLM responses with progress tracking and cancellation.

use crate::error::Result;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use tokio::sync::mpsc;

/// Streaming response wrapper
pub struct StreamingResponse {
    receiver: mpsc::Receiver<Result<StreamChunk>>,
}

/// Stream chunk data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    /// Chunk content
    pub content: String,
    /// Cumulative content
    pub cumulative: String,
    /// Chunk index
    pub index: usize,
    /// Done flag
    pub done: bool,
    /// Metadata
    pub metadata: Option<serde_json::Value>,
}

/// Stream handler for processing chunks
pub trait StreamHandler: Send + Sync {
    /// Handle a stream chunk
    fn on_chunk(&mut self, chunk: &StreamChunk) -> Result<()>;

    /// Handle stream completion
    fn on_complete(&mut self, final_content: &str) -> Result<()>;

    /// Handle stream error
    fn on_error(&mut self, error: &str);
}

/// Collector that gathers all chunks
pub struct StreamCollector {
    chunks: Vec<StreamChunk>,
    final_content: String,
}

/// Progress tracker for streaming
pub struct ProgressTracker {
    total_chunks: usize,
    total_chars: usize,
    start_time: std::time::Instant,
}

impl StreamingResponse {
    /// Create a new streaming response
    pub fn new(receiver: mpsc::Receiver<Result<StreamChunk>>) -> Self {
        Self { receiver }
    }

    /// Get the next chunk
    pub async fn next(&mut self) -> Option<Result<StreamChunk>> {
        self.receiver.recv().await
    }

    /// Collect all chunks into a single string
    pub async fn collect(mut self) -> Result<String> {
        let mut content = String::new();

        while let Some(result) = self.next().await {
            let chunk = result?;
            content.push_str(&chunk.content);

            if chunk.done {
                break;
            }
        }

        Ok(content)
    }

    /// Process stream with a handler
    pub async fn process_with<H: StreamHandler>(mut self, handler: &mut H) -> Result<String> {
        let mut final_content = String::new();

        while let Some(result) = self.next().await {
            match result {
                Ok(chunk) => {
                    handler.on_chunk(&chunk)?;
                    final_content = chunk.cumulative.clone();

                    if chunk.done {
                        handler.on_complete(&final_content)?;
                        break;
                    }
                },
                Err(e) => {
                    handler.on_error(&e.to_string());
                    return Err(e);
                },
            }
        }

        Ok(final_content)
    }

    /// Convert to async stream
    pub fn into_stream(mut self) -> Pin<Box<dyn Stream<Item = Result<StreamChunk>> + Send>> {
        Box::pin(async_stream::stream! {
            while let Some(chunk) = self.next().await {
                yield chunk;
            }
        })
    }
}

impl StreamChunk {
    /// Create a new stream chunk
    pub fn new(content: impl Into<String>, cumulative: impl Into<String>, index: usize) -> Self {
        Self {
            content: content.into(),
            cumulative: cumulative.into(),
            index,
            done: false,
            metadata: None,
        }
    }

    /// Mark as done
    pub fn with_done(mut self, done: bool) -> Self {
        self.done = done;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

impl StreamCollector {
    /// Create a new stream collector
    pub fn new() -> Self {
        Self { chunks: vec![], final_content: String::new() }
    }

    /// Get collected chunks
    pub fn chunks(&self) -> &[StreamChunk] {
        &self.chunks
    }

    /// Get final content
    pub fn content(&self) -> &str {
        &self.final_content
    }

    /// Get chunk count
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }
}

impl StreamHandler for StreamCollector {
    fn on_chunk(&mut self, chunk: &StreamChunk) -> Result<()> {
        self.chunks.push(chunk.clone());
        Ok(())
    }

    fn on_complete(&mut self, final_content: &str) -> Result<()> {
        self.final_content = final_content.to_string();
        Ok(())
    }

    fn on_error(&mut self, error: &str) {
        tracing::error!("Stream error: {}", error);
    }
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new() -> Self {
        Self { total_chunks: 0, total_chars: 0, start_time: std::time::Instant::now() }
    }

    /// Get elapsed time in seconds
    pub fn elapsed_secs(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }

    /// Get characters per second
    pub fn chars_per_second(&self) -> f64 {
        let elapsed = self.elapsed_secs();
        if elapsed > 0.0 {
            self.total_chars as f64 / elapsed
        } else {
            0.0
        }
    }

    /// Get chunks per second
    pub fn chunks_per_second(&self) -> f64 {
        let elapsed = self.elapsed_secs();
        if elapsed > 0.0 {
            self.total_chunks as f64 / elapsed
        } else {
            0.0
        }
    }
}

impl StreamHandler for ProgressTracker {
    fn on_chunk(&mut self, chunk: &StreamChunk) -> Result<()> {
        self.total_chunks += 1;
        self.total_chars += chunk.content.len();
        Ok(())
    }

    fn on_complete(&mut self, _final_content: &str) -> Result<()> {
        tracing::info!(
            "Stream complete: {} chunks, {} chars, {:.2} chars/sec",
            self.total_chunks,
            self.total_chars,
            self.chars_per_second()
        );
        Ok(())
    }

    fn on_error(&mut self, error: &str) {
        tracing::error!("Stream error after {:.2}s: {}", self.elapsed_secs(), error);
    }
}

impl Default for StreamCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ProgressTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_chunk_creation() {
        let chunk = StreamChunk::new("Hello", "Hello", 0).with_done(false);
        assert_eq!(chunk.content, "Hello");
        assert_eq!(chunk.index, 0);
        assert!(!chunk.done);
    }

    #[tokio::test]
    async fn test_streaming_response() {
        let (tx, rx) = mpsc::channel(10);

        tokio::spawn(async move {
            tx.send(Ok(StreamChunk::new("Hello ", "Hello ", 0))).await.unwrap();
            tx.send(Ok(StreamChunk::new("world", "Hello world", 1).with_done(true)))
                .await
                .unwrap();
        });

        let response = StreamingResponse::new(rx);
        let content = response.collect().await.unwrap();
        assert_eq!(content, "Hello world");
    }

    #[test]
    fn test_stream_collector() {
        let mut collector = StreamCollector::new();
        let chunk = StreamChunk::new("Test", "Test", 0);

        collector.on_chunk(&chunk).unwrap();
        collector.on_complete("Test").unwrap();

        assert_eq!(collector.chunk_count(), 1);
        assert_eq!(collector.content(), "Test");
    }

    #[test]
    fn test_progress_tracker() {
        let mut tracker = ProgressTracker::new();
        let chunk = StreamChunk::new("Hello", "Hello", 0);

        tracker.on_chunk(&chunk).unwrap();

        assert_eq!(tracker.total_chunks, 1);
        assert_eq!(tracker.total_chars, 5);
    }
}
