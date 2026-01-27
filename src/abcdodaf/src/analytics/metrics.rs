//! Real-time metrics collection system
//!
//! Collects and maintains process metrics including:
//! - Process completion metrics
//! - Performance metrics
//! - Cost metrics
//! - Resource metrics

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

/// Main metrics collector for process analytics
#[derive(Debug, Clone)]
pub struct MetricsCollector {
    /// Process metrics storage
    process_metrics: Arc<Mutex<HashMap<String, ProcessMetrics>>>,
    /// Performance metrics storage
    performance_metrics: Arc<Mutex<VecDeque<MetricPoint>>>,
    /// Cost metrics storage
    cost_metrics: Arc<Mutex<HashMap<String, CostMetrics>>>,
    /// Resource metrics storage
    resource_metrics: Arc<Mutex<VecDeque<ResourceMetricPoint>>>,
    /// Configuration
    config: MetricsConfig,
}

/// Configuration for metrics collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Maximum number of historical data points to retain
    pub max_history_points: usize,
    /// Interval for metric aggregation
    pub aggregation_interval_secs: u64,
    /// Enable detailed metric collection
    pub detailed_collection: bool,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self { max_history_points: 10000, aggregation_interval_secs: 60, detailed_collection: true }
    }
}

/// Process-level metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessMetrics {
    /// Process ID
    pub process_id: String,
    /// Total instances executed
    pub total_instances: u64,
    /// Completed instances
    pub completed_instances: u64,
    /// Failed instances
    pub failed_instances: u64,
    /// Completion rate (%)
    pub completion_rate: f64,
    /// Average execution time (milliseconds)
    pub avg_execution_time_ms: f64,
    /// Min execution time (milliseconds)
    pub min_execution_time_ms: f64,
    /// Max execution time (milliseconds)
    pub max_execution_time_ms: f64,
    /// 95th percentile execution time
    pub p95_execution_time_ms: f64,
    /// Last execution time (milliseconds)
    pub last_execution_time_ms: f64,
    /// Latest update timestamp
    pub updated_at: DateTime<Utc>,
}

impl ProcessMetrics {
    /// Create new process metrics
    pub fn new(process_id: impl Into<String>) -> Self {
        Self {
            process_id: process_id.into(),
            total_instances: 0,
            completed_instances: 0,
            failed_instances: 0,
            completion_rate: 0.0,
            avg_execution_time_ms: 0.0,
            min_execution_time_ms: f64::MAX,
            max_execution_time_ms: 0.0,
            p95_execution_time_ms: 0.0,
            last_execution_time_ms: 0.0,
            updated_at: Utc::now(),
        }
    }

    /// Record a completed execution
    pub fn record_completion(&mut self, duration_ms: f64) {
        self.total_instances += 1;
        self.completed_instances += 1;
        self.last_execution_time_ms = duration_ms;
        self.min_execution_time_ms = self.min_execution_time_ms.min(duration_ms);
        self.max_execution_time_ms = self.max_execution_time_ms.max(duration_ms);

        // Update running average
        let n = self.completed_instances as f64;
        self.avg_execution_time_ms = (self.avg_execution_time_ms * (n - 1.0) + duration_ms) / n;

        self.completion_rate =
            (self.completed_instances as f64 / self.total_instances as f64) * 100.0;
        self.updated_at = Utc::now();
    }

    /// Record a failed execution
    pub fn record_failure(&mut self) {
        self.total_instances += 1;
        self.failed_instances += 1;
        self.completion_rate =
            (self.completed_instances as f64 / self.total_instances as f64) * 100.0;
        self.updated_at = Utc::now();
    }
}

/// Performance metrics for real-time monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Average response time (ms)
    pub avg_response_time_ms: f64,
    /// P95 response time (ms)
    pub p95_response_time_ms: f64,
    /// P99 response time (ms)
    pub p99_response_time_ms: f64,
    /// Throughput (processes per second)
    pub throughput_pps: f64,
    /// Error rate (%)
    pub error_rate: f64,
    /// CPU utilization (%)
    pub cpu_utilization: f64,
    /// Memory utilization (%)
    pub memory_utilization: f64,
    /// Active process count
    pub active_processes: u64,
    /// Queued process count
    pub queued_processes: u64,
}

impl PerformanceMetrics {
    /// Create default performance metrics
    pub fn default() -> Self {
        Self {
            avg_response_time_ms: 0.0,
            p95_response_time_ms: 0.0,
            p99_response_time_ms: 0.0,
            throughput_pps: 0.0,
            error_rate: 0.0,
            cpu_utilization: 0.0,
            memory_utilization: 0.0,
            active_processes: 0,
            queued_processes: 0,
        }
    }
}

