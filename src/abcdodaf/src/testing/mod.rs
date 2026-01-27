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

pub mod assertions;
pub mod coverage;
pub mod dsl;
pub mod fixtures;
pub mod harness;
pub mod mock_handlers;
pub mod performance;
pub mod reporting;

pub use assertions::{AssertionResult, TaskAssertions, WorkflowAssertions};
pub use coverage::{BranchCoverage, CoverageAnalyzer, CoverageTracker, PathCoverage};
pub use dsl::{ScenarioBuilder, ScenarioDsl, StepBuilder, TestScenario, ValidationBuilder};
pub use fixtures::{FixtureBuilder, FixturePool, SampleDataGenerator, TestFixture};
pub use harness::{TaskTestCase, TaskTestResult, TestHarness, TestSummary};
pub use mock_handlers::{MockBehavior, MockHandlerBuilder, MockTaskHandler, RecordingMockHandler};
pub use performance::{BenchmarkResult, BenchmarkSuite, PerfTimer, PerformanceMetrics};
pub use reporting::{
    CoverageReport, CoverageStatus, ReportGenerator, TestReport, TestResultDetail, TestStatus,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_testing_framework_loads() {
        // Basic sanity check that the framework loads
        assert!(std::mem::size_of::<TestHarness>() > 0);
    }
}
