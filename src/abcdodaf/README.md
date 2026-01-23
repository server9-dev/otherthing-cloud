# ABCDODAF

**ABCDODAF** is a Rust library that combines the **BPM+ Triple Threat** (BPMN, CMMN, DMN) standards with the **DoDAF 2.02** (Department of Defense Architecture Framework) to model both **agentic** (AI-driven) and **non-agentic** (human/system) operations within an AI assistant workforce.

## Features

### BPM+ Triple Threat (OMG Standards)
- 🔄 **BPMN 2.0**: Business Process Model and Notation - Structured process flows and orchestration
- 📋 **CMMN 1.1**: Case Management Model and Notation - Adaptive, knowledge-intensive case management
- ⚖️ **DMN 1.3**: Decision Model and Notation - Reusable business decision logic with decision tables

### DoDAF & AI Workforce Integration
- 🏛️ **DoDAF 2.02 Framework**: Comprehensive architectural modeling following DoD standards
- 🤖 **AI Agent Orchestration**: Model and execute AI agent tasks with 12 specialized capabilities
- 👥 **Human-AI Collaboration**: Seamlessly integrate human tasks in automated workflows
- ⚙️ **System Integration**: Support for automated system tasks with retry policies
- 🔌 **MCP Integration**: Native support for Model Context Protocol (RhizOS)

## Architecture

```
┌───────────────────────────────────────────────────────────────────┐
│                       ABCDODAF Library                            │
├───────────────────────────────────────────────────────────────────┤
│                                                                   │
│                  BPM+ Triple Threat (OMG Standards)               │
│    ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│    │   BPMN 2.0   │  │   CMMN 1.1   │  │   DMN 1.3    │         │
│    │   Process    │  │     Case     │  │   Decision   │         │
│    │    Flows     │  │  Management  │  │    Logic     │         │
│    └──────┬───────┘  └──────┬───────┘  └──────┬───────┘         │
│           └──────────────────┴──────────────────┘                 │
│                              │                                    │
│           ┌──────────────────┴──────────────────┐                │
│           │                                     │                │
│  ┌────────▼────────┐              ┌─────────────▼──────────┐    │
│  │   DoDAF 2.02    │              │   AI Workforce         │    │
│  │  Architecture   │◄─────────────┤   Orchestration        │    │
│  │   Framework     │              │  (Agentic/Non-Agentic) │    │
│  └─────────────────┘              └────────────────────────┘    │
│                                                                   │
└───────────────────────────────────────────────────────────────────┘
```

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
abcdodaf = "0.1.0"
tokio = { version = "1", features = ["full"] }
```

### Basic Usage

```rust
use abcdodaf::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a workflow combining AI agents, humans, and systems
    let workflow = WorkflowBuilder::new("customer_support")
        .name("AI-Assisted Customer Support")
        .add_agent_task(
            "analyze",
            AgentTask::new(
                "analyze",
                "Analyze Request",
                AgentCapability::NaturalLanguageProcessing,
            ),
        )
        .add_human_task(
            "review",
            HumanTask::new("review", "Review Output", HumanRole::QualityAssurance),
        )
        .add_system_task(
            "save",
            SystemTask::new("save", "Save Results", SystemOperation::DatabaseWrite),
        )
        .build()?;

    // Execute the workflow
    let result = workflow.execute().await?;

    println!("Success rate: {:.1}%", result.metrics.success_rate * 100.0);

    Ok(())
}
```

## Core Concepts

### 1. BPMN Process Modeling

BPMN (Business Process Model and Notation) provides a standard way to model business processes:

```rust
use abcdodaf::bpmn::ProcessBuilder;

let process = ProcessBuilder::new("my_process", "My Process")
    .add_user_task("task1", "Human Task")
    .add_service_task("task2", "Automated Task")
    .add_flow("flow1", "task1", "task2")
    .build()?;
```

### 2. DoDAF 2.02 Framework

DoDAF provides architectural viewpoints for comprehensive system modeling:

#### Operational View (OV)
Models operational activities and information flows:

```rust
use abcdodaf::dodaf::OperationalActivity;

let activity = OperationalActivity::new("analyze", "Analyze Data")
    .with_type(ActivityType::Automated)
    .add_performer("AI Agent")
    .add_input("raw_data")
    .add_output("insights");
```

#### Capability View (CV)
Defines system capabilities:

```rust
use abcdodaf::dodaf::{Capability, CapabilityType};

