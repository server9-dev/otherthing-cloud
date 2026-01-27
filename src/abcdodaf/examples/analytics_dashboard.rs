//! Example: Analytics and Monitoring Dashboard
//!
//! Demonstrates comprehensive metrics collection, analysis, alerting, and reporting.

use abcdodaf::prelude::*;
use chrono::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("ABCDODAF Analytics & Monitoring Dashboard Example\n");

    // Create analytics dashboard
    let dashboard = AnalyticsDashboard::new();

    // Add SLA policies
    let sla_standard = SlaPolicy::new("Standard SLA", "order_processing")
        .with_max_response_time(5000.0)
        .with_max_error_rate(2.0)
        .with_min_completion_rate(98.0);

    let sla_critical = SlaPolicy::new("Critical SLA", "payment_processing")
        .with_max_response_time(2000.0)
        .with_max_error_rate(0.1)
        .with_min_completion_rate(99.9);

    dashboard.alerting.add_sla_policy(sla_standard);
    dashboard.alerting.add_sla_policy(sla_critical);

    // Add alert rules
    let high_latency_rule = abcdodaf::analytics::AlertRule::new(
        "High Latency Alert",
        "response_time",
        "greater_than",
        3000.0,
        AlertLevel::Warning,
    );

    let critical_error_rule = abcdodaf::analytics::AlertRule::new(
        "Critical Error Rate",
        "error_rate",
        "greater_than",
        5.0,
        AlertLevel::Critical,
    );

    dashboard.alerting.add_rule(high_latency_rule);
    dashboard.alerting.add_rule(critical_error_rule);

    // Simulate process executions and metrics collection
    println!("--- Simulating Process Executions ---\n");

    // Order processing metrics
    dashboard.collector.record_process_execution("order_processing", 2500.0, true);
    dashboard.collector.record_process_execution("order_processing", 3200.0, true);
    dashboard.collector.record_process_execution("order_processing", 2800.0, true);
    dashboard.collector.record_process_execution("order_processing", 5500.0, false); // High latency + failure

    // Payment processing metrics
    dashboard.collector.record_process_execution("payment_processing", 1200.0, true);
    dashboard.collector.record_process_execution("payment_processing", 1500.0, true);
    dashboard.collector.record_process_execution("payment_processing", 1100.0, true);

    // Record cost metrics
    dashboard
        .collector
        .record_cost("payment_processing", "compute".to_string(), 25.50, "USD");
    dashboard
        .collector
        .record_cost("payment_processing", "storage".to_string(), 5.00, "USD");
    dashboard
        .collector
        .record_cost("order_processing", "compute".to_string(), 15.75, "USD");

    // Get current snapshot
    let snapshot = dashboard.collector.current_snapshot();
    println!("Process Metrics Collected:");
    for (process_id, metrics) in &snapshot.process_metrics {
        println!(
            "  {}: {} executions, {:.1}% completion rate, {:.0}ms avg time",
            process_id,
            metrics.total_instances,
            metrics.completion_rate,
            metrics.avg_execution_time_ms
        );
    }
    println!();

    // Analyze metrics
    println!("--- Analysis Results ---\n");

    let bottleneck_analyzer = abcdodaf::analytics::BottleneckAnalyzer::new();
    let bottlenecks = bottleneck_analyzer.detect_bottlenecks(&snapshot);
    if !bottlenecks.is_empty() {
        println!("Detected Bottlenecks:");
        for bottleneck in bottlenecks {
            println!(
                "  [{}] {}: {} (Severity: {:.0}%)",
                bottleneck.detected_at.format("%H:%M:%S"),
                bottleneck.process_id,
                bottleneck.reason,
                bottleneck.severity
            );
            println!("    Recommendation: {}", bottleneck.recommendation);
        }
        println!();
    }

    // Trend analysis
    let trend_analyzer = abcdodaf::analytics::TrendAnalyzer::new();
    let trends = trend_analyzer.analyze_trends(&snapshot.recent_metrics, Duration::days(7));
    if !trends.is_empty() {
        println!("Trends (7-day period):");
        for trend in trends {
            println!(
                "  {}: {:.2}% change ({:?})",
                trend.metric_name, trend.change_percent, trend.direction
            );
        }
        println!();
    }

    // Check SLA compliance
    println!("--- SLA Compliance Check ---\n");

    let violations_order = dashboard.alerting.check_sla_compliance(
        "order_processing",
        Some(3500.0), // avg response time
        Some(99.5),   // availability
        Some(1.5),    // error rate
        Some(97.5),   // completion rate
    );

    let violations_payment = dashboard.alerting.check_sla_compliance(
        "payment_processing",
        Some(1300.0),
        Some(99.95),
        Some(0.05),
        Some(99.9),
    );

    if !violations_order.is_empty() {
        println!("Order Processing SLA Violations:");
        for violation in violations_order {
            println!(
                "  {}: Expected {}, Got {} ({})",
                violation.metric,
                violation.expected,
                violation.actual,
                if violation.severity == AlertLevel::Critical { "CRITICAL" } else { "WARNING" }
            );
        }
    } else {
        println!("Order Processing: SLA COMPLIANT");
    }

    if violations_payment.is_empty() {
        println!("Payment Processing: SLA COMPLIANT");
    }
    println!();

    // Generate alerts
    println!("--- Alert Generation ---\n");

    dashboard.alerting.evaluate_metric("response_time", 3500.0);
    dashboard.alerting.evaluate_metric("error_rate", 1.5);

    let active_alerts = dashboard.alerting.active_alerts();
    if !active_alerts.is_empty() {
        println!("Active Alerts:");
        for alert in &active_alerts {
            let level_str = match alert.level {
                AlertLevel::Info => "INFO",
                AlertLevel::Warning => "WARNING",
                AlertLevel::Critical => "CRITICAL",
            };
            println!("  [{}] {}: {}", level_str, alert.source, alert.message);
        }
    } else {
        println!("No active alerts");
    }
    println!();

    // Performance analysis
    println!("--- Performance Analysis ---\n");

    let performance_analyzer = abcdodaf::analytics::PerformanceAnalyzer::new();
    let perf_analysis = performance_analyzer.analyze(&snapshot);
    println!("Response Time Percentiles:");
    println!(
        "  P50: {:.0}ms, P95: {:.0}ms, P99: {:.0}ms",
        perf_analysis.response_time_percentiles.p50,
        perf_analysis.response_time_percentiles.p95,
        perf_analysis.response_time_percentiles.p99
    );
    println!(
        "Error Rate: {:.2}%, Total Errors: {}",
        perf_analysis.error_analysis.error_rate, perf_analysis.error_analysis.total_errors
    );
    println!();

    // Cost analysis
    println!("--- Cost Analysis ---\n");

    let cost_analyzer = abcdodaf::analytics::CostAnalyzer::new();
    let cost_analysis = cost_analyzer.analyze(&snapshot);
    println!("Costs by Resource:");
    for (resource, cost) in &cost_analysis.costs_by_resource {
        println!("  {}: ${:.2}", resource, cost);
    }
    println!("Costs by Category:");
    for (category, cost) in &cost_analysis.costs_by_category {
        println!("  {}: ${:.2}", category, cost);
    }
    println!();

    // Heatmap generation
    println!("--- Activity Heatmap ---\n");

    let heatmap_analyzer = abcdodaf::analytics::HeatmapAnalyzer::new();
    let heatmap = heatmap_analyzer.generate_heatmap(&snapshot.recent_metrics);
    println!("Activity Heatmap:");
    println!("  Total activities: {}", heatmap.cells.len());
    println!("  Max activity count: {}", heatmap.max_count);
    println!("  Period: {}", heatmap.period);
    println!();

    // Export metrics
    println!("--- Export Functionality ---\n");

    let csv_export = ExportManager::export(&snapshot, ExportFormat::Csv)?;
    println!("CSV Export (first 200 chars):\n{}\n", &csv_export[..csv_export.len().min(200)]);

    let json_export = ExportManager::export(&snapshot, ExportFormat::Json)?;
    println!("JSON Export (first 200 chars):\n{}\n", &json_export[..json_export.len().min(200)]);

    // Generate report
    println!("--- Analytics Report ---\n");

    let report =
        ExportManager::generate_report(&snapshot, "Weekly Analytics Report", ExportFormat::Json)?;
    println!("Report Generated:\n{}\n", &report[..report.len().min(300)]);

    // Dashboard snapshot
    println!("--- Dashboard Snapshot ---\n");

    let dash_snapshot = dashboard.snapshot();
    println!(
        "Dashboard Summary: {} active alerts, {} processes monitored",
        dash_snapshot.active_alerts.len(),
        dash_snapshot.metrics.process_metrics.len()
    );

    println!("\nAnalytics Dashboard Example Complete!");
    Ok(())
}
