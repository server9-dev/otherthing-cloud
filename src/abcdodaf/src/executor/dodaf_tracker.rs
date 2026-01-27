//! DODAF Tracker
//!
//! Tracks all DODAF 2.02 metadata during workflow execution:
//! - Operational Activities (OV-5)
//! - Resource Flows (OV-2, OV-3)
//! - Capabilities (CV-2, CV-6)
//! - System Mappings (SV-1, SV-4)
//! - Performer tracking
//! - Traceability matrices
//! - Security markings
//! - Cost/duration estimates

use crate::dodaf::*;
use crate::dodaf::ov5::ResourceFlow;
use crate::executor::database::{DatabaseManager};
use crate::error::Result;
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use std::collections::HashMap;

/// DODAF metadata tracker
pub struct DodafTracker {
    db: DatabaseManager,
}

impl DodafTracker {
    /// Create a new DODAF tracker
    pub fn new(db: DatabaseManager) -> Self {
        Self { db }
    }

    /// Track operational activity execution
    pub async fn track_operational_activity(
        &self,
        workflow_id: Uuid,
        activity: &OperationalActivity,
    ) -> Result<()> {
        let metadata = serde_json::to_value(activity)?;
        self.db.store_dodaf_metadata(workflow_id, "OV-5", metadata).await?;
        Ok(())
    }

    /// Track resource flow
    pub async fn track_resource_flow(
        &self,
        workflow_id: Uuid,
        flow: &ResourceFlow,
    ) -> Result<()> {
        // Get existing OV-3 data
        let existing = self.db.get_dodaf_metadata(workflow_id, Some("OV-3")).await?;

        let mut flows: Vec<ResourceFlow> = if let Some(record) = existing.first() {
            serde_json::from_value(record.metadata.clone()).unwrap_or_default()
        } else {
            Vec::new()
        };

        flows.push(flow.clone());
        let metadata = serde_json::to_value(&flows)?;
        self.db.store_dodaf_metadata(workflow_id, "OV-3", metadata).await?;
        Ok(())
    }

    /// Track capability mapping
    pub async fn track_capability(
        &self,
        workflow_id: Uuid,
        capability: &CapabilityMapping,
    ) -> Result<()> {
        let metadata = serde_json::to_value(capability)?;
        self.db.store_dodaf_metadata(workflow_id, "CV-6", metadata).await?;
        Ok(())
    }

    /// Track system-to-activity mapping
    pub async fn track_system_mapping(
        &self,
        workflow_id: Uuid,
        mapping: &SystemMapping,
    ) -> Result<()> {
        let metadata = serde_json::to_value(mapping)?;
        self.db.store_dodaf_metadata(workflow_id, "SV-1", metadata).await?;
        Ok(())
    }

    /// Track performer assignment
    pub async fn track_performer(
        &self,
        workflow_id: Uuid,
        performer: &PerformerAssignment,
    ) -> Result<()> {
        // Get existing OV-4 data
        let existing = self.db.get_dodaf_metadata(workflow_id, Some("OV-4")).await?;

        let mut performers: Vec<PerformerAssignment> = if let Some(record) = existing.first() {
            serde_json::from_value(record.metadata.clone()).unwrap_or_default()
        } else {
            Vec::new()
        };

        performers.push(performer.clone());
        let metadata = serde_json::to_value(&performers)?;
        self.db.store_dodaf_metadata(workflow_id, "OV-4", metadata).await?;
        Ok(())
    }

    /// Track security marking
    pub async fn track_security_marking(
        &self,
        workflow_id: Uuid,
        marking: &SecurityMarking,
    ) -> Result<()> {
        // Get existing security data
        let existing = self.db.get_dodaf_metadata(workflow_id, Some("SECURITY")).await?;

        let mut markings: Vec<SecurityMarking> = if let Some(record) = existing.first() {
            serde_json::from_value(record.metadata.clone()).unwrap_or_default()
        } else {
            Vec::new()
        };

        markings.push(marking.clone());
        let metadata = serde_json::to_value(&markings)?;
        self.db.store_dodaf_metadata(workflow_id, "SECURITY", metadata).await?;
        Ok(())
    }

