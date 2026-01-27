//! Built-in workflow templates library
//!
//! Pre-configured templates for common workflow patterns:
//! - Approval workflows (single, multi-level)
//! - ETL (Extract-Transform-Load) patterns
//! - Orchestration patterns
//! - Human-in-the-loop workflows
//! - Error handling patterns

use super::{Template, TemplateMetadata, TemplateCategory, TemplateLibrary};
use super::templates::ParameterConfig;

/// Create the built-in template library
pub fn create_builtin_library() -> TemplateLibrary {
    let mut library = TemplateLibrary::new("ABCDODAF Built-in Templates", "1.0.0");

    // Add all built-in templates
    library.add_template(create_simple_approval_template()).ok();
    library.add_template(create_multi_level_approval_template()).ok();
    library.add_template(create_etl_template()).ok();
    library.add_template(create_parallel_orchestration_template()).ok();
    library.add_template(create_human_in_loop_template()).ok();
    library.add_template(create_error_handling_template()).ok();
    library.add_template(create_notification_template()).ok();
    library.add_template(create_decision_logic_template()).ok();

    library
}

fn create_simple_approval_template() -> Template {
    let metadata = TemplateMetadata {
        id: "approval_simple".to_string(),
        name: "Simple Approval Workflow".to_string(),
        description: "A basic single-level approval workflow for requests".to_string(),
        category: TemplateCategory::ApprovalWorkflow,
        version: "1.0.0".to_string(),
        author: Some("ABCDODAF System".to_string()),
        created_at: chrono::Utc::now(),
        modified_at: chrono::Utc::now(),
        tags: vec![
            "approval".to_string(),
            "request".to_string(),
            "single-level".to_string(),
        ],
        complexity: "beginner".to_string(),
        use_cases: vec![
            "Simple request approval".to_string(),
            "Single approver workflows".to_string(),
        ],
        related_templates: vec!["approval_multilevel".to_string()],
    };

    let mut template = Template::new(
        metadata,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL">
  <bpmn:process id="SimpleApproval" name="Simple Approval">
    <bpmn:startEvent id="StartApproval"/>
    <bpmn:userTask id="SubmitRequest" name="Submit Request"/>
    <bpmn:userTask id="ApproveRequest" name="Approve Request"/>
    <bpmn:serviceTask id="ProcessApproval" name="Process Approval"/>
    <bpmn:endEvent id="ApprovalComplete"/>
  </bpmn:process>
</bpmn:definitions>"#.to_string()
    );

    template.add_parameter(
        "request_id",
        ParameterConfig {
            name: "request_id".to_string(),
            param_type: "string".to_string(),
            default: None,
            description: "Unique request identifier".to_string(),
            required: true,
        }
    );

    template.add_parameter(
        "approver_role",
        ParameterConfig {
            name: "approver_role".to_string(),
            param_type: "string".to_string(),
            default: Some("Manager".to_string()),
            description: "Role of the approver".to_string(),
            required: false,
        }
    );

    template.documentation = r#"# Simple Approval Workflow

## Overview
A straightforward single-level approval workflow for processing requests.

## Flow
1. Request is submitted by initiator
2. Approver reviews and approves/rejects
3. System processes the approval
4. Workflow completes

## Participants
- Initiator: Submits the request
- Approver: Reviews and approves/rejects

## Use Cases
- Purchase order approvals
- Leave request approvals
- Simple access requests
"#.to_string();

    template
}

