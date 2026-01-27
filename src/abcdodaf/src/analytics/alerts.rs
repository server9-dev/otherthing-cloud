//! SLA monitoring and alerting system
//!
//! Provides:
//! - Alert rules and thresholds
//! - SLA policy definition
//! - SLA violation tracking
//! - Alert generation and management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

/// An alert notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Alert ID
    pub id: String,
    /// Alert message
    pub message: String,
    /// Alert level
    pub level: AlertLevel,
    /// Related process or resource
    pub source: String,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Alert is resolved
    pub resolved: bool,
    /// Additional context
    pub context: HashMap<String, serde_json::Value>,
}

impl Alert {
    /// Create a new alert
    pub fn new(
        message: impl Into<String>,
        level: AlertLevel,
        source: impl Into<String>,
    ) -> Self {
        let id = format!("alert-{}", uuid::Uuid::new_v4());
        Self {
            id,
            message: message.into(),
            level,
            source: source.into(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            resolved: false,
            context: HashMap::new(),
        }
    }

    /// Mark alert as resolved
    pub fn resolve(&mut self) {
        self.resolved = true;
        self.updated_at = Utc::now();
    }

    /// Add context to alert
    pub fn with_context(mut self, key: String, value: serde_json::Value) -> Self {
        self.context.insert(key, value);
        self
    }
}

/// Alert rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    /// Rule ID
    pub id: String,
    /// Rule name
    pub name: String,
    /// Metric name to monitor
    pub metric_name: String,
    /// Condition (e.g., "greater_than", "less_than")
    pub condition: String,
    /// Threshold value
    pub threshold: f64,
    /// Alert level when triggered
    pub alert_level: AlertLevel,
    /// Enabled
    pub enabled: bool,
}

impl AlertRule {
    /// Create a new alert rule
    pub fn new(
        name: impl Into<String>,
        metric_name: impl Into<String>,
        condition: impl Into<String>,
        threshold: f64,
        alert_level: AlertLevel,
    ) -> Self {
        let id = format!("rule-{}", uuid::Uuid::new_v4());
        Self {
            id,
            name: name.into(),
            metric_name: metric_name.into(),
            condition: condition.into(),
            threshold,
            alert_level,
            enabled: true,
        }
    }

    /// Evaluate rule against a value
    pub fn evaluate(&self, value: f64) -> bool {
        if !self.enabled {
            return false;
        }

        match self.condition.as_str() {
            "greater_than" => value > self.threshold,
            "less_than" => value < self.threshold,
            "equals" => (value - self.threshold).abs() < 0.01,
            "greater_or_equal" => value >= self.threshold,
            "less_or_equal" => value <= self.threshold,
            _ => false,
        }
    }
}

/// SLA (Service Level Agreement) policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaPolicy {
    /// Policy ID
    pub id: String,
    /// Policy name
    pub name: String,
    /// Process or service this applies to
    pub target: String,
    /// Maximum allowed response time (ms)
    pub max_response_time_ms: Option<f64>,
    /// Minimum availability (%)
    pub min_availability: Option<f64>,
    /// Maximum error rate (%)
    pub max_error_rate: Option<f64>,
    /// Minimum completion rate (%)
    pub min_completion_rate: Option<f64>,
    /// Coverage window (e.g., "24x7", "business_hours")
    pub coverage_window: String,
}

impl SlaPolicy {
    /// Create a new SLA policy
    pub fn new(name: impl Into<String>, target: impl Into<String>) -> Self {
        let id = format!("sla-{}", uuid::Uuid::new_v4());
        Self {
            id,
            name: name.into(),
            target: target.into(),
            max_response_time_ms: None,
            min_availability: None,
            max_error_rate: None,
            min_completion_rate: None,
            coverage_window: "24x7".to_string(),
        }
    }

    /// Set maximum response time
    pub fn with_max_response_time(mut self, ms: f64) -> Self {
        self.max_response_time_ms = Some(ms);
        self
    }

    /// Set minimum availability
    pub fn with_min_availability(mut self, percent: f64) -> Self {
        self.min_availability = Some(percent);
        self
    }

    /// Set maximum error rate
    pub fn with_max_error_rate(mut self, percent: f64) -> Self {
        self.max_error_rate = Some(percent);
        self
    }

    /// Set minimum completion rate
    pub fn with_min_completion_rate(mut self, percent: f64) -> Self {
        self.min_completion_rate = Some(percent);
        self
    }

    /// Set coverage window
    pub fn with_coverage_window(mut self, window: impl Into<String>) -> Self {
        self.coverage_window = window.into();
        self
    }
}

