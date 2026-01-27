# ABCDODAF Analytics & Monitoring Dashboard Guide

## Overview

The ABCDODAF Analytics module provides a comprehensive metrics collection, analysis, alerting, and reporting system for monitoring process execution, performance, cost, and SLA compliance.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│           Analytics & Monitoring Dashboard                  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────────┐  ┌──────────────────┐               │
│  │ Metrics Collector│  │ Analytics Engine │               │
│  │  - Processes     │  │  - Bottlenecks   │               │
│  │  - Performance   │  │  - Trends        │               │
│  │  - Cost          │  │  - Heatmaps      │               │
│  │  - Resources     │  │  - Performance   │               │
│  └──────────────────┘  └──────────────────┘               │
│         │                      │                          │
│         └──────────┬───────────┘                          │
│                    │                                      │
│         ┌──────────▼──────────┐                          │
│         │  Alerting System    │                          │
│         │  - Alert Rules      │                          │
│         │  - SLA Policies     │                          │
│         │  - Violations       │                          │
│         └─────────┬────────────┘                         │
│                   │                                      │
│         ┌─────────▼──────────┐                          │
│         │ Export & Reports   │                          │
│         │  - CSV/JSON/YAML   │                          │
│         │  - Report Gen      │                          │
│         └────────────────────┘                          │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

## Core Components

### 1. MetricsCollector

Collects and maintains real-time metrics:

```rust
use abcdodaf::prelude::*;

let collector = MetricsCollector::new();

// Record process execution
collector.record_process_execution("order_process", 2500.0, true);
collector.record_process_execution("order_process", 3200.0, true);
collector.record_process_execution("order_process", 1500.0, false); // Failed

// Record cost metrics
collector.record_cost("resource_1", "compute".to_string(), 25.50, "USD");

// Get current snapshot
let snapshot = collector.current_snapshot();
for (process_id, metrics) in &snapshot.process_metrics {
    println!("Process: {}, Completion Rate: {:.1}%",
        process_id, metrics.completion_rate);
}
```

**ProcessMetrics** tracks:
- Total instances executed
- Completed vs failed instances
- Completion rate (%)
- Average execution time
- Min/Max execution times
- P95 percentile execution time

### 2. AnalyticsEngine

Performs intelligent analysis on collected metrics:

```rust
let engine = AnalyticsEngine::new();
let snapshot = collector.current_snapshot();

// Get overall performance analysis
let perf_analysis = engine.performance_analyzer.analyze(&snapshot);
println!("P95 Response Time: {:.0}ms", perf_analysis.response_time_percentiles.p95);
println!("Error Rate: {:.2}%", perf_analysis.error_analysis.error_rate);

// Detect bottlenecks
let bottlenecks = engine.bottleneck_analyzer.detect_bottlenecks(&snapshot);
for bottleneck in bottlenecks {
    println!("Bottleneck in {}: {} (Severity: {:.0}%)",
        bottleneck.process_id, bottleneck.reason, bottleneck.severity);
    println!("Recommendation: {}", bottleneck.recommendation);
}

// Analyze trends
let trends = engine.trend_analyzer.analyze_trends(
    &snapshot.recent_metrics,
    chrono::Duration::days(7),
);
for trend in trends {
    println!("{}: {:.2}% change ({:?})",
        trend.metric_name, trend.change_percent, trend.direction);
}

// Generate heatmaps
let heatmap = engine.heatmap_analyzer.generate_heatmap(&snapshot.recent_metrics);
for cell in &heatmap.cells {
    println!("Activity at ({},{}): {} occurrences", cell.x, cell.y, cell.count);
}

// Cost analysis
let cost_analysis = engine.cost_analyzer.analyze(&snapshot);
for (resource, cost) in &cost_analysis.costs_by_resource {
    println!("Resource {}: ${:.2}", resource, cost);
}
```

### 3. AlertingSystem

Manages alert rules, SLA policies, and violation tracking:

