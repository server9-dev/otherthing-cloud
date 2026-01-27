//! Comprehensive DoDAF 2.02 Architecture Example
//!
//! Demonstrates all implemented DoDAF views: OV-1, OV-2, OV-3, OV-6, SV-1, SV-2, SV-4, CV-1, CV-2

use abcdodaf::dodaf::*;
use chrono::Utc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Comprehensive DoDAF 2.02 Architecture Example ===\n");

    // 1. Create OV-1: High-Level Operational Concept
    println!("1. Building OV-1: Operational Concept Graphic...");
    let ov1 = create_ov1();
    println!("   - Name: {}", ov1.name);
    println!("   - Organizations: {}", ov1.organizations.len());
    println!("   - Scenarios: {}\n", ov1.operational_scenarios.len());

    // 2. Create OV-2: Operational Node Connectivity
    println!("2. Building OV-2: Node Connectivity...");
    let ov2 = create_ov2();
    println!("   - Nodes: {}", ov2.node_count);
    println!("   - Needlines: {}", ov2.needlines.len());
    println!("   - Material Needlines: {}\n", ov2.materialneedlines.len());

    // 3. Create OV-3: Information Exchange Matrix
    println!("3. Building OV-3: Exchange Matrix...");
    let ov3 = create_ov3();
    println!("   - Information Elements: {}", ov3.exchanges.len());
    println!("   - Exchange Pairs: {}", ov3.exchange_pairs.len());
    println!("   - Sources: {}, Targets: {}\n", ov3.row_headers.len(), ov3.column_headers.len());

    // 4. Create OV-6: Rules, States, and Events
    println!("4. Building OV-6 Components...");
    let ov6a = create_ov6a();
    let ov6b = create_ov6b();
    println!("   OV-6a - Rules: {}", ov6a.rules.len());
    println!("   OV-6b - States: {}, Transitions: {}\n", ov6b.states.len(), ov6b.transitions.len());

    // 5. Create SV-1: Systems Interface
    println!("5. Building SV-1: Systems Interface...");
    let sv1 = create_sv1();
    println!("   - Systems: {}", sv1.systems.len());
    println!("   - Interfaces: {}", sv1.interfaces.len());
    println!("   - Ports: {}\n", sv1.ports.len());

    // 6. Create SV-2: Systems Resource Flow
    println!("6. Building SV-2: Resource Flow...");
    let sv2 = create_sv2();
    println!("   - Communication Systems: {}", sv2.communications_systems.len());
    println!("   - Communication Links: {}", sv2.communications_links.len());
    println!("   - Networks: {}\n", sv2.communications_networks.len());

    // 7. Create SV-4: Systems Functionality
    println!("7. Building SV-4: Functionality...");
    let sv4 = create_sv4();
    println!("   - Functions: {}", sv4.functions.len());
    println!("   - Data Flows: {}", sv4.data_flows.len());
    println!("   - Hierarchies: {}\n", sv4.functional_hierarchies.len());

    // 8. Create CV-1: Capability Vision
    println!("8. Building CV-1: Capability Vision...");
    let cv1 = create_cv1();
    println!("   - Vision: {}", cv1.name);
    println!("   - Strategic Objectives: {}", cv1.strategic_objectives.len());
    println!("   - Capability Increments: {}\n", cv1.capability_increments.len());

    // 9. Create CV-2: Capability Taxonomy
    println!("9. Building CV-2: Capability Taxonomy...");
    let cv2 = create_cv2();
    println!("   - Total Capabilities: {}", cv2.capabilities.len());
    println!("   - Root Capabilities: {}", cv2.hierarchy.root_nodes.len());
    println!("   - Hierarchy Levels: {}\n", cv2.hierarchy.levels);

    // 10. Export examples
    println!("10. Exporting examples...");
    if let Ok(json) = serde_json::to_string_pretty(&ov1) {
        println!("    OV-1 JSON export: {} bytes", json.len());
    }
    if let Ok(json) = serde_json::to_string_pretty(&cv2) {
        println!("    CV-2 JSON export: {} bytes\n", json.len());
    }

    println!("=== Architecture Summary ===");
    println!("Successfully created a comprehensive DoDAF 2.02 architecture with:");
    println!("- Operational Views (OV-1, OV-2, OV-3, OV-6a/b)");
    println!("- Systems Views (SV-1, SV-2, SV-4)");
    println!("- Capability Views (CV-1, CV-2)");

    Ok(())
}

