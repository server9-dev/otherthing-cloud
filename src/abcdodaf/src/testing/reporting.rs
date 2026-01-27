//! Test report generation and formatting

use crate::testing::harness::TestSummary;
use crate::testing::performance::BenchmarkResult;
use crate::testing::coverage::PathCoverage;
use crate::testing::assertions::AssertionResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Complete test report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestReport {
    /// Report ID
    pub id: String,
    /// Report title
    pub title: String,
    /// Test suite name
    pub suite_name: String,
    /// When report was generated
    pub generated_at: DateTime<Utc>,
    /// Test summary
    pub summary: TestSummary,
    /// Detailed test results
    pub test_results: Vec<TestResultDetail>,
    /// Performance metrics
    pub performance: Option<PerformanceSection>,
    /// Coverage metrics
    pub coverage: Option<CoverageSection>,
    /// System information
    pub system_info: SystemInfo,
}

/// Individual test result details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResultDetail {
    /// Test ID
    pub test_id: String,
    /// Test name
    pub test_name: String,
    /// Pass/fail status
    pub status: TestStatus,
    /// Execution duration
    pub duration_ms: u64,
    /// Error message if failed
    pub error: Option<String>,
    /// Assertions that were checked
    pub assertions: Vec<AssertionResult>,
    /// Category tags
    pub tags: Vec<String>,
}

/// Test status enum
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TestStatus {
    /// Test passed
    Passed,
    /// Test failed
    Failed,
    /// Test was skipped
    Skipped,
    /// Test encountered an error
    Error,
}

impl std::fmt::Display for TestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestStatus::Passed => write!(f, "PASSED"),
            TestStatus::Failed => write!(f, "FAILED"),
            TestStatus::Skipped => write!(f, "SKIPPED"),
            TestStatus::Error => write!(f, "ERROR"),
        }
    }
}

/// Performance section of report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSection {
    /// Benchmark results
    pub benchmarks: Vec<BenchmarkResult>,
    /// Average throughput
    pub average_throughput: f64,
    /// Performance notes
    pub notes: Vec<String>,
}

/// Coverage section of report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageSection {
    /// Path coverage data
    pub path_coverage: PathCoverage,
    /// Coverage percentage
    pub coverage_percent: f64,
    /// Coverage status
    pub status: CoverageStatus,
}

/// Coverage status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CoverageStatus {
    /// Excellent coverage
    Excellent,
    /// Good coverage
    Good,
    /// Acceptable coverage
    Acceptable,
    /// Poor coverage
    Poor,
}

impl CoverageStatus {
    /// Get status from percentage
    pub fn from_percent(percent: f64) -> Self {
        if percent >= 0.95 {
            CoverageStatus::Excellent
        } else if percent >= 0.8 {
            CoverageStatus::Good
        } else if percent >= 0.6 {
            CoverageStatus::Acceptable
        } else {
            CoverageStatus::Poor
        }
    }
}

/// System information in report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Operating system
    pub os: String,
    /// Architecture
    pub arch: String,
    /// Test runner version
    pub test_framework_version: String,
    /// Environment variables relevant to testing
    pub environment: HashMap<String, String>,
}

