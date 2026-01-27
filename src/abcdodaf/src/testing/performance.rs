//! Performance testing and benchmarking tools

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Performance metrics for a task or workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Name of what was measured
    pub name: String,
    /// Execution times in milliseconds
    pub durations_ms: Vec<u64>,
    /// Memory usage in bytes (if tracked)
    pub memory_usage: Option<Vec<u64>>,
    /// CPU usage percentage (if tracked)
    pub cpu_usage: Option<Vec<f64>>,
    /// Timestamp when measurements were taken
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl PerformanceMetrics {
    /// Create new metrics
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            durations_ms: Vec::new(),
            memory_usage: None,
            cpu_usage: None,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Add duration measurement
    pub fn add_duration(&mut self, ms: u64) {
        self.durations_ms.push(ms);
    }

    /// Get average duration
    pub fn average_duration(&self) -> f64 {
        if self.durations_ms.is_empty() {
            return 0.0;
        }
        let sum: u64 = self.durations_ms.iter().sum();
        sum as f64 / self.durations_ms.len() as f64
    }

    /// Get min duration
    pub fn min_duration(&self) -> Option<u64> {
        self.durations_ms.iter().min().copied()
    }

    /// Get max duration
    pub fn max_duration(&self) -> Option<u64> {
        self.durations_ms.iter().max().copied()
    }

    /// Get median duration
    pub fn median_duration(&self) -> Option<f64> {
        if self.durations_ms.is_empty() {
            return None;
        }

        let mut sorted = self.durations_ms.clone();
        sorted.sort_unstable();

        let len = sorted.len();
        if len % 2 == 0 {
            Some((sorted[len / 2 - 1] as f64 + sorted[len / 2] as f64) / 2.0)
        } else {
            Some(sorted[len / 2] as f64)
        }
    }

    /// Get p95 (95th percentile)
    pub fn p95_duration(&self) -> Option<u64> {
        if self.durations_ms.is_empty() {
            return None;
        }

        let mut sorted = self.durations_ms.clone();
        sorted.sort_unstable();
        let idx = ((95 * sorted.len()) / 100).max(0);
        Some(sorted[idx.min(sorted.len() - 1)])
    }

    /// Get p99 (99th percentile)
    pub fn p99_duration(&self) -> Option<u64> {
        if self.durations_ms.is_empty() {
            return None;
        }

        let mut sorted = self.durations_ms.clone();
        sorted.sort_unstable();
        let idx = ((99 * sorted.len()) / 100).max(0);
        Some(sorted[idx.min(sorted.len() - 1)])
    }

    /// Get measurement count
    pub fn count(&self) -> usize {
        self.durations_ms.len()
    }
}

/// Benchmark result for a single test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Name of the benchmark
    pub name: String,
    /// Performance metrics
    pub metrics: PerformanceMetrics,
    /// Operations per second
    pub ops_per_second: f64,
    /// Whether it passed performance criteria
    pub passed_criteria: bool,
    /// Failure message if applicable
    pub failure_reason: Option<String>,
}

impl BenchmarkResult {
    /// Create new benchmark result
    pub fn new(name: impl Into<String>, metrics: PerformanceMetrics) -> Self {
        let avg = metrics.average_duration();
        let ops_per_second = if avg > 0.0 { 1000.0 / avg } else { 0.0 };

        Self {
            name: name.into(),
            metrics,
            ops_per_second,
            passed_criteria: true,
            failure_reason: None,
        }
    }

    /// Set whether criteria were met
    pub fn with_criteria(mut self, passed: bool, reason: Option<String>) -> Self {
        self.passed_criteria = passed;
        self.failure_reason = reason;
        self
    }
}

/// Benchmark suite for running multiple benchmarks
pub struct BenchmarkSuite {
    /// Name of the suite
    pub name: String,
    /// Benchmarks to run
    benchmarks: HashMap<String, BenchmarkDef>,
    /// Results
    results: Arc<RwLock<Vec<BenchmarkResult>>>,
}

/// Benchmark definition
#[derive(Clone)]
pub struct BenchmarkDef {
    /// Name
    pub name: String,
    /// Number of iterations
    pub iterations: usize,
    /// Warmup iterations
    pub warmup: usize,
    /// Max acceptable duration in milliseconds
    pub max_duration_ms: Option<u64>,
    /// Min acceptable throughput (ops/sec)
    pub min_throughput: Option<f64>,
}

impl BenchmarkDef {
    /// Create new benchmark definition
    pub fn new(name: impl Into<String>, iterations: usize) -> Self {
        Self {
            name: name.into(),
            iterations,
            warmup: 0,
            max_duration_ms: None,
            min_throughput: None,
        }
    }

    /// Set warmup iterations
    pub fn with_warmup(mut self, warmup: usize) -> Self {
        self.warmup = warmup;
        self
    }

    /// Set max acceptable duration
    pub fn with_max_duration(mut self, ms: u64) -> Self {
        self.max_duration_ms = Some(ms);
        self
    }

    /// Set min throughput requirement
    pub fn with_min_throughput(mut self, ops_per_sec: f64) -> Self {
        self.min_throughput = Some(ops_per_sec);
        self
    }
}

