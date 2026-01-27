//! BPMN 2.0 ↔ DoDAF 2.02 Integration and Mapping
//!
//! Provides bidirectional mapping between BPMN 2.0 processes and DoDAF 2.02 Operational Views.
//! Implements the official DoDAF BPMN Persona for OV-6c.

use crate::bpmn::elements::*;
use crate::dodaf::ov5::*;
use crate::error::Result;
use std::collections::HashMap;

/// BPMN to DoDAF Mapper
pub struct BpmnDodafMapper {
    /// Mapping configuration
    config: MappingConfig,
    /// Element mapping cache
    element_map: HashMap<String, String>,
}

/// Mapping Configuration
#[derive(Debug, Clone)]
pub struct MappingConfig {
    /// Map BPMN pools to DoDAF operational nodes
    pub map_pools_to_nodes: bool,
    /// Map BPMN lanes to DoDAF performers
    pub map_lanes_to_performers: bool,
    /// Map BPMN data objects to DoDAF information elements
    pub map_data_to_information: bool,
    /// Preserve BPMN diagram interchange (visual layout)
    pub preserve_visual_layout: bool,
}

impl Default for MappingConfig {
    fn default() -> Self {
        Self {
            map_pools_to_nodes: true,
            map_lanes_to_performers: true,
            map_data_to_information: true,
            preserve_visual_layout: true,
        }
    }
}

impl BpmnDodafMapper {
    pub fn new() -> Self {
        Self { config: MappingConfig::default(), element_map: HashMap::new() }
    }

    pub fn with_config(config: MappingConfig) -> Self {
        Self { config, element_map: HashMap::new() }
    }

    // ========================================================================
    // BPMN → DoDAF OV-5 Mapping
    // ========================================================================

    /// Convert BPMN Process to DoDAF OV-5 Operational Activity Model
    pub fn bpmn_to_ov5(&mut self, process: &BpmnProcess) -> Result<OperationalActivityModel> {
        let mut activities = Vec::new();
        let mut resource_flows = Vec::new();
        let mut performers = Vec::new();

        // Map BPMN Tasks to DoDAF Activities
        for task in &process.tasks {
            let activity = self.map_task_to_activity(task)?;
            activities.push(activity);
        }

        // Map BPMN Subprocesses to hierarchical activities
        for subprocess in &process.subprocesses {
            let activity = self.map_subprocess_to_activity(subprocess)?;
            activities.push(activity);
        }

        // Map BPMN Sequence Flows to DoDAF Resource Flows
        for flow in &process.sequence_flows {
            if let Some(resource_flow) = self.map_sequence_flow_to_resource_flow(flow)? {
                resource_flows.push(resource_flow);
            }
        }

        // Map BPMN Lanes to DoDAF Performers
        if self.config.map_lanes_to_performers {
            for lane in &process.lanes {
                let performer = self.map_lane_to_performer(lane)?;
                performers.push(performer);
            }
        }

        Ok(OperationalActivityModel {
            id: process.id.clone(),
            name: process.name.clone().unwrap_or_default(),
            description: process.documentation.clone(),
            version: "1.0".to_string(),
            activities,
            resource_flows,
            performers,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata: process.metadata.clone(),
        })
    }

    /// Map BPMN Task to DoDAF Operational Activity
    fn map_task_to_activity(&mut self, task: &BpmnTask) -> Result<OperationalActivity> {
        let mut activity = OperationalActivity::new(
            task.id.clone(),
            task.name.clone().unwrap_or_default(),
            task.documentation.clone().unwrap_or_default(),
        );

        // Map task type to activity characteristics
        match &task.task_type {
            BpmnTaskType::User { .. } => {
                activity.performer = PerformerRef {
                    performer_id: "human_performer".to_string(),
                    role: Some("User".to_string()),
                };
            },
            BpmnTaskType::Service { .. } => {
                activity.performer = PerformerRef {
                    performer_id: "system_performer".to_string(),
                    role: Some("Service".to_string()),
                };
            },
            BpmnTaskType::Script { .. } => {
                activity.performer = PerformerRef {
                    performer_id: "system_performer".to_string(),
                    role: Some("Script".to_string()),
                };
            },
            BpmnTaskType::Manual { .. } => {
                activity.performer = PerformerRef {
                    performer_id: "human_performer".to_string(),
                    role: Some("Manual".to_string()),
                };
            },
            BpmnTaskType::Send { .. } => {
                activity.performer = PerformerRef {
                    performer_id: "system_performer".to_string(),
                    role: Some("Sender".to_string()),
                };
            },
            BpmnTaskType::Receive { .. } => {
                activity.performer = PerformerRef {
                    performer_id: "system_performer".to_string(),
                    role: Some("Receiver".to_string()),
                };
            },
            BpmnTaskType::BusinessRule { .. } => {
                activity.performer = PerformerRef {
                    performer_id: "system_performer".to_string(),
                    role: Some("RulesEngine".to_string()),
                };
            },
            BpmnTaskType::Abstract => {
                activity.performer =
                    PerformerRef { performer_id: "unassigned".to_string(), role: None };
            },
        }

        // Map loop characteristics to frequency
        if let Some(loop_char) = &task.loop_characteristics {
            match loop_char {
                LoopCharacteristics::Standard { loop_maximum, .. } => {
                    if let Some(max) = loop_maximum {
                        activity.frequency =
                            Some(Frequency { occurrences: *max, per_time_unit: TimeUnit::Hours });
                    }
                },
                LoopCharacteristics::MultiInstance { loop_cardinality, .. } => {
                    if let Some(cardinality) = loop_cardinality {
                        activity.frequency = Some(Frequency {
                            occurrences: *cardinality,
                            per_time_unit: TimeUnit::Hours,
                        });
                    }
                },
            }
        }

        // Map properties
        activity.metadata = task.properties.clone();

        // Store mapping
        self.element_map.insert(task.id.clone(), activity.id.clone());

        Ok(activity)
    }

