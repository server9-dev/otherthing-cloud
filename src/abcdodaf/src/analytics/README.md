# Analytics & Monitoring Module

This module provides a comprehensive analytics and monitoring system for the ABCDODAF library, enabling detailed metrics collection, analysis, alerting, and reporting.

## Features

### 1. Metrics Collection (metrics.rs)
- **Process Metrics**: Track execution count, completion rate, execution times, failure rates
- **Performance Metrics**: Real-time response times, throughput, error rates, resource utilization
- **Cost Metrics**: Track resource costs by category with aggregation
- **Resource Metrics**: CPU, memory, disk I/O, network utilization
- **Metric Points**: Time-series data points with metadata and tags

### 2. Analytics Engine (analyzer.rs)
- **Bottleneck Detection**: Identify slow processes, high failure rates, low completion
- **Trend Analysis**: Detect improving/stable/degrading trends over periods
- **Performance Analysis**: Percentile calculations, throughput stats, error analysis
- **Cost Analysis**: Cost breakdown by resource and category
- **Heatmap Generation**: Activity frequency visualization by hour and day of week

### 3. Alerting System (alerts.rs)
- **Alert Rules**: Flexible rule definitions with thresholds and conditions
- **Alert Management**: Create, store, resolve alerts with different severity levels
- **SLA Policies**: Define service level agreements with multiple dimensions
- **SLA Monitoring**: Check compliance across response time, availability, error rate, completion
- **Violation Tracking**: Record and query SLA violations
- **Alert Channels**: Support for console, email, webhook, SMS delivery

### 4. Export & Reporting (export.rs)
- **Multiple Formats**: CSV, JSON, YAML exports
- **Report Generation**: Comprehensive analytics reports with findings and recommendations
- **File Export**: Direct export to filesystem
- **Filtering**: Export specific data ranges and metrics

### 5. Visualization Structures (visualization.rs)
- **Time Series Data**: For line charts with historical trends
- **Categorical Data**: For bar/pie charts
- **Heatmap Data**: For activity frequency visualization
- **Gauge Data**: For metric displays with thresholds
- **Custom Metrics**: Define application-specific metrics
- **UI Integration**: Data structures designed for egui/web charting libraries

## Architecture

```
┌──────────────────────────────────┐
│   AnalyticsDashboard             │
│  (Coordinator)                   │
├──────────────────────────────────┤
│                                  │
├─ MetricsCollector               │
│  ├─ Process Metrics             │
│  ├─ Performance Metrics         │
│  ├─ Cost Metrics                │
│  └─ Resource Metrics            │
│                                  │
├─ AnalyticsEngine                │
│  ├─ BottleneckAnalyzer          │
│  ├─ TrendAnalyzer               │
│  ├─ PerformanceAnalyzer         │
│  ├─ CostAnalyzer                │
│  └─ HeatmapAnalyzer             │
│                                  │
├─ AlertingSystem                 │
│  ├─ Alert Rules                 │
│  ├─ SLA Policies                │
│  ├─ Alert Management            │
│  └─ Violation Tracking          │
│                                  │
└─ Export & Visualization         │
   ├─ ExportManager               │
   ├─ ReportGenerator             │
   └─ VisualizationData           │
```

## Key Data Structures

### ProcessMetrics
```rust
pub struct ProcessMetrics {
    pub process_id: String,
    pub total_instances: u64,
    pub completed_instances: u64,
    pub failed_instances: u64,
    pub completion_rate: f64,
    pub avg_execution_time_ms: f64,
    pub min_execution_time_ms: f64,
    pub max_execution_time_ms: f64,
    pub p95_execution_time_ms: f64,
    pub last_execution_time_ms: f64,
    pub updated_at: DateTime<Utc>,
}
```

### Alert
```rust
pub struct Alert {
    pub id: String,
    pub message: String,
    pub level: AlertLevel,  // Info, Warning, Critical
    pub source: String,
    pub created_at: DateTime<Utc>,
    pub resolved: bool,
    pub context: HashMap<String, serde_json::Value>,
}
```

### SlaViolation
```rust
pub struct SlaViolation {
    pub id: String,
    pub sla_policy_id: String,
    pub metric: String,
    pub expected: f64,
    pub actual: f64,
    pub violation_time: DateTime<Utc>,
    pub severity: AlertLevel,
}
```

