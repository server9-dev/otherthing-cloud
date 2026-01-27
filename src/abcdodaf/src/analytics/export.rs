//! Export functionality for metrics and reports
//!
//! Provides export to CSV, JSON, and other formats

use super::metrics::MetricSnapshot;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::io::Write;

/// Export format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    /// CSV format
    Csv,
    /// JSON format
    Json,
    /// YAML format
    Yaml,
}

/// Main exporter trait
pub trait Exporter {
    /// Export metrics snapshot
    fn export(&self, snapshot: &MetricSnapshot) -> Result<String>;
}

/// CSV exporter
pub struct CsvExporter;

impl CsvExporter {
    /// Create new CSV exporter
    pub fn new() -> Self {
        Self
    }

    /// Export to CSV writer
    pub fn export_to_writer(&self, snapshot: &MetricSnapshot, writer: &mut dyn Write) -> Result<()> {
        // Write header
        writeln!(
            writer,
            "Process ID,Total Instances,Completed,Failed,Completion Rate,Avg Time (ms),Min Time (ms),Max Time (ms),P95 Time (ms),Last Time (ms),Updated At"
        )?;

        // Write data rows
        for (_process_id, metrics) in &snapshot.process_metrics {
            writeln!(
                writer,
                "{},{},{},{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{}",
                metrics.process_id,
                metrics.total_instances,
                metrics.completed_instances,
                metrics.failed_instances,
                metrics.completion_rate,
                metrics.avg_execution_time_ms,
                metrics.min_execution_time_ms,
                metrics.max_execution_time_ms,
                metrics.p95_execution_time_ms,
                metrics.last_execution_time_ms,
                metrics.updated_at.to_rfc3339()
            )?;
        }

        Ok(())
    }
}

impl Default for CsvExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Exporter for CsvExporter {
    fn export(&self, snapshot: &MetricSnapshot) -> Result<String> {
        let mut output = Vec::new();
        self.export_to_writer(snapshot, &mut output)?;
        Ok(String::from_utf8_lossy(&output).to_string())
    }
}

/// JSON exporter
pub struct JsonExporter;

impl JsonExporter {
    /// Create new JSON exporter
    pub fn new() -> Self {
        Self
    }

    /// Export with pretty printing
    pub fn export_pretty(&self, snapshot: &MetricSnapshot) -> Result<String> {
        let json = serde_json::to_string_pretty(snapshot)?;
        Ok(json)
    }

    /// Export compact
    pub fn export_compact(&self, snapshot: &MetricSnapshot) -> Result<String> {
        let json = serde_json::to_string(snapshot)?;
        Ok(json)
    }
}

impl Default for JsonExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Exporter for JsonExporter {
    fn export(&self, snapshot: &MetricSnapshot) -> Result<String> {
        self.export_pretty(snapshot)
    }
}