/// SLA violation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaViolation {
    /// Violation ID
    pub id: String,
    /// SLA policy ID
    pub sla_policy_id: String,
    /// Violated metric
    pub metric: String,
    /// Expected value (threshold)
    pub expected: f64,
    /// Actual value
    pub actual: f64,
    /// Timestamp of violation
    pub violation_time: DateTime<Utc>,
    /// Severity
    pub severity: AlertLevel,
}

impl SlaViolation {
    /// Create new SLA violation
    pub fn new(
        sla_policy_id: String,
        metric: String,
        expected: f64,
        actual: f64,
    ) -> Self {
        let id = format!("violation-{}", uuid::Uuid::new_v4());
        Self {
            id,
            sla_policy_id,
            metric,
            expected,
            actual,
            violation_time: Utc::now(),
            severity: AlertLevel::Critical,
        }
    }
}

/// Main alerting system
#[derive(Debug, Clone)]
pub struct AlertingSystem {
    /// Alert storage
    alerts: Arc<Mutex<HashMap<String, Alert>>>,
    /// Alert rules
    rules: Arc<Mutex<Vec<AlertRule>>>,
    /// SLA policies
    sla_policies: Arc<Mutex<Vec<SlaPolicy>>>,
    /// SLA violations
    violations: Arc<Mutex<Vec<SlaViolation>>>,
    /// Alert delivery channels
    channels: Arc<Mutex<Vec<AlertChannel>>>,
}

/// Alert delivery channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertChannel {
    /// Log to console
    Console,
    /// Send via email
    Email { recipients: Vec<String> },
    /// Send to webhook
    Webhook { url: String },
    /// Send via SMS
    Sms { phone_numbers: Vec<String> },
}

