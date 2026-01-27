//! CMMN 1.1 (Case Management Model and Notation) Implementation
//!
//! This module implements CMMN 1.1 elements for case management.
//! CMMN is used for modeling knowledge-intensive, unstructured work that adapts based on context.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// CMMN Case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CmmnCase {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub case_plan_model: CasePlanModel,
    pub case_file: CaseFile,
}

/// Case Plan Model - the main container for case planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CasePlanModel {
    pub id: String,
    pub name: String,
    pub plan_items: Vec<PlanItem>,
    pub sentries: Vec<Sentry>,
}

/// Plan Items - elements within a case
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PlanItem {
    /// Human Task
    HumanTask {
        id: String,
        name: String,
        performer: Option<String>,
        documentation: Option<String>,
        entry_criteria: Vec<String>, // Sentry IDs
        exit_criteria: Vec<String>,  // Sentry IDs
        required: bool,
        repeatable: bool,
    },
    /// Process Task (calls BPMN process)
    ProcessTask {
        id: String,
        name: String,
        process_ref: String, // Reference to BPMN process
        entry_criteria: Vec<String>,
        exit_criteria: Vec<String>,
        required: bool,
    },
    /// Case Task (calls another case)
    CaseTask {
        id: String,
        name: String,
        case_ref: String, // Reference to another CMMN case
        entry_criteria: Vec<String>,
        exit_criteria: Vec<String>,
    },
    /// Decision Task (calls DMN decision)
    DecisionTask {
        id: String,
        name: String,
        decision_ref: String, // Reference to DMN decision
        entry_criteria: Vec<String>,
        exit_criteria: Vec<String>,
    },
    /// Milestone - represents an achievable goal
    Milestone {
        id: String,
        name: String,
        entry_criteria: Vec<String>,
    },
    /// Stage - a grouping of plan items
    Stage {
        id: String,
        name: String,
        plan_items: Vec<PlanItem>,
        entry_criteria: Vec<String>,
        exit_criteria: Vec<String>,
        auto_complete: bool,
    },
    /// Event Listener
    EventListener {
        id: String,
        name: String,
        event_type: EventType,
    },
}

