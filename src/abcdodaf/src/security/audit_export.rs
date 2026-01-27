//! Audit log export functionality
//!
//! Provides export of audit logs in various formats for compliance reporting
//! and long-term archival.

use crate::security::audit::AuditEvent;
use crate::security::error::SecurityResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Export format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    /// JSON format
    Json,
    /// CSV format
    Csv,
    /// XML format
    Xml,
    /// Binary format
    Binary,
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportFormat::Json => write!(f, "JSON"),
            ExportFormat::Csv => write!(f, "CSV"),
            ExportFormat::Xml => write!(f, "XML"),
            ExportFormat::Binary => write!(f, "Binary"),
        }
    }
}

/// Export options
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// Export format
    pub format: ExportFormat,
    /// Include sensitive data
    pub include_sensitive: bool,
    /// Compress output
    pub compress: bool,
    /// Sign with digital signature
    pub sign: bool,
    /// Include metadata
    pub include_metadata: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ExportFormat::Json,
            include_sensitive: false,
            compress: false,
            sign: true,
            include_metadata: true,
        }
    }
}

/// Export metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    /// Export ID
    pub export_id: String,
    /// Export timestamp
    pub exported_at: chrono::DateTime<chrono::Utc>,
    /// Number of events exported
    pub event_count: usize,
    /// Start of date range
    pub range_start: chrono::DateTime<chrono::Utc>,
    /// End of date range
    pub range_end: chrono::DateTime<chrono::Utc>,
    /// Filter applied
    pub filter: Option<String>,
    /// Export format
    pub format: String,
    /// Exporter (user/service)
    pub exported_by: String,
    /// Hash of exported data (for integrity verification)
    pub data_hash: Option<String>,
    /// Signature (if signed)
    pub signature: Option<Vec<u8>>,
}

impl ExportMetadata {
    /// Create new export metadata
    pub fn new(exported_by: impl Into<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            export_id: uuid::Uuid::new_v4().to_string(),
            exported_at: now,
            event_count: 0,
            range_start: now,
            range_end: now,
            filter: None,
            format: String::new(),
            exported_by: exported_by.into(),
            data_hash: None,
            signature: None,
        }
    }
}

/// Audit log export wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogExport {
    /// Export metadata
    pub metadata: ExportMetadata,
    /// Exported events
    pub events: Vec<AuditEventExport>,
}

impl AuditLogExport {
    /// Create new export
    pub fn new(metadata: ExportMetadata) -> Self {
        Self {
            metadata,
            events: Vec::new(),
        }
    }

    /// Add an event
    pub fn add_event(&mut self, event: AuditEventExport) {
        self.events.push(event);
        self.metadata.event_count = self.events.len();
    }
}

/// Audit event for export (may have sensitive data redacted)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEventExport {
    /// Event ID
    pub event_id: String,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Level
    pub level: String,
    /// Category
    pub category: String,
    /// Subject ID (may be redacted)
    pub subject_id: String,
    /// Subject type
    pub subject_type: Option<String>,
    /// Action
    pub action: String,
    /// Resource type
    pub resource_type: Option<String>,
    /// Resource ID (may be redacted)
    pub resource_id: Option<String>,
    /// Result
    pub result: String,
    /// Error message (may be redacted)
    pub error_message: Option<String>,
    /// IP address (may be redacted)
    pub ip_address: Option<String>,
    /// Session ID (may be redacted)
    pub session_id: Option<String>,
    /// Details (may be redacted)
    pub details: HashMap<String, String>,
}

impl AuditEventExport {
    /// Create from audit event
    pub fn from_event(event: AuditEvent, include_sensitive: bool) -> Self {
        let mut result = Self {
            event_id: event.event_id,
            timestamp: event.timestamp,
            level: event.level.to_string(),
            category: event.category.to_string(),
            subject_id: event.subject_id,
            subject_type: event.subject_type,
            action: event.action,
            resource_type: event.resource_type,
            resource_id: event.resource_id,
            result: event.result,
            error_message: event.error_message,
            ip_address: event.ip_address,
            session_id: event.session_id,
            details: event.details,
        };

        if !include_sensitive {
            // Redact sensitive information
            result.subject_id = redact_id(&result.subject_id);
            result.ip_address = result.ip_address.map(|_| "[REDACTED]".to_string());
            result.session_id = result.session_id.map(|_| "[REDACTED]".to_string());

            // Redact details that might contain sensitive data
            for (key, value) in result.details.iter_mut() {
                if key.contains("password") || key.contains("secret") || key.contains("key")
                    || key.contains("token")
                {
                    *value = "[REDACTED]".to_string();
                }
            }
        }

        result
    }
}

/// Redact an identifier
fn redact_id(id: &str) -> String {
    if id.len() > 4 {
        format!("***{}", &id[id.len() - 4..])
    } else {
        "***".to_string()
    }
}

/// Audit exporter
pub struct AuditExporter;

impl AuditExporter {
    /// Export events to JSON format
    pub fn to_json(export: &AuditLogExport) -> SecurityResult<String> {
        // Wrap in audit_export object for consistency
        let wrapped = serde_json::json!({
            "audit_export": export
        });
        Ok(serde_json::to_string_pretty(&wrapped)?)
    }

    /// Export events to JSON lines format (one event per line)
    pub fn to_jsonlines(export: &AuditLogExport) -> SecurityResult<String> {
        let lines: Vec<String> = export
            .events
            .iter()
            .filter_map(|e| serde_json::to_string(e).ok())
            .collect();

        Ok(lines.join("\n"))
    }