impl AlertingSystem {
    /// Create a new alerting system
    pub fn new() -> Self {
        Self {
            alerts: Arc::new(Mutex::new(HashMap::new())),
            rules: Arc::new(Mutex::new(Vec::new())),
            sla_policies: Arc::new(Mutex::new(Vec::new())),
            violations: Arc::new(Mutex::new(Vec::new())),
            channels: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add an alert rule
    pub fn add_rule(&self, rule: AlertRule) {
        let mut rules = self.rules.lock().unwrap();
        rules.push(rule);
    }

    /// Add an SLA policy
    pub fn add_sla_policy(&self, policy: SlaPolicy) {
        let mut policies = self.sla_policies.lock().unwrap();
        policies.push(policy);
    }

    /// Create and store an alert
    pub fn create_alert(
        &self,
        message: impl Into<String>,
        level: AlertLevel,
        source: impl Into<String>,
    ) -> Alert {
        let alert = Alert::new(message, level, source);
        let mut alerts = self.alerts.lock().unwrap();
        alerts.insert(alert.id.clone(), alert.clone());
        alert
    }

    /// Get all active alerts
    pub fn active_alerts(&self) -> Vec<Alert> {
        let alerts = self.alerts.lock().unwrap();
        alerts
            .values()
            .filter(|a| !a.resolved)
            .cloned()
            .collect()
    }

    /// Get alerts by source
    pub fn alerts_by_source(&self, source: &str) -> Vec<Alert> {
        let alerts = self.alerts.lock().unwrap();
        alerts
            .values()
            .filter(|a| a.source == source && !a.resolved)
            .cloned()
            .collect()
    }

    /// Get alerts by level
    pub fn alerts_by_level(&self, level: AlertLevel) -> Vec<Alert> {
        let alerts = self.alerts.lock().unwrap();
        alerts
            .values()
            .filter(|a| a.level == level && !a.resolved)
            .cloned()
            .collect()
    }

    /// Resolve an alert
    pub fn resolve_alert(&self, alert_id: &str) {
        let mut alerts = self.alerts.lock().unwrap();
        if let Some(alert) = alerts.get_mut(alert_id) {
            alert.resolve();
        }
    }

    /// Evaluate a metric against all rules
    pub fn evaluate_metric(&self, metric_name: &str, value: f64) -> Vec<Alert> {
        let mut triggered_alerts = Vec::new();
        let rules = self.rules.lock().unwrap();

        for rule in rules.iter() {
            if rule.metric_name == metric_name && rule.evaluate(value) {
                let alert = self.create_alert(
                    format!(
                        "Rule '{}' triggered: {} {} {} (actual: {})",
                        rule.name, metric_name, rule.condition, rule.threshold, value
                    ),
                    rule.alert_level,
                    &rule.metric_name,
                );
                triggered_alerts.push(alert);
            }
        }

        triggered_alerts
    }

    /// Check SLA compliance
    pub fn check_sla_compliance(
        &self,
        target: &str,
        response_time: Option<f64>,
        availability: Option<f64>,
        error_rate: Option<f64>,
        completion_rate: Option<f64>,
    ) -> Vec<SlaViolation> {
        let mut violations = Vec::new();
        let policies = self.sla_policies.lock().unwrap();

        for policy in policies.iter() {
            if policy.target == target {
                // Check response time
                if let (Some(max_rt), Some(rt)) = (policy.max_response_time_ms, response_time) {
                    if rt > max_rt {
                        violations.push(SlaViolation::new(
                            policy.id.clone(),
                            "response_time".to_string(),
                            max_rt,
                            rt,
                        ));
                    }
                }

                // Check availability
                if let (Some(min_avail), Some(avail)) = (policy.min_availability, availability) {
                    if avail < min_avail {
                        violations.push(SlaViolation::new(
                            policy.id.clone(),
                            "availability".to_string(),
                            min_avail,
                            avail,
                        ));
                    }
                }

                // Check error rate
                if let (Some(max_err), Some(err)) = (policy.max_error_rate, error_rate) {
                    if err > max_err {
                        violations.push(SlaViolation::new(
                            policy.id.clone(),
                            "error_rate".to_string(),
                            max_err,
                            err,
                        ));
                    }
                }

                // Check completion rate
                if let (Some(min_comp), Some(comp)) = (policy.min_completion_rate, completion_rate)
                {
                    if comp < min_comp {
                        violations.push(SlaViolation::new(
                            policy.id.clone(),
                            "completion_rate".to_string(),
                            min_comp,
                            comp,
                        ));
                    }
                }
            }
        }

        violations
    }

    /// Add alert channel
    pub fn add_channel(&self, channel: AlertChannel) {
        let mut channels = self.channels.lock().unwrap();
        channels.push(channel);
    }

    /// Get all violations
    pub fn violations(&self) -> Vec<SlaViolation> {
        self.violations.lock().unwrap().clone()
    }

    /// Record a violation
    pub fn record_violation(&self, violation: SlaViolation) {
        let mut violations = self.violations.lock().unwrap();
        violations.push(violation);
    }

    /// Clear old alerts (older than hours)
    pub fn cleanup_old_alerts(&self, hours: i64) {
        let cutoff = Utc::now() - chrono::Duration::hours(hours);
        let mut alerts = self.alerts.lock().unwrap();
        alerts.retain(|_, alert| alert.created_at > cutoff);
    }
}

impl Default for AlertingSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_creation() {
        let alert = Alert::new("Test alert", AlertLevel::Warning, "test_process");
        assert_eq!(alert.message, "Test alert");
        assert_eq!(alert.level, AlertLevel::Warning);
        assert!(!alert.resolved);
    }

    #[test]
    fn test_alert_resolution() {
        let mut alert = Alert::new("Test alert", AlertLevel::Warning, "test_process");
        alert.resolve();
        assert!(alert.resolved);
    }

    #[test]
    fn test_alert_rule() {
        let rule = AlertRule::new(
            "High response time",
            "response_time",
            "greater_than",
            1000.0,
            AlertLevel::Critical,
        );

        assert!(rule.evaluate(1500.0));
        assert!(!rule.evaluate(500.0));
    }

    #[test]
    fn test_sla_policy() {
        let policy = SlaPolicy::new("Standard SLA", "process_1")
            .with_max_response_time(5000.0)
            .with_min_completion_rate(95.0);

        assert_eq!(policy.max_response_time_ms, Some(5000.0));
        assert_eq!(policy.min_completion_rate, Some(95.0));
    }

    #[test]
    fn test_alerting_system() {
        let system = AlertingSystem::new();
        let rule = AlertRule::new(
            "Test rule",
            "test_metric",
            "greater_than",
            100.0,
            AlertLevel::Warning,
        );

        system.add_rule(rule);
        let alerts = system.evaluate_metric("test_metric", 150.0);
        assert!(!alerts.is_empty());
    }

    #[test]
    fn test_sla_violation() {
        let system = AlertingSystem::new();
        let policy = SlaPolicy::new("Test SLA", "process_1")
            .with_max_response_time(1000.0)
            .with_max_error_rate(5.0);

        system.add_sla_policy(policy);
        let violations = system.check_sla_compliance("process_1", Some(1500.0), None, Some(10.0), None);
        assert!(!violations.is_empty());
    }
}
