//! Audit logging system for tracking all user actions and workflow executions
//!
//! Provides comprehensive audit trail with configurable levels, filtering,
//! and structured logging of security-relevant events.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Audit event severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "UPPERCASE")]
pub enum AuditLevel {
    /// Informational events (default)
    Info = 0,
    /// Warning events (may indicate unusual behavior)
    Warning = 1,
    /// Error events (failed operations, access denied)
    Error = 2,
    /// Critical security events
    Critical = 3,
}

impl std::fmt::Display for AuditLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditLevel::Info => write!(f, "INFO"),
            AuditLevel::Warning => write!(f, "WARNING"),
            AuditLevel::Error => write!(f, "ERROR"),
            AuditLevel::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Category of audit event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EventCategory {
    /// User authentication and session events
    Authentication,
    /// Authorization and permission checks
    Authorization,
    /// Workflow operations
    WorkflowOperation,
    /// Task operations
    TaskOperation,
    /// Execution operations
    ExecutionOperation,
    /// Configuration changes
    ConfigurationChange,
    /// Data access
    DataAccess,
    /// Security-related events
    SecurityEvent,
    /// Compliance-related events
    ComplianceEvent,
    /// System events
    SystemEvent,
}

impl std::fmt::Display for EventCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventCategory::Authentication => write!(f, "Authentication"),
            EventCategory::Authorization => write!(f, "Authorization"),
            EventCategory::WorkflowOperation => write!(f, "WorkflowOperation"),
            EventCategory::TaskOperation => write!(f, "TaskOperation"),
            EventCategory::ExecutionOperation => write!(f, "ExecutionOperation"),
            EventCategory::ConfigurationChange => write!(f, "ConfigurationChange"),
            EventCategory::DataAccess => write!(f, "DataAccess"),
            EventCategory::SecurityEvent => write!(f, "SecurityEvent"),
            EventCategory::ComplianceEvent => write!(f, "ComplianceEvent"),
            EventCategory::SystemEvent => write!(f, "SystemEvent"),
        }
    }
}

/// Single audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event ID
    pub event_id: String,
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
    /// Severity level
    pub level: AuditLevel,
    /// Event category
    pub category: EventCategory,
    /// ID of the subject (user/service) who performed the action
    pub subject_id: String,
    /// Type of subject (user, service, admin, etc.)
    pub subject_type: Option<String>,
    /// Action performed
    pub action: String,
    /// Resource affected
    pub resource_type: Option<String>,
    /// Resource ID
    pub resource_id: Option<String>,
    /// Result of the action (success, failure, etc.)
    pub result: String,
    /// Error message if action failed
    pub error_message: Option<String>,
    /// IP address of the requester
    pub ip_address: Option<String>,
    /// Session ID
    pub session_id: Option<String>,
    /// Additional structured data
    pub details: HashMap<String, String>,
}

impl AuditEvent {
    /// Create a new audit event
    pub fn new(
        subject_id: impl Into<String>,
        action: impl Into<String>,
        category: EventCategory,
    ) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            level: AuditLevel::Info,
            category,
            subject_id: subject_id.into(),
            subject_type: None,
            action: action.into(),
            resource_type: None,
            resource_id: None,
            result: "success".to_string(),
            error_message: None,
            ip_address: None,
            session_id: None,
            details: HashMap::new(),
        }
    }

    /// Set the severity level
    pub fn with_level(mut self, level: AuditLevel) -> Self {
        self.level = level;
        self
    }

    /// Set subject type
    pub fn with_subject_type(mut self, subject_type: impl Into<String>) -> Self {
        self.subject_type = Some(subject_type.into());
        self
    }

    /// Set resource information
    pub fn with_resource(
        mut self,
        resource_type: impl Into<String>,
        resource_id: impl Into<String>,
    ) -> Self {
        self.resource_type = Some(resource_type.into());
        self.resource_id = Some(resource_id.into());
        self
    }

    /// Set the result of the action
    pub fn with_result(mut self, result: impl Into<String>) -> Self {
        self.result = result.into();
        self
    }

    /// Set error message for failed actions
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error_message = Some(error.into());
        self.result = "failure".to_string();
        self
    }

    /// Set IP address
    pub fn with_ip_address(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    /// Set session ID
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Add detail field
    pub fn with_detail(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.details.insert(key.into(), value.into());
        self
    }

    /// Add multiple details
    pub fn with_details(mut self, details: HashMap<String, String>) -> Self {
        self.details.extend(details);
        self
    }
}

/// Configuration for the audit logger
#[derive(Debug, Clone)]
pub struct AuditLoggerConfig {
    /// Minimum level to log
    pub min_level: AuditLevel,
    /// Maximum number of events to store in memory
    pub max_events: usize,
    /// Whether to include sensitive data in logs
    pub include_sensitive_data: bool,
    /// Categories to track (empty = all)
    pub tracked_categories: Vec<EventCategory>,
}

impl Default for AuditLoggerConfig {
    fn default() -> Self {
        Self {
            min_level: AuditLevel::Info,
            max_events: 100_000,
            include_sensitive_data: false,
            tracked_categories: Vec::new(), // Track all
        }
    }
}

/// Audit logger for recording security events
#[derive(Debug, Clone)]
pub struct AuditLogger {
    events: Arc<RwLock<Vec<AuditEvent>>>,
    config: Arc<RwLock<AuditLoggerConfig>>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(config: AuditLoggerConfig) -> Self {
        Self { events: Arc::new(RwLock::new(Vec::new())), config: Arc::new(RwLock::new(config)) }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(AuditLoggerConfig::default())
    }

