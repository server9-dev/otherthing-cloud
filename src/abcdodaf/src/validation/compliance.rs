//! Compliance scoring and reporting
//!
//! Calculate compliance scores and generate detailed compliance reports.

use super::engine::ValidationResult;
use super::standards::Standard;
use super::violations::{Violation, ViolationSeverity};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Compliance level categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ComplianceLevel {
    /// Non-compliant (< 60%)
    NonCompliant,
    /// Partially compliant (60-79%)
    PartiallyCompliant,
    /// Substantially compliant (80-94%)
    SubstantiallyCompliant,
    /// Fully compliant (95-99%)
    FullyCompliant,
    /// Perfect (100%)
    Perfect,
}

impl ComplianceLevel {
    /// Get compliance level from score percentage
    pub fn from_percentage(percentage: f64) -> Self {
        if percentage >= 100.0 {
            Self::Perfect
        } else if percentage >= 95.0 {
            Self::FullyCompliant
        } else if percentage >= 80.0 {
            Self::SubstantiallyCompliant
        } else if percentage >= 60.0 {
            Self::PartiallyCompliant
        } else {
            Self::NonCompliant
        }
    }

    /// Get display string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Perfect => "Perfect",
            Self::FullyCompliant => "Fully Compliant",
            Self::SubstantiallyCompliant => "Substantially Compliant",
            Self::PartiallyCompliant => "Partially Compliant",
            Self::NonCompliant => "Non-Compliant",
        }
    }
}

/// Compliance score calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceScore {
    /// Overall percentage score (0-100)
    pub percentage: f64,
    /// Compliance level
    pub level: ComplianceLevel,
    /// Total possible points
    pub total_points: u32,
    /// Points earned
    pub points_earned: u32,
    /// Points deducted
    pub points_deducted: u32,
    /// Breakdown by severity
    pub severity_breakdown: HashMap<String, u32>,
}

impl ComplianceScore {
    /// Calculate score from validation result
    pub fn from_validation_result(result: &ValidationResult) -> Self {
        let mut total_points = 0u32;
        let mut points_deducted = 0u32;
        let mut severity_breakdown = HashMap::new();

        // Calculate points based on rule severity
        for rule_result in &result.results {
            if let Some(ref violation) = rule_result.violation {
                let points = (violation.severity as ViolationSeverity).score();
                total_points += points;
                points_deducted += points;

                let severity_key = format!("{:?}", violation.severity);
                *severity_breakdown.entry(severity_key).or_insert(0) += points;
            } else {
                // Rule passed, add to total but not deducted
                total_points += 5; // Assume average weight for passed rules
            }
        }

        let points_earned = total_points.saturating_sub(points_deducted);
        let percentage = if total_points > 0 {
            (points_earned as f64 / total_points as f64) * 100.0
        } else {
            100.0
        };

        let level = ComplianceLevel::from_percentage(percentage);

        Self {
            percentage,
            level,
            total_points,
            points_earned,
            points_deducted,
            severity_breakdown,
        }
    }

    /// Get grade letter (A-F)
    pub fn grade(&self) -> char {
        if self.percentage >= 90.0 {
            'A'
        } else if self.percentage >= 80.0 {
            'B'
        } else if self.percentage >= 70.0 {
            'C'
        } else if self.percentage >= 60.0 {
            'D'
        } else {
            'F'
        }
    }
}