/// Create a sample OV-1: High-Level Operational Concept
fn create_ov1() -> OperationalConceptGraphic {
    use ov1::{OperationalOrganization, OperationalScenario, OrganizationType};

    let scenario = OperationalScenario {
        id: "scenario_primary".to_string(),
        name: "Primary Operations".to_string(),
        description: "Primary operational scenario".to_string(),
        initiating_event: Some("Alert received".to_string()),
        key_activities: vec!["Assess".to_string(), "Plan".to_string(), "Execute".to_string()],
        duration: None,
    };

    let cmd_org =
        OperationalOrganization::new("org_cmd", "Command Center", OrganizationType::Command)
            .add_responsibility("Overall mission command");

    let ops_org =
        OperationalOrganization::new("org_ops", "Operations Center", OrganizationType::Operations)
            .add_responsibility("Execute tactical operations");

    OperationalConceptGraphic::new(
        "ov1_demo",
        "Military Operations Architecture",
        "Execute joint operations",
    )
    .with_description("Demonstrates OV-1 operational concept")
    .with_vision("Integrated command and control across all domains")
    .add_scenario(scenario)
    .add_organization(cmd_org)
    .add_organization(ops_org)
}

/// Create a sample OV-2: Operational Node Connectivity
fn create_ov2() -> OperationalNodeConnectivity {
    use ov2::{Criticality, Needline, OperationalNode, OperationalNodeType};

    let node1 =
        OperationalNode::new("node_cmd", "Command Center", OperationalNodeType::Organization)
            .with_description("Central command authority")
            .add_activity("activity_cmd")
            .add_performer("performer_commander");

    let node2 = OperationalNode::new("node_field", "Field Unit", OperationalNodeType::Organization)
        .with_description("Forward operational unit")
        .add_activity("activity_field")
        .add_performer("performer_unit_leader");

    let needline = Needline::new(
        "nl_cmd_field",
        "Command Orders",
        "node_cmd",
        "node_field",
        Criticality::Critical,
    )
    .add_information_element("tactical_order");

    OperationalNodeConnectivity::new("ov2_demo", "Operational Node Connectivity")
        .with_description("Shows nodes and information flows")
        .add_node(node1)
        .add_node(node2)
        .add_needline(needline)
}

/// Create a sample OV-3: Information Exchange Matrix
fn create_ov3() -> InformationExchangeMatrix {
    use ov3::{
        ExchangeAttributes, ExchangeMedia, ExchangePair, InformationElement, InformationType,
        InteroperabilityLevel,
    };

    let ie1 = InformationElement::new("ie_order", "Tactical Order", InformationType::Command)
        .with_creator("Command Center");

    let ie2 = InformationElement::new("ie_status", "Status Report", InformationType::Report)
        .with_consumer("Command Center");

    let attrs = ExchangeAttributes::with_quality(95.0, 100.0, 99.0)
        .with_media(ExchangeMedia::Electronic)
        .with_interoperability(InteroperabilityLevel::Level4);

    let pair = ExchangePair::new("ep_1", "node_cmd", "node_field")
        .add_information_element("ie_order")
        .with_attributes(attrs)
        .with_criticality(ov3::Criticality::Critical);

    InformationExchangeMatrix::new("ov3_demo", "Information Exchange Matrix")
        .with_description("Exchange requirements between nodes")
        .add_information_element(ie1)
        .add_information_element(ie2)
        .add_exchange_pair(pair)
}