/// Single metric data point with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    /// Timestamp of measurement
    pub timestamp: DateTime<Utc>,
    /// Metric name
    pub metric_name: String,
    /// Metric value
    pub value: f64,
    /// Associated tags
    pub tags: HashMap<String, String>,
}

impl MetricPoint {
    /// Create a new metric point
    pub fn new(metric_name: impl Into<String>, value: f64) -> Self {
        Self { timestamp: Utc::now(), metric_name: metric_name.into(), value, tags: HashMap::new() }
    }

    /// Add a tag to metric point
    pub fn with_tag(mut self, key: String, value: String) -> Self {
        self.tags.insert(key, value);
        self
    }
}

/// Cost analysis metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostMetrics {
    /// Resource ID
    pub resource_id: String,
    /// Total cost incurred
    pub total_cost: f64,
    /// Cost per execution
    pub cost_per_execution: f64,
    /// Cumulative cost
    pub cumulative_cost: f64,
    /// Currency (e.g., USD)
    pub currency: String,
    /// Cost breakdown by category
    pub cost_breakdown: HashMap<String, f64>,
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

impl CostMetrics {
    /// Create new cost metrics
    pub fn new(resource_id: impl Into<String>, currency: impl Into<String>) -> Self {
        Self {
            resource_id: resource_id.into(),
            total_cost: 0.0,
            cost_per_execution: 0.0,
            cumulative_cost: 0.0,
            currency: currency.into(),
            cost_breakdown: HashMap::new(),
            updated_at: Utc::now(),
        }
    }

    /// Record a cost entry
    pub fn record_cost(&mut self, category: String, amount: f64) {
        *self.cost_breakdown.entry(category).or_insert(0.0) += amount;
        self.total_cost += amount;
        self.cumulative_cost += amount;
        self.updated_at = Utc::now();
    }
}

/// Resource utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetricPoint {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// CPU utilization percentage
    pub cpu_percent: f64,
    /// Memory utilization percentage
    pub memory_percent: f64,
    /// Disk I/O percentage
    pub disk_io_percent: f64,
    /// Network utilization percentage
    pub network_percent: f64,
    /// Active connections
    pub active_connections: u64,
    /// Queue depth
    pub queue_depth: u64,
}

impl ResourceMetricPoint {
    /// Create new resource metric point
    pub fn new() -> Self {
        Self {
            timestamp: Utc::now(),
            cpu_percent: 0.0,
            memory_percent: 0.0,
            disk_io_percent: 0.0,
            network_percent: 0.0,
            active_connections: 0,
            queue_depth: 0,
        }
    }
}

impl Default for ResourceMetricPoint {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of all collected metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSnapshot {
    /// Timestamp of snapshot
    pub timestamp: DateTime<Utc>,
    /// Process metrics
    pub process_metrics: HashMap<String, ProcessMetrics>,
    /// Overall performance metrics
    pub performance_metrics: PerformanceMetrics,
    /// Cost metrics
    pub cost_metrics: HashMap<String, CostMetrics>,
    /// Recent metric points
    pub recent_metrics: Vec<MetricPoint>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self::with_config(MetricsConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: MetricsConfig) -> Self {
        Self {
            process_metrics: Arc::new(Mutex::new(HashMap::new())),
            performance_metrics: Arc::new(Mutex::new(VecDeque::new())),
            cost_metrics: Arc::new(Mutex::new(HashMap::new())),
            resource_metrics: Arc::new(Mutex::new(VecDeque::new())),
            config,
        }
    }

    /// Record a process execution
    pub fn record_process_execution(
        &self,
        process_id: impl Into<String>,
        duration_ms: f64,
        success: bool,
    ) {
        let process_id = process_id.into();
        let mut metrics = self.process_metrics.lock().unwrap();
        let entry = metrics
            .entry(process_id.clone())
            .or_insert_with(|| ProcessMetrics::new(process_id));

        if success {
            entry.record_completion(duration_ms);
        } else {
            entry.record_failure();
        }
    }

    /// Record a metric point
    pub fn record_metric(&self, point: MetricPoint) {
        let mut metrics = self.performance_metrics.lock().unwrap();
        metrics.push_back(point);

        // Trim to max history
        while metrics.len() > self.config.max_history_points {
            metrics.pop_front();
        }
    }

    /// Record cost
    pub fn record_cost(
        &self,
        resource_id: impl Into<String>,
        category: String,
        amount: f64,
        currency: impl Into<String>,
    ) {
        let resource_id = resource_id.into();
        let mut costs = self.cost_metrics.lock().unwrap();
        let entry = costs
            .entry(resource_id.clone())
            .or_insert_with(|| CostMetrics::new(resource_id, currency));

        entry.record_cost(category, amount);
    }