impl BenchmarkSuite {
    /// Create new benchmark suite
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            benchmarks: HashMap::new(),
            results: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add benchmark
    pub fn add_benchmark(mut self, def: BenchmarkDef) -> Self {
        self.benchmarks.insert(def.name.clone(), def);
        self
    }

    /// Run a synchronous benchmark
    pub async fn run_sync_benchmark<F>(
        &self,
        name: &str,
        iterations: usize,
        mut func: F,
    ) -> BenchmarkResult
    where
        F: FnMut(),
    {
        let mut metrics = PerformanceMetrics::new(name);

        // Warmup
        for _ in 0..10 {
            func();
        }

        // Benchmark
        for _ in 0..iterations {
            let start = Instant::now();
            func();
            metrics.add_duration(start.elapsed().as_millis() as u64);
        }

        BenchmarkResult::new(name, metrics)
    }

    /// Run an async benchmark
    pub async fn run_async_benchmark<F, Fut>(
        &self,
        name: &str,
        iterations: usize,
        mut func: F,
    ) -> BenchmarkResult
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        let mut metrics = PerformanceMetrics::new(name);

        // Warmup
        for _ in 0..10 {
            func().await;
        }

        // Benchmark
        for _ in 0..iterations {
            let start = Instant::now();
            func().await;
            metrics.add_duration(start.elapsed().as_millis() as u64);
        }

        BenchmarkResult::new(name, metrics)
    }

    /// Get all results
    pub async fn get_results(&self) -> Vec<BenchmarkResult> {
        self.results.read().await.clone()
    }

    /// Clear results
    pub async fn clear_results(&self) {
        self.results.write().await.clear();
    }

    /// Get summary
    pub async fn get_summary(&self) -> BenchmarkSummary {
        let results = self.results.read().await;

        let total_benchmarks = results.len();
        let passed = results.iter().filter(|r| r.passed_criteria).count();
        let failed = total_benchmarks - passed;

        let avg_ops_per_sec = if !results.is_empty() {
            results.iter().map(|r| r.ops_per_second).sum::<f64>() / results.len() as f64
        } else {
            0.0
        };

        BenchmarkSummary {
            total_benchmarks,
            passed_benchmarks: passed,
            failed_benchmarks: failed,
            average_ops_per_second: avg_ops_per_sec,
        }
    }
}

/// Summary of benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSummary {
    /// Total benchmarks run
    pub total_benchmarks: usize,
    /// Number that passed criteria
    pub passed_benchmarks: usize,
    /// Number that failed criteria
    pub failed_benchmarks: usize,
    /// Average operations per second across all benchmarks
    pub average_ops_per_second: f64,
}

/// Timer utility for performance tracking
pub struct PerfTimer {
    start: Instant,
    name: String,
}

impl PerfTimer {
    /// Start a performance timer
    pub fn start(name: impl Into<String>) -> Self {
        Self { start: Instant::now(), name: name.into() }
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    /// Get elapsed time in seconds
    pub fn elapsed_secs(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }

    /// Stop and return duration
    pub fn stop(self) -> u64 {
        self.elapsed_ms()
    }

    /// Stop and return duration with name
    pub fn stop_and_log(self) -> (String, u64) {
        let duration = self.elapsed_ms();
        (self.name, duration)
    }
}

/// Builder for creating performance test scenarios
pub struct PerfTestBuilder {
    suite: BenchmarkSuite,
}

impl PerfTestBuilder {
    /// Create new performance test builder
    pub fn new(name: impl Into<String>) -> Self {
        Self { suite: BenchmarkSuite::new(name) }
    }

    /// Add benchmark
    pub fn add_benchmark(mut self, def: BenchmarkDef) -> Self {
        self.suite = self.suite.add_benchmark(def);
        self
    }

    /// Build suite
    pub fn build(self) -> BenchmarkSuite {
        self.suite
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_metrics() {
        let mut metrics = PerformanceMetrics::new("test");
        metrics.add_duration(100);
        metrics.add_duration(200);
        metrics.add_duration(300);

        assert_eq!(metrics.count(), 3);
        assert_eq!(metrics.average_duration(), 200.0);
        assert_eq!(metrics.min_duration(), Some(100));
        assert_eq!(metrics.max_duration(), Some(300));
    }

    #[test]
    fn test_percentile_calculations() {
        let mut metrics = PerformanceMetrics::new("test");
        for i in 1..=100 {
            metrics.add_duration(i as u64);
        }

        let p95 = metrics.p95_duration();
        assert!(p95.is_some());
        let p99 = metrics.p99_duration();
        assert!(p99.is_some());
    }

    #[test]
    fn test_benchmark_result() {
        let mut metrics = PerformanceMetrics::new("bench");
        metrics.add_duration(100);
        metrics.add_duration(100);

        let result = BenchmarkResult::new("bench", metrics);
        assert_eq!(result.ops_per_second, 10.0);
    }

    #[tokio::test]
    async fn test_async_benchmark() {
        let suite = BenchmarkSuite::new("test_suite");

        let result = suite
            .run_async_benchmark("async_test", 10, || async {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            })
            .await;

        assert_eq!(result.metrics.count(), 10);
    }

    #[test]
    fn test_perf_timer() {
        let timer = PerfTimer::start("test");
        std::thread::sleep(std::time::Duration::from_millis(50));
        let elapsed = timer.elapsed_ms();
        assert!(elapsed >= 50);
    }
}