    /// Map BPMN Subprocess to DoDAF Hierarchical Activity
    fn map_subprocess_to_activity(
        &mut self,
        subprocess: &Subprocess,
    ) -> Result<OperationalActivity> {
        let mut activity = OperationalActivity::new(
            subprocess.id.clone(),
            subprocess.name.clone().unwrap_or_default(),
            subprocess.documentation.clone().unwrap_or_default(),
        );

        // If subprocess has a process, map child activities
        if let Some(process) = &subprocess.process {
            for task in &process.tasks {
                activity.child_activities.push(task.id.clone());
            }
        }

        self.element_map.insert(subprocess.id.clone(), activity.id.clone());
        Ok(activity)
    }

    /// Map BPMN Sequence Flow to DoDAF Resource Flow
    fn map_sequence_flow_to_resource_flow(
        &self,
        flow: &SequenceFlow,
    ) -> Result<Option<ResourceFlow>> {
        // Sequence flows represent control flow, which we map to information flow
        let resource_flow = ResourceFlow::new(
            flow.id.clone(),
            flow.source_ref.clone(),
            flow.target_ref.clone(),
            ResourceType::Information,
        );

        Ok(Some(resource_flow))
    }

    /// Map BPMN Lane to DoDAF Performer
    fn map_lane_to_performer(&self, lane: &Lane) -> Result<Performer> {
        let performer = Performer {
            id: lane.id.clone(),
            name: lane.name.clone().unwrap_or_default(),
            description: None,
            performer_type: PerformerType::Organization {
                org_type: "Department".to_string(),
                parent_org: None,
            },
            properties: HashMap::new(),
        };

        Ok(performer)
    }

    // ========================================================================
    // DoDAF OV-5 → BPMN Mapping
    // ========================================================================

    /// Convert DoDAF OV-5 to BPMN Process
    pub fn ov5_to_bpmn(&mut self, model: &OperationalActivityModel) -> Result<BpmnProcess> {
        let mut tasks = Vec::new();
        let mut sequence_flows = Vec::new();
        let mut lanes = Vec::new();

        // Map DoDAF Activities to BPMN Tasks
        for activity in &model.activities {
            let task = self.map_activity_to_task(activity)?;
            tasks.push(task);
        }

        // Map DoDAF Resource Flows to BPMN Sequence Flows
        for resource_flow in &model.resource_flows {
            if let Some(flow) = self.map_resource_flow_to_sequence_flow(resource_flow)? {
                sequence_flows.push(flow);
            }
        }

        // Map DoDAF Performers to BPMN Lanes
        for performer in &model.performers {
            let lane = self.map_performer_to_lane(performer)?;
            lanes.push(lane);
        }

        Ok(BpmnProcess {
            id: model.id.clone(),
            name: Some(model.name.clone()),
            documentation: model.description.clone(),
            is_executable: true,
            process_type: ProcessType::None,
            start_events: vec![],
            end_events: vec![],
            intermediate_events: vec![],
            tasks,
            subprocesses: vec![],
            gateways: vec![],
            sequence_flows,
            data_objects: vec![],
            data_associations: vec![],
            text_annotations: vec![],
            groups: vec![],
            lanes,
            metadata: model.metadata.clone(),
        })
    }