```rust
let alerting = AlertingSystem::new();

// Add alert rules
let rule = AlertRule::new(
    "High Response Time",
    "response_time",
    "greater_than",
    3000.0,
    AlertLevel::Warning,
);
alerting.add_rule(rule);

// Define SLA policies
let sla = SlaPolicy::new("Standard SLA", "payment_process")
    .with_max_response_time(2000.0)
    .with_max_error_rate(0.5)
    .with_min_completion_rate(99.5);
alerting.add_sla_policy(sla);

// Evaluate metrics against rules
let alerts = alerting.evaluate_metric("response_time", 3500.0);
println!("Triggered {} alerts", alerts.len());

// Check SLA compliance
let violations = alerting.check_sla_compliance(
    "payment_process",
    Some(2500.0),      // response_time
    Some(99.0),        // availability
    Some(1.0),         // error_rate
    Some(98.5),        // completion_rate
);
for violation in violations {
    println!("SLA Violation: {} - Expected {}, Got {}",
        violation.metric, violation.expected, violation.actual);
}

// Get active alerts
let active = alerting.active_alerts();
for alert in active {
    println!("[{}] {}: {}",
        match alert.level {
            AlertLevel::Info => "INFO",
            AlertLevel::Warning => "WARN",
            AlertLevel::Critical => "CRIT",
        },
        alert.source, alert.message);
}

// Resolve alerts
alerting.resolve_alert(&alert_id);
```

### 4. ExportManager

Export metrics and reports in multiple formats:

```rust
use abcdodaf::analytics::{ExportFormat, ExportManager};

let snapshot = collector.current_snapshot();

// Export in different formats
let csv = ExportManager::export(&snapshot, ExportFormat::Csv)?;
let json = ExportManager::export(&snapshot, ExportFormat::Json)?;
let yaml = ExportManager::export(&snapshot, ExportFormat::Yaml)?;

// Export to file
ExportManager::export_to_file(
    &snapshot,
    std::path::Path::new("/tmp/metrics.csv"),
    ExportFormat::Csv
)?;

// Generate report
let report = ExportManager::generate_report(
    &snapshot,
    "Weekly Analytics Report",
    ExportFormat::Json,
)?;
```

### 5. Visualization Data Structures

Create data structures for UI charting:

```rust
use abcdodaf::analytics::{MetricVisualization, VisualizationType};

// Time series visualization
let mut ts = MetricVisualization::time_series("response_time", "ms");
ts.add_point(Utc::now(), 100.0);
ts.add_point(Utc::now() + Duration::minutes(1), 150.0);

// Categorical data (bar chart)
let mut cat = MetricVisualization::categorical();
cat.add_category("process_1");
cat.add_category("process_2");
cat.add_series("completion_rate", vec![95.0, 87.0]);

// Heatmap data
let mut heatmap = MetricVisualization::heatmap(
    vec!["0:00".to_string(), "6:00".to_string(), "12:00".to_string()],
    vec!["Mon".to_string(), "Tue".to_string()],
);
heatmap.add_cell(0, 0, 45.0, Some("12".to_string()));
heatmap.set_color_scheme("cool");

// Gauge visualization
let gauge = MetricVisualization::gauge("CPU", 65.0, 0.0, 100.0, "%");
println!("CPU Status: {} ({}%)", gauge.current_color(), gauge.value);

// Custom metric definition
let metric = CustomMetricDefinition::new(
    "Avg Response Time",
    "Average response time for all processes",
    CustomMetricType::Derived,
)
.with_data_source("process_metrics", "payment_process", "avg_execution_time_ms")
.with_unit("ms")
.with_aggregation(AggregationMethod::Average)
.with_visualization_type(VisualizationType::TimeSeries);
```

## AnalyticsDashboard - Complete Integration

The `AnalyticsDashboard` coordinates all components:

