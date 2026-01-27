# ABCDODAF Agent Workflow Definitions

This directory contains BPMN 2.0-compliant JSON workflow definitions documenting the execution of agent tasks for the ABCDODAF library expansion project.

## Overview

Each agent task is documented as a BPMN 2.0 process with:
- **Process metadata** (BPMN process definition)
- **Task metadata** (agent, model, status, timing)
- **DoDAF metadata** (operational activity, capability, cost, security)
- **Workflow steps** (BPMN activities and events)
- **Sequence flows** (process flow connections)
- **Deliverables** (files, documentation, tests)
- **Success criteria** (functional, quality, business value)

## File Structure

```
workflows/
├── README.md (this file)
├── agent_tasks_master.json (master task list and coordination)
├── WORKFLOW_TEMPLATE.json (template for creating new workflows)
├── task_01_bpmn_xml.json (BPMN XML import/export workflow)
├── task_03_dmn_implementation.json (DMN decision tables workflow)
├── task_08_analytics_dashboard.json (Analytics dashboard workflow - COMPLETED)
├── task_10_ai_ollama.json (AI/LLM Ollama integration workflow)
└── task_XX_name.json (additional task workflows)
```

## Workflow Format

### BPMN 2.0 Elements Used

#### Start/End Events
```json
{
  "id": "start_event",
  "name": "Start Task",
  "type": "startEvent",
  "eventType": "none"
}
```

#### Service Tasks (Automated Activities)
```json
{
  "id": "implement_step",
  "name": "Implement Component",
  "type": "serviceTask",
  "taskType": "code_generation",
  "implementation": "rust_code_generation",
  "inputs": ["architecture"],
  "outputs": ["code_modules"],
  "duration_minutes": 5
}
```

#### Gateways (Decision Points)
```json
{
  "id": "quality_gate",
  "name": "Tests Pass?",
  "type": "exclusiveGateway"
}
```

```json
{
  "id": "parallel_split",
  "name": "Parallel Development",
  "type": "parallelGateway",
  "diverging": true
}
```

#### Sequence Flows (Connections)
```json
{
  "id": "flow1",
  "sourceRef": "start_event",
  "targetRef": "research_step"
}
```

### Common Task Types

| Task Type | Description | Examples |
|-----------|-------------|----------|
| `research` | Research specifications, patterns, best practices | Specification research, literature review |
| `design` | Architectural and component design | System architecture, data structures |
| `code_generation` | Write implementation code | Rust modules, structs, functions |
| `testing` | Create and run tests | Unit tests, integration tests |
| `documentation` | Write documentation | User guides, API docs, tutorials |
| `integration` | Integrate with existing systems | Module integration, API integration |
| `debugging` | Fix issues and errors | Bug fixes, error handling |
| `optimization` | Improve performance or quality | Performance tuning, refactoring |

### DoDAF Integration

Each workflow includes DoDAF 2.02 operational activity metadata:

```json
"dodaf_metadata": {
  "operational_activity": {
    "id": "OA-XXX",
    "name": "Activity_Name",
    "type": "Automated",
    "performer": "AI_Agent_Type",
    "security_domain": "Development",
    "cost": {
      "api_calls": 45,
      "tokens_consumed": 95000,
      "estimated_usd": 0.15
    },
    "duration_minutes": 20
  },
  "capability": "System_Capability",
  "inputs": ["input_data"],
  "outputs": ["output_data"]
}
```

## Example Workflows

### 1. Sequential Workflow (Simple)
The Analytics Dashboard (task_08) demonstrates a simple sequential workflow:
1. Research → Design → Implement → Test → Document

### 2. Parallel Workflow
The DMN Implementation (task_03) uses parallel gateways:
1. Research
2. **Split** into parallel branches:
   - Implement decision tables
   - Implement FEEL engine
   - Implement DRD
3. **Join** all branches
4. Integration → Testing → Documentation

### 3. Iterative Workflow with Decision Points
The BPMN XML workflow (task_01) includes quality gates:
1. Research → Design → Implement → Test
2. **Decision**: Tests pass?
   - **No**: Fix issues → Return to test
   - **Yes**: Document → Complete

## Using Workflows with ABCDODAF

These workflow definitions are compatible with the ABCDODAF execution engine and can be:

### 1. Imported as BPMN Processes
```rust
use abcdodaf::prelude::*;

// Load workflow from JSON
let workflow_json = std::fs::read_to_string("workflows/task_08_analytics_dashboard.json")?;
let workflow: Process = serde_json::from_str(&workflow_json)?;

// Execute the workflow
let result = workflow.execute().await?;
```

### 2. Visualized in the UI
```rust
// Open in the enhanced UI editor
cargo run --example enhanced_ui_editor --features ui

// Import workflow: File → Import → Select JSON file
```