    /// Track traceability link
    pub async fn track_traceability(
        &self,
        workflow_id: Uuid,
        link: &TraceabilityLink,
    ) -> Result<()> {
        // Get existing traceability matrix
        let existing = self.db.get_dodaf_metadata(workflow_id, Some("TRACEABILITY")).await?;

        let mut links: Vec<TraceabilityLink> = if let Some(record) = existing.first() {
            serde_json::from_value(record.metadata.clone()).unwrap_or_default()
        } else {
            Vec::new()
        };

        links.push(link.clone());
        let metadata = serde_json::to_value(&links)?;
        self.db.store_dodaf_metadata(workflow_id, "TRACEABILITY", metadata).await?;
        Ok(())
    }

    /// Track cost and duration estimate
    pub async fn track_cost_duration(
        &self,
        workflow_id: Uuid,
        estimate: &CostDurationEstimate,
    ) -> Result<()> {
        let metadata = serde_json::to_value(estimate)?;
        self.db.store_dodaf_metadata(workflow_id, "OV-5-METRICS", metadata).await?;
        Ok(())
    }

    /// Get complete DODAF view for a workflow
    pub async fn get_dodaf_view(&self, workflow_id: Uuid) -> Result<DodafView> {
        let records = self.db.get_dodaf_metadata(workflow_id, None).await?;

        let mut view = DodafView::default();

        for record in records {
            match record.view_type.as_str() {
                "OV-5" => {
                    view.operational_activity = serde_json::from_value(record.metadata).ok();
                }
                "OV-3" => {
                    view.resource_flows = serde_json::from_value(record.metadata).unwrap_or_default();
                }
                "OV-4" => {
                    view.performers = serde_json::from_value(record.metadata).unwrap_or_default();
                }
                "CV-6" => {
                    view.capability_mapping = serde_json::from_value(record.metadata).ok();
                }
                "SV-1" => {
                    view.system_mapping = serde_json::from_value(record.metadata).ok();
                }
                "SECURITY" => {
                    view.security_markings = serde_json::from_value(record.metadata).unwrap_or_default();
                }
                "TRACEABILITY" => {
                    view.traceability_links = serde_json::from_value(record.metadata).unwrap_or_default();
                }
                "OV-5-METRICS" => {
                    view.cost_duration = serde_json::from_value(record.metadata).ok();
                }
                _ => {}
            }
        }

        Ok(view)
    }

    /// Generate compliance report for stored workflow
    pub async fn generate_compliance_report(&self, workflow_id: Uuid) -> Result<DodafComplianceReport> {
        let view = self.get_dodaf_view(workflow_id).await?;

        let mut report = DodafComplianceReport {
            workflow_id,
            timestamp: chrono::Utc::now(),
            overall_score: 0.0,
            view_compliance: HashMap::new(),
            missing_views: Vec::new(),
            recommendations: Vec::new(),
        };

        // Check OV-5 (Operational Activity)
        if view.operational_activity.is_some() {
            report.view_compliance.insert("OV-5".to_string(), 100.0);
        } else {
            report.view_compliance.insert("OV-5".to_string(), 0.0);
            report.missing_views.push("OV-5".to_string());
            report.recommendations.push("Add operational activity metadata to workflow".to_string());
        }

        // Check OV-3 (Resource Flows)
        let resource_flow_score = if !view.resource_flows.is_empty() {
            100.0
        } else {
            0.0
        };
        report.view_compliance.insert("OV-3".to_string(), resource_flow_score);

        if view.resource_flows.is_empty() {
            report.missing_views.push("OV-3".to_string());
            report.recommendations.push("Define resource flows between activities".to_string());
        }

        // Check OV-4 (Performers)
        let performer_score = if !view.performers.is_empty() {
            100.0
        } else {
            0.0
        };
        report.view_compliance.insert("OV-4".to_string(), performer_score);

        if view.performers.is_empty() {
            report.missing_views.push("OV-4".to_string());
            report.recommendations.push("Assign performers to activities".to_string());
        }

        // Check CV-6 (Capability Mapping)
        if view.capability_mapping.is_some() {
            report.view_compliance.insert("CV-6".to_string(), 100.0);
        } else {
            report.view_compliance.insert("CV-6".to_string(), 0.0);
            report.missing_views.push("CV-6".to_string());
            report.recommendations.push("Map workflow to organizational capabilities".to_string());
        }

        // Check SV-1 (System Mapping)
        if view.system_mapping.is_some() {
            report.view_compliance.insert("SV-1".to_string(), 100.0);
        } else {
            report.view_compliance.insert("SV-1".to_string(), 0.0);
            report.missing_views.push("SV-1".to_string());
            report.recommendations.push("Map systems to activities".to_string());
        }

        // Calculate overall score
        let total: f64 = report.view_compliance.values().sum();
        report.overall_score = total / report.view_compliance.len() as f64;

        Ok(report)
    }
}