/// Create a sample OV-6a: Operational Rules
fn create_ov6a() -> OperationalRulesModel {
    use ov6::{Constraint, ConstraintType, OperationalRule, RuleApplicability, RuleType};

    let rule = OperationalRule::new(
        "rule_auth",
        "Authorization Rule",
        "All orders must come from authorized command authority",
        RuleType::Policy,
        "order.source == AUTHORIZED_COMMAND",
        "execute_order(order)",
    )
    .with_applicability(RuleApplicability::Always)
    .with_priority(1)
    .add_applicable_activity("activity_execute");

    let constraint = Constraint {
        id: "const_timing".to_string(),
        name: "Timing Constraint".to_string(),
        constraint_type: ConstraintType::Temporal,
        expression: "execution_start < deadline".to_string(),
        applies_to: vec!["activity_execute".to_string()],
    };

    OperationalRulesModel::new("ov6a_demo", "Operational Rules")
        .with_description("Business rules and constraints")
        .add_rule(rule)
        .add_constraint(constraint)
}

/// Create a sample OV-6b: State Transitions
fn create_ov6b() -> StateTransitionDescription {
    use ov6::{EventType, OperationalState, StateTransition, StateType, TransitionEvent};

    let idle = OperationalState::new("state_idle", "Idle", StateType::Idle).as_initial();
    let active = OperationalState::new("state_active", "Active", StateType::Active);
    let done = OperationalState::new("state_done", "Done", StateType::Terminal).as_terminal();

    let event = TransitionEvent {
        id: "event_start".to_string(),
        name: "Start Event".to_string(),
        description: None,
        event_type: EventType::Signal,
        triggers_transitions: vec!["trans_start".to_string()],
    };

    let transition = StateTransition {
        id: "trans_start".to_string(),
        name: Some("Transition to Active".to_string()),
        source_state: "state_idle".to_string(),
        target_state: "state_active".to_string(),
        trigger_event: "event_start".to_string(),
        guard_condition: None,
        probability: Some(1.0),
        action: Some("activate_operations()".to_string()),
        action_sequence: Vec::new(),
    };

    StateTransitionDescription::new("ov6b_demo", "State Transitions", "state_idle")
        .with_description("Operational state machine")
        .add_state(idle)
        .add_state(active)
        .add_state(done)
        .add_event(event)
        .add_transition(transition)
        .add_terminal_state("state_done")
}

/// Create a sample SV-1: Systems Interface
fn create_sv1() -> SystemsInterfaceDescription {
    use sv1::{
        InterfaceType, PortDirection, PortType, System, SystemInterface, SystemPort, SystemType,
    };

    let sys1 =
        System::new("sys_cmd", "Command System", SystemType::Software).add_function("func_cmd");

    let sys2 =
        System::new("sys_sensor", "Sensor System", SystemType::Hardware).add_function("func_sense");

    let port1 = SystemPort::new(
        "port_out",
        "Command Output",
        PortType::Output,
        "sys_cmd",
        PortDirection::Out,
    )
    .with_protocol("TCP/IP");

    let port2 = SystemPort::new(
        "port_in",
        "Sensor Input",
        PortType::Input,
        "sys_sensor",
        PortDirection::In,
    )
    .with_protocol("TCP/IP");

    let iface = SystemInterface::new("iface_1", "sys_cmd", "sys_sensor", "port_out", "port_in")
        .with_name("Command to Sensor")
        .with_type(InterfaceType::Data);

    SystemsInterfaceDescription::new("sv1_demo", "Systems Interface")
        .with_description("System architecture and interfaces")
        .add_system(sys1)
        .add_system(sys2)
        .add_port(port1)
        .add_port(port2)
        .add_interface(iface)
}

