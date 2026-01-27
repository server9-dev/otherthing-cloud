//! Analytics and monitoring dashboard for ABCD ODAF
//!
//! This module provides comprehensive analytics capabilities for:
//! - Process metrics (completion rate, duration, throughput)
//! - Performance analytics (bottleneck detection, resource utilization)
//! - Cost analysis using DoDAF cost metadata
//! - SLA monitoring and alerting
//! - Real-time process statistics collection
//! - Historical trend analysis
//! - Activity frequency heatmaps
//! - Drill-down capabilities
//! - Export functionality (CSV/JSON)
//! - Custom metric definitions
//! - Integration with DoDAF operational views

pub mod metrics;
pub mod analyzer;
pub mod alerts;
pub mod export;
pub mod visualization;

pub use metrics::{
    MetricsCollector, ProcessMetrics, PerformanceMetrics, CostMetrics,
    ResourceMetricPoint, MetricPoint, MetricSnapshot,
};
pub use analyzer::{
    AnalyticsEngine, BottleneckAnalyzer, TrendAnalyzer, HeatmapAnalyzer,
    PerformanceAnalyzer, CostAnalyzer,
};
pub use alerts::{
    AlertingSystem, Alert, AlertLevel, AlertRule, SlaPolicy, SlaViolation,
};
pub use export::{
    ExportFormat, Exporter, CsvExporter, JsonExporter, ExportManager,
};
pub use visualization::{
    VisualizationData, ChartData, HeatmapData, TimeSeriesData,
    MetricVisualization, CustomMetricDefinition,
};

use serde::{Deserialize, Serialize};

/// Main analytics dashboard coordinator
#[derive(Debug, Clone)]
pub struct AnalyticsDashboard {
    /// Metrics collector
    pub collector: MetricsCollector,
    /// Analytics engine
    pub analyzer: AnalyticsEngine,
    /// Alerting system
    pub alerting: AlertingSystem,
}

impl AnalyticsDashboard {
    /// Create a new analytics dashboard
    pub fn new() -> Self {
        Self {
            collector: MetricsCollector::new(),
            analyzer: AnalyticsEngine::new(),
            alerting: AlertingSystem::new(),
        }
    }

    /// Initialize dashboard with custom SLA policies
    pub fn with_sla_policies(mut self, policies: Vec<SlaPolicy>) -> Self {
        for policy in policies {
            self.alerting.add_sla_policy(policy);
        }
        self
    }

    /// Get current dashboard state snapshot
    pub fn snapshot(&self) -> DashboardSnapshot {
        DashboardSnapshot {
            metrics: self.collector.current_snapshot(),
            active_alerts: self.alerting.active_alerts(),
            performance_summary: self.analyzer.performance_summary(),
        }
    }
}

impl Default for AnalyticsDashboard {
    fn default() -> Self {
        Self::new()
    }
}

/// Dashboard state snapshot for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSnapshot {
    /// Current metrics snapshot
    pub metrics: MetricSnapshot,
    /// Active alerts
    pub active_alerts: Vec<Alert>,
    /// Performance summary
    pub performance_summary: PerformanceSummary,
}

/// High-level performance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    /// Overall process health (0-100)
    pub health_score: f64,
    /// Average completion time
    pub avg_completion_time_ms: f64,
    /// Process completion rate (%)
    pub completion_rate: f64,
    /// Throughput (processes per minute)
    pub throughput: f64,
    /// Resource utilization (%)
    pub resource_utilization: f64,
    /// Number of active bottlenecks
    pub active_bottlenecks: usize,
    /// SLA compliance rate (%)
    pub sla_compliance_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_dashboard_creation() {
        let dashboard = AnalyticsDashboard::new();
        let snapshot = dashboard.snapshot();
        assert_eq!(snapshot.active_alerts.len(), 0);
    }
}
