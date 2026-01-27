//! Comprehensive Testing Framework for ABCDODAF Workflows
//!
//! This module provides a complete testing solution for BPMN/DoDAF workflows including:
//! - Unit testing API for individual tasks
//! - Integration testing framework for complete workflows
//! - Mock task handler system
//! - Test data generation and fixtures
//! - Assertion framework for workflow outcomes
//! - Performance benchmarking tools
//! - Test scenario DSL (Domain-Specific Language)
//! - Code coverage tracking
//! - Test report generation

pub mod harness;
pub mod mock_handlers;
pub mod assertions;
pub mod fixtures;
pub mod dsl;
pub mod performance;
pub mod reporting;
pub mod coverage;

pub use harness::{TestHarness, TaskTestCase, TaskTestResult, TestSummary};
pub use mock_handlers::{MockTaskHandler, MockHandlerBuilder, RecordingMockHandler, MockBehavior};
pub use assertions::{WorkflowAssertions, TaskAssertions, AssertionResult};
pub use fixtures::{TestFixture, FixtureBuilder, SampleDataGenerator, FixturePool};
pub use dsl::{TestScenario, ScenarioBuilder, ScenarioDsl, StepBuilder, ValidationBuilder};
pub use performance::{PerformanceMetrics, BenchmarkSuite, BenchmarkResult, PerfTimer};
pub use reporting::{TestReport, ReportGenerator, CoverageReport, TestResultDetail, TestStatus, CoverageStatus};
pub use coverage::{CoverageTracker, PathCoverage, CoverageAnalyzer, BranchCoverage};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_testing_framework_loads() {
        // Basic sanity check that the framework loads
        assert!(std::mem::size_of::<TestHarness>() > 0);
    }
}