    /// Export events to CSV format
    pub fn to_csv(export: &AuditLogExport) -> SecurityResult<String> {
        let mut csv = String::new();

        // Header
        csv.push_str("event_id,timestamp,level,category,subject_id,subject_type,action,resource_type,resource_id,result,error_message,ip_address,session_id\n");

        // Events
        for event in &export.events {
            csv.push_str(&format!(
                "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
                escape_csv(&event.event_id),
                event.timestamp,
                escape_csv(&event.level),
                escape_csv(&event.category),
                escape_csv(&event.subject_id),
                event.subject_type.as_deref().unwrap_or(""),
                escape_csv(&event.action),
                event.resource_type.as_deref().unwrap_or(""),
                event.resource_id.as_deref().unwrap_or(""),
                escape_csv(&event.result),
                event.error_message.as_deref().unwrap_or(""),
                event.ip_address.as_deref().unwrap_or(""),
                event.session_id.as_deref().unwrap_or("")
            ));
        }

        Ok(csv)
    }

    /// Export events to XML format
    pub fn to_xml(export: &AuditLogExport) -> SecurityResult<String> {
        let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<audit_export>\n");

        // Metadata
        xml.push_str("  <metadata>\n");
        xml.push_str(&format!(
            "    <export_id>{}</export_id>\n",
            escape_xml(&export.metadata.export_id)
        ));
        xml.push_str(&format!(
            "    <exported_at>{}</exported_at>\n",
            escape_xml(&export.metadata.exported_at.to_string())
        ));
        xml.push_str(&format!(
            "    <event_count>{}</event_count>\n",
            export.metadata.event_count
        ));
        xml.push_str("  </metadata>\n");

        // Events
        xml.push_str("  <events>\n");
        for event in &export.events {
            xml.push_str("    <event>\n");
            xml.push_str(&format!(
                "      <event_id>{}</event_id>\n",
                escape_xml(&event.event_id)
            ));
            xml.push_str(&format!(
                "      <timestamp>{}</timestamp>\n",
                escape_xml(&event.timestamp.to_string())
            ));
            xml.push_str(&format!(
                "      <level>{}</level>\n",
                escape_xml(&event.level)
            ));
            xml.push_str(&format!(
                "      <category>{}</category>\n",
                escape_xml(&event.category)
            ));
            xml.push_str(&format!(
                "      <subject_id>{}</subject_id>\n",
                escape_xml(&event.subject_id)
            ));
            xml.push_str(&format!(
                "      <action>{}</action>\n",
                escape_xml(&event.action)
            ));
            xml.push_str(&format!(
                "      <result>{}</result>\n",
                escape_xml(&event.result)
            ));
            xml.push_str("    </event>\n");
        }
        xml.push_str("  </events>\n");

        xml.push_str("</audit_export>\n");
        Ok(xml)
    }
}

/// Escape CSV field
fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Escape XML field
fn escape_xml(s: &str) -> String {
    s.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::audit::EventCategory;

    #[test]
    fn test_export_metadata_creation() {
        let metadata = ExportMetadata::new("admin");
        assert_eq!(metadata.exported_by, "admin");
        assert!(!metadata.export_id.is_empty());
    }

    #[test]
    fn test_audit_event_export_redaction() {
        let event = AuditEvent::new("user123", "login", EventCategory::Authentication)
            .with_ip_address("192.168.1.1")
            .with_session_id("session123");

        let export = AuditEventExport::from_event(event, false);
        assert_eq!(export.ip_address, Some("[REDACTED]".to_string()));
        assert_eq!(export.session_id, Some("[REDACTED]".to_string()));
    }

    #[test]
    fn test_audit_event_export_no_redaction() {
        let event = AuditEvent::new("user123", "login", EventCategory::Authentication)
            .with_ip_address("192.168.1.1");

        let export = AuditEventExport::from_event(event, true);
        assert_eq!(export.ip_address, Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_export_to_json() {
        let metadata = ExportMetadata::new("admin");
        let export = AuditLogExport::new(metadata);

        let json = AuditExporter::to_json(&export).unwrap();
        assert!(json.contains("audit_export"));
    }

    #[test]
    fn test_export_to_csv() {
        let metadata = ExportMetadata::new("admin");
        let mut export = AuditLogExport::new(metadata);

        let event = AuditEventExport {
            event_id: "event1".to_string(),
            timestamp: chrono::Utc::now(),
            level: "INFO".to_string(),
            category: "Authentication".to_string(),
            subject_id: "user1".to_string(),
            subject_type: Some("User".to_string()),
            action: "login".to_string(),
            resource_type: None,
            resource_id: None,
            result: "success".to_string(),
            error_message: None,
            ip_address: None,
            session_id: None,
            details: HashMap::new(),
        };

        export.add_event(event);

        let csv = AuditExporter::to_csv(&export).unwrap();
        assert!(csv.contains("event_id"));
        assert!(csv.contains("event1"));
    }

    #[test]
    fn test_export_to_xml() {
        let metadata = ExportMetadata::new("admin");
        let export = AuditLogExport::new(metadata);

        let xml = AuditExporter::to_xml(&export).unwrap();
        assert!(xml.contains("<?xml"));
        assert!(xml.contains("<audit_export>"));
    }

    #[test]
    fn test_csv_escaping() {
        let result = escape_csv("field,with,commas");
        assert!(result.contains("\""));

        let result = escape_csv("field\"with\"quotes");
        assert_eq!(result, "\"field\"\"with\"\"quotes\"");
    }

    #[test]
    fn test_xml_escaping() {
        let result = escape_xml("<tag>data</tag>");
        assert!(result.contains("&lt;"));
        assert!(result.contains("&gt;"));
    }
}