### 3. Analyzed with DoDAF Views
```rust
use abcdodaf::dodaf::*;

// Extract operational activities
let activities = extract_operational_activities(&workflow);

// Generate OV-5 (Operational Activity Decomposition)
let ov5 = generate_ov5_view(activities);
```

### 4. Monitored with Analytics
```rust
use abcdodaf::analytics::*;

let dashboard = AnalyticsDashboard::new();

// Track workflow execution
dashboard.record_process_start("task_08_analytics_dashboard");
// ... execution ...
dashboard.record_process_completion("task_08_analytics_dashboard", duration);

// Analyze performance
let bottlenecks = dashboard.analyze_bottlenecks();
let metrics = dashboard.get_process_metrics();
```

## Creating New Workflows

Use the `WORKFLOW_TEMPLATE.json` as a starting point:

1. Copy the template:
   ```bash
   cp WORKFLOW_TEMPLATE.json task_XX_your_task.json
   ```

2. Fill in the metadata:
   - Update task_id, agent_id, model
   - Set priority and category
   - Add DoDAF operational activity details

3. Define workflow steps:
   - Add startEvent
   - Add serviceTask for each major activity
   - Add gateways for decisions or parallelism
   - Add endEvent

4. Define sequence flows:
   - Connect all steps with flows
   - Specify conditions for exclusive gateways

5. Document deliverables:
   - List expected files
   - Define success criteria
   - Note dependencies

6. Update the master file:
   ```bash
   # Add your task to agent_tasks_master.json
   ```

## Workflow Analysis

### Metrics Tracked

For each workflow, we track:
- **Duration**: Total execution time
- **Cost**: API calls, tokens, estimated USD
- **Quality**: Test coverage, warnings, errors
- **Deliverables**: Files created, lines of code, documentation
- **Status**: pending, in_progress, completed, failed

### Example Analysis
```bash
# Count total tasks
cat agent_tasks_master.json | jq '.tasks | length'

# List completed tasks
cat agent_tasks_master.json | jq '.tasks[] | select(.status == "completed")'

# Calculate total cost (hypothetical)
cat workflows/task_*.json | jq -r '.dodaf_metadata.operational_activity.cost.estimated_usd' | awk '{sum+=$1} END {print sum}'

# Count total deliverables
cat workflows/task_08_analytics_dashboard.json | jq '.deliverables.statistics'
```

## Integration with CI/CD

These workflows can be used for:

1. **Automated Testing**: Validate that delivered files exist
   ```bash
   # Check if all expected modules were created
   for file in $(jq -r '.deliverables.modules[]' task_08_analytics_dashboard.json); do
     test -f "$file" && echo "✓ $file" || echo "✗ Missing: $file"
   done
   ```

2. **Documentation Generation**: Auto-generate project reports
   ```bash
   # Generate completion report
   python scripts/generate_completion_report.py workflows/
   ```

3. **Cost Tracking**: Monitor API usage across all agents
   ```bash
   # Sum total tokens consumed
   jq -r '.dodaf_metadata.operational_activity.cost.tokens_consumed // 0' workflows/task_*.json | awk '{sum+=$1} END {print "Total tokens:", sum}'
   ```

## Visualization

Workflows can be visualized using:

1. **ABCDODAF Enhanced UI** (built-in)
2. **BPMN.io** (online viewer)
3. **Camunda Modeler** (desktop app)
4. **Custom renderers** (using the JSON structure)

### Example: Generate SVG Diagram
```python
# Python script to generate SVG from workflow JSON
import json
from bpmn_python import BpmnDiagram

with open('task_08_analytics_dashboard.json') as f:
    workflow = json.load(f)

diagram = BpmnDiagram.from_json(workflow)
diagram.export_to_svg('task_08_diagram.svg')
```

## Compliance and Standards

These workflows follow:
- **BPMN 2.0** specification
- **DoDAF 2.02** framework
- **JSON Schema** for validation
- **Semantic versioning** for workflow versions

## Future Enhancements

Planned improvements:
1. **BPMN XML Export**: Convert JSON to standard BPMN 2.0 XML
2. **Real-time Updates**: Live workflow status updates
3. **Collaboration**: Multi-agent workflow coordination
4. **Validation**: JSON schema validation
5. **Simulation**: What-if analysis for workflow optimization

## Related Documentation

- `../EXPANSION_ROADMAP.md` - Overall expansion plan
- `../docs/BPMN_SPEC_SUMMARY.md` - BPMN specification
- `../docs/DODAF_SPEC_SUMMARY.md` - DoDAF framework
- `../ANALYTICS_GUIDE.md` - Analytics system guide

## Questions?

For questions or issues related to workflow definitions:
1. Review the WORKFLOW_TEMPLATE.json for structure
2. Check existing task workflows for examples
3. Consult the BPMN 2.0 specification
4. See the ABCDODAF library documentation

---

**Last Updated**: 2026-01-27
**Format Version**: 1.0.0
**Standard**: BPMN 2.0 + DoDAF 2.02
