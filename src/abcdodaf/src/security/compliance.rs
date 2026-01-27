//! Compliance validation and reporting
//!
//! Provides framework for defining and validating compliance rules,
//! generating compliance reports, and enforcing compliance policies.

use crate::security::error::{SecurityError, SecurityResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Compliance framework standard
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComplianceStandard {
    /// SOC 2 Type II
    SOC2TypeII,
    /// ISO 27001
    ISO27001,
    /// HIPAA
    HIPAA,
    /// GDPR
    GDPR,
    /// PCI DSS
    PCIDSS,
    /// Custom
    Custom,
}

impl std::fmt::Display for ComplianceStandard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComplianceStandard::SOC2TypeII => write!(f, "SOC 2 Type II"),
            ComplianceStandard::ISO27001 => write!(f, "ISO 27001"),
            ComplianceStandard::HIPAA => write!(f, "HIPAA"),
            ComplianceStandard::GDPR => write!(f, "GDPR"),
            ComplianceStandard::PCIDSS => write!(f, "PCI DSS"),
            ComplianceStandard::Custom => write!(f, "Custom"),
        }
    }
}

/// Rule status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleStatus {
    /// Rule is compliant
    Compliant,
    /// Rule is not compliant
    NonCompliant,
    /// Rule status is unknown
    Unknown,
    /// Rule is not applicable
    NotApplicable,
}

impl std::fmt::Display for RuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuleStatus::Compliant => write!(f, "Compliant"),
            RuleStatus::NonCompliant => write!(f, "Non-Compliant"),
            RuleStatus::Unknown => write!(f, "Unknown"),
            RuleStatus::NotApplicable => write!(f, "Not Applicable"),
        }
    }
}

/// Compliance rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRule {
    /// Unique rule identifier
    pub id: String,
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: Option<String>,
    /// Applicable standards
    pub standards: Vec<ComplianceStandard>,
    /// Validation logic (description of what is checked)
    pub validation_logic: String,
    /// Current rule status
    pub status: RuleStatus,
    /// Last checked timestamp
    pub last_checked_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Remediation steps if non-compliant
    pub remediation_steps: Option<Vec<String>>,
    /// Evidence of compliance
    pub evidence: Option<String>,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

impl ComplianceRule {
    /// Create a new compliance rule
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            standards: Vec::new(),
            validation_logic: String::new(),
            status: RuleStatus::Unknown,
            last_checked_at: None,
            remediation_steps: None,
            evidence: None,
            metadata: HashMap::new(),
        }
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add applicable standard
    pub fn with_standard(mut self, standard: ComplianceStandard) -> Self {
        if !self.standards.contains(&standard) {
            self.standards.push(standard);
        }
        self
    }

    /// Set validation logic
    pub fn with_validation_logic(mut self, logic: impl Into<String>) -> Self {
        self.validation_logic = logic.into();
        self
    }

    /// Set remediation steps
    pub fn with_remediation(mut self, steps: Vec<String>) -> Self {
        self.remediation_steps = Some(steps);
        self
    }

    /// Mark as compliant
    pub fn mark_compliant(&mut self) {
        self.status = RuleStatus::Compliant;
        self.last_checked_at = Some(chrono::Utc::now());
    }

    /// Mark as non-compliant
    pub fn mark_non_compliant(&mut self) {
        self.status = RuleStatus::NonCompliant;
        self.last_checked_at = Some(chrono::Utc::now());
    }

    /// Set evidence
    pub fn with_evidence(mut self, evidence: impl Into<String>) -> Self {
        self.evidence = Some(evidence.into());
        self
    }
}

/// Compliance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    /// Report ID
    pub id: String,
    /// Report generation timestamp
    pub generated_at: chrono::DateTime<chrono::Utc>,
    /// Applicable standards
    pub standards: Vec<ComplianceStandard>,
    /// Total rules
    pub total_rules: usize,
    /// Compliant rules
    pub compliant_rules: usize,
    /// Non-compliant rules
    pub non_compliant_rules: usize,
    /// Unknown status rules
    pub unknown_rules: usize,
    /// Overall compliance score (0-100)
    pub compliance_score: f32,
    /// Rule details
    pub rules: Vec<ComplianceRule>,
    /// Summary findings
    pub findings: Vec<String>,
}

