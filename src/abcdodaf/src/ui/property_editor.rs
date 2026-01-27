//! Property Editor for BPMN Nodes
//!
//! Provides interactive editing UI for all BPMN 2.0 node types and DoDAF metadata.

use super::enhanced_nodes::*;
use crate::bpmn::elements::*;
use crate::dodaf::ov5::*;
use egui::{Color32, RichText, Ui};
use std::collections::HashMap;

/// Property editor state
#[derive(Default)]
pub struct PropertyEditor {
    /// Edit buffers for string fields
    #[allow(dead_code)]
    edit_buffers: HashMap<String, String>,
    /// Edit buffers for numeric fields
    numeric_buffers: HashMap<String, String>,
    /// Checkbox states
    #[allow(dead_code)]
    checkbox_states: HashMap<String, bool>,
}

impl PropertyEditor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Show property editor for a node
    pub fn show_properties(&mut self, ui: &mut Ui, node: &mut EnhancedBpmnNode) -> bool {
        let mut changed = false;

        // Node ID (read-only)
        ui.horizontal(|ui| {
            ui.label(RichText::new("ID:").strong());
            ui.label(&node.id);
        });

        ui.separator();

        // Edit node based on type
        match &mut node.node_type {
            BpmnNodeType::StartEvent(event) => {
                changed |= self.edit_start_event(ui, event);
            },
            BpmnNodeType::EndEvent(event) => {
                changed |= self.edit_end_event(ui, event);
            },
            BpmnNodeType::IntermediateEvent(event) => {
                changed |= self.edit_intermediate_event(ui, event);
            },
            BpmnNodeType::Task(task) => {
                changed |= self.edit_task(ui, task);
            },
            BpmnNodeType::Gateway(gateway) => {
                changed |= self.edit_gateway(ui, gateway);
            },
            BpmnNodeType::Subprocess(subprocess) => {
                changed |= self.edit_subprocess(ui, subprocess);
            },
            BpmnNodeType::DataObject(data_obj) => {
                changed |= self.edit_data_object(ui, data_obj);
            },
            BpmnNodeType::DataStore(data_store) => {
                changed |= self.edit_data_store(ui, data_store);
            },
            BpmnNodeType::TextAnnotation(annotation) => {
                changed |= self.edit_text_annotation(ui, annotation);
            },
            BpmnNodeType::Group(group) => {
                changed |= self.edit_group(ui, group);
            },
        }

        ui.separator();

        // Visual Markers Editor
        ui.heading("Visual Markers");
        changed |= self.edit_visual_markers(ui, &mut node.visual.markers);

        ui.separator();

        // DoDAF Metadata Editor
        ui.heading(RichText::new("DoDAF Metadata").color(Color32::BLUE));

        if node.dodaf_metadata.is_none() {
            if ui.button("➕ Add DoDAF Metadata").clicked() {
                node.dodaf_metadata = Some(DodafNodeMetadata {
                    activity_ref: None,
                    performer: None,
                    cost: None,
                    duration: None,
                    security_domain: None,
                    dodaf_properties: HashMap::new(),
                });
                changed = true;
            }
        } else {
            changed |= self.edit_dodaf_metadata(ui, node.dodaf_metadata.as_mut().unwrap());

            if ui.button("➖ Remove DoDAF Metadata").clicked() {
                node.dodaf_metadata = None;
                changed = true;
            }
        }

        changed
    }

    // ========================================================================
    // Event Editors
    // ========================================================================

    fn edit_start_event(&mut self, ui: &mut Ui, event: &mut StartEventNode) -> bool {
        let mut changed = false;

        ui.heading("Start Event");

        // Name
        changed |= self.edit_text_field(ui, "Name", &mut event.name);

        // Documentation
        changed |= self.edit_optional_text_area(ui, "Documentation", &mut event.documentation);

        // Is Interrupting
        ui.horizontal(|ui| {
            ui.label("Interrupting:");
            if ui.checkbox(&mut event.is_interrupting, "").changed() {
                changed = true;
            }
        });

        // Event Definition
        changed |= self.edit_event_definition(ui, &mut event.event_definition);

        changed
    }

    fn edit_end_event(&mut self, ui: &mut Ui, event: &mut EndEventNode) -> bool {
        let mut changed = false;

        ui.heading("End Event");

        // Name
        changed |= self.edit_text_field(ui, "Name", &mut event.name);

        // Documentation
        changed |= self.edit_optional_text_area(ui, "Documentation", &mut event.documentation);

        // Event Definition
        changed |= self.edit_event_definition(ui, &mut event.event_definition);

        changed
    }

    fn edit_intermediate_event(&mut self, ui: &mut Ui, event: &mut IntermediateEventNode) -> bool {
        let mut changed = false;

        ui.heading("Intermediate Event");

        // Name
        changed |= self.edit_text_field(ui, "Name", &mut event.name);

        // Documentation
        changed |= self.edit_optional_text_area(ui, "Documentation", &mut event.documentation);

        // Event properties
        ui.horizontal(|ui| {
            ui.label("Catching:");
            if ui.checkbox(&mut event.is_catching, "").changed() {
                changed = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Interrupting:");
            if ui.checkbox(&mut event.is_interrupting, "").changed() {
                changed = true;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Boundary:");
            if ui.checkbox(&mut event.is_boundary, "").changed() {
                changed = true;
            }
        });

        // Event Definition
        changed |= self.edit_event_definition(ui, &mut event.event_definition);

        changed
    }

    fn edit_event_definition(
        &mut self,
        ui: &mut Ui,
        event_def: &mut Option<EventDefinition>,
    ) -> bool {
        let mut changed = false;

        ui.collapsing("Event Definition", |ui| {
            let current_type = match event_def {
                None => "None",
                Some(EventDefinition::Message { .. }) => "Message",
                Some(EventDefinition::Timer { .. }) => "Timer",
                Some(EventDefinition::Signal { .. }) => "Signal",
                Some(EventDefinition::Error { .. }) => "Error",
                Some(EventDefinition::Escalation { .. }) => "Escalation",
                Some(EventDefinition::Cancel) => "Cancel",
                Some(EventDefinition::Compensation) => "Compensation",
                Some(EventDefinition::Conditional { .. }) => "Conditional",
                Some(EventDefinition::Link { .. }) => "Link",
                Some(EventDefinition::Terminate) => "Terminate",
                Some(EventDefinition::Multiple { is_parallel: false, .. }) => "Multiple",
                Some(EventDefinition::Multiple { is_parallel: true, .. }) => "Parallel Multiple",
                Some(EventDefinition::None) => "None",
            };

            egui::ComboBox::from_label("Type")
                .selected_text(current_type)
                .show_ui(ui, |ui| {
                    if ui.selectable_label(current_type == "None", "None").clicked() {
                        *event_def = None;
                        changed = true;
                    }
                    if ui.selectable_label(current_type == "Message", "Message").clicked() {
                        *event_def = Some(EventDefinition::Message { message_ref: None });
                        changed = true;
                    }
                    if ui.selectable_label(current_type == "Timer", "Timer").clicked() {
                        *event_def =
                            Some(EventDefinition::Timer { time_expression: "PT1H".to_string() });
                        changed = true;
                    }
                    if ui.selectable_label(current_type == "Signal", "Signal").clicked() {
                        *event_def = Some(EventDefinition::Signal { signal_ref: None });
                        changed = true;
                    }
                    if ui.selectable_label(current_type == "Error", "Error").clicked() {
                        *event_def = Some(EventDefinition::Error { error_ref: None });
                        changed = true;
                    }
                    if ui.selectable_label(current_type == "Terminate", "Terminate").clicked() {
                        *event_def = Some(EventDefinition::Terminate);
                        changed = true;
                    }
                });

            // Edit specific event definition fields
            if let Some(def) = event_def {
                match def {
                    EventDefinition::Message { message_ref } => {
                        changed |= self.edit_optional_text_field(ui, "Message Ref", message_ref);
                    },
                    EventDefinition::Timer { time_expression } => {
                        changed |= self.edit_text_field(ui, "Time Expression", time_expression);
                    },
                    EventDefinition::Signal { signal_ref } => {
                        changed |= self.edit_optional_text_field(ui, "Signal Ref", signal_ref);
                    },
                    EventDefinition::Error { error_ref } => {
                        changed |= self.edit_optional_text_field(ui, "Error Ref", error_ref);
                    },
                    EventDefinition::Conditional { condition } => {
                        changed |= self.edit_text_field(ui, "Condition", condition);
                    },
                    _ => {},
                }
            }
        });

        changed
    }

    // ========================================================================
    // Task Editor
    // ========================================================================

    fn edit_task(&mut self, ui: &mut Ui, task: &mut TaskNode) -> bool {
        let mut changed = false;

        ui.heading("Task");

        // Name
        changed |= self.edit_text_field(ui, "Name", &mut task.name);

        // Documentation
        changed |= self.edit_optional_text_area(ui, "Documentation", &mut task.documentation);

        // Task Type
        let task_type_name = match &task.task_type {
            BpmnTaskType::Abstract => "Abstract",
            BpmnTaskType::User { .. } => "User",
            BpmnTaskType::Service { .. } => "Service",
            BpmnTaskType::Script { .. } => "Script",
            BpmnTaskType::BusinessRule { .. } => "Business Rule",
            BpmnTaskType::Manual => "Manual",
            BpmnTaskType::Send { .. } => "Send",
            BpmnTaskType::Receive { .. } => "Receive",
        };

        egui::ComboBox::from_label("Task Type")
            .selected_text(task_type_name)
            .show_ui(ui, |ui| {
                if ui.selectable_label(task_type_name == "User", "User").clicked() {
                    task.task_type = BpmnTaskType::User { implementation: None, rendering: None };
                    changed = true;
                }
                if ui.selectable_label(task_type_name == "Service", "Service").clicked() {
                    task.task_type =
                        BpmnTaskType::Service { implementation: None, operation_ref: None };
                    changed = true;
                }
                if ui.selectable_label(task_type_name == "Script", "Script").clicked() {
                    task.task_type = BpmnTaskType::Script {
                        script_format: "javascript".to_string(),
                        script: String::new(),
                    };
                    changed = true;
                }
                if ui
                    .selectable_label(task_type_name == "Business Rule", "Business Rule")
                    .clicked()
                {
                    task.task_type =
                        BpmnTaskType::BusinessRule { implementation: None, rule_ref: None };
                    changed = true;
                }
                if ui.selectable_label(task_type_name == "Manual", "Manual").clicked() {
                    task.task_type = BpmnTaskType::Manual;
                    changed = true;
                }
            });

        // Task-type specific fields
        match &mut task.task_type {
            BpmnTaskType::User { implementation, rendering } => {
                changed |= self.edit_optional_text_field(ui, "Implementation", implementation);
                changed |= self.edit_optional_text_field(ui, "Rendering", rendering);
            },
            BpmnTaskType::Service { implementation, operation_ref } => {
                changed |= self.edit_optional_text_field(ui, "Implementation", implementation);
                changed |= self.edit_optional_text_field(ui, "Operation Ref", operation_ref);
            },
            BpmnTaskType::Script { script_format, script } => {
                changed |= self.edit_text_field(ui, "Script Format", script_format);
                changed |= self.edit_text_area(ui, "Script", script);
            },
            BpmnTaskType::BusinessRule { implementation, rule_ref } => {
                changed |= self.edit_optional_text_field(ui, "Implementation", implementation);
                changed |= self.edit_optional_text_field(ui, "Rule Ref", rule_ref);
            },
            _ => {},
        }

        // Compensation
        ui.horizontal(|ui| {
            ui.label("For Compensation:");
            if ui.checkbox(&mut task.is_for_compensation, "").changed() {
                changed = true;
            }
        });

        changed
    }

    // ========================================================================
    // Gateway Editor
    // ========================================================================

    fn edit_gateway(&mut self, ui: &mut Ui, gateway: &mut GatewayNode) -> bool {
        let mut changed = false;

        ui.heading("Gateway");

        // Name
        changed |= self.edit_text_field(ui, "Name", &mut gateway.name);

        // Documentation
        changed |= self.edit_optional_text_area(ui, "Documentation", &mut gateway.documentation);

        // Gateway Type
        let gateway_type_name = match &gateway.gateway_type {
            BpmnGatewayType::Exclusive => "Exclusive (XOR)",
            BpmnGatewayType::Parallel => "Parallel (AND)",
            BpmnGatewayType::Inclusive => "Inclusive (OR)",
            BpmnGatewayType::EventBased { .. } => "Event-Based",
            BpmnGatewayType::ParallelEventBased => "Parallel Event-Based",
            BpmnGatewayType::Complex { .. } => "Complex",
        };

        ui.label("Gateway Type:");
        ui.label(gateway_type_name);

        // Gateway Direction
        let direction_name = match gateway.gateway_direction {
            GatewayDirection::Unspecified => "Unspecified",
            GatewayDirection::Diverging => "Diverging (Split)",
            GatewayDirection::Converging => "Converging (Merge)",
            GatewayDirection::Mixed => "Mixed",
        };

        egui::ComboBox::from_label("Direction")
            .selected_text(direction_name)
            .show_ui(ui, |ui| {
                if ui.selectable_label(direction_name == "Unspecified", "Unspecified").clicked() {
                    gateway.gateway_direction = GatewayDirection::Unspecified;
                    changed = true;
                }
                if ui
                    .selectable_label(direction_name == "Diverging (Split)", "Diverging (Split)")
                    .clicked()
                {
                    gateway.gateway_direction = GatewayDirection::Diverging;
                    changed = true;
                }
                if ui
                    .selectable_label(direction_name == "Converging (Merge)", "Converging (Merge)")
                    .clicked()
                {
                    gateway.gateway_direction = GatewayDirection::Converging;
                    changed = true;
                }
                if ui.selectable_label(direction_name == "Mixed", "Mixed").clicked() {
                    gateway.gateway_direction = GatewayDirection::Mixed;
                    changed = true;
                }
            });

        changed
    }

    // ========================================================================
    // Other Node Type Editors
    // ========================================================================

    fn edit_subprocess(&mut self, ui: &mut Ui, subprocess: &mut SubprocessNode) -> bool {
        let mut changed = false;
        ui.heading("Subprocess");
        changed |= self.edit_text_field(ui, "Name", &mut subprocess.name);
        changed |= self.edit_optional_text_area(ui, "Documentation", &mut subprocess.documentation);
        changed
    }

    fn edit_data_object(&mut self, ui: &mut Ui, data_obj: &mut DataObjectNode) -> bool {
        let mut changed = false;
        ui.heading("Data Object");
        changed |= self.edit_text_field(ui, "Name", &mut data_obj.name);
        ui.horizontal(|ui| {
            ui.label("Is Collection:");
            if ui.checkbox(&mut data_obj.is_collection, "").changed() {
                changed = true;
            }
        });
        changed |= self.edit_optional_text_field(ui, "Data State", &mut data_obj.data_state);
        changed
    }

    fn edit_data_store(&mut self, ui: &mut Ui, data_store: &mut DataStoreNode) -> bool {
        let mut changed = false;
        ui.heading("Data Store");
        changed |= self.edit_text_field(ui, "Name", &mut data_store.name);
        ui.horizontal(|ui| {
            ui.label("Unlimited:");
            if ui.checkbox(&mut data_store.is_unlimited, "").changed() {
                changed = true;
            }
        });
        if !data_store.is_unlimited {
            changed |= self.edit_optional_i32_field(ui, "Capacity", &mut data_store.capacity);
        }
        changed
    }

    fn edit_text_annotation(&mut self, ui: &mut Ui, annotation: &mut TextAnnotationNode) -> bool {
        let mut changed = false;
        ui.heading("Text Annotation");
        changed |= self.edit_text_area(ui, "Text", &mut annotation.text);
        changed |= self.edit_text_field(ui, "Text Format", &mut annotation.text_format);
        changed
    }

    fn edit_group(&mut self, ui: &mut Ui, group: &mut GroupNode) -> bool {
        let mut changed = false;
        ui.heading("Group");
        changed |= self.edit_optional_text_field(ui, "Category", &mut group.category);
        changed
    }

    // ========================================================================
    // Visual Markers Editor
    // ========================================================================

    fn edit_visual_markers(&mut self, ui: &mut Ui, markers: &mut Vec<VisualMarker>) -> bool {
        let mut changed = false;

        ui.label("Add markers to indicate special behavior:");

        // Loop marker
        let has_loop = markers.iter().any(|m| matches!(m, VisualMarker::Loop));
        let mut loop_checked = has_loop;
        ui.horizontal(|ui| {
            if ui.checkbox(&mut loop_checked, "↻ Loop").changed() {
                if loop_checked && !has_loop {
                    markers.push(VisualMarker::Loop);
                    changed = true;
                } else if !loop_checked && has_loop {
                    markers.retain(|m| !matches!(m, VisualMarker::Loop));
                    changed = true;
                }
            }
            ui.label("(Activity repeats)");
        });

        // Multi-instance marker
        let has_multi = markers.iter().any(|m| matches!(m, VisualMarker::MultiInstance));
        let mut multi_checked = has_multi;
        ui.horizontal(|ui| {
            if ui.checkbox(&mut multi_checked, "||| Multi-Instance").changed() {
                if multi_checked && !has_multi {
                    markers.push(VisualMarker::MultiInstance);
                    changed = true;
                } else if !multi_checked && has_multi {
                    markers.retain(|m| !matches!(m, VisualMarker::MultiInstance));
                    changed = true;
                }
            }
            ui.label("(Parallel instances)");
        });

        // Compensation marker
        let has_comp = markers.iter().any(|m| matches!(m, VisualMarker::Compensation));
        let mut comp_checked = has_comp;
        ui.horizontal(|ui| {
            if ui.checkbox(&mut comp_checked, "⏪ Compensation").changed() {
                if comp_checked && !has_comp {
                    markers.push(VisualMarker::Compensation);
                    changed = true;
                } else if !comp_checked && has_comp {
                    markers.retain(|m| !matches!(m, VisualMarker::Compensation));
                    changed = true;
                }
            }
            ui.label("(Undo action)");
        });

        // Ad-hoc marker
        let has_adhoc = markers.iter().any(|m| matches!(m, VisualMarker::AdHoc));
        let mut adhoc_checked = has_adhoc;
        ui.horizontal(|ui| {
            if ui.checkbox(&mut adhoc_checked, "~ Ad-Hoc").changed() {
                if adhoc_checked && !has_adhoc {
                    markers.push(VisualMarker::AdHoc);
                    changed = true;
                } else if !adhoc_checked && has_adhoc {
                    markers.retain(|m| !matches!(m, VisualMarker::AdHoc));
                    changed = true;
                }
            }
            ui.label("(No predefined sequence)");
        });

        // Collapsed marker
        let has_collapsed = markers.iter().any(|m| matches!(m, VisualMarker::Collapsed));
        let mut collapsed_checked = has_collapsed;
        ui.horizontal(|ui| {
            if ui.checkbox(&mut collapsed_checked, "+ Collapsed").changed() {
                if collapsed_checked && !has_collapsed {
                    markers.push(VisualMarker::Collapsed);
                    changed = true;
                } else if !collapsed_checked && has_collapsed {
                    markers.retain(|m| !matches!(m, VisualMarker::Collapsed));
                    changed = true;
                }
            }
            ui.label("(Subprocess collapsed)");
        });

        changed
    }

    // ========================================================================
    // DoDAF Metadata Editor
    // ========================================================================

    fn edit_dodaf_metadata(&mut self, ui: &mut Ui, metadata: &mut DodafNodeMetadata) -> bool {
        let mut changed = false;

        // Activity Reference
        changed |= self.edit_optional_text_field(ui, "Activity Ref", &mut metadata.activity_ref);

        // Performer
        ui.collapsing("Performer", |ui| {
            if metadata.performer.is_none() {
                if ui.button("➕ Add Performer").clicked() {
                    metadata.performer =
                        Some(PerformerRef { performer_id: "performer_1".to_string(), role: None });
                    changed = true;
                }
            } else if let Some(performer) = &mut metadata.performer {
                changed |= self.edit_text_field(ui, "Performer ID", &mut performer.performer_id);
                changed |= self.edit_optional_text_field(ui, "Role", &mut performer.role);

                if ui.button("➖ Remove Performer").clicked() {
                    metadata.performer = None;
                    changed = true;
                }
            }
        });

        // Cost
        ui.collapsing("Cost", |ui| {
            if metadata.cost.is_none() {
                if ui.button("➕ Add Cost").clicked() {
                    metadata.cost = Some(Cost {
                        amount: 0.0,
                        currency: "USD".to_string(),
                        cost_type: CostType::Estimated,
                    });
                    changed = true;
                }
            } else if let Some(cost) = &mut metadata.cost {
                let mut amount_str = self
                    .numeric_buffers
                    .entry("cost_amount".to_string())
                    .or_insert_with(|| cost.amount.to_string())
                    .clone();

                ui.horizontal(|ui| {
                    ui.label("Amount:");
                    if ui.text_edit_singleline(&mut amount_str).changed() {
                        if let Ok(amount) = amount_str.parse::<f64>() {
                            cost.amount = amount;
                            changed = true;
                        }
                        self.numeric_buffers.insert("cost_amount".to_string(), amount_str);
                    }
                });

                changed |= self.edit_text_field(ui, "Currency", &mut cost.currency);

                let cost_type_name = match cost.cost_type {
                    CostType::Actual => "Actual",
                    CostType::Estimated => "Estimated",
                    CostType::Fixed => "Fixed",
                    CostType::Variable => "Variable",
                };

                egui::ComboBox::from_label("Cost Type").selected_text(cost_type_name).show_ui(
                    ui,
                    |ui| {
                        if ui.selectable_label(cost_type_name == "Actual", "Actual").clicked() {
                            cost.cost_type = CostType::Actual;
                            changed = true;
                        }
                        if ui.selectable_label(cost_type_name == "Estimated", "Estimated").clicked()
                        {
                            cost.cost_type = CostType::Estimated;
                            changed = true;
                        }
                        if ui.selectable_label(cost_type_name == "Fixed", "Fixed").clicked() {
                            cost.cost_type = CostType::Fixed;
                            changed = true;
                        }
                        if ui.selectable_label(cost_type_name == "Variable", "Variable").clicked() {
                            cost.cost_type = CostType::Variable;
                            changed = true;
                        }
                    },
                );

                if ui.button("➖ Remove Cost").clicked() {
                    metadata.cost = None;
                    self.numeric_buffers.remove("cost_amount");
                    changed = true;
                }
            }
        });

        // Duration
        ui.collapsing("Duration", |ui| {
            if metadata.duration.is_none() {
                if ui.button("➕ Add Duration").clicked() {
                    metadata.duration =
                        Some(Duration { value: 1.0, unit: TimeUnit::Hours, is_estimated: true });
                    changed = true;
                }
            } else if let Some(duration) = &mut metadata.duration {
                let mut value_str = self
                    .numeric_buffers
                    .entry("duration_value".to_string())
                    .or_insert_with(|| duration.value.to_string())
                    .clone();

                ui.horizontal(|ui| {
                    ui.label("Value:");
                    if ui.text_edit_singleline(&mut value_str).changed() {
                        if let Ok(value) = value_str.parse::<f64>() {
                            duration.value = value;
                            changed = true;
                        }
                        self.numeric_buffers.insert("duration_value".to_string(), value_str);
                    }
                });

                let unit_name = match duration.unit {
                    TimeUnit::Seconds => "Seconds",
                    TimeUnit::Minutes => "Minutes",
                    TimeUnit::Hours => "Hours",
                    TimeUnit::Days => "Days",
                    TimeUnit::Weeks => "Weeks",
                    TimeUnit::Months => "Months",
                };

                egui::ComboBox::from_label("Unit").selected_text(unit_name).show_ui(ui, |ui| {
                    if ui.selectable_label(unit_name == "Seconds", "Seconds").clicked() {
                        duration.unit = TimeUnit::Seconds;
                        changed = true;
                    }
                    if ui.selectable_label(unit_name == "Minutes", "Minutes").clicked() {
                        duration.unit = TimeUnit::Minutes;
                        changed = true;
                    }
                    if ui.selectable_label(unit_name == "Hours", "Hours").clicked() {
                        duration.unit = TimeUnit::Hours;
                        changed = true;
                    }
                    if ui.selectable_label(unit_name == "Days", "Days").clicked() {
                        duration.unit = TimeUnit::Days;
                        changed = true;
                    }
                    if ui.selectable_label(unit_name == "Weeks", "Weeks").clicked() {
                        duration.unit = TimeUnit::Weeks;
                        changed = true;
                    }
                    if ui.selectable_label(unit_name == "Months", "Months").clicked() {
                        duration.unit = TimeUnit::Months;
                        changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Estimated:");
                    if ui.checkbox(&mut duration.is_estimated, "").changed() {
                        changed = true;
                    }
                });

                if ui.button("➖ Remove Duration").clicked() {
                    metadata.duration = None;
                    self.numeric_buffers.remove("duration_value");
                    changed = true;
                }
            }
        });

        // Security Domain
        ui.collapsing("Security Domain", |ui| {
            if metadata.security_domain.is_none() {
                if ui.button("➕ Add Security Domain").clicked() {
                    metadata.security_domain = Some(SecurityDomain {
                        classification: SecurityClassification::Unclassified,
                        access_control: Vec::new(),
                        constraints: Vec::new(),
                    });
                    changed = true;
                }
            } else if let Some(domain) = &mut metadata.security_domain {
                // Classification level
                let current_classification = domain.classification.clone();
                let classification_name = match &current_classification {
                    SecurityClassification::Unclassified => "Unclassified",
                    SecurityClassification::Confidential => "Confidential",
                    SecurityClassification::Secret => "Secret",
                    SecurityClassification::TopSecret => "Top Secret",
                    SecurityClassification::Custom(s) => s.as_str(),
                };

                egui::ComboBox::from_label("Classification")
                    .selected_text(classification_name)
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_label(
                                matches!(current_classification, SecurityClassification::Unclassified),
                                "Unclassified",
                            )
                            .clicked()
                        {
                            domain.classification = SecurityClassification::Unclassified;
                            changed = true;
                        }
                        if ui
                            .selectable_label(
                                matches!(current_classification, SecurityClassification::Confidential),
                                "Confidential",
                            )
                            .clicked()
                        {
                            domain.classification = SecurityClassification::Confidential;
                            changed = true;
                        }
                        if ui
                            .selectable_label(
                                matches!(current_classification, SecurityClassification::Secret),
                                "Secret",
                            )
                            .clicked()
                        {
                            domain.classification = SecurityClassification::Secret;
                            changed = true;
                        }
                        if ui
                            .selectable_label(
                                matches!(current_classification, SecurityClassification::TopSecret),
                                "Top Secret",
                            )
                            .clicked()
                        {
                            domain.classification = SecurityClassification::TopSecret;
                            changed = true;
                        }
                    });

                ui.separator();

                // Access Control list
                ui.label("Access Control:");
                ui.indent("access_control_indent", |ui| {
                    let mut to_remove = None;
                    for (idx, control) in domain.access_control.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            if ui.text_edit_singleline(control).changed() {
                                changed = true;
                            }
                            if ui.button("✖").clicked() {
                                to_remove = Some(idx);
                            }
                        });
                    }
                    if let Some(idx) = to_remove {
                        domain.access_control.remove(idx);
                        changed = true;
                    }
                    if ui.button("➕ Add Access Control").clicked() {
                        domain.access_control.push(String::new());
                        changed = true;
                    }
                });

                ui.separator();

                // Constraints list
                ui.label("Constraints:");
                ui.indent("constraints_indent", |ui| {
                    let mut to_remove = None;
                    for (idx, constraint) in domain.constraints.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            if ui.text_edit_singleline(constraint).changed() {
                                changed = true;
                            }
                            if ui.button("✖").clicked() {
                                to_remove = Some(idx);
                            }
                        });
                    }
                    if let Some(idx) = to_remove {
                        domain.constraints.remove(idx);
                        changed = true;
                    }
                    if ui.button("➕ Add Constraint").clicked() {
                        domain.constraints.push(String::new());
                        changed = true;
                    }
                });

                ui.separator();

                if ui.button("➖ Remove Security Domain").clicked() {
                    metadata.security_domain = None;
                    changed = true;
                }
            }
        });

        changed
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    fn edit_text_field(&mut self, ui: &mut Ui, label: &str, value: &mut String) -> bool {
        ui.horizontal(|ui| {
            ui.label(format!("{}:", label));
            ui.text_edit_singleline(value).changed()
        })
        .inner
    }

    fn edit_text_area(&mut self, ui: &mut Ui, label: &str, value: &mut String) -> bool {
        ui.label(format!("{}:", label));
        ui.text_edit_multiline(value).changed()
    }

    fn edit_optional_text_field(
        &mut self,
        ui: &mut Ui,
        label: &str,
        value: &mut Option<String>,
    ) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label(format!("{}:", label));
            if value.is_none() {
                if ui.button("➕").clicked() {
                    *value = Some(String::new());
                    changed = true;
                }
            } else {
                let mut text = value.as_ref().unwrap().clone();
                if ui.text_edit_singleline(&mut text).changed() {
                    *value = Some(text);
                    changed = true;
                }
                if ui.button("✖").clicked() {
                    *value = None;
                    changed = true;
                }
            }
        });
        changed
    }

    fn edit_optional_text_area(
        &mut self,
        ui: &mut Ui,
        label: &str,
        value: &mut Option<String>,
    ) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label(format!("{}:", label));
            if value.is_none() {
                if ui.button("➕").clicked() {
                    *value = Some(String::new());
                    changed = true;
                }
            } else if ui.button("✖").clicked() {
                *value = None;
                changed = true;
            }
        });

        if let Some(text) = value {
            if ui.text_edit_multiline(text).changed() {
                changed = true;
            }
        }

        changed
    }

    #[allow(dead_code)]
    fn edit_optional_numeric_field(
        &mut self,
        ui: &mut Ui,
        label: &str,
        value: &mut Option<u64>,
    ) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label(format!("{}:", label));
            if value.is_none() {
                if ui.button("➕").clicked() {
                    *value = Some(0);
                    changed = true;
                }
            } else {
                let key = format!("numeric_{}", label);
                let mut text = self
                    .numeric_buffers
                    .entry(key.clone())
                    .or_insert_with(|| value.unwrap().to_string())
                    .clone();

                if ui.text_edit_singleline(&mut text).changed() {
                    if let Ok(num) = text.parse::<u64>() {
                        *value = Some(num);
                        changed = true;
                    }
                    self.numeric_buffers.insert(key.clone(), text);
                }
                if ui.button("✖").clicked() {
                    *value = None;
                    self.numeric_buffers.remove(&key);
                    changed = true;
                }
            }
        });
        changed
    }

    fn edit_optional_i32_field(
        &mut self,
        ui: &mut Ui,
        label: &str,
        value: &mut Option<i32>,
    ) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label(format!("{}:", label));
            if value.is_none() {
                if ui.button("➕").clicked() {
                    *value = Some(0);
                    changed = true;
                }
            } else {
                let key = format!("numeric_i32_{}", label);
                let mut text = self
                    .numeric_buffers
                    .entry(key.clone())
                    .or_insert_with(|| value.unwrap().to_string())
                    .clone();

                if ui.text_edit_singleline(&mut text).changed() {
                    if let Ok(num) = text.parse::<i32>() {
                        *value = Some(num);
                        changed = true;
                    }
                    self.numeric_buffers.insert(key.clone(), text);
                }
                if ui.button("✖").clicked() {
                    *value = None;
                    self.numeric_buffers.remove(&key);
                    changed = true;
                }
            }
        });
        changed
    }
}
