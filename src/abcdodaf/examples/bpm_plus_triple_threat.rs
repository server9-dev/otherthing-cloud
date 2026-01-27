//! BPM+ Triple Threat Example
//!
//! This example demonstrates the integration of all three OMG standards:
//! - BPMN 2.0 (Business Process Model and Notation)
//! - CMMN 1.1 (Case Management Model and Notation)
//! - DMN 1.3 (Decision Model and Notation)
//!
//! Scenario: AI-powered customer support system
//! - BPMN: Structured ticket triage process
//! - DMN: Decision logic for ticket routing and priority
//! - CMMN: Adaptive case management for complex customer issues

use abcdodaf::bpm_plus::*;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== BPM+ Triple Threat Example ===\n");
    println!("This example demonstrates how BPMN, CMMN, and DMN work together");
    println!("to model an AI-powered customer support system.\n");

    // ====================================================================
    // 1. Create DMN Decisions
    // ====================================================================
    println!("📊 Step 1: Creating DMN Decision Models\n");

    // Decision 1: Ticket Priority
    let priority_table = dmn::DecisionTable {
        hit_policy: dmn::HitPolicy::First,
        inputs: vec![
            dmn::InputClause {
                id: "i1".to_string(),
                label: "urgency".to_string(),
                input_expression: "ticket.urgency".to_string(),
                input_values: Some(vec![
                    "high".to_string(),
                    "medium".to_string(),
                    "low".to_string(),
                ]),
            },
            dmn::InputClause {
                id: "i2".to_string(),
                label: "impact".to_string(),
                input_expression: "ticket.impact".to_string(),
                input_values: Some(vec![
                    "high".to_string(),
                    "medium".to_string(),
                    "low".to_string(),
                ]),
            },
        ],
        outputs: vec![dmn::OutputClause {
            id: "o1".to_string(),
            label: "Priority".to_string(),
            name: "priority".to_string(),
            output_values: Some(vec![
                "critical".to_string(),
                "high".to_string(),
                "medium".to_string(),
                "low".to_string(),
            ]),
        }],
        rules: vec![
            dmn::DecisionRule {
                id: "r1".to_string(),
                input_entries: vec!["high".to_string(), "high".to_string()],
                output_entries: vec!["critical".to_string()],
                annotation: Some("Both urgency and impact high".to_string()),
            },
            dmn::DecisionRule {
                id: "r2".to_string(),
                input_entries: vec!["high".to_string(), "-".to_string()],
                output_entries: vec!["high".to_string()],
                annotation: Some("High urgency".to_string()),
            },
            dmn::DecisionRule {
                id: "r3".to_string(),
                input_entries: vec!["-".to_string(), "high".to_string()],
                output_entries: vec!["high".to_string()],
                annotation: Some("High impact".to_string()),
            },
            dmn::DecisionRule {
                id: "r4".to_string(),
                input_entries: vec!["medium".to_string(), "medium".to_string()],
                output_entries: vec!["medium".to_string()],
                annotation: None,
            },
            dmn::DecisionRule {
                id: "r5".to_string(),
                input_entries: vec!["-".to_string(), "-".to_string()],
                output_entries: vec!["low".to_string()],
                annotation: Some("Default case".to_string()),
            },
        ],
    };

    let mut priority_decision = dmn::DmnDecision::with_decision_table(
        "ticket_priority",
        "Ticket Priority Decision",
        priority_table,
    );
    priority_decision.description =
        Some("Determines ticket priority based on urgency and impact".to_string());

    println!("✅ Created DMN Decision: Ticket Priority");
    println!("   - Inputs: urgency, impact");
    println!("   - Output: priority (critical/high/medium/low)");
    println!("   - Rules: {} decision rules\n", priority_decision.decision_logic.get_rule_count());

    // Decision 2: Agent Assignment
    let assignment_table = dmn::DecisionTable {
        hit_policy: dmn::HitPolicy::First,
        inputs: vec![
            dmn::InputClause {
                id: "i1".to_string(),
                label: "category".to_string(),
                input_expression: "ticket.category".to_string(),
                input_values: Some(vec![
                    "technical".to_string(),
                    "billing".to_string(),
                    "general".to_string(),
                ]),
            },
            dmn::InputClause {
                id: "i2".to_string(),
                label: "complexity".to_string(),
                input_expression: "ticket.complexity".to_string(),
                input_values: Some(vec!["simple".to_string(), "complex".to_string()]),
            },
        ],
        outputs: vec![dmn::OutputClause {
            id: "o1".to_string(),
            label: "Agent Type".to_string(),
            name: "agent_type".to_string(),
            output_values: Some(vec![
                "ai_agent".to_string(),
                "human_expert".to_string(),
                "hybrid".to_string(),
            ]),
        }],
        rules: vec![
            dmn::DecisionRule {
                id: "r1".to_string(),
                input_entries: vec!["technical".to_string(), "complex".to_string()],
                output_entries: vec!["human_expert".to_string()],
                annotation: Some("Complex technical issues need human experts".to_string()),
            },
            dmn::DecisionRule {
                id: "r2".to_string(),
                input_entries: vec!["technical".to_string(), "simple".to_string()],
                output_entries: vec!["ai_agent".to_string()],
                annotation: Some("Simple technical issues can be handled by AI".to_string()),
            },
            dmn::DecisionRule {
                id: "r3".to_string(),
                input_entries: vec!["billing".to_string(), "-".to_string()],
                output_entries: vec!["hybrid".to_string()],
                annotation: Some("Billing requires both AI and human oversight".to_string()),
            },
            dmn::DecisionRule {
                id: "r4".to_string(),
                input_entries: vec!["-".to_string(), "-".to_string()],
                output_entries: vec!["ai_agent".to_string()],
                annotation: Some("Default to AI agent".to_string()),
            },
        ],
    };

    let assignment_decision = dmn::DmnDecision::with_decision_table(
        "agent_assignment",
        "Agent Assignment Decision",
        assignment_table,
    );

    println!("✅ Created DMN Decision: Agent Assignment");
    println!("   - Inputs: category, complexity");
    println!("   - Output: agent_type (ai_agent/human_expert/hybrid)\n");

    // ====================================================================
    // 2. Create BPMN Process (Structured Ticket Triage)
    // ====================================================================
    println!("🔄 Step 2: Creating BPMN Process Model\n");

    let mut triage_process =
        bpmn::BpmnProcess::new("ticket_triage", "Customer Ticket Triage Process");
    triage_process.description =
        Some("Automated process for triaging customer support tickets".to_string());

    // Start event
    triage_process.add_flow_element(bpmn::FlowElement::StartEvent {
        id: "start".to_string(),
        name: "Ticket Received".to_string(),
        outgoing: vec!["flow1".to_string()],
    });

    // Service task: AI-powered ticket classification
    triage_process.add_flow_element(bpmn::FlowElement::ServiceTask {
        id: "classify".to_string(),
        name: "Classify Ticket (AI)".to_string(),
        implementation: bpmn::ServiceImplementation::Agent {
            agent_type: "nlp_classifier".to_string(),
        },
        incoming: vec!["flow1".to_string()],
        outgoing: vec!["flow2".to_string()],
    });

    // Business rule task: Determine priority (calls DMN)
    triage_process.add_flow_element(bpmn::FlowElement::BusinessRuleTask {
        id: "priority".to_string(),
        name: "Determine Priority (DMN)".to_string(),
        decision_ref: Some("ticket_priority".to_string()),
        incoming: vec!["flow2".to_string()],
        outgoing: vec!["flow3".to_string()],
    });

    // Business rule task: Assign agent (calls DMN)
    triage_process.add_flow_element(bpmn::FlowElement::BusinessRuleTask {
        id: "assign".to_string(),
        name: "Assign Agent (DMN)".to_string(),
        decision_ref: Some("agent_assignment".to_string()),
        incoming: vec!["flow3".to_string()],
        outgoing: vec!["flow4".to_string()],
    });

    // Exclusive gateway: Route based on priority
    triage_process.add_flow_element(bpmn::FlowElement::ExclusiveGateway {
        id: "gateway1".to_string(),
        name: "Check Priority".to_string(),
        incoming: vec!["flow4".to_string()],
        outgoing: vec!["flow5_critical".to_string(), "flow5_normal".to_string()],
        default_flow: Some("flow5_normal".to_string()),
    });

    // Call activity: Create case for critical tickets (calls CMMN)
    triage_process.add_flow_element(bpmn::FlowElement::CallActivity {
        id: "create_case".to_string(),
        name: "Create Complex Case (CMMN)".to_string(),
        called_element: "complex_inquiry_case".to_string(),
        incoming: vec!["flow5_critical".to_string()],
        outgoing: vec!["flow6".to_string()],
    });

    // Service task: Route to queue for normal tickets
    triage_process.add_flow_element(bpmn::FlowElement::ServiceTask {
        id: "route_queue".to_string(),
        name: "Route to Queue".to_string(),
        implementation: bpmn::ServiceImplementation::Expression {
            expression: "queue.add(ticket)".to_string(),
        },
        incoming: vec!["flow5_normal".to_string()],
        outgoing: vec!["flow7".to_string()],
    });

    // End events
    triage_process.add_flow_element(bpmn::FlowElement::EndEvent {
        id: "end_case".to_string(),
        name: "Case Created".to_string(),
        incoming: vec!["flow6".to_string()],
    });

    triage_process.add_flow_element(bpmn::FlowElement::EndEvent {
        id: "end_queued".to_string(),
        name: "Ticket Queued".to_string(),
        incoming: vec!["flow7".to_string()],
    });

    // Sequence flows
    triage_process.add_flow_element(bpmn::FlowElement::SequenceFlow {
        id: "flow1".to_string(),
        name: None,
        source_ref: "start".to_string(),
        target_ref: "classify".to_string(),
        condition: None,
    });

    triage_process.add_flow_element(bpmn::FlowElement::SequenceFlow {
        id: "flow2".to_string(),
        name: None,
        source_ref: "classify".to_string(),
        target_ref: "priority".to_string(),
        condition: None,
    });

    triage_process.add_flow_element(bpmn::FlowElement::SequenceFlow {
        id: "flow3".to_string(),
        name: None,
        source_ref: "priority".to_string(),
        target_ref: "assign".to_string(),
        condition: None,
    });

    triage_process.add_flow_element(bpmn::FlowElement::SequenceFlow {
        id: "flow4".to_string(),
        name: None,
        source_ref: "assign".to_string(),
        target_ref: "gateway1".to_string(),
        condition: None,
    });

    triage_process.add_flow_element(bpmn::FlowElement::SequenceFlow {
        id: "flow5_critical".to_string(),
        name: Some("Critical".to_string()),
        source_ref: "gateway1".to_string(),
        target_ref: "create_case".to_string(),
        condition: Some("priority == 'critical'".to_string()),
    });

    triage_process.add_flow_element(bpmn::FlowElement::SequenceFlow {
        id: "flow5_normal".to_string(),
        name: Some("Normal".to_string()),
        source_ref: "gateway1".to_string(),
        target_ref: "route_queue".to_string(),
        condition: None,
    });

    triage_process.add_flow_element(bpmn::FlowElement::SequenceFlow {
        id: "flow6".to_string(),
        name: None,
        source_ref: "create_case".to_string(),
        target_ref: "end_case".to_string(),
        condition: None,
    });

    triage_process.add_flow_element(bpmn::FlowElement::SequenceFlow {
        id: "flow7".to_string(),
        name: None,
        source_ref: "route_queue".to_string(),
        target_ref: "end_queued".to_string(),
        condition: None,
    });

    println!("✅ Created BPMN Process: Ticket Triage");
    println!("   - Flow elements: {}", triage_process.flow_elements.len());
    println!("   - Integrates with 2 DMN decisions");
    println!("   - Calls CMMN case for critical tickets\n");

    // ====================================================================
    // 3. Create CMMN Case (Adaptive Complex Inquiry Handling)
    // ====================================================================
    println!("📋 Step 3: Creating CMMN Case Model\n");

    let mut complex_case =
        cmmn::CmmnCase::new("complex_inquiry_case", "Complex Customer Inquiry Case");
    complex_case.description = Some(
        "Adaptive case for handling complex customer issues requiring multiple steps and decisions"
            .to_string(),
    );

    // Add case file items (data)
    complex_case.add_case_file_item(cmmn::CaseFileItem {
        id: "customer_info".to_string(),
        name: "Customer Information".to_string(),
        definition_ref: None,
        multiplicity: cmmn::Multiplicity::ExactlyOne,
    });

    complex_case.add_case_file_item(cmmn::CaseFileItem {
        id: "investigation_notes".to_string(),
        name: "Investigation Notes".to_string(),
        definition_ref: None,
        multiplicity: cmmn::Multiplicity::ZeroOrMore,
    });

    // Sentry: Initial investigation complete
    complex_case.add_sentry(cmmn::Sentry {
        id: "sentry_investigation_complete".to_string(),
        name: "Investigation Complete".to_string(),
        on_parts: vec![cmmn::OnPart {
            source_ref: "investigate".to_string(),
            standard_event: cmmn::StandardEvent::Complete,
        }],
        if_part: None,
    });

    // Human task: Initial investigation
    complex_case.add_plan_item(cmmn::PlanItem::HumanTask {
        id: "investigate".to_string(),
        name: "Investigate Issue".to_string(),
        performer: Some("senior_agent".to_string()),
        documentation: Some("Deep dive into customer issue".to_string()),
        entry_criteria: vec![],
        exit_criteria: vec![],
        required: true,
        repeatable: false,
    });

    // Decision task: Determine resolution approach
    complex_case.add_plan_item(cmmn::PlanItem::DecisionTask {
        id: "resolution_decision".to_string(),
        name: "Decide Resolution Approach (DMN)".to_string(),
        decision_ref: "resolution_approach".to_string(),
        entry_criteria: vec!["sentry_investigation_complete".to_string()],
        exit_criteria: vec![],
    });

    // Stage: Resolution execution (can contain multiple tasks)
    complex_case.add_plan_item(cmmn::PlanItem::Stage {
        id: "execute_resolution".to_string(),
        name: "Execute Resolution".to_string(),
        plan_items: vec![
            cmmn::PlanItem::HumanTask {
                id: "implement_fix".to_string(),
                name: "Implement Solution".to_string(),
                performer: Some("specialist".to_string()),
                documentation: None,
                entry_criteria: vec![],
                exit_criteria: vec![],
                required: true,
                repeatable: false,
            },
            cmmn::PlanItem::HumanTask {
                id: "verify_fix".to_string(),
                name: "Verify Solution".to_string(),
                performer: Some("qa_agent".to_string()),
                documentation: None,
                entry_criteria: vec![],
                exit_criteria: vec![],
                required: true,
                repeatable: false,
            },
        ],
        entry_criteria: vec![],
        exit_criteria: vec![],
        auto_complete: true,
    });

    // Milestone: Case resolved
    complex_case.add_plan_item(cmmn::PlanItem::Milestone {
        id: "case_resolved".to_string(),
        name: "Issue Resolved".to_string(),
        entry_criteria: vec![],
    });

    println!("✅ Created CMMN Case: Complex Customer Inquiry");
    println!("   - Plan items: {}", complex_case.case_plan_model.plan_items.len());
    println!("   - Case file items: {}", complex_case.case_file.items.len());
    println!("   - Adaptive workflow with sentries and milestones");
    println!("   - Integrates with DMN for decision points\n");

    // ====================================================================
    // 4. Build Integrated BPM+ Model
    // ====================================================================
    println!("🔗 Step 4: Integrating All Standards\n");

    let model = BpmPlusBuilder::new(
        "AI Support System",
        "Complete AI-powered customer support system using BPM+ triple threat",
    )
    .add_decision(priority_decision)
    .add_decision(assignment_decision)
    .add_process(triage_process)
    .add_case(complex_case)
    .link_decision_to_process("ticket_priority", "ticket_triage", "priority")
    .link_decision_to_process("agent_assignment", "ticket_triage", "assign")
    .build()?;

    println!("✅ Successfully integrated BPM+ model!");
    println!("   - {} BPMN processes", model.processes.len());
    println!("   - {} CMMN cases", model.cases.len());
    println!("   - {} DMN decisions", model.decisions.len());
    println!("   - {} integrations", model.integrations.len());
    println!();

    // ====================================================================
    // 5. Validate the Model
    // ====================================================================
    println!("✅ Step 5: Validating Model\n");
    model.validate()?;
    println!("✅ Model validation passed!\n");

    // ====================================================================
    // 6. Test DMN Decision Execution
    // ====================================================================
    println!("🧪 Step 6: Testing DMN Decision Execution\n");

    let decision = &model.decisions[0]; // Priority decision

    let mut inputs = HashMap::new();
    inputs.insert("urgency".to_string(), serde_json::json!("high"));
    inputs.insert("impact".to_string(), serde_json::json!("high"));

    println!("Test case: urgency=high, impact=high");
    let result = decision.execute(&inputs)?;
    println!("Result: {}\n", serde_json::to_string_pretty(&result)?);

    // ====================================================================
    // 7. Export to XML
    // ====================================================================
    println!("📄 Step 7: Exporting to Standard Formats\n");

    println!("BPMN 2.0 XML Export:");
    println!("{}\n", "─".repeat(80));
    let bpmn_xml = model.processes[0].to_bpmn_xml();
    println!("{}", &bpmn_xml[0..bpmn_xml.len().min(500)]);
    if bpmn_xml.len() > 500 {
        println!("... (truncated)");
    }
    println!();

    println!("DMN 1.3 XML Export:");
    println!("{}\n", "─".repeat(80));
    let dmn_xml = model.decisions[0].to_dmn_xml();
    println!("{}", &dmn_xml[0..dmn_xml.len().min(500)]);
    if dmn_xml.len() > 500 {
        println!("... (truncated)");
    }
    println!();

    println!("CMMN 1.1 XML Export:");
    println!("{}\n", "─".repeat(80));
    let cmmn_xml = model.cases[0].to_cmmn_xml();
    println!("{}", &cmmn_xml[0..cmmn_xml.len().min(500)]);
    if cmmn_xml.len() > 500 {
        println!("... (truncated)");
    }
    println!();

    // ====================================================================
    // Summary
    // ====================================================================
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("🎉 BPM+ Triple Threat Example Complete!");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    println!("This example demonstrated:");
    println!("  ✓ BPMN 2.0 - Structured process flows with gateways and tasks");
    println!("  ✓ CMMN 1.1 - Adaptive case management with sentries and milestones");
    println!("  ✓ DMN 1.3  - Business decision tables with hit policies");
    println!("  ✓ Seamless integration between all three standards");
    println!("  ✓ Export to standard XML formats (BPMN, CMMN, DMN)");
    println!("  ✓ Runtime decision execution");
    println!();
    println!("The BPM+ triple threat provides comprehensive coverage for:");
    println!("  • Structured processes (BPMN)");
    println!("  • Adaptive knowledge work (CMMN)");
    println!("  • Reusable decision logic (DMN)");
    println!();

    Ok(())
}

// Helper trait to get rule count from DecisionLogic
trait RuleCounter {
    fn get_rule_count(&self) -> usize;
}

impl RuleCounter for dmn::DecisionLogic {
    fn get_rule_count(&self) -> usize {
        match self {
            dmn::DecisionLogic::DecisionTable { table } => table.rules.len(),
            _ => 0,
        }
    }
}