impl ComplianceReport {
    /// Create a new compliance report
    pub fn new(standards: Vec<ComplianceStandard>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            generated_at: chrono::Utc::now(),
            standards,
            total_rules: 0,
            compliant_rules: 0,
            non_compliant_rules: 0,
            unknown_rules: 0,
            compliance_score: 0.0,
            rules: Vec::new(),
            findings: Vec::new(),
        }
    }

    /// Calculate compliance score
    pub fn calculate_score(&mut self) {
        if self.total_rules == 0 {
            self.compliance_score = 100.0;
            return;
        }

        self.compliance_score = (self.compliant_rules as f32 / self.total_rules as f32) * 100.0;
    }

    /// Add finding
    pub fn add_finding(&mut self, finding: impl Into<String>) {
        self.findings.push(finding.into());
    }
}

/// Compliance validator
#[derive(Clone)]
pub struct ComplianceValidator {
    /// Stored rules
    rules: Arc<RwLock<HashMap<String, ComplianceRule>>>,
}

impl ComplianceValidator {
    /// Create a new compliance validator
    pub fn new() -> Self {
        Self { rules: Arc::new(RwLock::new(HashMap::new())) }
    }

    /// Initialize with standard rules
    pub async fn with_standard_rules() -> Self {
        let validator = Self::new();

        // Audit logging rule
        let audit_rule = ComplianceRule::new("audit_001", "Comprehensive Audit Logging")
            .with_description("All security-relevant events must be logged")
            .with_standard(ComplianceStandard::SOC2TypeII)
            .with_standard(ComplianceStandard::ISO27001)
            .with_validation_logic("Check that audit logger is enabled and events are recorded");

        // Access control rule
        let access_rule = ComplianceRule::new("access_001", "Role-Based Access Control")
            .with_description("Access to resources must be controlled via RBAC")
            .with_standard(ComplianceStandard::SOC2TypeII)
            .with_standard(ComplianceStandard::ISO27001)
            .with_standard(ComplianceStandard::GDPR)
            .with_validation_logic("Verify RBAC system is implemented and enforced");

        // Encryption rule
        let encryption_rule = ComplianceRule::new("encryption_001", "Data Encryption")
            .with_description("Sensitive data must be encrypted at rest and in transit")
            .with_standard(ComplianceStandard::SOC2TypeII)
            .with_standard(ComplianceStandard::HIPAA)
            .with_standard(ComplianceStandard::GDPR)
            .with_validation_logic("Check encryption of sensitive data");

        // Session management rule
        let session_rule = ComplianceRule::new("session_001", "Session Management")
            .with_description("User sessions must have proper timeout and termination")
            .with_standard(ComplianceStandard::SOC2TypeII)
            .with_validation_logic("Verify session timeout and termination mechanisms");

        // Secret management rule
        let secret_rule = ComplianceRule::new("secret_001", "Secret Management")
            .with_description("Credentials and secrets must be securely stored")
            .with_standard(ComplianceStandard::SOC2TypeII)
            .with_standard(ComplianceStandard::ISO27001)
            .with_validation_logic("Check that secrets are encrypted and access is logged");

        // Data retention rule
        let retention_rule = ComplianceRule::new("retention_001", "Data Retention")
            .with_description("Data must be retained according to policy")
            .with_standard(ComplianceStandard::GDPR)
            .with_validation_logic("Verify data retention policies are implemented");

        validator.add_rule(audit_rule).await.ok();
        validator.add_rule(access_rule).await.ok();
        validator.add_rule(encryption_rule).await.ok();
        validator.add_rule(session_rule).await.ok();
        validator.add_rule(secret_rule).await.ok();
        validator.add_rule(retention_rule).await.ok();

        validator
    }

    /// Add a compliance rule
    pub async fn add_rule(&self, rule: ComplianceRule) -> SecurityResult<()> {
        let mut rules = self.rules.write().await;
        rules.insert(rule.id.clone(), rule);
        Ok(())
    }