fn create_multi_level_approval_template() -> Template {
    let metadata = TemplateMetadata {
        id: "approval_multilevel".to_string(),
        name: "Multi-Level Approval Workflow".to_string(),
        description: "Sequential approval workflow with multiple approval levels".to_string(),
        category: TemplateCategory::ApprovalWorkflow,
        version: "1.0.0".to_string(),
        author: Some("ABCDODAF System".to_string()),
        created_at: chrono::Utc::now(),
        modified_at: chrono::Utc::now(),
        tags: vec![
            "approval".to_string(),
            "multi-level".to_string(),
            "hierarchical".to_string(),
        ],
        complexity: "intermediate".to_string(),
        use_cases: vec![
            "Multi-level authorization".to_string(),
            "Hierarchical approvals".to_string(),
        ],
        related_templates: vec!["approval_simple".to_string()],
    };

    let mut template = Template::new(
        metadata,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL">
  <bpmn:process id="MultiLevelApproval" name="Multi-Level Approval">
    <bpmn:startEvent id="Start"/>
    <bpmn:userTask id="SubmitRequest" name="Submit Request"/>
    <bpmn:userTask id="Level1Approval" name="Level 1 Approval"/>
    <bpmn:exclusiveGateway id="Level1Decision"/>
    <bpmn:userTask id="Level2Approval" name="Level 2 Approval"/>
    <bpmn:exclusiveGateway id="Level2Decision"/>
    <bpmn:serviceTask id="ApplyApproval" name="Apply Approval"/>
    <bpmn:endEvent id="End"/>
  </bpmn:process>
</bpmn:definitions>"#.to_string()
    );

    template.documentation = r#"# Multi-Level Approval Workflow

## Overview
Sequential approval process with multiple hierarchical levels.

## Flow
1. Request submission
2. Level 1 Approval (e.g., Department Manager)
3. If approved → Level 2 Approval (e.g., Director)
4. If approved → Process approval and complete
5. If rejected at any level → Reject and end

## Approval Levels
- Level 1: Department/Team Manager
- Level 2: Director or Senior Manager

## Escalation
Rejections are communicated back to initiator.
"#.to_string();

    template
}

fn create_etl_template() -> Template {
    let metadata = TemplateMetadata {
        id: "etl_basic".to_string(),
        name: "Basic ETL Workflow".to_string(),
        description: "Extract-Transform-Load pattern for data processing".to_string(),
        category: TemplateCategory::DataTransformation,
        version: "1.0.0".to_string(),
        author: Some("ABCDODAF System".to_string()),
        created_at: chrono::Utc::now(),
        modified_at: chrono::Utc::now(),
        tags: vec![
            "etl".to_string(),
            "data".to_string(),
            "pipeline".to_string(),
        ],
        complexity: "intermediate".to_string(),
        use_cases: vec![
            "Data migration".to_string(),
            "Data warehouse loading".to_string(),
            "Data integration".to_string(),
        ],
        related_templates: vec![],
    };

    let mut template = Template::new(
        metadata,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL">
  <bpmn:process id="BasicETL" name="Basic ETL">
    <bpmn:startEvent id="Start"/>
    <bpmn:serviceTask id="Extract" name="Extract Data"/>
    <bpmn:serviceTask id="Transform" name="Transform Data"/>
    <bpmn:serviceTask id="Load" name="Load Data"/>
    <bpmn:serviceTask id="Validate" name="Validate"/>
    <bpmn:endEvent id="End"/>
  </bpmn:process>
</bpmn:definitions>"#.to_string()
    );

    template.documentation = r#"# ETL Workflow Pattern

## Overview
Extract-Transform-Load pattern for processing and moving data.

## Phases

### Extract
- Source data collection
- Data connection establishment
- Initial data validation

### Transform
- Data cleaning
- Format conversion
- Business rule application
- Data enrichment

### Load
- Target system preparation
- Data insertion/update
- Referential integrity checks

### Validate
- Completeness checks
- Quality verification
- Reconciliation

## Error Handling
- Extract errors → Retry logic
- Transform errors → Exception handling
- Load errors → Rollback capability
"#.to_string();

    template
}