/// Comprehensive compliance report
#[derive(Debug, Serialize, Deserialize)]
pub struct ComplianceReport {
    /// Standard being checked
    pub standard_id: String,
    /// Standard name
    pub standard_name: String,
    /// Standard version
    pub standard_version: String,
    /// Overall compliance score
    pub score: ComplianceScore,
    /// Validation results
    pub validation_summary: ValidationSummary,
    /// All violations
    pub violations: Vec<Violation>,
    /// Recommendations
    pub recommendations: Vec<String>,
    /// Report generation timestamp
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

/// Validation summary
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationSummary {
    /// Total rules checked
    pub total_rules: usize,
    /// Rules passed
    pub passed: usize,
    /// Rules failed
    pub failed: usize,
    /// Pass rate percentage
    pub pass_rate: f64,
    /// Critical violations
    pub critical_count: usize,
    /// Error violations
    pub error_count: usize,
    /// Warning violations
    pub warning_count: usize,
    /// Info violations
    pub info_count: usize,
}

impl ValidationSummary {
    /// Create from validation result
    pub fn from_validation_result(result: &ValidationResult) -> Self {
        Self {
            total_rules: result.total_rules,
            passed: result.passed,
            failed: result.failed,
            pass_rate: result.pass_rate(),
            critical_count: result.critical_count,
            error_count: result.error_count,
            warning_count: result.warning_count,
            info_count: result.results.len() - result.critical_count - result.error_count - result.warning_count,
        }
    }
}

impl ComplianceReport {
    /// Generate a compliance report
    pub fn generate(
        standard: &Standard,
        validation_result: &ValidationResult,
    ) -> Self {
        let score = ComplianceScore::from_validation_result(validation_result);
        let validation_summary = ValidationSummary::from_validation_result(validation_result);
        let violations = validation_result.violations().into_iter().cloned().collect();

        let mut recommendations = Vec::new();

        // Generate recommendations based on violations
        if validation_result.critical_count > 0 {
            recommendations.push(format!(
                "Address {} critical violation(s) immediately",
                validation_result.critical_count
            ));
        }

        if validation_result.error_count > 0 {
            recommendations.push(format!(
                "Fix {} error(s) to improve compliance",
                validation_result.error_count
            ));
        }

        if score.percentage < 80.0 {
            recommendations.push(
                "Consider reviewing standard requirements and implementing missing elements".to_string()
            );
        }

        Self {
            standard_id: standard.id.clone(),
            standard_name: standard.name.clone(),
            standard_version: standard.version.version.clone(),
            score,
            validation_summary,
            violations,
            recommendations,
            generated_at: chrono::Utc::now(),
        }
    }

    /// Export report to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Generate markdown summary
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        md.push_str(&format!("# {} Compliance Report\n\n", self.standard_name));
        md.push_str(&format!("**Version:** {}\n", self.standard_version));
        md.push_str(&format!("**Generated:** {}\n\n", self.generated_at.format("%Y-%m-%d %H:%M:%S UTC")));

        md.push_str("## Overall Score\n\n");
        md.push_str(&format!("- **Score:** {:.1}% (Grade: {})\n", self.score.percentage, self.score.grade()));
        md.push_str(&format!("- **Level:** {}\n", self.score.level.as_str()));
        md.push_str(&format!("- **Points:** {}/{}\n\n", self.score.points_earned, self.score.total_points));

        md.push_str("## Validation Summary\n\n");
        md.push_str(&format!("- **Total Rules:** {}\n", self.validation_summary.total_rules));
        md.push_str(&format!("- **Passed:** {} ({:.1}%)\n", self.validation_summary.passed, self.validation_summary.pass_rate));
        md.push_str(&format!("- **Failed:** {}\n\n", self.validation_summary.failed));

        md.push_str("### Violations by Severity\n\n");
        md.push_str(&format!("- **Critical:** {}\n", self.validation_summary.critical_count));
        md.push_str(&format!("- **Errors:** {}\n", self.validation_summary.error_count));
        md.push_str(&format!("- **Warnings:** {}\n\n", self.validation_summary.warning_count));

        if !self.violations.is_empty() {
            md.push_str("## Violations\n\n");
            for violation in &self.violations {
                md.push_str(&format!("### {}\n\n", violation.rule_name));
                md.push_str(&format!("- **Severity:** {:?}\n", violation.severity));
                md.push_str(&format!("- **Location:** {}\n", violation.location));
                md.push_str(&format!("- **Message:** {}\n", violation.message));
                if let Some(ref fix) = violation.fix_suggestion {
                    md.push_str(&format!("- **Fix:** {}\n", fix));
                }
                md.push_str("\n");
            }
        }

        if !self.recommendations.is_empty() {
            md.push_str("## Recommendations\n\n");
            for rec in &self.recommendations {
                md.push_str(&format!("- {}\n", rec));
            }
        }

        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compliance_level() {
        assert_eq!(ComplianceLevel::from_percentage(100.0), ComplianceLevel::Perfect);
        assert_eq!(ComplianceLevel::from_percentage(95.0), ComplianceLevel::FullyCompliant);
        assert_eq!(ComplianceLevel::from_percentage(85.0), ComplianceLevel::SubstantiallyCompliant);
        assert_eq!(ComplianceLevel::from_percentage(65.0), ComplianceLevel::PartiallyCompliant);
        assert_eq!(ComplianceLevel::from_percentage(50.0), ComplianceLevel::NonCompliant);
    }

    #[test]
    fn test_compliance_score_grade() {
        let score = ComplianceScore {
            percentage: 92.0,
            level: ComplianceLevel::FullyCompliant,
            total_points: 100,
            points_earned: 92,
            points_deducted: 8,
            severity_breakdown: HashMap::new(),
        };

        assert_eq!(score.grade(), 'A');
    }
}