    /// Record resource metrics
    pub fn record_resource_metrics(&self, point: ResourceMetricPoint) {
        let mut metrics = self.resource_metrics.lock().unwrap();
        metrics.push_back(point);

        while metrics.len() > self.config.max_history_points {
            metrics.pop_front();
        }
    }

    /// Get process metrics
    pub fn get_process_metrics(&self, process_id: &str) -> Option<ProcessMetrics> {
        self.process_metrics.lock().unwrap().get(process_id).cloned()
    }

    /// Get all process metrics
    pub fn all_process_metrics(&self) -> HashMap<String, ProcessMetrics> {
        self.process_metrics.lock().unwrap().clone()
    }

    /// Get recent metric points
    pub fn recent_metrics(&self, limit: usize) -> Vec<MetricPoint> {
        let metrics = self.performance_metrics.lock().unwrap();
        metrics
            .iter()
            .rev()
            .take(limit)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Get resource metrics
    pub fn resource_metrics(&self) -> Vec<ResourceMetricPoint> {
        self.resource_metrics.lock().unwrap().iter().cloned().collect()
    }

    /// Get current snapshot
    pub fn current_snapshot(&self) -> MetricSnapshot {
        MetricSnapshot {
            timestamp: Utc::now(),
            process_metrics: self.all_process_metrics(),
            performance_metrics: PerformanceMetrics::default(),
            cost_metrics: self.cost_metrics.lock().unwrap().clone(),
            recent_metrics: self.recent_metrics(100),
        }
    }

    /// Clear old metrics (older than duration)
    pub fn clear_old_metrics(&self, older_than: Duration) {
        let cutoff = Utc::now() - older_than;

        let mut perf = self.performance_metrics.lock().unwrap();
        perf.retain(|p| p.timestamp > cutoff);

        let mut res = self.resource_metrics.lock().unwrap();
        res.retain(|r| r.timestamp > cutoff);
    }

    /// Get metrics configuration
    pub fn config(&self) -> &MetricsConfig {
        &self.config
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_metrics_recording() {
        let mut metrics = ProcessMetrics::new("test_process");
        metrics.record_completion(100.0);
        metrics.record_completion(200.0);

        assert_eq!(metrics.total_instances, 2);
        assert_eq!(metrics.completed_instances, 2);
        assert_eq!(metrics.completion_rate, 100.0);
        assert_eq!(metrics.avg_execution_time_ms, 150.0);
        assert_eq!(metrics.min_execution_time_ms, 100.0);
        assert_eq!(metrics.max_execution_time_ms, 200.0);
    }

    #[test]
    fn test_process_metrics_failure() {
        let mut metrics = ProcessMetrics::new("test_process");
        metrics.record_completion(100.0);
        metrics.record_failure();

        assert_eq!(metrics.total_instances, 2);
        assert_eq!(metrics.completed_instances, 1);
        assert_eq!(metrics.failed_instances, 1);
        assert_eq!(metrics.completion_rate, 50.0);
    }

    #[test]
    fn test_cost_metrics() {
        let mut cost = CostMetrics::new("resource_1", "USD");
        cost.record_cost("compute".to_string(), 10.0);
        cost.record_cost("storage".to_string(), 5.0);
        cost.record_cost("compute".to_string(), 15.0);

        assert_eq!(cost.total_cost, 30.0);
        assert_eq!(cost.cost_breakdown.get("compute").unwrap(), &25.0);
        assert_eq!(cost.cost_breakdown.get("storage").unwrap(), &5.0);
    }

    #[test]
    fn test_metrics_collector() {
        let collector = MetricsCollector::new();
        collector.record_process_execution("proc1", 100.0, true);
        collector.record_process_execution("proc1", 200.0, true);
        collector.record_process_execution("proc1", 150.0, false);

        let metrics = collector.get_process_metrics("proc1").unwrap();
        assert_eq!(metrics.total_instances, 3);
        assert_eq!(metrics.completed_instances, 2);
        assert_eq!(metrics.failed_instances, 1);
    }

    #[test]
    fn test_metric_point_with_tags() {
        let point = MetricPoint::new("response_time", 125.5)
            .with_tag("endpoint".to_string(), "/api/users".to_string())
            .with_tag("method".to_string(), "GET".to_string());

        assert_eq!(point.value, 125.5);
        assert_eq!(point.tags.get("endpoint").unwrap(), "/api/users");
    }
}