let capability = Capability::new(
    "nlp",
    "Natural Language Processing",
    CapabilityType::Cognitive,
)
.add_resource("LLM Model")
.add_metric("accuracy", serde_json::json!(0.95));
```

#### Services View (SvcV)
Describes service-oriented architecture:

```rust
use abcdodaf::dodaf::{Service, ServiceType};

let service = Service::new("ai_svc", "AI Service", ServiceType::AiInference)
    .with_endpoint("http://localhost:8080")
    .provides("nlp_capability");
```

### 3. Workforce Modeling

#### Agentic Operations (AI Agents)

```rust
use abcdodaf::workforce::{AgentTask, AgentCapability, AgentType};

let agent_task = AgentTask::new(
    "code_gen",
    "Generate Code",
    AgentCapability::CodeGeneration,
)
.with_agent_type(AgentType::CodeAgent)
.with_autonomy(0.8)  // 80% autonomous
.add_parameter("temperature", serde_json::json!(0.7));
```

**Agent Capabilities**:
- `NaturalLanguageProcessing` - Understanding and generating text
- `ComputerVision` - Image understanding and generation
- `CodeGeneration` - Generating and analyzing code
- `MathematicalReasoning` - Mathematical problem solving
- `LogicalReasoning` - Multi-step logical reasoning
- `InformationRetrieval` - Searching and retrieving information
- `ToolUsage` - Using external tools and APIs
- `KnowledgeSynthesis` - Combining information from multiple sources
- `DecisionMaking` - Making informed decisions
- `PatternRecognition` - Identifying patterns in data

#### Non-Agentic Operations (Humans & Systems)

**Human Tasks**:
```rust
use abcdodaf::workforce::{HumanTask, HumanRole};

let human_task = HumanTask::new(
    "review",
    "Expert Review",
    HumanRole::Expert,
)
.with_complexity(4)  // 1-5 scale
.with_duration(30)   // minutes
.delegatable();
```

**System Tasks**:
```rust
use abcdodaf::workforce::{SystemTask, SystemOperation, RetryPolicy};

let system_task = SystemTask::new(
    "api_call",
    "Call External API",
    SystemOperation::ApiCall,
)
.with_timeout(30)
.with_retry_policy(
    RetryPolicy::new(3)
        .with_initial_delay(1000)
        .with_backoff(2.0)
);
```

### 4. BPM+ Triple Threat (BPMN, CMMN, DMN)

The library implements the three OMG standards for comprehensive business modeling:

1. **BPMN 2.0** - Structured, repeatable process flows with gateways and orchestration
2. **CMMN 1.1** - Adaptive, knowledge-intensive case management with sentries
3. **DMN 1.3** - Reusable business decision logic with decision tables and hit policies

```rust
use abcdodaf::bpm_plus::*;
use std::collections::HashMap;

// Create DMN decision
let decision = DmnDecision::with_decision_table(
    "priority_decision",
    "Determine Priority",
    decision_table,
);

// Create BPMN process with decision task
let mut process = BpmnProcess::new("triage", "Ticket Triage");
process.add_flow_element(FlowElement::BusinessRuleTask {
    id: "decide_priority".to_string(),
    name: "Determine Priority".to_string(),
    decision_ref: Some("priority_decision".to_string()),
    incoming: vec!["flow1".to_string()],
    outgoing: vec!["flow2".to_string()],
});

// Create CMMN case for complex scenarios
let mut case = CmmnCase::new("complex_case", "Complex Inquiry");
case.add_plan_item(PlanItem::DecisionTask {
    id: "decision1".to_string(),
    name: "Route Case".to_string(),
    decision_ref: "routing_decision".to_string(),
    entry_criteria: vec![],
    exit_criteria: vec![],
});

// Integrate all three standards
let model = BpmPlusBuilder::new("Support System", "AI Support")
    .add_decision(decision)
    .add_process(process)
    .add_case(case)
    .link_decision_to_process("priority_decision", "triage", "decide_priority")
    .build()?;

// Execute DMN decision
let mut inputs = HashMap::new();
inputs.insert("urgency".to_string(), serde_json::json!("high"));
let result = model.decisions[0].execute(&inputs)?;

