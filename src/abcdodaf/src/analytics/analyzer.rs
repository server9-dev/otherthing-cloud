//! Analytics and analysis algorithms
//!
//! Provides:
//! - Bottleneck detection
//! - Trend analysis
//! - Performance analysis
//! - Cost analysis
//! - Heatmap generation

use super::metrics::{MetricPoint, MetricSnapshot, ProcessMetrics};
use super::PerformanceSummary;
use chrono::{DateTime, Datelike, Duration, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main analytics engine
#[derive(Debug, Clone)]
pub struct AnalyticsEngine {
    bottleneck_analyzer: BottleneckAnalyzer,
    #[allow(dead_code)]
    trend_analyzer: TrendAnalyzer,
    #[allow(dead_code)]
    heatmap_analyzer: HeatmapAnalyzer,
    #[allow(dead_code)]
    performance_analyzer: PerformanceAnalyzer,
    #[allow(dead_code)]
    cost_analyzer: CostAnalyzer,
}

impl AnalyticsEngine {
    /// Create a new analytics engine
    pub fn new() -> Self {
        Self {
            bottleneck_analyzer: BottleneckAnalyzer::new(),
            trend_analyzer: TrendAnalyzer::new(),
            heatmap_analyzer: HeatmapAnalyzer::new(),
            performance_analyzer: PerformanceAnalyzer::new(),
            cost_analyzer: CostAnalyzer::new(),
        }
    }

    /// Analyze metrics and produce summary
    pub fn analyze(&self, snapshot: &MetricSnapshot) -> PerformanceSummary {
        let completion_rate = self.calculate_completion_rate(snapshot);
        let throughput = self.calculate_throughput(snapshot);
        let health_score = self.calculate_health_score(snapshot);

        PerformanceSummary {
            health_score,
            avg_completion_time_ms: self.calculate_avg_completion_time(snapshot),
            completion_rate,
            throughput,
            resource_utilization: self.calculate_resource_utilization(snapshot),
            active_bottlenecks: self.bottleneck_analyzer.detect_bottlenecks(snapshot).len(),
            sla_compliance_rate: 100.0, // Will be updated with SLA data
        }
    }

    /// Get performance summary
    pub fn performance_summary(&self) -> PerformanceSummary {
        PerformanceSummary {
            health_score: 100.0,
            avg_completion_time_ms: 0.0,
            completion_rate: 0.0,
            throughput: 0.0,
            resource_utilization: 0.0,
            active_bottlenecks: 0,
            sla_compliance_rate: 100.0,
        }
    }

    fn calculate_completion_rate(&self, snapshot: &MetricSnapshot) -> f64 {
        if snapshot.process_metrics.is_empty() {
            return 0.0;
        }

        let total_completed: u64 =
            snapshot.process_metrics.values().map(|m| m.completed_instances).sum();
        let total: u64 = snapshot.process_metrics.values().map(|m| m.total_instances).sum();

        if total == 0 {
            0.0
        } else {
            (total_completed as f64 / total as f64) * 100.0
        }
    }

    fn calculate_throughput(&self, snapshot: &MetricSnapshot) -> f64 {
        // Calculate processes per minute based on recent metrics
        let now = Utc::now();
        let one_min_ago = now - Duration::minutes(1);

        let count = snapshot.recent_metrics.iter().filter(|m| m.timestamp > one_min_ago).count();

        count as f64
    }

    fn calculate_avg_completion_time(&self, snapshot: &MetricSnapshot) -> f64 {
        if snapshot.process_metrics.is_empty() {
            return 0.0;
        }

        let sum: f64 = snapshot
            .process_metrics
            .values()
            .map(|m| m.avg_execution_time_ms * m.completed_instances as f64)
            .sum();

        let total: u64 = snapshot.process_metrics.values().map(|m| m.completed_instances).sum();

        if total == 0 {
            0.0
        } else {
            sum / total as f64
        }
    }

    fn calculate_health_score(&self, snapshot: &MetricSnapshot) -> f64 {
        let completion_rate = self.calculate_completion_rate(snapshot);
        let resource_util = self.calculate_resource_utilization(snapshot);

        // Health score based on completion rate and resource efficiency
        let efficiency = if resource_util > 100.0 { 0.0 } else { 100.0 - resource_util };

        (completion_rate * 0.7 + efficiency * 0.3).min(100.0).max(0.0)
    }

    fn calculate_resource_utilization(&self, _snapshot: &MetricSnapshot) -> f64 {
        // Will be populated with actual resource metrics
        50.0
    }
}

impl Default for AnalyticsEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Detects bottlenecks in process execution
#[derive(Debug, Clone)]
pub struct BottleneckAnalyzer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bottleneck {
    /// Process ID
    pub process_id: String,
    /// Bottleneck severity (0-100)
    pub severity: f64,
    /// Reason for bottleneck
    pub reason: String,
    /// Detected timestamp
    pub detected_at: DateTime<Utc>,
    /// Recommended action
    pub recommendation: String,
}

impl BottleneckAnalyzer {
    /// Create new bottleneck analyzer
    pub fn new() -> Self {
        Self
    }

    /// Detect bottlenecks in process execution
    pub fn detect_bottlenecks(&self, snapshot: &MetricSnapshot) -> Vec<Bottleneck> {
        let mut bottlenecks = Vec::new();

        for (process_id, metrics) in &snapshot.process_metrics {
            // High execution time bottleneck
            if metrics.avg_execution_time_ms > 5000.0 {
                bottlenecks.push(Bottleneck {
                    process_id: process_id.clone(),
                    severity: (metrics.avg_execution_time_ms / 10000.0 * 100.0).min(100.0),
                    reason: format!("High execution time: {:.0}ms", metrics.avg_execution_time_ms),
                    detected_at: Utc::now(),
                    recommendation: "Optimize process logic or increase parallelization"
                        .to_string(),
                });
            }

            // High failure rate bottleneck
            let failure_rate = if metrics.total_instances > 0 {
                (metrics.failed_instances as f64 / metrics.total_instances as f64) * 100.0
            } else {
                0.0
            };

            if failure_rate > 10.0 {
                bottlenecks.push(Bottleneck {
                    process_id: process_id.clone(),
                    severity: failure_rate.min(100.0),
                    reason: format!("High failure rate: {:.1}%", failure_rate),
                    detected_at: Utc::now(),
                    recommendation: "Investigate root causes of failures".to_string(),
                });
            }

            // Low completion rate bottleneck
            if metrics.completion_rate < 80.0 && metrics.total_instances > 10 {
                bottlenecks.push(Bottleneck {
                    process_id: process_id.clone(),
                    severity: (100.0 - metrics.completion_rate),
                    reason: format!("Low completion rate: {:.1}%", metrics.completion_rate),
                    detected_at: Utc::now(),
                    recommendation: "Review process requirements and dependencies".to_string(),
                });
            }
        }

        bottlenecks
    }

    /// Analyze execution time trend
    pub fn analyze_execution_time_trend(&self, metrics: &ProcessMetrics) -> ExecutionTimeTrend {
        let trend = if metrics.last_execution_time_ms > metrics.avg_execution_time_ms * 1.2 {
            TrendDirection::Degrading
        } else if metrics.last_execution_time_ms < metrics.avg_execution_time_ms * 0.8 {
            TrendDirection::Improving
        } else {
            TrendDirection::Stable
        };

        ExecutionTimeTrend {
            direction: trend,
            current: metrics.last_execution_time_ms,
            average: metrics.avg_execution_time_ms,
            variance: metrics.max_execution_time_ms - metrics.min_execution_time_ms,
        }
    }
}

impl Default for BottleneckAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Stable,
    Degrading,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTimeTrend {
    pub direction: TrendDirection,
    pub current: f64,
    pub average: f64,
    pub variance: f64,
}

/// Analyzes trends over time
#[derive(Debug, Clone)]
pub struct TrendAnalyzer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trend {
    /// Metric name
    pub metric_name: String,
    /// Start value
    pub start_value: f64,
    /// End value
    pub end_value: f64,
    /// Change percentage
    pub change_percent: f64,
    /// Direction
    pub direction: TrendDirection,
    /// Period (days)
    pub period_days: i32,
}

impl TrendAnalyzer {
    /// Create new trend analyzer
    pub fn new() -> Self {
        Self
    }

    /// Analyze trends in metric points
    pub fn analyze_trends(&self, points: &[MetricPoint], period: Duration) -> Vec<Trend> {
        if points.len() < 2 {
            return Vec::new();
        }

        let now = Utc::now();
        let start_time = now - period;

        // Group points by metric name
        let mut metrics_by_name: HashMap<String, Vec<&MetricPoint>> = HashMap::new();
        for point in points {
            if point.timestamp >= start_time {
                metrics_by_name
                    .entry(point.metric_name.clone())
                    .or_insert_with(Vec::new)
                    .push(point);
            }
        }

        // Calculate trends
        let mut trends = Vec::new();
        for (metric_name, metric_points) in metrics_by_name {
            if metric_points.len() >= 2 {
                let start_value = metric_points.first().map(|p| p.value).unwrap_or(0.0);
                let end_value = metric_points.last().map(|p| p.value).unwrap_or(0.0);

                let change_percent = if start_value != 0.0 {
                    ((end_value - start_value) / start_value) * 100.0
                } else {
                    0.0
                };

                let direction = if change_percent > 5.0 {
                    TrendDirection::Improving
                } else if change_percent < -5.0 {
                    TrendDirection::Degrading
                } else {
                    TrendDirection::Stable
                };

                trends.push(Trend {
                    metric_name,
                    start_value,
                    end_value,
                    change_percent,
                    direction,
                    period_days: period.num_days() as i32,
                });
            }
        }

        trends
    }
}

impl Default for TrendAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Generates heatmaps for activity frequency
#[derive(Debug, Clone)]
pub struct HeatmapAnalyzer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatmapCell {
    /// X coordinate (hour of day)
    pub x: u32,
    /// Y coordinate (day of week)
    pub y: u32,
    /// Intensity value (0-100)
    pub intensity: f64,
    /// Count of activities
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityHeatmap {
    /// Heatmap cells
    pub cells: Vec<HeatmapCell>,
    /// Max activity count
    pub max_count: u64,
    /// Min activity count
    pub min_count: u64,
    /// Period covered
    pub period: String,
}

impl HeatmapAnalyzer {
    /// Create new heatmap analyzer
    pub fn new() -> Self {
        Self
    }

    /// Generate activity heatmap by hour and day
    pub fn generate_heatmap(&self, points: &[MetricPoint]) -> ActivityHeatmap {
        let mut heatmap: HashMap<(u32, u32), u64> = HashMap::new();

        for point in points {
            let hour = point.timestamp.hour();
            let weekday = point.timestamp.weekday() as u32;
            *heatmap.entry((hour, weekday)).or_insert(0) += 1;
        }

        let max_count = heatmap.values().max().cloned().unwrap_or(0);
        let min_count = heatmap.values().min().cloned().unwrap_or(0);

        let cells = heatmap
            .into_iter()
            .map(|((x, y), count)| HeatmapCell {
                x,
                y,
                intensity: if max_count > 0 {
                    (count as f64 / max_count as f64) * 100.0
                } else {
                    0.0
                },
                count,
            })
            .collect();

        ActivityHeatmap { cells, max_count, min_count, period: "Weekly".to_string() }
    }
}

impl Default for HeatmapAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Analyzes performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceAnalyzer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysis {
    /// Response time percentiles
    pub response_time_percentiles: ResponseTimePercentiles,
    /// Throughput statistics
    pub throughput_stats: ThroughputStats,
    /// Error analysis
    pub error_analysis: ErrorAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTimePercentiles {
    pub p50: f64,
    pub p75: f64,
    pub p90: f64,
    pub p95: f64,
    pub p99: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputStats {
    pub mean: f64,
    pub stddev: f64,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorAnalysis {
    pub total_errors: u64,
    pub error_rate: f64,
    pub top_errors: HashMap<String, u64>,
}

impl PerformanceAnalyzer {
    /// Create new performance analyzer
    pub fn new() -> Self {
        Self
    }

    /// Analyze performance metrics
    pub fn analyze(&self, snapshot: &MetricSnapshot) -> PerformanceAnalysis {
        let response_times: Vec<f64> =
            snapshot.process_metrics.values().map(|m| m.avg_execution_time_ms).collect();

        let response_time_percentiles = self.calculate_percentiles(&response_times);

        let throughput_stats = self.calculate_throughput_stats(snapshot);

        let error_analysis = self.calculate_error_analysis(snapshot);

        PerformanceAnalysis { response_time_percentiles, throughput_stats, error_analysis }
    }

    fn calculate_percentiles(&self, values: &[f64]) -> ResponseTimePercentiles {
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let len = sorted.len();
        let get_percentile = |p: f64| -> f64 {
            if len == 0 {
                0.0
            } else {
                let idx = ((p / 100.0) * (len - 1) as f64).round() as usize;
                sorted[idx]
            }
        };

        ResponseTimePercentiles {
            p50: get_percentile(50.0),
            p75: get_percentile(75.0),
            p90: get_percentile(90.0),
            p95: get_percentile(95.0),
            p99: get_percentile(99.0),
        }
    }

    fn calculate_throughput_stats(&self, snapshot: &MetricSnapshot) -> ThroughputStats {
        let completion_times: Vec<f64> = snapshot
            .process_metrics
            .values()
            .map(|m| m.avg_execution_time_ms)
            .filter(|t| *t > 0.0)
            .collect();

        if completion_times.is_empty() {
            return ThroughputStats { mean: 0.0, stddev: 0.0, min: 0.0, max: 0.0 };
        }

        let mean = completion_times.iter().sum::<f64>() / completion_times.len() as f64;
        let variance = completion_times.iter().map(|t| (t - mean).powi(2)).sum::<f64>()
            / completion_times.len() as f64;
        let stddev = variance.sqrt();

        ThroughputStats {
            mean,
            stddev,
            min: completion_times.iter().cloned().fold(f64::INFINITY, f64::min),
            max: completion_times.iter().cloned().fold(0.0, f64::max),
        }
    }

    fn calculate_error_analysis(&self, snapshot: &MetricSnapshot) -> ErrorAnalysis {
        let total_errors: u64 = snapshot.process_metrics.values().map(|m| m.failed_instances).sum();

        let total_instances: u64 =
            snapshot.process_metrics.values().map(|m| m.total_instances).sum();

        let error_rate = if total_instances > 0 {
            (total_errors as f64 / total_instances as f64) * 100.0
        } else {
            0.0
        };

        let mut top_errors: HashMap<String, u64> = HashMap::new();
        for (process_id, metrics) in &snapshot.process_metrics {
            if metrics.failed_instances > 0 {
                top_errors.insert(process_id.clone(), metrics.failed_instances);
            }
        }

        ErrorAnalysis { total_errors, error_rate, top_errors }
    }
}

impl Default for PerformanceAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Analyzes costs
#[derive(Debug, Clone)]
pub struct CostAnalyzer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostAnalysis {
    /// Total costs by resource
    pub costs_by_resource: HashMap<String, f64>,
    /// Total costs by category
    pub costs_by_category: HashMap<String, f64>,
    /// Cost trends
    pub trends: Vec<CostTrend>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostTrend {
    /// Resource or category
    pub name: String,
    /// Cost this period
    pub current_cost: f64,
    /// Cost previous period
    pub previous_cost: f64,
    /// Change percentage
    pub change_percent: f64,
}

impl CostAnalyzer {
    /// Create new cost analyzer
    pub fn new() -> Self {
        Self
    }

    /// Analyze costs
    pub fn analyze(&self, snapshot: &MetricSnapshot) -> CostAnalysis {
        let mut costs_by_resource: HashMap<String, f64> = HashMap::new();
        let mut costs_by_category: HashMap<String, f64> = HashMap::new();

        for (resource_id, cost_metrics) in &snapshot.cost_metrics {
            costs_by_resource.insert(resource_id.clone(), cost_metrics.total_cost);

            for (category, amount) in &cost_metrics.cost_breakdown {
                *costs_by_category.entry(category.clone()).or_insert(0.0) += amount;
            }
        }

        CostAnalysis { costs_by_resource, costs_by_category, trends: Vec::new() }
    }
}

impl Default for CostAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bottleneck_detection() {
        let analyzer = BottleneckAnalyzer::new();
        let mut snapshot = MetricSnapshot {
            timestamp: Utc::now(),
            process_metrics: HashMap::new(),
            performance_metrics: super::super::metrics::PerformanceMetrics::default(),
            cost_metrics: HashMap::new(),
            recent_metrics: Vec::new(),
        };

        let mut metrics = ProcessMetrics::new("slow_process");
        metrics.avg_execution_time_ms = 6000.0;
        snapshot.process_metrics.insert("slow_process".to_string(), metrics);

        let bottlenecks = analyzer.detect_bottlenecks(&snapshot);
        assert!(!bottlenecks.is_empty());
    }

    #[test]
    fn test_heatmap_generation() {
        let analyzer = HeatmapAnalyzer::new();
        let mut points = Vec::new();
        for i in 0..24 {
            points.push(MetricPoint::new("test", i as f64));
        }

        let heatmap = analyzer.generate_heatmap(&points);
        assert!(!heatmap.cells.is_empty());
    }
}