/// Sentry - guards that control plan item lifecycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sentry {
    pub id: String,
    pub name: String,
    pub on_parts: Vec<OnPart>,  // Events that trigger the sentry
    pub if_part: Option<String>, // Condition expression
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnPart {
    pub source_ref: String,     // Plan item ID
    pub standard_event: StandardEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StandardEvent {
    Create,
    Enable,
    Disable,
    Start,
    Complete,
    Terminate,
    Suspend,
    Resume,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    Timer,
    User,
    Signal,
}

/// Case File - data context for the case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseFile {
    pub items: HashMap<String, CaseFileItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseFileItem {
    pub id: String,
    pub name: String,
    pub definition_ref: Option<String>,
    pub multiplicity: Multiplicity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Multiplicity {
    ZeroOrOne,
    ExactlyOne,
    ZeroOrMore,
    OneOrMore,
}

impl CmmnCase {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            case_plan_model: CasePlanModel {
                id: "case_plan_1".to_string(),
                name: "Main Plan".to_string(),
                plan_items: Vec::new(),
                sentries: Vec::new(),
            },
            case_file: CaseFile {
                items: HashMap::new(),
            },
        }
    }

    pub fn add_plan_item(&mut self, item: PlanItem) -> &mut Self {
        self.case_plan_model.plan_items.push(item);
        self
    }

    pub fn add_sentry(&mut self, sentry: Sentry) -> &mut Self {
        self.case_plan_model.sentries.push(sentry);
        self
    }

    pub fn add_case_file_item(&mut self, item: CaseFileItem) -> &mut Self {
        self.case_file.items.insert(item.id.clone(), item);
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        // Validate that all referenced sentries exist
        let sentry_ids: Vec<&String> = self.case_plan_model.sentries.iter().map(|s| &s.id).collect();

        for item in &self.case_plan_model.plan_items {
            let (entry_criteria, exit_criteria) = match item {
                PlanItem::HumanTask { entry_criteria, exit_criteria, .. } => (entry_criteria, exit_criteria),
                PlanItem::ProcessTask { entry_criteria, exit_criteria, .. } => (entry_criteria, exit_criteria),
                PlanItem::CaseTask { entry_criteria, exit_criteria, .. } => (entry_criteria, exit_criteria),
                PlanItem::DecisionTask { entry_criteria, exit_criteria, .. } => (entry_criteria, exit_criteria),
                PlanItem::Milestone { entry_criteria, .. } => (entry_criteria, &vec![]),
                PlanItem::Stage { entry_criteria, exit_criteria, .. } => (entry_criteria, exit_criteria),
                PlanItem::EventListener { .. } => continue,
            };

            for sentry_ref in entry_criteria.iter().chain(exit_criteria.iter()) {
                if !sentry_ids.contains(&sentry_ref) {
                    return Err(format!("Sentry '{}' referenced but not defined", sentry_ref));
                }
            }
        }

        // Validate sentries reference existing plan items
        for sentry in &self.case_plan_model.sentries {
            for on_part in &sentry.on_parts {
                if !self.has_plan_item(&on_part.source_ref) {
                    return Err(format!("Sentry '{}' references non-existent plan item '{}'",
                        sentry.id, on_part.source_ref));
                }
            }
        }

        Ok(())
    }

    pub(crate) fn has_plan_item(&self, item_id: &str) -> bool {
        self.case_plan_model.plan_items.iter().any(|item| {
            match item {
                PlanItem::HumanTask { id, .. } => id == item_id,
                PlanItem::ProcessTask { id, .. } => id == item_id,
                PlanItem::CaseTask { id, .. } => id == item_id,
                PlanItem::DecisionTask { id, .. } => id == item_id,
                PlanItem::Milestone { id, .. } => id == item_id,
                PlanItem::Stage { id, .. } => id == item_id,
                PlanItem::EventListener { id, .. } => id == item_id,
            }
        })
    }

    /// Get a plan item by ID
    pub fn get_plan_item(&self, item_id: &str) -> Option<&PlanItem> {
        self.case_plan_model.plan_items.iter().find(|item| {
            match item {
                PlanItem::HumanTask { id, .. } => id == item_id,
                PlanItem::ProcessTask { id, .. } => id == item_id,
                PlanItem::CaseTask { id, .. } => id == item_id,
                PlanItem::DecisionTask { id, .. } => id == item_id,
                PlanItem::Milestone { id, .. } => id == item_id,
                PlanItem::Stage { id, .. } => id == item_id,
                PlanItem::EventListener { id, .. } => id == item_id,
            }
        })
    }

    /// Get a sentry by ID
    pub fn get_sentry(&self, sentry_id: &str) -> Option<&Sentry> {
        self.case_plan_model.sentries.iter().find(|s| s.id == sentry_id)
    }

    /// Get all sentries that reference a specific plan item
    pub fn get_sentries_for_item(&self, item_id: &str) -> Vec<&Sentry> {
        self.case_plan_model.sentries.iter().filter(|s| {
            s.on_parts.iter().any(|on_part| on_part.source_ref == item_id)
        }).collect()
    }

    /// Get all plan items with a specific type
    pub fn get_items_of_type<T: std::any::Any>(&self) -> Vec<&PlanItem> {
        self.case_plan_model.plan_items.iter().collect()
    }

    /// Get required plan items
    pub fn get_required_items(&self) -> Vec<&PlanItem> {
        self.case_plan_model.plan_items.iter().filter(|item| {
            match item {
                PlanItem::HumanTask { required, .. } => *required,
                PlanItem::ProcessTask { required, .. } => *required,
                _ => false,
            }
        }).collect()
    }

    /// Get discretionary items (optional plan items)
    pub fn get_discretionary_items(&self) -> Vec<&PlanItem> {
        self.case_plan_model.plan_items.iter().filter(|item| {
            match item {
                PlanItem::HumanTask { required, .. } => !*required,
                PlanItem::ProcessTask { required, .. } => !*required,
                _ => true,
            }
        }).collect()
    }

    /// Count plan items by type
    pub fn count_items_by_type(&self) -> std::collections::HashMap<&'static str, usize> {
        let mut counts = std::collections::HashMap::new();
        for item in &self.case_plan_model.plan_items {
            let type_name = match item {
                PlanItem::HumanTask { .. } => "HumanTask",
                PlanItem::ProcessTask { .. } => "ProcessTask",
                PlanItem::CaseTask { .. } => "CaseTask",
                PlanItem::DecisionTask { .. } => "DecisionTask",
                PlanItem::Milestone { .. } => "Milestone",
                PlanItem::Stage { .. } => "Stage",
                PlanItem::EventListener { .. } => "EventListener",
            };
            *counts.entry(type_name).or_insert(0) += 1;
        }
        counts
    }

    /// Export to CMMN 1.1 XML (simplified)
    pub fn to_cmmn_xml(&self) -> String {
        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<definitions xmlns=\"http://www.omg.org/spec/CMMN/20151109/MODEL\">\n");
        xml.push_str(&format!("  <case id=\"{}\" name=\"{}\">\n", self.id, self.name));
        xml.push_str(&format!("    <casePlanModel id=\"{}\" name=\"{}\">\n",
            self.case_plan_model.id, self.case_plan_model.name));

        for item in &self.case_plan_model.plan_items {
            xml.push_str(&self.plan_item_to_xml(item, 6));
        }

        xml.push_str("    </casePlanModel>\n");
        xml.push_str("  </case>\n");
        xml.push_str("</definitions>");
        xml
    }

    fn plan_item_to_xml(&self, item: &PlanItem, indent: usize) -> String {
        let spaces = " ".repeat(indent);
        match item {
            PlanItem::HumanTask { id, name, .. } => {
                format!("{}<planItem id=\"{}\" name=\"{}\" definitionRef=\"humanTask_{}\" />\n",
                    spaces, id, name, id)
            }
            PlanItem::ProcessTask { id, name, process_ref, .. } => {
                format!("{}<planItem id=\"{}\" name=\"{}\" definitionRef=\"{}\" />\n",
                    spaces, id, name, process_ref)
            }
            PlanItem::Milestone { id, name, .. } => {
                format!("{}<planItem id=\"{}\" name=\"{}\" definitionRef=\"milestone_{}\" />\n",
                    spaces, id, name, id)
            }
            _ => format!("{}<!-- Plan item {} not yet implemented in XML export -->\n",
                spaces, self.get_plan_item_id(item)),
        }
    }

    fn get_plan_item_id<'a>(&self, item: &'a PlanItem) -> &'a str {
        match item {
            PlanItem::HumanTask { id, .. } => id,
            PlanItem::ProcessTask { id, .. } => id,
            PlanItem::CaseTask { id, .. } => id,
            PlanItem::DecisionTask { id, .. } => id,
            PlanItem::Milestone { id, .. } => id,
            PlanItem::Stage { id, .. } => id,
            PlanItem::EventListener { id, .. } => id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmmn_case_creation() {
        let mut case = CmmnCase::new("case1", "Customer Inquiry");

        case.add_plan_item(PlanItem::HumanTask {
            id: "task1".to_string(),
            name: "Review Inquiry".to_string(),
            performer: Some("agent".to_string()),
            documentation: None,
            entry_criteria: vec![],
            exit_criteria: vec![],
            required: true,
            repeatable: false,
        });

        case.add_plan_item(PlanItem::Milestone {
            id: "milestone1".to_string(),
            name: "Inquiry Resolved".to_string(),
            entry_criteria: vec![],
        });

        assert_eq!(case.case_plan_model.plan_items.len(), 2);
        assert!(case.validate().is_ok());
    }

    #[test]
    fn test_cmmn_sentry_validation() {
        let mut case = CmmnCase::new("case1", "Test");

        case.add_plan_item(PlanItem::HumanTask {
            id: "task1".to_string(),
            name: "Task".to_string(),
            performer: None,
            documentation: None,
            entry_criteria: vec!["sentry1".to_string()],
            exit_criteria: vec![],
            required: false,
            repeatable: false,
        });

        // Should fail - sentry not defined
        assert!(case.validate().is_err());

        // Add the sentry
        case.add_sentry(Sentry {
            id: "sentry1".to_string(),
            name: "Entry Sentry".to_string(),
            on_parts: vec![],
            if_part: None,
        });

        // Should pass now
        assert!(case.validate().is_ok());
    }
}