/// Complete DODAF view for a workflow
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DodafView {
    pub operational_activity: Option<OperationalActivity>,
    pub resource_flows: Vec<ResourceFlow>,
    pub performers: Vec<PerformerAssignment>,
    pub capability_mapping: Option<CapabilityMapping>,
    pub system_mapping: Option<SystemMapping>,
    pub security_markings: Vec<SecurityMarking>,
    pub traceability_links: Vec<TraceabilityLink>,
    pub cost_duration: Option<CostDurationEstimate>,
}

// Re-export resource flow types from dodaf module
pub use crate::dodaf::ov5::ResourceType;

/// Performer assignment to activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformerAssignment {
    pub activity_id: String,
    pub performer_id: String,
    pub performer_type: PerformerType,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformerType {
    Person { role: String, organization: String },
    Organization { name: String, org_type: String },
    Service { name: String, interface: String },
    System { name: String, version: String },
}

/// Capability mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityMapping {
    pub workflow_id: String,
    pub capabilities: Vec<String>,
    pub capability_gaps: Vec<String>,
}

/// System-to-activity mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMapping {
    pub activity_id: String,
    pub systems: Vec<SystemReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemReference {
    pub system_id: String,
    pub system_name: String,
    pub interfaces: Vec<String>,
}

/// Security marking for classified workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMarking {
    pub element_id: String,
    pub classification: String,  // "Unclassified", "Confidential", "Secret", "Top Secret"
    pub caveats: Vec<String>,    // "NOFORN", "FOUO", etc.
    pub handling: String,
}

/// Traceability link between elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceabilityLink {
    pub source_id: String,
    pub source_type: String,  // "activity", "capability", "system", "requirement"
    pub target_id: String,
    pub target_type: String,
    pub link_type: String,    // "implements", "supports", "derives_from"
}

/// Cost and duration estimate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostDurationEstimate {
    pub activity_id: String,
    pub estimated_duration_hours: f64,
    pub estimated_cost_usd: f64,
    pub resource_requirements: HashMap<String, i32>,
}

/// DODAF compliance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DodafComplianceReport {
    pub workflow_id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub overall_score: f64,
    pub view_compliance: HashMap<String, f64>,
    pub missing_views: Vec<String>,
    pub recommendations: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_flow_serialization() {
        let flow = ResourceFlow {
            id: "flow_1".to_string(),
            name: "Test Flow".to_string(),
            description: Some("Data transfer".to_string()),
            source_activity: "task_a".to_string(),
            target_activity: "task_b".to_string(),
            resource_ref: "resource_1".to_string(),
            attributes: crate::dodaf::ov5::ResourceFlowAttributes {
                timeliness: None,
                availability: None,
                security_classification: None,
                performance_requirements: None,
            },
            is_needline: false,
            metadata: std::collections::HashMap::new(),
        };

        let json = serde_json::to_string(&flow).unwrap();
        assert!(json.contains("task_a"));
    }
}