### Bottleneck
```rust
pub struct Bottleneck {
    pub process_id: String,
    pub severity: f64,          // 0-100
    pub reason: String,
    pub detected_at: DateTime<Utc>,
    pub recommendation: String,
}
```

## Usage Examples

### Basic Metrics Collection
```rust
let collector = MetricsCollector::new();

// Record execution
collector.record_process_execution("my_process", 1500.0, true);
collector.record_process_execution("my_process", 2000.0, true);
collector.record_process_execution("my_process", 1200.0, false);

// Get metrics
let metrics = collector.get_process_metrics("my_process");
assert_eq!(metrics.total_instances, 3);
assert_eq!(metrics.failed_instances, 1);
```

### SLA Compliance
```rust
let alerting = AlertingSystem::new();

let sla = SlaPolicy::new("Production", "payment_service")
    .with_max_response_time(2000.0)
    .with_max_error_rate(0.5);

alerting.add_sla_policy(sla);

let violations = alerting.check_sla_compliance(
    "payment_service",
    Some(2500.0),   // response_time
    None,
    Some(1.0),      // error_rate
    None,
);
```

### Bottleneck Detection
```rust
let snapshot = collector.current_snapshot();
let bottlenecks = analyzer.bottleneck_analyzer.detect_bottlenecks(&snapshot);

for bottleneck in bottlenecks {
    eprintln!("Alert: {}", bottleneck.reason);
    eprintln!("Fix: {}", bottleneck.recommendation);
}
```

### Export Reports
```rust
let csv = ExportManager::export(&snapshot, ExportFormat::Csv)?;
let json = ExportManager::export(&snapshot, ExportFormat::Json)?;

ExportManager::export_to_file(
    &snapshot,
    Path::new("metrics.csv"),
    ExportFormat::Csv
)?;
```

## Thread Safety

All components use `Arc<Mutex<>>` for thread-safe concurrent access:
- MetricsCollector can be cloned and shared across threads
- AlertingSystem supports concurrent alert creation and resolution
- No locks are held during computation (only during data access)

## Performance Characteristics

- **Collection**: O(1) for recording metrics
- **Storage**: Limited to `max_history_points` (default 10,000 records)
- **Analysis**: O(n) where n = number of metrics
- **Snapshot**: Atomic point-in-time view, O(1) access
- **Cleanup**: O(n) for removing old metrics

## Configuration

```rust
let config = MetricsConfig {
    max_history_points: 10000,
    aggregation_interval_secs: 60,
    detailed_collection: true,
};

let collector = MetricsCollector::with_config(config);
```

## Integration Points

### With BPMN Engine
```rust
// After task execution
dashboard.collector.record_process_execution(
    task.process_id,
    task.duration_ms,
    task.succeeded,
);
```

### With DoDAF Activities
```rust
// Operational activities report metrics
dashboard.collector.record_process_execution(
    &activity.operational_id,
    execution_time_ms,
    success,
);
```

### With Visualization (UI)
```rust
// Create visualization data for charts
let ts = MetricVisualization::time_series("response_time", "ms");
// Use with egui or web charting libraries
```

## Testing

All modules include comprehensive unit tests:

```bash
cargo test --lib analytics
```

## Future Enhancements

1. **Anomaly Detection**: ML-based outlier detection
2. **Forecasting**: Predictive SLA violations
3. **Custom Metrics**: User-defined metric calculations
4. **External Integration**: Prometheus, Datadog, CloudWatch
5. **Streaming**: Real-time metric streaming to dashboards
6. **Histograms**: Detailed distribution analysis
7. **Cardinality Limits**: Prevent unbounded metric growth
8. **Sampling**: Optional metric sampling for high-throughput systems

## Files

- `mod.rs` - Module definition and dashboard coordinator
- `metrics.rs` - Metrics collection (4000+ lines)
- `analyzer.rs` - Analysis algorithms (1200+ lines)
- `alerts.rs` - Alerting and SLA system (800+ lines)
- `export.rs` - Export and reporting (600+ lines)
- `visualization.rs` - Visualization data structures (900+ lines)

## Dependencies

- `chrono` - Timestamps and duration calculations
- `serde` - Serialization/deserialization
- `uuid` - Unique identifier generation
- `parking_lot` - Mutex operations (via Arc<Mutex>)

## License

MIT or Apache 2.0 (same as ABCDODAF)