    /// Log an audit event
    pub async fn log(&self, event: AuditEvent) -> crate::security::error::SecurityResult<()> {
        let config = self.config.read().await;

        // Check if event meets minimum level
        if event.level < config.min_level {
            return Ok(());
        }

        // Check if category is tracked
        if !config.tracked_categories.is_empty()
            && !config.tracked_categories.contains(&event.category)
        {
            return Ok(());
        }

        drop(config);

        // Store event
        let mut events = self.events.write().await;

        if events.len() >= self.config.read().await.max_events {
            events.remove(0);
        }

        events.push(event);
        Ok(())
    }

    /// Get all events
    pub async fn get_events(&self) -> crate::security::error::SecurityResult<Vec<AuditEvent>> {
        let events = self.events.read().await;
        Ok(events.clone())
    }

    /// Get events filtered by level
    pub async fn get_events_by_level(
        &self,
        level: AuditLevel,
    ) -> crate::security::error::SecurityResult<Vec<AuditEvent>> {
        let events = self.events.read().await;
        Ok(events.iter().filter(|e| e.level >= level).cloned().collect())
    }

    /// Get events for a specific subject
    pub async fn get_subject_events(
        &self,
        subject_id: &str,
    ) -> crate::security::error::SecurityResult<Vec<AuditEvent>> {
        let events = self.events.read().await;
        Ok(events.iter().filter(|e| e.subject_id == subject_id).cloned().collect())
    }

    /// Get events for a specific resource
    pub async fn get_resource_events(
        &self,
        resource_id: &str,
    ) -> crate::security::error::SecurityResult<Vec<AuditEvent>> {
        let events = self.events.read().await;
        Ok(events
            .iter()
            .filter(|e| e.resource_id.as_ref().map(|r| r == resource_id).unwrap_or(false))
            .cloned()
            .collect())
    }

    /// Get events in a time range
    pub async fn get_events_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> crate::security::error::SecurityResult<Vec<AuditEvent>> {
        let events = self.events.read().await;
        Ok(events
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .cloned()
            .collect())
    }

    /// Get events by category
    pub async fn get_events_by_category(
        &self,
        category: EventCategory,
    ) -> crate::security::error::SecurityResult<Vec<AuditEvent>> {
        let events = self.events.read().await;
        Ok(events.iter().filter(|e| e.category == category).cloned().collect())
    }

    /// Clear all events
    pub async fn clear(&self) -> crate::security::error::SecurityResult<()> {
        let mut events = self.events.write().await;
        events.clear();
        Ok(())
    }

    /// Get total event count
    pub async fn count(&self) -> crate::security::error::SecurityResult<usize> {
        let events = self.events.read().await;
        Ok(events.len())
    }

    /// Update configuration
    pub async fn update_config(
        &self,
        config: AuditLoggerConfig,
    ) -> crate::security::error::SecurityResult<()> {
        let mut cfg = self.config.write().await;
        *cfg = config;
        Ok(())
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_creation() {
        let event = AuditEvent::new("user1", "login", EventCategory::Authentication);

        assert_eq!(event.subject_id, "user1");
        assert_eq!(event.action, "login");
        assert_eq!(event.level, AuditLevel::Info);
        assert_eq!(event.result, "success");
    }

    #[test]
    fn test_audit_event_with_details() {
        let event = AuditEvent::new("user1", "create_workflow", EventCategory::WorkflowOperation)
            .with_resource("workflow", "wf123")
            .with_level(AuditLevel::Warning)
            .with_detail("status", "review_pending");

        assert_eq!(event.resource_type, Some("workflow".to_string()));
        assert_eq!(event.level, AuditLevel::Warning);
        assert_eq!(event.details.get("status"), Some(&"review_pending".to_string()));
    }

    #[tokio::test]
    async fn test_audit_logger() {
        let logger = AuditLogger::with_defaults();

        let event = AuditEvent::new("user1", "login", EventCategory::Authentication);
        logger.log(event).await.unwrap();

        let events = logger.get_events().await.unwrap();
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn test_audit_logger_filtering() {
        let logger = AuditLogger::with_defaults();

        logger
            .log(
                AuditEvent::new("user1", "login", EventCategory::Authentication)
                    .with_level(AuditLevel::Info),
            )
            .await
            .unwrap();

        logger
            .log(
                AuditEvent::new("user1", "unauthorized_access", EventCategory::Authorization)
                    .with_level(AuditLevel::Critical),
            )
            .await
            .unwrap();

        let events = logger.get_events_by_level(AuditLevel::Critical).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].level, AuditLevel::Critical);
    }

    #[tokio::test]
    async fn test_audit_logger_subject_filtering() {
        let logger = AuditLogger::with_defaults();

        logger
            .log(AuditEvent::new("user1", "login", EventCategory::Authentication))
            .await
            .unwrap();

        logger
            .log(AuditEvent::new("user2", "login", EventCategory::Authentication))
            .await
            .unwrap();

        let events = logger.get_subject_events("user1").await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].subject_id, "user1");
    }

    #[tokio::test]
    async fn test_audit_logger_time_filtering() {
        let logger = AuditLogger::with_defaults();

        let now = Utc::now();
        logger
            .log(AuditEvent::new("user1", "action1", EventCategory::WorkflowOperation))
            .await
            .unwrap();

        let events = logger
            .get_events_in_range(
                now - chrono::Duration::minutes(1),
                Utc::now() + chrono::Duration::minutes(1),
            )
            .await
            .unwrap();

        assert_eq!(events.len(), 1);
    }
}
