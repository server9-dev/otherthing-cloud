//! Agent performance metrics and cost tracking
//!
//! Tracks agent performance, token usage, latency, and costs.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent performance tracker
pub struct PerformanceTracker {
    /// Metrics by agent ID
    metrics: HashMap<String, AgentMetrics>,
    /// Cost tracker
    cost_tracker: CostTracker,
}

/// Agent metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    /// Agent ID
    pub agent_id: String,
    /// Total requests
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Total tokens used
    pub total_tokens: u64,
    /// Average latency in ms
    pub avg_latency_ms: f64,
    /// Min latency in ms
    pub min_latency_ms: u64,
    /// Max latency in ms
    pub max_latency_ms: u64,
    /// Total execution time in ms
    pub total_execution_ms: u64,
    /// Average confidence score
    pub avg_confidence: f32,
    /// First request time
    pub first_request_at: Option<DateTime<Utc>>,
    /// Last request time
    pub last_request_at: Option<DateTime<Utc>>,
}

/// Cost tracking
pub struct CostTracker {
    /// Cost entries
    entries: Vec<CostEntry>,
    /// Total cost
    total_cost: f64,
}

/// Cost entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEntry {
    /// Entry ID
    pub id: String,
    /// Agent ID
    pub agent_id: String,
    /// Model used
    pub model: String,
    /// Tokens used
    pub tokens: u64,
    /// Cost in USD
    pub cost: f64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Request metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetrics {
    /// Request ID
    pub request_id: String,
    /// Agent ID
    pub agent_id: String,
    /// Latency in ms
    pub latency_ms: u64,
    /// Tokens used
    pub tokens: u64,
    /// Success flag
    pub success: bool,
    /// Confidence score
    pub confidence: f32,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Performance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    /// Total agents tracked
    pub total_agents: usize,
    /// Total requests
    pub total_requests: u64,
    /// Success rate
    pub success_rate: f64,
    /// Average latency
    pub avg_latency_ms: f64,
    /// Total tokens
    pub total_tokens: u64,
    /// Total cost
    pub total_cost: f64,
}

impl PerformanceTracker {
    /// Create a new performance tracker
    pub fn new() -> Self {
        Self { metrics: HashMap::new(), cost_tracker: CostTracker::new() }
    }

    /// Record a request
    pub fn record_request(&mut self, metrics: RequestMetrics) {
        let agent_metrics = self
            .metrics
            .entry(metrics.agent_id.clone())
            .or_insert_with(|| AgentMetrics::new(&metrics.agent_id));

        agent_metrics.record_request(&metrics);
    }

    /// Record cost
    pub fn record_cost(&mut self, agent_id: &str, model: &str, tokens: u64, cost_per_1k: f64) {
        let cost = (tokens as f64 / 1000.0) * cost_per_1k;
        self.cost_tracker.add_entry(CostEntry {
            id: uuid::Uuid::new_v4().to_string(),
            agent_id: agent_id.to_string(),
            model: model.to_string(),
            tokens,
            cost,
            timestamp: Utc::now(),
        });
    }

    /// Get metrics for an agent
    pub fn get_metrics(&self, agent_id: &str) -> Option<&AgentMetrics> {
        self.metrics.get(agent_id)
    }

    /// Get all metrics
    pub fn all_metrics(&self) -> Vec<&AgentMetrics> {
        self.metrics.values().collect()
    }

    /// Get performance summary
    pub fn summary(&self) -> PerformanceSummary {
        let total_requests: u64 = self.metrics.values().map(|m| m.total_requests).sum();
        let successful_requests: u64 = self.metrics.values().map(|m| m.successful_requests).sum();

        let success_rate = if total_requests > 0 {
            successful_requests as f64 / total_requests as f64
        } else {
            0.0
        };

        let avg_latency = if !self.metrics.is_empty() {
            self.metrics.values().map(|m| m.avg_latency_ms).sum::<f64>() / self.metrics.len() as f64
        } else {
            0.0
        };

        let total_tokens: u64 = self.metrics.values().map(|m| m.total_tokens).sum();

        PerformanceSummary {
            total_agents: self.metrics.len(),
            total_requests,
            success_rate,
            avg_latency_ms: avg_latency,
            total_tokens,
            total_cost: self.cost_tracker.total_cost(),
        }
    }

    /// Reset all metrics
    pub fn reset(&mut self) {
        self.metrics.clear();
        self.cost_tracker = CostTracker::new();
    }

    /// Get cost tracker
    pub fn cost_tracker(&self) -> &CostTracker {
        &self.cost_tracker
    }
}