impl TestReport {
    /// Create new test report
    pub fn new(
        suite_name: impl Into<String>,
        summary: TestSummary,
    ) -> Self {
        let suite_name_str = suite_name.into();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: format!("Test Report: {}", suite_name_str),
            suite_name: suite_name_str,
            generated_at: Utc::now(),
            summary,
            test_results: Vec::new(),
            performance: None,
            coverage: None,
            system_info: SystemInfo::default(),
        }
    }

    /// Add test result
    pub fn with_test_result(mut self, result: TestResultDetail) -> Self {
        self.test_results.push(result);
        self
    }

    /// Add performance section
    pub fn with_performance(mut self, perf: PerformanceSection) -> Self {
        self.performance = Some(perf);
        self
    }

    /// Add coverage section
    pub fn with_coverage(mut self, coverage: CoverageSection) -> Self {
        self.coverage = Some(coverage);
        self
    }

    /// Get HTML representation
    pub fn to_html(&self) -> String {
        let mut html = String::new();

        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("<meta charset=\"utf-8\">\n");
        html.push_str(&format!("<title>{}</title>\n", self.title));
        html.push_str("<style>\n");
        html.push_str(Self::css_styles());
        html.push_str("</style>\n");
        html.push_str("</head>\n<body>\n");

        // Header
        html.push_str(&format!("<h1>{}</h1>\n", self.title));
        html.push_str(&format!(
            "<p>Generated: {}</p>\n",
            self.generated_at.format("%Y-%m-%d %H:%M:%S UTC")
        ));

        // Summary section
        html.push_str("<h2>Summary</h2>\n");
        html.push_str(&self.summary_to_html());

        // Results section
        if !self.test_results.is_empty() {
            html.push_str("<h2>Test Results</h2>\n");
            html.push_str(&self.results_to_html());
        }

        // Performance section
        if let Some(perf) = &self.performance {
            html.push_str("<h2>Performance</h2>\n");
            html.push_str(&self.performance_to_html(perf));
        }

        // Coverage section
        if let Some(coverage) = &self.coverage {
            html.push_str("<h2>Coverage</h2>\n");
            html.push_str(&self.coverage_to_html(coverage));
        }

        html.push_str("</body>\n</html>");
        html
    }

    /// Get JSON representation
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    fn summary_to_html(&self) -> String {
        let pass_rate = (self.summary.pass_rate * 100.0) as u32;
        let status_color = if pass_rate >= 90 { "green" } else if pass_rate >= 70 { "orange" } else { "red" };

        format!(
            "<div class=\"summary-box\">\
            <div class=\"stat\">Total Tests: <strong>{}</strong></div>\
            <div class=\"stat\">Passed: <strong style=\"color: green;\">{}</strong></div>\
            <div class=\"stat\">Failed: <strong style=\"color: red;\">{}</strong></div>\
            <div class=\"stat\">Duration: <strong>{}</strong>ms</div>\
            <div class=\"stat\">Pass Rate: <strong style=\"color: {};\">{}</strong>%</div>\
            </div>",
            self.summary.total_tests,
            self.summary.passed_tests,
            self.summary.failed_tests,
            self.summary.total_duration_ms,
            status_color,
            pass_rate
        )
    }

    fn results_to_html(&self) -> String {
        let mut html = String::from("<table class=\"results-table\">\n<tr><th>Test</th><th>Status</th><th>Duration</th><th>Error</th></tr>\n");

        for result in &self.test_results {
            let status_class = match result.status {
                TestStatus::Passed => "passed",
                TestStatus::Failed => "failed",
                TestStatus::Error => "error",
                TestStatus::Skipped => "skipped",
            };

            html.push_str(&format!(
                "<tr><td>{}</td><td class=\"{}\">{}</td><td>{}ms</td><td>{}</td></tr>\n",
                result.test_name,
                status_class,
                result.status,
                result.duration_ms,
                result.error.as_deref().unwrap_or("")
            ));
        }

        html.push_str("</table>\n");
        html
    }

    fn performance_to_html(&self, perf: &PerformanceSection) -> String {
        let mut html = String::from("<div class=\"performance\">\n");

        html.push_str(&format!(
            "<p>Average Throughput: <strong>{:.2}</strong> ops/sec</p>\n",
            perf.average_throughput
        ));

        if !perf.benchmarks.is_empty() {
            html.push_str("<table class=\"benchmark-table\">\n<tr><th>Benchmark</th><th>Avg Duration</th><th>Min</th><th>Max</th><th>Ops/sec</th></tr>\n");

            for bench in &perf.benchmarks {
                html.push_str(&format!(
                    "<tr><td>{}</td><td>{:.2}ms</td><td>{}ms</td><td>{}ms</td><td>{:.2}</td></tr>\n",
                    bench.name,
                    bench.metrics.average_duration(),
                    bench.metrics.min_duration().unwrap_or(0),
                    bench.metrics.max_duration().unwrap_or(0),
                    bench.ops_per_second
                ));
            }

            html.push_str("</table>\n");
        }

        html.push_str("</div>\n");
        html
    }

    fn coverage_to_html(&self, coverage: &CoverageSection) -> String {
        let coverage_percent = (coverage.coverage_percent * 100.0) as u32;
        let status_color = match coverage.status {
            CoverageStatus::Excellent => "green",
            CoverageStatus::Good => "lightgreen",
            CoverageStatus::Acceptable => "orange",
            CoverageStatus::Poor => "red",
        };

        format!(
            "<div class=\"coverage-box\">\
            <div class=\"coverage-bar\" style=\"background-color: {}; width: {}%;\"></div>\
            <p>Code Coverage: <strong style=\"color: {};\">{}</strong>% ({}/{})</p>\
            <p>Status: <strong>{:?}</strong></p>\
            </div>",
            status_color,
            coverage_percent,
            status_color,
            coverage_percent,
            coverage.path_coverage.covered_paths,
            coverage.path_coverage.total_paths,
            coverage.status
        )
    }

    fn css_styles() -> &'static str {
        r#"
        body { font-family: Arial, sans-serif; margin: 20px; background-color: #f5f5f5; }
        h1, h2 { color: #333; }
        .summary-box { background: white; padding: 15px; border-radius: 5px; margin: 10px 0; }
        .stat { margin: 5px 0; }
        .results-table, .benchmark-table { width: 100%; border-collapse: collapse; margin: 10px 0; background: white; }
        .results-table th, .benchmark-table th { background: #333; color: white; padding: 10px; text-align: left; }
        .results-table td, .benchmark-table td { padding: 10px; border-bottom: 1px solid #ddd; }
        .results-table tr:hover, .benchmark-table tr:hover { background: #f9f9f9; }
        .passed { color: green; font-weight: bold; }
        .failed { color: red; font-weight: bold; }
        .error { color: darkred; font-weight: bold; }
        .skipped { color: gray; font-weight: bold; }
        .coverage-box { background: white; padding: 15px; border-radius: 5px; margin: 10px 0; }
        .coverage-bar { height: 30px; border-radius: 3px; }
        "#
    }
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            test_framework_version: crate::VERSION.to_string(),
            environment: HashMap::new(),
        }
    }
}

/// Report generator
pub struct ReportGenerator;

impl ReportGenerator {
    /// Generate JSON report
    pub fn generate_json(report: &TestReport) -> Result<String, serde_json::Error> {
        report.to_json()
    }

    /// Generate HTML report
    pub fn generate_html(report: &TestReport) -> String {
        report.to_html()
    }

    /// Save report to file
    pub fn save_html_report(report: &TestReport, path: &str) -> std::io::Result<()> {
        let html = report.to_html();
        std::fs::write(path, html)
    }

    /// Save report to file
    pub fn save_json_report(report: &TestReport, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = report.to_json()?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

/// Coverage report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    /// Total lines
    pub total_lines: usize,
    /// Covered lines
    pub covered_lines: usize,
    /// Branch coverage
    pub branch_coverage: f64,
    /// Function coverage
    pub function_coverage: f64,
    /// Line coverage
    pub line_coverage: f64,
}

impl CoverageReport {
    /// Create new coverage report
    pub fn new() -> Self {
        Self {
            total_lines: 0,
            covered_lines: 0,
            branch_coverage: 0.0,
            function_coverage: 0.0,
            line_coverage: 0.0,
        }
    }

    /// Calculate line coverage
    pub fn calculate_line_coverage(&mut self) {
        if self.total_lines > 0 {
            self.line_coverage = self.covered_lines as f64 / self.total_lines as f64;
        }
    }
}

impl Default for CoverageReport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_report_creation() {
        let summary = TestSummary {
            total_tests: 10,
            passed_tests: 9,
            failed_tests: 1,
            total_duration_ms: 1000,
            pass_rate: 0.9,
        };

        let report = TestReport::new("test_suite", summary);
        assert_eq!(report.suite_name, "test_suite");
    }

    #[test]
    fn test_coverage_status_from_percent() {
        assert_eq!(CoverageStatus::from_percent(0.95), CoverageStatus::Excellent);
        assert_eq!(CoverageStatus::from_percent(0.8), CoverageStatus::Good);
        assert_eq!(CoverageStatus::from_percent(0.6), CoverageStatus::Acceptable);
        assert_eq!(CoverageStatus::from_percent(0.5), CoverageStatus::Poor);
    }

    #[test]
    fn test_test_status_display() {
        assert_eq!(TestStatus::Passed.to_string(), "PASSED");
        assert_eq!(TestStatus::Failed.to_string(), "FAILED");
    }

    #[test]
    fn test_report_html_generation() {
        let summary = TestSummary {
            total_tests: 5,
            passed_tests: 5,
            failed_tests: 0,
            total_duration_ms: 500,
            pass_rate: 1.0,
        };

        let report = TestReport::new("test", summary);
        let html = report.to_html();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Test Report"));
    }

    #[test]
    fn test_report_json_generation() {
        let summary = TestSummary {
            total_tests: 5,
            passed_tests: 5,
            failed_tests: 0,
            total_duration_ms: 500,
            pass_rate: 1.0,
        };

        let report = TestReport::new("test", summary);
        let json = report.to_json();
        assert!(json.is_ok());
    }
}
