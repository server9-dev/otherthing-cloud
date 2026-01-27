//! Code coverage tracking for workflow paths

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Coverage tracker for workflow execution paths
pub struct CoverageTracker {
    /// Tracked paths
    paths: Arc<RwLock<HashMap<String, PathInfo>>>,
    /// Current execution context
    context: Arc<RwLock<String>>,
}

/// Information about a code path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathInfo {
    /// Path identifier
    pub path_id: String,
    /// Human readable name
    pub name: String,
    /// Number of times executed
    pub execution_count: u64,
    /// Conditions that lead to this path
    pub conditions: Vec<String>,
    /// Whether this is a critical path
    pub is_critical: bool,
    /// Last execution timestamp
    pub last_executed: Option<chrono::DateTime<chrono::Utc>>,
}

/// Path coverage report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathCoverage {
    /// Total paths defined
    pub total_paths: usize,
    /// Paths executed at least once
    pub covered_paths: usize,
    /// Coverage percentage (0.0 to 1.0)
    pub coverage_percent: f64,
    /// Coverage details by path
    pub paths: Vec<PathInfo>,
    /// Critical paths missed
    pub missed_critical_paths: Vec<String>,
}

impl CoverageTracker {
    /// Create new coverage tracker
    pub fn new() -> Self {
        Self {
            paths: Arc::new(RwLock::new(HashMap::new())),
            context: Arc::new(RwLock::new(String::new())),
        }
    }

    /// Register a code path
    pub async fn register_path(&self, path_id: impl Into<String>, name: impl Into<String>) {
        let path_id = path_id.into();
        let name = name.into();

        let mut paths = self.paths.write().await;
        paths.entry(path_id.clone()).or_insert_with(|| PathInfo {
            path_id,
            name,
            execution_count: 0,
            conditions: Vec::new(),
            is_critical: false,
            last_executed: None,
        });
    }

    /// Mark a path as critical
    pub async fn mark_critical(&self, path_id: &str) {
        let mut paths = self.paths.write().await;
        if let Some(path) = paths.get_mut(path_id) {
            path.is_critical = true;
        }
    }

    /// Add condition to a path
    pub async fn add_condition(&self, path_id: &str, condition: impl Into<String>) {
        let mut paths = self.paths.write().await;
        if let Some(path) = paths.get_mut(path_id) {
            path.conditions.push(condition.into());
        }
    }

    /// Execute a path (increment counter)
    pub async fn execute_path(&self, path_id: &str) {
        let mut paths = self.paths.write().await;
        if let Some(path) = paths.get_mut(path_id) {
            path.execution_count += 1;
            path.last_executed = Some(chrono::Utc::now());
        }
    }

    /// Set current execution context
    pub async fn set_context(&self, context: impl Into<String>) {
        *self.context.write().await = context.into();
    }

    /// Get current execution context
    pub async fn get_context(&self) -> String {
        self.context.read().await.clone()
    }

    /// Generate coverage report
    pub async fn generate_report(&self) -> PathCoverage {
        let paths = self.paths.read().await;

        let total_paths = paths.len();
        let covered_paths = paths.values().filter(|p| p.execution_count > 0).count();
        let coverage_percent = if total_paths > 0 {
            (covered_paths as f64) / (total_paths as f64)
        } else {
            0.0
        };

        let mut path_list: Vec<_> = paths.values().cloned().collect();
        path_list.sort_by(|a, b| b.execution_count.cmp(&a.execution_count));

        let missed_critical_paths = paths
            .values()
            .filter(|p| p.is_critical && p.execution_count == 0)
            .map(|p| p.path_id.clone())
            .collect();

        PathCoverage {
            total_paths,
            covered_paths,
            coverage_percent,
            paths: path_list,
            missed_critical_paths,
        }
    }

    /// Clear all tracking data
    pub async fn clear(&self) {
        self.paths.write().await.clear();
    }

    /// Get coverage percent
    pub async fn coverage_percent(&self) -> f64 {
        let paths = self.paths.read().await;
        let total = paths.len();
        if total == 0 {
            return 0.0;
        }

        let covered = paths.values().filter(|p| p.execution_count > 0).count();
        (covered as f64) / (total as f64)
    }
}

impl Default for CoverageTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Coverage metrics analyzer
pub struct CoverageAnalyzer;

impl CoverageAnalyzer {
    /// Analyze coverage and identify gaps
    pub fn analyze_coverage(report: &PathCoverage) -> CoverageAnalysis {
        let critical_gap = !report.missed_critical_paths.is_empty();

        let confidence = if report.coverage_percent >= 0.95 {
            "High".to_string()
        } else if report.coverage_percent >= 0.8 {
            "Medium".to_string()
        } else {
            "Low".to_string()
        };

        let recommendations = Self::get_recommendations(report);

        CoverageAnalysis {
            coverage_percent: report.coverage_percent,
            has_critical_gaps: critical_gap,
            confidence,
            recommendations,
            total_paths: report.total_paths,
            covered_paths: report.covered_paths,
        }
    }