impl AgentMetrics {
    /// Create new agent metrics
    pub fn new(agent_id: &str) -> Self {
        Self {
            agent_id: agent_id.to_string(),
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            total_tokens: 0,
            avg_latency_ms: 0.0,
            min_latency_ms: u64::MAX,
            max_latency_ms: 0,
            total_execution_ms: 0,
            avg_confidence: 0.0,
            first_request_at: None,
            last_request_at: None,
        }
    }

    /// Record a request
    pub fn record_request(&mut self, metrics: &RequestMetrics) {
        self.total_requests += 1;

        if metrics.success {
            self.successful_requests += 1;
        } else {
            self.failed_requests += 1;
        }

        self.total_tokens += metrics.tokens;
        self.total_execution_ms += metrics.latency_ms;

        self.min_latency_ms = self.min_latency_ms.min(metrics.latency_ms);
        self.max_latency_ms = self.max_latency_ms.max(metrics.latency_ms);

        self.avg_latency_ms = self.total_execution_ms as f64 / self.total_requests as f64;

        // Update average confidence
        self.avg_confidence = (self.avg_confidence * (self.total_requests - 1) as f32
            + metrics.confidence)
            / self.total_requests as f32;

        if self.first_request_at.is_none() {
            self.first_request_at = Some(metrics.timestamp);
        }
        self.last_request_at = Some(metrics.timestamp);
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_requests > 0 {
            self.successful_requests as f64 / self.total_requests as f64
        } else {
            0.0
        }
    }

    /// Get requests per second
    pub fn requests_per_second(&self) -> f64 {
        if let (Some(first), Some(last)) = (self.first_request_at, self.last_request_at) {
            let duration_secs = (last - first).num_seconds() as f64;
            if duration_secs > 0.0 {
                return self.total_requests as f64 / duration_secs;
            }
        }
        0.0
    }
}

impl CostTracker {
    /// Create a new cost tracker
    pub fn new() -> Self {
        Self { entries: vec![], total_cost: 0.0 }
    }

    /// Add a cost entry
    pub fn add_entry(&mut self, entry: CostEntry) {
        self.total_cost += entry.cost;
        self.entries.push(entry);
    }

    /// Get total cost
    pub fn total_cost(&self) -> f64 {
        self.total_cost
    }

    /// Get cost by agent
    pub fn cost_by_agent(&self, agent_id: &str) -> f64 {
        self.entries.iter().filter(|e| e.agent_id == agent_id).map(|e| e.cost).sum()
    }

    /// Get cost by model
    pub fn cost_by_model(&self, model: &str) -> f64 {
        self.entries.iter().filter(|e| e.model == model).map(|e| e.cost).sum()
    }

    /// Get all entries
    pub fn entries(&self) -> &[CostEntry] {
        &self.entries
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
        self.total_cost = 0.0;
    }
}

impl Default for PerformanceTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CostTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_tracker() {
        let mut tracker = PerformanceTracker::new();

        tracker.record_request(RequestMetrics {
            request_id: "req1".to_string(),
            agent_id: "agent1".to_string(),
            latency_ms: 100,
            tokens: 150,
            success: true,
            confidence: 0.9,
            timestamp: Utc::now(),
        });

        let metrics = tracker.get_metrics("agent1").unwrap();
        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.successful_requests, 1);
    }

    #[test]
    fn test_cost_tracking() {
        let mut tracker = PerformanceTracker::new();
        tracker.record_cost("agent1", "gpt-4", 1000, 0.03);

        assert!(tracker.cost_tracker().total_cost() > 0.0);
    }

    #[test]
    fn test_success_rate() {
        let mut metrics = AgentMetrics::new("test");

        metrics.record_request(&RequestMetrics {
            request_id: "1".to_string(),
            agent_id: "test".to_string(),
            latency_ms: 100,
            tokens: 100,
            success: true,
            confidence: 0.9,
            timestamp: Utc::now(),
        });

        metrics.record_request(&RequestMetrics {
            request_id: "2".to_string(),
            agent_id: "test".to_string(),
            latency_ms: 100,
            tokens: 100,
            success: false,
            confidence: 0.5,
            timestamp: Utc::now(),
        });

        assert_eq!(metrics.success_rate(), 0.5);
    }

    #[test]
    fn test_summary() {
        let mut tracker = PerformanceTracker::new();

        tracker.record_request(RequestMetrics {
            request_id: "1".to_string(),
            agent_id: "agent1".to_string(),
            latency_ms: 100,
            tokens: 150,
            success: true,
            confidence: 0.9,
            timestamp: Utc::now(),
        });

        let summary = tracker.summary();
        assert_eq!(summary.total_requests, 1);
        assert_eq!(summary.success_rate, 1.0);
    }
}