fn create_parallel_orchestration_template() -> Template {
    let metadata = TemplateMetadata {
        id: "orchestration_parallel".to_string(),
        name: "Parallel Orchestration Pattern".to_string(),
        description: "Execute multiple tasks in parallel with synchronization".to_string(),
        category: TemplateCategory::Orchestration,
        version: "1.0.0".to_string(),
        author: Some("ABCDODAF System".to_string()),
        created_at: chrono::Utc::now(),
        modified_at: chrono::Utc::now(),
        tags: vec![
            "orchestration".to_string(),
            "parallel".to_string(),
            "async".to_string(),
        ],
        complexity: "intermediate".to_string(),
        use_cases: vec![
            "Parallel task execution".to_string(),
            "Concurrent microservices".to_string(),
        ],
        related_templates: vec![],
    };

    let template = Template::new(
        metadata,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL">
  <bpmn:process id="ParallelOrch" name="Parallel Orchestration">
    <bpmn:startEvent id="Start"/>
    <bpmn:parallelGateway id="Fork"/>
    <bpmn:serviceTask id="Task1" name="Task 1"/>
    <bpmn:serviceTask id="Task2" name="Task 2"/>
    <bpmn:serviceTask id="Task3" name="Task 3"/>
    <bpmn:parallelGateway id="Join"/>
    <bpmn:endEvent id="End"/>
  </bpmn:process>
</bpmn:definitions>"#.to_string()
    );

    template
}

fn create_human_in_loop_template() -> Template {
    let metadata = TemplateMetadata {
        id: "hitl_basic".to_string(),
        name: "Human-in-the-Loop Workflow".to_string(),
        description: "Workflow combining automated and human decision points".to_string(),
        category: TemplateCategory::HumanInTheLoop,
        version: "1.0.0".to_string(),
        author: Some("ABCDODAF System".to_string()),
        created_at: chrono::Utc::now(),
        modified_at: chrono::Utc::now(),
        tags: vec![
            "human-in-loop".to_string(),
            "decision".to_string(),
            "hybrid".to_string(),
        ],
        complexity: "intermediate".to_string(),
        use_cases: vec![
            "Exception handling".to_string(),
            "Quality assurance".to_string(),
        ],
        related_templates: vec![],
    };

    let mut template = Template::new(
        metadata,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL">
  <bpmn:process id="HumanInLoop" name="Human-in-the-Loop">
    <bpmn:startEvent id="Start"/>
    <bpmn:serviceTask id="AutoProcess" name="Automated Processing"/>
    <bpmn:exclusiveGateway id="NeedHuman"/>
    <bpmn:userTask id="HumanReview" name="Human Review"/>
    <bpmn:userTask id="HumanDecision" name="Make Decision"/>
    <bpmn:serviceTask id="ApplyDecision" name="Apply Decision"/>
    <bpmn:endEvent id="End"/>
  </bpmn:process>
</bpmn:definitions>"#.to_string()
    );

    template.documentation = r#"# Human-in-the-Loop Pattern

## Overview
Combines automated processing with human review and decision-making.

## Phases
1. Automated initial processing
2. Decision point: Does this need human review?
3. If yes → Human review and decision
4. Apply human decision
5. Complete

## Use Cases
- Complex decision scenarios
- Exception handling
- Quality assurance gates
- Learning and continuous improvement
"#.to_string();

    template
}

fn create_error_handling_template() -> Template {
    let metadata = TemplateMetadata {
        id: "errorhandling_retry".to_string(),
        name: "Error Handling with Retry".to_string(),
        description: "Resilient error handling with retry and escalation".to_string(),
        category: TemplateCategory::ErrorHandling,
        version: "1.0.0".to_string(),
        author: Some("ABCDODAF System".to_string()),
        created_at: chrono::Utc::now(),
        modified_at: chrono::Utc::now(),
        tags: vec![
            "error".to_string(),
            "retry".to_string(),
            "resilience".to_string(),
        ],
        complexity: "intermediate".to_string(),
        use_cases: vec![
            "Transient error handling".to_string(),
            "Service reliability".to_string(),
        ],
        related_templates: vec![],
    };

    let template = Template::new(
        metadata,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL">
  <bpmn:process id="ErrorHandling" name="Error Handling">
    <bpmn:startEvent id="Start"/>
    <bpmn:serviceTask id="Task" name="Main Task"/>
    <bpmn:boundaryEvent id="ErrorBoundary" attachedToRef="Task"/>
    <bpmn:serviceTask id="Retry" name="Retry Task"/>
    <bpmn:exclusiveGateway id="RetryDecision"/>
    <bpmn:userTask id="Escalate" name="Manual Escalation"/>
    <bpmn:endEvent id="End"/>
  </bpmn:process>
</bpmn:definitions>"#.to_string()
    );

    template
}