```rust
use abcdodaf::prelude::*;

// Create dashboard
let dashboard = AnalyticsDashboard::new();

// Add SLA policies
let sla = SlaPolicy::new("Production SLA", "critical_process")
    .with_max_response_time(5000.0)
    .with_max_error_rate(1.0)
    .with_min_completion_rate(99.0);
dashboard.alerting.add_sla_policy(sla);

// Record metrics
dashboard.collector.record_process_execution("critical_process", 2500.0, true);
dashboard.collector.record_cost("resource_1", "compute".to_string(), 10.0, "USD");

// Get complete snapshot
let snapshot = dashboard.snapshot();
println!("Active Alerts: {}", snapshot.active_alerts.len());
println!("Health Score: {:.0}%", snapshot.performance_summary.health_score);
println!("Throughput: {:.2} processes/min", snapshot.performance_summary.throughput);
```

## Metrics Collection Configuration

```rust
use abcdodaf::analytics::MetricsConfig;

let config = MetricsConfig {
    max_history_points: 10000,      // Retain last 10k data points
    aggregation_interval_secs: 60,  // Aggregate every minute
    detailed_collection: true,      // Enable detailed metrics
};

let collector = MetricsCollector::with_config(config);
```

## Bottleneck Detection Algorithm

Detects bottlenecks based on:

1. **High Execution Time**: When avg time > 5000ms
   - Severity = (avg_time / 10000) * 100
   - Recommendation: Optimize logic or increase parallelization

2. **High Failure Rate**: When failures > 10%
   - Severity = failure_rate
   - Recommendation: Investigate root causes

3. **Low Completion Rate**: When completion_rate < 80%
   - Severity = 100 - completion_rate
   - Recommendation: Review dependencies and requirements

## Trend Analysis

Analyzes metric changes over time periods:

```rust
let trends = analyzer.trend_analyzer.analyze_trends(
    &snapshot.recent_metrics,
    chrono::Duration::days(7),
);

for trend in trends {
    match trend.direction {
        TrendDirection::Improving => println!("Performance improving!"),
        TrendDirection::Degrading => println!("Performance degrading!"),
        TrendDirection::Stable => println!("Performance stable"),
    }
}
```

## SLA Policy Definition

SLA policies track multiple dimensions:

```rust
let sla = SlaPolicy::new("Enterprise SLA", "order_system")
    .with_max_response_time(3000.0)      // Max 3 seconds
    .with_min_availability(99.9)         // 99.9% uptime
    .with_max_error_rate(0.1)            // < 0.1% errors
    .with_min_completion_rate(99.5)      // 99.5% completion
    .with_coverage_window("24x7");       // Round-the-clock
```

## Cost Analysis

Track costs by resource and category:

```rust
// Record costs
collector.record_cost("gpu_node_1", "compute".to_string(), 50.0, "USD");
collector.record_cost("gpu_node_1", "storage".to_string(), 10.0, "USD");
collector.record_cost("gpu_node_2", "compute".to_string(), 45.0, "USD");

// Analyze
let cost_analysis = analyzer.cost_analyzer.analyze(&snapshot);

// By resource
for (resource, cost) in &cost_analysis.costs_by_resource {
    println!("{}: ${:.2}", resource, cost);
}

// By category
for (category, cost) in &cost_analysis.costs_by_category {
    println!("{}: ${:.2}", category, cost);
}
```

## Performance Percentiles

Calculate and monitor response time percentiles:

```rust
let perf = analyzer.performance_analyzer.analyze(&snapshot);

println!("P50 (Median): {:.0}ms", perf.response_time_percentiles.p50);
println!("P95: {:.0}ms", perf.response_time_percentiles.p95);
println!("P99: {:.0}ms", perf.response_time_percentiles.p99);
```

## Activity Heatmap

Generate heatmaps showing activity frequency by time:

```rust
let heatmap = analyzer.heatmap_analyzer.generate_heatmap(&snapshot.recent_metrics);

// X-axis: Hours (0-23)
// Y-axis: Days of week (0-6)
for cell in &heatmap.cells {
    if cell.intensity > 50.0 {
        println!("High activity: {}:00 on day {} ({} occurrences)",
            cell.x, cell.y, cell.count);
    }
}
```