    /// Map DoDAF Activity to BPMN Task
    fn map_activity_to_task(&mut self, activity: &OperationalActivity) -> Result<BpmnTask> {
        // Determine BPMN task type from performer
        let task_type = match &activity.performer.role {
            Some(role) if role == "User" || role == "Manual" => {
                BpmnTaskType::User { implementation: None, rendering: None }
            },
            Some(role) if role == "Service" => {
                BpmnTaskType::Service { implementation: None, operation_ref: None }
            },
            Some(role) if role == "Script" => BpmnTaskType::Script {
                script_format: "application/javascript".to_string(),
                script: String::new(),
            },
            _ => BpmnTaskType::Abstract,
        };

        let task = BpmnTask {
            id: activity.id.clone(),
            name: Some(activity.name.clone()),
            documentation: Some(activity.description.clone()),
            task_type,
            default_flow: None,
            io_specification: None,
            properties: activity.metadata.clone(),
            loop_characteristics: None,
            is_for_compensation: false,
        };

        self.element_map.insert(activity.id.clone(), task.id.clone());
        Ok(task)
    }

    /// Map DoDAF Resource Flow to BPMN Sequence Flow
    fn map_resource_flow_to_sequence_flow(
        &self,
        flow: &ResourceFlow,
    ) -> Result<Option<SequenceFlow>> {
        // Only map information flows to sequence flows
        if flow.resource_type == ResourceType::Information {
            let sequence_flow = SequenceFlow {
                id: flow.id.clone(),
                name: Some(flow.name.clone()),
                source_ref: flow.source_activity.clone(),
                target_ref: flow.target_activity.clone(),
                condition_expression: None,
                is_immediate: false,
            };
            Ok(Some(sequence_flow))
        } else {
            Ok(None)
        }
    }

    /// Map DoDAF Performer to BPMN Lane
    fn map_performer_to_lane(&self, performer: &Performer) -> Result<Lane> {
        let lane = Lane {
            id: performer.id.clone(),
            name: Some(performer.name.clone()),
            partition_element_ref: None,
            child_lanes: vec![],
            flow_node_refs: vec![],
        };

        Ok(lane)
    }

    // ========================================================================
    // OV-6c BPMN Persona
    // ========================================================================

    /// Create OV-6c Event-Trace from BPMN Process
    pub fn bpmn_to_ov6c(
        &self,
        process: &BpmnProcess,
        scenario: String,
    ) -> Result<EventTraceDescription> {
        let mut trace_events = Vec::new();
        let mut sequence = 0;

        // Traverse process in execution order
        for task in &process.tasks {
            // Activity start
            trace_events.push(TraceEvent {
                timestamp: chrono::Utc::now(),
                sequence_number: sequence,
                event_type: TraceEventType::ActivityStart,
                activity_id: task.id.clone(),
                performer_id: "performer_ref".to_string(),
                description: format!("Start: {}", task.name.clone().unwrap_or_default()),
            });
            sequence += 1;

            // Activity end
            trace_events.push(TraceEvent {
                timestamp: chrono::Utc::now(),
                sequence_number: sequence,
                event_type: TraceEventType::ActivityEnd,
                activity_id: task.id.clone(),
                performer_id: "performer_ref".to_string(),
                description: format!("End: {}", task.name.clone().unwrap_or_default()),
            });
            sequence += 1;
        }

        Ok(EventTraceDescription {
            id: format!("{}_trace", process.id),
            name: format!("{} Event Trace", process.name.clone().unwrap_or_default()),
            description: process.documentation.clone(),
            scenario,
            trace_events,
            bpmn_ref: Some(process.id.clone()),
        })
    }
}

impl Default for BpmnDodafMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_to_activity_mapping() {
        let mut mapper = BpmnDodafMapper::new();

        let task = BpmnTask {
            id: "task1".to_string(),
            name: Some("Review Application".to_string()),
            documentation: Some("Review submitted application".to_string()),
            task_type: BpmnTaskType::User { implementation: None, rendering: None },
            default_flow: None,
            io_specification: None,
            properties: HashMap::new(),
            loop_characteristics: None,
            is_for_compensation: false,
        };

        let activity = mapper.map_task_to_activity(&task).unwrap();
        assert_eq!(activity.id, "task1");
        assert_eq!(activity.name, "Review Application");
    }

    #[test]
    fn test_sequence_flow_to_resource_flow() {
        let mapper = BpmnDodafMapper::new();

        let flow = SequenceFlow {
            id: "flow1".to_string(),
            name: Some("Next".to_string()),
            source_ref: "task1".to_string(),
            target_ref: "task2".to_string(),
            condition_expression: None,
            is_immediate: false,
        };

        let resource_flow = mapper.map_sequence_flow_to_resource_flow(&flow).unwrap();
        assert!(resource_flow.is_some());

        let rf = resource_flow.unwrap();
        assert_eq!(rf.resource_type, ResourceType::Information);
    }
}