fn create_notification_template() -> Template {
    let metadata = TemplateMetadata {
        id: "notification_alert".to_string(),
        name: "Notification and Alert Pattern".to_string(),
        description: "Workflow for triggering notifications and alerts".to_string(),
        category: TemplateCategory::Notification,
        version: "1.0.0".to_string(),
        author: Some("ABCDODAF System".to_string()),
        created_at: chrono::Utc::now(),
        modified_at: chrono::Utc::now(),
        tags: vec![
            "notification".to_string(),
            "alert".to_string(),
            "communication".to_string(),
        ],
        complexity: "beginner".to_string(),
        use_cases: vec![
            "Event notifications".to_string(),
            "Alert generation".to_string(),
        ],
        related_templates: vec![],
    };

    let template = Template::new(
        metadata,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL">
  <bpmn:process id="Notification" name="Notification">
    <bpmn:startEvent id="TriggerEvent"/>
    <bpmn:serviceTask id="BuildMessage" name="Build Message"/>
    <bpmn:parallelGateway id="SendMultiple"/>
    <bpmn:serviceTask id="EmailNotify" name="Send Email"/>
    <bpmn:serviceTask id="SlackNotify" name="Send Slack"/>
    <bpmn:parallelGateway id="Sync"/>
    <bpmn:endEvent id="End"/>
  </bpmn:process>
</bpmn:definitions>"#.to_string()
    );

    template
}

fn create_decision_logic_template() -> Template {
    let metadata = TemplateMetadata {
        id: "decision_dmn".to_string(),
        name: "Decision Logic Pattern".to_string(),
        description: "Complex decision logic using gateways and rules".to_string(),
        category: TemplateCategory::DecisionLogic,
        version: "1.0.0".to_string(),
        author: Some("ABCDODAF System".to_string()),
        created_at: chrono::Utc::now(),
        modified_at: chrono::Utc::now(),
        tags: vec![
            "decision".to_string(),
            "logic".to_string(),
            "rules".to_string(),
        ],
        complexity: "intermediate".to_string(),
        use_cases: vec![
            "Business rule evaluation".to_string(),
            "Complex routing".to_string(),
        ],
        related_templates: vec![],
    };

    let template = Template::new(
        metadata,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL">
  <bpmn:process id="DecisionLogic" name="Decision Logic">
    <bpmn:startEvent id="Start"/>
    <bpmn:serviceTask id="EvaluateConditions" name="Evaluate Conditions"/>
    <bpmn:exclusiveGateway id="Decision"/>
    <bpmn:serviceTask id="ProcessA" name="Process Path A"/>
    <bpmn:serviceTask id="ProcessB" name="Process Path B"/>
    <bpmn:serviceTask id="ProcessC" name="Process Path C"/>
    <bpmn:exclusiveGateway id="Merge"/>
    <bpmn:endEvent id="End"/>
  </bpmn:process>
</bpmn:definitions>"#.to_string()
    );

    template
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_builtin_library() {
        let library = create_builtin_library();
        assert!(library.templates.len() > 0);
        assert_eq!(library.name, "ABCDODAF Built-in Templates");
    }

    #[test]
    fn test_simple_approval_template() {
        let template = create_simple_approval_template();
        assert_eq!(template.metadata.name, "Simple Approval Workflow");
        assert_eq!(template.metadata.complexity, "beginner");
    }

    #[test]
    fn test_etl_template() {
        let template = create_etl_template();
        assert_eq!(template.metadata.category, TemplateCategory::DataTransformation);
        assert!(!template.documentation.is_empty());
    }
}
