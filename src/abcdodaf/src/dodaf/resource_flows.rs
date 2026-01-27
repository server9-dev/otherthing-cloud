//! DODAF Resource Flow Types
//!
//! Enhanced resource flow taxonomy for OV-2 (Operational Resource Flow Description)
//! and OV-3 (Operational Resource Flow Matrix)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Resource flow between activities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceFlow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub source_activity: String,
    pub target_activity: String,
    pub resource_type: ResourceFlowType,
    pub attributes: ResourceFlowAttributes,
    pub is_needline: bool,
    pub metadata: HashMap<String, String>,
}

/// Comprehensive resource flow type taxonomy
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "category")]
pub enum ResourceFlowType {
    /// Information/data flow
    Information {
        data_type: InformationType,
        format: Option<String>,
        schema: Option<String>,
    },

    /// Financial resources
    Funding {
        currency: String,
        amount_min: Option<f64>,
        amount_max: Option<f64>,
        budget_code: Option<String>,
    },

    /// Personnel movement/assignment
    Personnel {
        personnel_type: PersonnelType,
        count: Option<u32>,
        duration: Option<String>,
    },

    /// Physical materials
    Materiel {
        materiel_type: MaterielType,
        quantity: Option<u32>,
        unit_of_measure: Option<String>,
    },

    /// Service invocation/consumption
    Service {
        service_name: String,
        service_type: String,
        protocol: Option<String>,
    },

    /// Energy/power
    Energy {
        energy_type: EnergyType,
        power_rating: Option<String>,
    },

    /// Communication/signal
    Communication {
        comm_type: CommunicationType,
        bandwidth: Option<String>,
        frequency: Option<String>,
    },
}

/// Information type categories
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InformationType {
    Command,
    Control,
    Intelligence,
    Surveillance,
    Reconnaissance,
    Logistics,
    Administrative,
    Tactical,
    Strategic,
    Operational,
    Technical,
    Raw,
    Processed,
    Analyzed,
}

/// Personnel type categories
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PersonnelType {
    Military,
    Civilian,
    Contractor,
    Analyst,
    Engineer,
    Operator,
    Administrator,
    Manager,
}

/// Materiel type categories
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaterielType {
    Equipment,
    Supplies,
    Ammunition,
    Fuel,
    Parts,
    Tools,
    Vehicles,
    Hardware,
    Software,
}

/// Energy type categories
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnergyType {
    Electrical,
    Mechanical,
    Thermal,
    Chemical,
    Nuclear,
}

/// Communication type categories
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommunicationType {
    Voice,
    Data,
    Video,
    Text,
    Signal,
    Telemetry,
}

/// Resource flow attributes (OV-3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceFlowAttributes {
    /// When the resource must be available
    pub timeliness: Option<TimelinessCriteria>,

    /// Reliability and uptime requirements
    pub availability: Option<f64>,

    /// Security classification
    pub security_classification: Option<String>,

    /// Performance requirements
    pub performance_requirements: Option<PerformanceRequirements>,

    /// Quality of service
    pub qos: Option<QualityOfService>,

    /// Frequency of flow
    pub frequency: Option<FlowFrequency>,
}

/// Timeliness criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelinessCriteria {
    pub max_latency_ms: Option<u64>,
    pub max_age_seconds: Option<u64>,
    pub real_time_required: bool,
}

/// Performance requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRequirements {
    pub throughput: Option<String>,
    pub response_time_ms: Option<u64>,
    pub concurrency: Option<u32>,
}

/// Quality of service parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityOfService {
    pub priority: QoSPriority,
    pub guaranteed_delivery: bool,
    pub ordering_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QoSPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// Flow frequency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FlowFrequency {
    Continuous,
    Periodic { interval_seconds: u64 },
    OnDemand,
    Event { trigger: String },
}

/// Mapping from BPMN to DODAF resource flows
pub struct ResourceFlowMapper;

impl ResourceFlowMapper {
    /// Create resource flow from BPMN message flow
    pub fn from_bpmn_message_flow(
        flow_id: &str,
        source: &str,
        target: &str,
        message_name: Option<&str>,
    ) -> ResourceFlow {
        ResourceFlow {
            id: flow_id.to_string(),
            name: message_name.unwrap_or(flow_id).to_string(),
            description: None,
            source_activity: source.to_string(),
            target_activity: target.to_string(),
            resource_type: ResourceFlowType::Information {
                data_type: InformationType::Operational,
                format: Some("JSON".to_string()),
                schema: None,
            },
            attributes: ResourceFlowAttributes {
                timeliness: Some(TimelinessCriteria {
                    max_latency_ms: Some(1000),
                    max_age_seconds: None,
                    real_time_required: false,
                }),
                availability: Some(0.99),
                security_classification: None,
                performance_requirements: None,
                qos: Some(QualityOfService {
                    priority: QoSPriority::Medium,
                    guaranteed_delivery: true,
                    ordering_required: false,
                }),
                frequency: Some(FlowFrequency::OnDemand),
            },
            is_needline: false,
            metadata: HashMap::new(),
        }
    }

    /// Create resource flow from BPMN data object
    pub fn from_bpmn_data_flow(
        flow_id: &str,
        source: &str,
        target: &str,
        data_name: &str,
    ) -> ResourceFlow {
        ResourceFlow {
            id: flow_id.to_string(),
            name: format!("{} data", data_name),
            description: None,
            source_activity: source.to_string(),
            target_activity: target.to_string(),
            resource_type: ResourceFlowType::Information {
                data_type: InformationType::Raw,
                format: None,
                schema: None,
            },
            attributes: ResourceFlowAttributes {
                timeliness: None,
                availability: None,
                security_classification: None,
                performance_requirements: None,
                qos: None,
                frequency: Some(FlowFrequency::OnDemand),
            },
            is_needline: false,
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_flow_from_message() {
        let flow = ResourceFlowMapper::from_bpmn_message_flow(
            "flow_1",
            "task_a",
            "task_b",
            Some("Request"),
        );

        assert_eq!(flow.id, "flow_1");
        assert_eq!(flow.source_activity, "task_a");
        assert_eq!(flow.target_activity, "task_b");

        if let ResourceFlowType::Information { data_type, .. } = flow.resource_type {
            assert_eq!(data_type, InformationType::Operational);
        } else {
            panic!("Expected Information flow type");
        }
    }

    #[test]
    fn test_resource_flow_attributes() {
        let flow = ResourceFlowMapper::from_bpmn_message_flow(
            "flow_1",
            "task_a",
            "task_b",
            None,
        );

        assert!(flow.attributes.timeliness.is_some());
        assert_eq!(flow.attributes.availability, Some(0.99));
    }
}