    /// Get a rule
    pub async fn get_rule(&self, rule_id: &str) -> SecurityResult<Option<ComplianceRule>> {
        let rules = self.rules.read().await;
        Ok(rules.get(rule_id).cloned())
    }

    /// Update rule status
    pub async fn update_rule_status(
        &self,
        rule_id: &str,
        status: RuleStatus,
    ) -> SecurityResult<()> {
        let mut rules = self.rules.write().await;

        if let Some(rule) = rules.get_mut(rule_id) {
            rule.status = status;
            rule.last_checked_at = Some(chrono::Utc::now());
            Ok(())
        } else {
            Err(SecurityError::Other(format!("Rule {} not found", rule_id)))
        }
    }

    /// Get rules for a standard
    pub async fn get_rules_for_standard(
        &self,
        standard: ComplianceStandard,
    ) -> SecurityResult<Vec<ComplianceRule>> {
        let rules = self.rules.read().await;
        Ok(rules.values().filter(|r| r.standards.contains(&standard)).cloned().collect())
    }

    /// Generate compliance report
    pub async fn generate_report(
        &self,
        standards: Vec<ComplianceStandard>,
    ) -> SecurityResult<ComplianceReport> {
        let mut report = ComplianceReport::new(standards.clone());

        let rules = self.rules.read().await;
        let applicable_rules: Vec<_> = rules
            .values()
            .filter(|r| standards.iter().any(|s| r.standards.contains(s)))
            .cloned()
            .collect();

        report.total_rules = applicable_rules.len();

        for rule in &applicable_rules {
            match rule.status {
                RuleStatus::Compliant => report.compliant_rules += 1,
                RuleStatus::NonCompliant => {
                    report.non_compliant_rules += 1;
                    report.add_finding(format!("Non-compliant: {}", rule.name));
                },
                RuleStatus::Unknown => report.unknown_rules += 1,
                RuleStatus::NotApplicable => {},
            }
        }

        report.rules = applicable_rules;
        report.calculate_score();

        Ok(report)
    }

    /// Get all rules
    pub async fn list_rules(&self) -> SecurityResult<Vec<ComplianceRule>> {
        let rules = self.rules.read().await;
        Ok(rules.values().cloned().collect())
    }

    /// Count compliant rules
    pub async fn count_compliant(&self) -> SecurityResult<usize> {
        let rules = self.rules.read().await;
        Ok(rules.values().filter(|r| r.status == RuleStatus::Compliant).count())
    }
}

impl Default for ComplianceValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compliance_rule_creation() {
        let rule = ComplianceRule::new("rule1", "Test Rule")
            .with_description("A test rule")
            .with_standard(ComplianceStandard::SOC2TypeII);

        assert_eq!(rule.id, "rule1");
        assert_eq!(rule.name, "Test Rule");
        assert!(rule.standards.contains(&ComplianceStandard::SOC2TypeII));
    }

    #[test]
    fn test_compliance_rule_status() {
        let mut rule = ComplianceRule::new("rule1", "Test Rule");
        assert_eq!(rule.status, RuleStatus::Unknown);

        rule.mark_compliant();
        assert_eq!(rule.status, RuleStatus::Compliant);
    }

    #[test]
    fn test_compliance_report_score() {
        let mut report = ComplianceReport::new(vec![ComplianceStandard::SOC2TypeII]);
        report.total_rules = 10;
        report.compliant_rules = 8;
        report.non_compliant_rules = 2;

        report.calculate_score();
        assert_eq!(report.compliance_score, 80.0);
    }

    #[tokio::test]
    async fn test_compliance_validator() {
        let validator = ComplianceValidator::with_standard_rules().await;

        let rules = validator.list_rules().await.unwrap();
        assert!(!rules.is_empty());
    }

    #[tokio::test]
    async fn test_compliance_report_generation() {
        let validator = ComplianceValidator::with_standard_rules().await;

        let report = validator.generate_report(vec![ComplianceStandard::SOC2TypeII]).await.unwrap();

        assert!(!report.rules.is_empty());
        assert!(report.compliance_score >= 0.0 && report.compliance_score <= 100.0);
    }
}