/// Create a sample SV-2: Systems Resource Flow
fn create_sv2() -> SystemsResourceFlowDescription {
    use sv2::{
        CommunicationLink, CommunicationNetwork, CommunicationSystem, CommunicationType, MediaType,
        NetworkType,
    };

    let comm_sys = CommunicationSystem::new(
        "comm_tcp",
        "TCP/IP Network",
        CommunicationType::Data,
        MediaType::Fiber,
        "TCP/IP",
    );

    let link = CommunicationLink::new("link_1", "sys_cmd", "sys_sensor", "comm_tcp")
        .with_bandwidth("100 Mbps")
        .with_latency("10ms")
        .with_reliability(0.99);

    let network = CommunicationNetwork::new("net_core", "Core Network", NetworkType::Intranet)
        .add_member_system("sys_cmd")
        .add_member_system("sys_sensor")
        .add_link("link_1");

    SystemsResourceFlowDescription::new("sv2_demo", "Systems Resource Flow")
        .with_description("Communication systems and links")
        .add_system("sys_cmd")
        .add_system("sys_sensor")
        .add_communication_system(comm_sys)
        .add_link(link)
        .add_network(network)
}

/// Create a sample SV-4: Systems Functionality
fn create_sv4() -> SystemsFunctionalityDescription {
    use sv4::{
        FunctionDataFlow, FunctionType, FunctionalHierarchy, HierarchyLevel, SystemFunction,
    };

    let func_cmd =
        SystemFunction::new("func_cmd", "Command Generation", FunctionType::Primary, "sys_cmd")
            .add_output("command_data");

    let func_execute =
        SystemFunction::new("func_execute", "Execute Command", FunctionType::Primary, "sys_sensor")
            .add_input("command_data")
            .add_output("execution_status");

    let data_flow = FunctionDataFlow::new("flow_1", "func_cmd", "func_execute")
        .add_data_element("command_data")
        .with_timing("Real-time");

    let mut level0 = HierarchyLevel::new(0);
    level0.functions.push("func_cmd".to_string());

    let hierarchy = FunctionalHierarchy::new("func_cmd").add_level(level0);

    SystemsFunctionalityDescription::new("sv4_demo", "Systems Functionality")
        .with_description("System functions and data flows")
        .add_function(func_cmd)
        .add_function(func_execute)
        .add_data_flow(data_flow)
        .add_hierarchy(hierarchy)
}

/// Create a sample CV-1: Capability Vision
fn create_cv1() -> CapabilityVision {
    use cv1::{CapabilityIncrement, StrategicObjective, TimeFrame};

    let obj = StrategicObjective::new(
        "obj_1",
        "Integrated Operations",
        "Achieve fully integrated operational capability",
    )
    .with_priority(1)
    .add_measure("Integration success rate");

    let now = Utc::now();
    let increment = CapabilityIncrement::new("inc_phase1", "Phase 1", now)
        .add_enabled_capability("cap_command")
        .add_enabled_capability("cap_sense");

    let tf = TimeFrame::with_all("0-2 years", "2-5 years", "5+ years");

    CapabilityVision::new(
        "cv1_demo",
        "2026-2030 Strategic Vision",
        "Strategic capability vision",
        "Achieve superior operational capability",
    )
    .add_mission_area("Command & Control")
    .add_objective(obj)
    .add_increment(increment)
    .with_timeframe(tf)
}

/// Create a sample CV-2: Capability Taxonomy
fn create_cv2() -> CapabilityTaxonomy {
    use cv2::{Capability, CapabilityMeasure};

    let cap_root = Capability::new("cap_command", "Command Capability", "C2", "Core")
        .with_maturity_level(4)
        .with_readiness_level(85);

    let cap_plan = Capability::new("cap_plan", "Planning", "C2", "Planning")
        .with_parent("cap_command", 1)
        .add_realizing_operation("ov_plan")
        .add_realizing_system("sys_cmd");

    let cap_execute = Capability::new("cap_execute", "Execute", "C2", "Execution")
        .with_parent("cap_command", 1)
        .add_realizing_operation("ov_execute")
        .add_realizing_system("sys_sensor");

    let measure = CapabilityMeasure::new("m_response", "Response Time", "seconds")
        .with_condition("Normal operations")
        .with_target("60")
        .with_current("45");

    let cap_with_measure = cap_plan.add_measure(measure);

    CapabilityTaxonomy::new("cv2_demo", "C2 Capability Taxonomy")
        .with_description("Hierarchical organization of command and control capabilities")
        .add_capability(cap_root)
        .add_capability(cap_with_measure)
        .add_capability(cap_execute)
}