// Export to standard XML formats
let bpmn_xml = model.processes[0].to_bpmn_xml();
let dmn_xml = model.decisions[0].to_dmn_xml();
let cmmn_xml = model.cases[0].to_cmmn_xml();
```

See `examples/bpm_plus_triple_threat.rs` for a complete AI customer support example.

### 5. MCP Integration

Integrate with RhizOS Model Context Protocol for distributed execution:

```rust
use abcdodaf::integration::McpIntegration;

let mcp = McpIntegration::new("http://localhost:3000")
    .with_config(McpConfig {
        timeout_secs: 300,
        retry_attempts: 3,
        gpu_enabled: true,
    });

let job_id = mcp.submit_agent_task(&agent_task).await?;
```

## Examples

### Simple Customer Support Workflow

```bash
cargo run --example simple_workflow
```

This example demonstrates:
- AI agent analyzing customer inquiries
- AI agent generating responses
- Human QA review
- Automated email sending and logging

### Advanced Multi-Agent Orchestration

```bash
cargo run --example agent_orchestration
```

This example shows:
- Complete DoDAF 2.02 architecture definition
- BPM+ triple threat (BPMN, CMMN, DMN)
- Multi-agent coordination (Reasoner, Researcher, Code Generator)
- Human expert integration
- Architecture export to JSON

### BPM+ Triple Threat Demo

```bash
cargo run --example bpm_plus_triple_threat
```

This comprehensive example demonstrates:
- **BPMN 2.0**: Customer ticket triage process with service tasks, business rule tasks, and gateways
- **DMN 1.3**: Decision tables for ticket priority and agent assignment
- **CMMN 1.1**: Adaptive case management for complex customer inquiries
- Seamless integration between all three standards
- Runtime DMN decision execution
- Export to standard XML formats (BPMN, CMMN, DMN)
- Full validation and error checking

## DoDAF 2.02 Viewpoints

### Operational Viewpoint (OV)
- **OV-1**: High-level operational concept
- **OV-2**: Operational resource flow (via operational activities)
- **OV-3**: Operational information exchange (via information exchanges)
- **OV-5**: Operational activity model (fully supported)

### Capability Viewpoint (CV)
- **CV-1**: Vision (via capability descriptions)
- **CV-2**: Capability taxonomy (via capability hierarchy)
- **CV-4**: Capability dependencies (via capability mappings)

### Services Viewpoint (SvcV)
- **SvcV-1**: Services context (via service view)
- **SvcV-4**: Services functionality (via service operations)
- **SvcV-7**: Services interaction (via service interactions)

## Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run integration tests
cargo test --test integration_tests
```

## Use Cases

1. **AI Customer Support**: Orchestrate AI agents, human experts, and automated systems for customer service
2. **Code Generation Workflows**: Coordinate code generation agents with human review and testing
3. **Data Analysis Pipelines**: Chain together AI analysis, human validation, and data storage
4. **Multi-Agent Research**: Orchestrate multiple specialized AI agents for complex research tasks
5. **Hybrid Decision Making**: Combine AI recommendations with human expertise for critical decisions

## Architecture Integration

ABCDODAF is designed to integrate seamlessly with the RhizOS cloud infrastructure:

- **Node Agent**: Submit tasks to compute nodes via MCP
- **Orchestrator**: Workflow management and job scheduling
- **Desktop App**: Visual workflow monitoring and control
- **MCP Adapters**: Hardware-agnostic execution (Docker, CUDA, WASM)

## Performance Considerations

- **Async Execution**: Built on Tokio for efficient async operations
- **Parallel Task Execution**: Support for concurrent task execution (future enhancement)
- **Resource Management**: DoDAF resource requirements for optimal scheduling
- **Retry Policies**: Configurable retry logic for resilient execution

## Roadmap

- [ ] Visual workflow designer integration
- [ ] Real-time workflow monitoring
- [ ] Advanced BPMN elements (subprocesses, events)
- [ ] Workflow persistence and recovery
- [ ] Performance metrics and analytics
- [ ] Integration with more MCP servers
- [ ] BPMN 2.0 XML import/export (complete spec)

## License

MIT

## Contributing

Contributions are welcome! Please see the main project repository for guidelines.

## References

- [BPMN 2.0 Specification](https://www.omg.org/spec/BPMN/2.0/)
- [DoDAF 2.02 Documentation](https://dodcio.defense.gov/Library/DoD-Architecture-Framework/)
- [Model Context Protocol](https://modelcontextprotocol.io/)
- [RhizOS Project](https://github.com/server9-dev/otherthing-cloud)

---

Built with ❤️ for the AI workforce revolution