## Alert Channels

Configure where alerts are sent:

```rust
alerting.add_channel(AlertChannel::Console);
alerting.add_channel(AlertChannel::Email {
    recipients: vec!["ops@example.com".to_string()]
});
alerting.add_channel(AlertChannel::Webhook {
    url: "https://alerts.example.com/webhook".to_string()
});
alerting.add_channel(AlertChannel::Sms {
    phone_numbers: vec!["+1234567890".to_string()]
});
```

## Report Generation

Generate comprehensive analytics reports:

```rust
let report_json = ExportManager::generate_report(
    &snapshot,
    "Monthly Performance Report",
    ExportFormat::Json,
)?;

// Report includes:
// - Executive summary (total processes, success rate, avg time, costs)
// - Findings (bottlenecks, high-failure processes, issues)
// - Recommendations for optimization
// - Generated timestamp and period
```

## Integration with DoDAF

Connect metrics to DoDAF operational views:

```rust
// Operational activities can record their execution metrics
let activity = OperationalActivity::new("Process Order", ActivityType::BusinessActivity);

// Collection during execution
dashboard.collector.record_process_execution(
    &activity.id,
    execution_duration_ms,
    success,
);

// Later analyze with DoDAF context
let snapshot = dashboard.collector.current_snapshot();
// Correlate with operational context for architecture insights
```

## Best Practices

1. **Record Consistently**: Record metrics at process completion, not at random intervals
2. **Set Realistic SLAs**: Base SLA thresholds on historical performance data
3. **Monitor Health**: Review dashboard health score regularly (target: >80%)
4. **Act on Alerts**: Don't ignore critical alerts; investigate bottlenecks promptly
5. **Review Trends**: Monitor 7-day and 30-day trends for pattern detection
6. **Cost Management**: Track cost trends to identify optimization opportunities
7. **Export Regularly**: Export metrics for long-term storage and analysis
8. **Clean Old Data**: Use `cleanup_old_metrics()` to manage storage

## Performance Considerations

- MetricsCollector uses Arc<Mutex<>> for thread-safe access
- History is limited to `max_history_points` (default: 10,000)
- Old metrics can be cleared with `clear_old_metrics(duration)`
- Aggregation reduces memory footprint over time
- Snapshot generation is atomic and consistent

## Example: Complete Monitoring Flow

```rust
use abcdodaf::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize dashboard
    let dashboard = AnalyticsDashboard::new();

    // Configure SLAs
    let sla = SlaPolicy::new("Production", "my_service")
        .with_max_response_time(2000.0)
        .with_min_completion_rate(99.0);
    dashboard.alerting.add_sla_policy(sla);

    // Collect metrics during execution
    for _ in 0..1000 {
        let start = std::time::Instant::now();
        // ... execute process ...
        let duration = start.elapsed().as_millis() as f64;
        dashboard.collector.record_process_execution("my_service", duration, true);
    }

    // Analyze
    let snapshot = dashboard.collector.current_snapshot();
    let analysis = dashboard.analyzer.analyze(&snapshot);

    // Check SLA
    let violations = dashboard.alerting.check_sla_compliance(
        "my_service",
        Some(analysis.avg_completion_time_ms),
        None,
        None,
        Some(analysis.completion_rate),
    );

    // Report
    let report = ExportManager::generate_report(
        &snapshot,
        "Execution Report",
        ExportFormat::Json,
    )?;

    println!("Report: {}", report);
    Ok(())
}
```

## Future Enhancements

- Machine learning-based anomaly detection
- Predictive alerting based on trends
- Integration with external monitoring systems (Prometheus, Grafana)
- Real-time dashboard streaming
- Custom metric calculations
- Historical data warehousing
- Performance benchmarking and comparison

---

For more details, see the API documentation and examples.