/// Analytics report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsReport {
    /// Report title
    pub title: String,
    /// Report description
    pub description: Option<String>,
    /// Generated timestamp
    pub generated_at: chrono::DateTime<chrono::Utc>,
    /// Period covered
    pub period: ReportPeriod,
    /// Executive summary
    pub executive_summary: ExecutiveSummary,
    /// Detailed findings
    pub findings: Vec<Finding>,
    /// Recommendations
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportPeriod {
    /// Start time
    pub start: chrono::DateTime<chrono::Utc>,
    /// End time
    pub end: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutiveSummary {
    /// Total processes executed
    pub total_processes: u64,
    /// Success rate
    pub success_rate: f64,
    /// Average execution time
    pub avg_execution_time_ms: f64,
    /// Total cost
    pub total_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Finding title
    pub title: String,
    /// Finding description
    pub description: String,
    /// Severity (info, warning, critical)
    pub severity: String,
    /// Impact
    pub impact: String,
}

/// Report generator
pub struct ReportGenerator;

impl ReportGenerator {
    /// Create new report generator
    pub fn new() -> Self {
        Self
    }

    /// Generate analytics report
    pub fn generate_report(
        &self,
        snapshot: &MetricSnapshot,
        title: impl Into<String>,
    ) -> AnalyticsReport {
        let total_processes: u64 = snapshot
            .process_metrics
            .values()
            .map(|m| m.total_instances)
            .sum();

        let completed: u64 = snapshot
            .process_metrics
            .values()
            .map(|m| m.completed_instances)
            .sum();

        let success_rate = if total_processes > 0 {
            (completed as f64 / total_processes as f64) * 100.0
        } else {
            0.0
        };

        let avg_exec_time = if completed > 0 {
            snapshot
                .process_metrics
                .values()
                .map(|m| m.avg_execution_time_ms * m.completed_instances as f64)
                .sum::<f64>()
                / completed as f64
        } else {
            0.0
        };

        let total_cost: f64 = snapshot
            .cost_metrics
            .values()
            .map(|c| c.total_cost)
            .sum();

        let mut findings = Vec::new();

        // Analyze for findings
        for (process_id, metrics) in &snapshot.process_metrics {
            if metrics.avg_execution_time_ms > 5000.0 {
                findings.push(Finding {
                    title: format!("High execution time in {}", process_id),
                    description: format!(
                        "Process {} has an average execution time of {:.0}ms",
                        process_id, metrics.avg_execution_time_ms
                    ),
                    severity: "warning".to_string(),
                    impact: "Performance degradation".to_string(),
                });
            }

            let failure_rate = if metrics.total_instances > 0 {
                (metrics.failed_instances as f64 / metrics.total_instances as f64) * 100.0
            } else {
                0.0
            };

            if failure_rate > 10.0 {
                findings.push(Finding {
                    title: format!("High failure rate in {}", process_id),
                    description: format!(
                        "Process {} has a failure rate of {:.1}%",
                        process_id, failure_rate
                    ),
                    severity: "critical".to_string(),
                    impact: "Service reliability".to_string(),
                });
            }
        }

        let recommendations = vec![
            "Monitor process execution times closely".to_string(),
            "Consider process optimization if average execution times exceed SLA".to_string(),
            "Review failure causes and implement mitigations".to_string(),
            "Analyze cost trends to identify optimization opportunities".to_string(),
        ];

        AnalyticsReport {
            title: title.into(),
            description: Some("Analytics report generated from metrics snapshot".to_string()),
            generated_at: chrono::Utc::now(),
            period: ReportPeriod {
                start: snapshot.timestamp - chrono::Duration::days(7),
                end: snapshot.timestamp,
            },
            executive_summary: ExecutiveSummary {
                total_processes,
                success_rate,
                avg_execution_time_ms: avg_exec_time,
                total_cost,
            },
            findings,
            recommendations,
        }
    }
}

impl Default for ReportGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Export data in different formats
pub struct ExportManager;

impl ExportManager {
    /// Export snapshot in specified format
    pub fn export(
        snapshot: &MetricSnapshot,
        format: ExportFormat,
    ) -> Result<String> {
        match format {
            ExportFormat::Csv => {
                let exporter = CsvExporter::new();
                exporter.export(snapshot)
            }
            ExportFormat::Json => {
                let exporter = JsonExporter::new();
                exporter.export(snapshot)
            }
            ExportFormat::Yaml => {
                let json_str = serde_json::to_value(snapshot)?;
                let yaml = serde_yaml::to_string(&json_str)
                    .map_err(|e| {
                        crate::error::AbcdodafError::YamlSerializationError(e.to_string())
                    })?;
                Ok(yaml)
            }
        }
    }

    /// Export to file
    pub fn export_to_file(
        snapshot: &MetricSnapshot,
        path: &std::path::Path,
        format: ExportFormat,
    ) -> Result<()> {
        let content = Self::export(snapshot, format)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Generate and export report
    pub fn generate_report(
        snapshot: &MetricSnapshot,
        title: impl Into<String>,
        format: ExportFormat,
    ) -> Result<String> {
        let generator = ReportGenerator::new();
        let report = generator.generate_report(snapshot, title);

        match format {
            ExportFormat::Json => Ok(serde_json::to_string_pretty(&report)?),
            ExportFormat::Yaml => {
                let json = serde_json::to_value(&report)?;
                let yaml = serde_yaml::to_string(&json)
                    .map_err(|e| {
                        crate::error::AbcdodafError::YamlSerializationError(e.to_string())
                    })?;
                Ok(yaml)
            }
            ExportFormat::Csv => {
                // For reports, JSON is more appropriate
                Ok(serde_json::to_string_pretty(&report)?)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::metrics::ProcessMetrics;
    use chrono::Utc;
    use std::collections::HashMap;

    #[test]
    fn test_csv_exporter() {
        let mut snapshot = MetricSnapshot {
            timestamp: Utc::now(),
            process_metrics: HashMap::new(),
            performance_metrics: super::super::metrics::PerformanceMetrics::default(),
            cost_metrics: HashMap::new(),
            recent_metrics: Vec::new(),
        };

        let mut metrics = ProcessMetrics::new("test_process");
        metrics.record_completion(100.0);
        snapshot.process_metrics.insert("test_process".to_string(), metrics);

        let exporter = CsvExporter::new();
        let csv = exporter.export(&snapshot).unwrap();
        assert!(csv.contains("test_process"));
    }

    #[test]
    fn test_json_exporter() {
        let snapshot = MetricSnapshot {
            timestamp: Utc::now(),
            process_metrics: HashMap::new(),
            performance_metrics: super::super::metrics::PerformanceMetrics::default(),
            cost_metrics: HashMap::new(),
            recent_metrics: Vec::new(),
        };

        let exporter = JsonExporter::new();
        let json = exporter.export(&snapshot).unwrap();
        assert!(json.contains("process_metrics"));
    }

    #[test]
    fn test_report_generation() {
        let snapshot = MetricSnapshot {
            timestamp: Utc::now(),
            process_metrics: HashMap::new(),
            performance_metrics: super::super::metrics::PerformanceMetrics::default(),
            cost_metrics: HashMap::new(),
            recent_metrics: Vec::new(),
        };

        let generator = ReportGenerator::new();
        let report = generator.generate_report(&snapshot, "Test Report");
        assert_eq!(report.title, "Test Report");
    }
}