    fn get_recommendations(report: &PathCoverage) -> Vec<String> {
        let mut recs = Vec::new();

        if !report.missed_critical_paths.is_empty() {
            recs.push(format!(
                "Add tests for critical paths: {}",
                report.missed_critical_paths.join(", ")
            ));
        }

        if report.coverage_percent < 0.8 {
            recs.push(format!(
                "Coverage is below 80% (currently {:.1}%). Add more test cases.",
                report.coverage_percent * 100.0
            ));
        }

        if report.coverage_percent < 0.95 {
            let uncovered = report.total_paths - report.covered_paths;
            recs.push(format!(
                "Consider adding tests for {} uncovered paths",
                uncovered
            ));
        }

        recs
    }

    /// Compare two coverage reports
    pub fn compare_coverage(before: &PathCoverage, after: &PathCoverage) -> CoverageComparison {
        let coverage_improvement = after.coverage_percent - before.coverage_percent;
        let new_paths_covered = after.covered_paths - before.covered_paths;
        let new_total_paths = after.total_paths - before.total_paths;

        let status = if coverage_improvement > 0.0 {
            "Improved".to_string()
        } else if coverage_improvement < 0.0 {
            "Regressed".to_string()
        } else {
            "Unchanged".to_string()
        };

        CoverageComparison {
            before: before.coverage_percent,
            after: after.coverage_percent,
            improvement: coverage_improvement,
            new_paths_covered,
            new_total_paths,
            status,
        }
    }
}

/// Coverage analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageAnalysis {
    /// Current coverage percentage
    pub coverage_percent: f64,
    /// Whether there are uncovered critical paths
    pub has_critical_gaps: bool,
    /// Confidence level
    pub confidence: String,
    /// Recommendations for improvement
    pub recommendations: Vec<String>,
    /// Total paths in analysis
    pub total_paths: usize,
    /// Covered paths
    pub covered_paths: usize,
}

/// Comparison between two coverage reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageComparison {
    /// Coverage before
    pub before: f64,
    /// Coverage after
    pub after: f64,
    /// Improvement
    pub improvement: f64,
    /// New paths covered
    pub new_paths_covered: usize,
    /// New total paths
    pub new_total_paths: usize,
    /// Status
    pub status: String,
}

/// Branch coverage tracker
pub struct BranchCoverage {
    /// All branches tracked
    branches: Arc<RwLock<HashMap<String, BranchInfo>>>,
}

/// Information about a branch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    /// Branch ID
    pub branch_id: String,
    /// Branch name
    pub name: String,
    /// Whether branch was taken
    pub taken: bool,
    /// Number of times taken
    pub taken_count: u64,
}

impl BranchCoverage {
    /// Create new branch coverage tracker
    pub fn new() -> Self {
        Self {
            branches: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a branch
    pub async fn register_branch(&self, branch_id: impl Into<String>, name: impl Into<String>) {
        let branch_id = branch_id.into();
        let name = name.into();

        let mut branches = self.branches.write().await;
        branches.entry(branch_id.clone()).or_insert_with(|| BranchInfo {
            branch_id,
            name,
            taken: false,
            taken_count: 0,
        });
    }

    /// Mark branch as taken
    pub async fn take_branch(&self, branch_id: &str) {
        let mut branches = self.branches.write().await;
        if let Some(branch) = branches.get_mut(branch_id) {
            branch.taken = true;
            branch.taken_count += 1;
        }
    }

    /// Get branch coverage percentage
    pub async fn coverage_percent(&self) -> f64 {
        let branches = self.branches.read().await;
        let total = branches.len();
        if total == 0 {
            return 0.0;
        }

        let taken = branches.values().filter(|b| b.taken).count();
        (taken as f64) / (total as f64)
    }

    /// Get all branch info
    pub async fn get_branches(&self) -> Vec<BranchInfo> {
        self.branches
            .read()
            .await
            .values()
            .cloned()
            .collect()
    }
}

impl Default for BranchCoverage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_coverage_tracker() {
        let tracker = CoverageTracker::new();

        tracker.register_path("path1", "Path 1").await;
        tracker.register_path("path2", "Path 2").await;
        tracker.mark_critical("path1").await;

        tracker.execute_path("path1").await;

        assert_eq!(tracker.coverage_percent().await, 0.5);
    }

    #[tokio::test]
    async fn test_path_coverage_report() {
        let tracker = CoverageTracker::new();

        tracker.register_path("p1", "Path 1").await;
        tracker.register_path("p2", "Path 2").await;
        tracker.register_path("p3", "Path 3").await;

        tracker.execute_path("p1").await;
        tracker.execute_path("p3").await;

        let report = tracker.generate_report().await;
        assert_eq!(report.total_paths, 3);
        assert_eq!(report.covered_paths, 2);
    }

    #[test]
    fn test_coverage_analysis() {
        let report = PathCoverage {
            total_paths: 10,
            covered_paths: 8,
            coverage_percent: 0.8,
            paths: Vec::new(),
            missed_critical_paths: Vec::new(),
        };

        let analysis = CoverageAnalyzer::analyze_coverage(&report);
        assert_eq!(analysis.coverage_percent, 0.8);
        assert_eq!(analysis.confidence, "Medium");
    }

    #[tokio::test]
    async fn test_branch_coverage() {
        let coverage = BranchCoverage::new();

        coverage.register_branch("b1", "Branch 1").await;
        coverage.register_branch("b2", "Branch 2").await;

        coverage.take_branch("b1").await;

        assert_eq!(coverage.coverage_percent().await, 0.5);
    }
}
