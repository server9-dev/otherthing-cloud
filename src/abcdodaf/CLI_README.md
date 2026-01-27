# ABCDODAF CLI - Command-Line Tool for BPMN JSON Workflows

A comprehensive command-line interface for working with BPMN JSON workflow files. Provides validation, conversion, information display, auto-fixing, and batch processing capabilities.

## Installation

```bash
# Install from source
cargo install --path . --bin abcdodaf-cli

# Or build for development
cargo build --bin abcdodaf-cli

# Binary location (development)
./target/debug/abcdodaf-cli

# Binary location (release)
./target/release/abcdodaf-cli
```

## Features

- **Validation**: Comprehensive BPMN 2.0 workflow validation with detailed error messages
- **Information Display**: View workflow metadata, statistics, and node details
- **Auto-Fix**: Automatically fix common issues like whitespace, version format, and empty names
- **Format Conversion**: Convert between workflow formats (BPMN JSON, Snarl, Diagram)
- **Batch Processing**: List and validate all workflows in a directory
- **JSON Output**: Machine-readable JSON output for automation and scripting
- **Colored Output**: User-friendly colored terminal output (can be disabled)
- **Exit Codes**: Proper exit codes for CI/CD integration

## Exit Codes

- `0`: Success
- `1`: Validation error (workflow has errors or warnings)
- `2`: File error (file not found, permission denied, parse error)

## Commands

### 1. Validate

Validate a BPMN JSON workflow file against the schema and BPMN 2.0 rules.

```bash
# Basic validation
abcdodaf-cli validate workflow.json

# Show only errors (hide warnings and info)
abcdodaf-cli validate workflow.json --errors-only

# Exit with success even if there are warnings
abcdodaf-cli validate workflow.json --warn-as-success

# Get JSON output for automation
abcdodaf-cli validate workflow.json --json
```

**Validation checks:**
- JSON schema conformance
- Required fields (process ID, node IDs, flow references)
- Node type validity (BPMN 2.0 element types)
- Flow reference integrity (sourceRef and targetRef exist)
- Duplicate ID detection
- Structural requirements (start/end events, gateway connections)
- Type-specific validation (service tasks, script tasks, events)
- Version format validation
- Disconnected node detection

**Example output:**
```
Success: Workflow 'test_workflow.json' is valid!
```

```
[ERROR] [BrokenReference] Sequence flow 'flow1' references non-existent target node: 'missing_node' (flow1)
  Suggestion: Ensure the targetRef matches an existing node ID
[WARNING] [MissingField] Process name is empty (bpmn_process.name)
  Suggestion: Provide a descriptive name for the process

Validation summary: 1 error(s), 1 warning(s), 0 info message(s)
```

### 2. Info

Display workflow information including metadata, statistics, and node details.

```bash
# Basic information
abcdodaf-cli info workflow.json

# Show detailed node information
abcdodaf-cli info workflow.json --detailed

# Show only statistics
abcdodaf-cli info workflow.json --stats-only

# Get JSON output
abcdodaf-cli info workflow.json --json
```

**Example output:**
```
Workflow Information

File: test_workflow.json
Process ID: test_workflow
Process Name: Test Workflow
Version: 1.0.0
Executable: true
Process Type: agent_workflow

Statistics

Total Nodes: 6
Total Flows: 6
Start Events: 1
End Events: 1
Tasks: 3
Gateways: 1

Node Types:
  startEvent: 1
  serviceTask: 3
  exclusiveGateway: 1
  endEvent: 1
```

With `--detailed`:
```
Nodes

• Start (start_event_1)
  Type: startEvent
  Event Type: none

• Research Task (task_research)
  Type: serviceTask
  Task Type: research
  Inputs: requirements, context
  Outputs: findings
```

### 3. Convert

Convert between workflow formats (currently supports BPMN JSON pretty-printing).

```bash
# Convert (pretty-print) BPMN JSON
abcdodaf-cli convert input.json output.json --format bpmn-json

# Overwrite existing output file
abcdodaf-cli convert input.json output.json --format bpmn-json --force

# Get JSON output
abcdodaf-cli convert input.json output.json --format bpmn-json --json
```

**Supported formats:**
- `bpmn-json`: BPMN JSON format (current implementation)
- `snarl`: Snarl diagram format (planned)
- `diagram`: Internal diagram format (planned)

### 4. Fix

Automatically fix common workflow issues.

```bash
# Preview fixes (output to stdout)
abcdodaf-cli fix workflow.json

# Apply fixes in-place
abcdodaf-cli fix workflow.json --in-place

# Save fixed workflow to a new file
abcdodaf-cli fix workflow.json --output fixed_workflow.json

# Get JSON output
abcdodaf-cli fix workflow.json --json
```

**Auto-fix capabilities:**
- Trim whitespace from IDs and names
- Ensure version format (convert "1" to "1.0.0")
- Fill empty process names with process ID
- Fill empty node names with node ID

**Example:**
```bash
$ abcdodaf-cli fix broken_workflow.json --in-place
Success: Applied 6 fix(es) to 'broken_workflow.json'

$ abcdodaf-cli validate broken_workflow.json
Success: Workflow 'broken_workflow.json' is valid!
```

### 5. List

List and validate all workflow files in a directory.

```bash
# List workflow files
abcdodaf-cli list workflows/

# Recursively scan subdirectories
abcdodaf-cli list workflows/ --recursive

# Validate each file (slower but more thorough)
abcdodaf-cli list workflows/ --validate

# Show only files with errors
abcdodaf-cli list workflows/ --validate --errors-only

# Get JSON output for automation
abcdodaf-cli list workflows/ --validate --json
```

**Example output:**
```
Workflow Files

Directory: workflows/examples/
Total Files: 5
Valid: 4
With Errors: 1

✓ ./simple_workflow.json
✓ ./complex_workflow.json
⚠ ./warning_workflow.json (2 warnings)
✗ ./broken_workflow.json (3 errors)
✓ ./decision_flow.json
```

## Global Options

### `--json`

Output results in JSON format for machine processing.

```bash
abcdodaf-cli validate workflow.json --json
```

**JSON output structure (validate):**
```json
{
  "success": true,
  "file": "workflow.json",
  "errors": [],
  "summary": {
    "errors": 0,
    "warnings": 0,
    "info": 0
  }
}
```

### `--no-color`

Disable colored output for terminals that don't support ANSI colors or for piping to files.

```bash
abcdodaf-cli validate workflow.json --no-color > validation.txt
```

## BPMN JSON Format

The CLI works with BPMN JSON workflow files in this format:

```json
{
  "bpmn_process": {
    "id": "process_id",
    "name": "Process Name",
    "version": "1.0.0",
    "isExecutable": true,
    "processType": "agent_workflow"
  },
  "workflow_steps": [
    {
      "id": "start_1",
      "name": "Start Event",
      "type": "startEvent",
      "eventType": "none"
    },
    {
      "id": "task_1",
      "name": "My Task",
      "type": "serviceTask",
      "taskType": "research",
      "inputs": ["input1"],
      "outputs": ["output1"],
      "duration_minutes": 30
    },
    {
      "id": "end_1",
      "name": "End Event",
      "type": "endEvent"
    }
  ],
  "sequence_flows": [
    {
      "id": "flow_1",
      "sourceRef": "start_1",
      "targetRef": "task_1"
    },
    {
      "id": "flow_2",
      "sourceRef": "task_1",
      "targetRef": "end_1"
    }
  ]
}
```

### Valid Node Types

**Events:**
- `startEvent`, `endEvent`
- `intermediateCatchEvent`, `intermediateThrowEvent`
- `boundaryEvent`

**Tasks:**
- `task`, `serviceTask`, `userTask`, `manualTask`
- `scriptTask`, `businessRuleTask`
- `sendTask`, `receiveTask`

**Gateways:**
- `exclusiveGateway` (XOR)
- `parallelGateway` (AND)
- `inclusiveGateway` (OR)
- `eventBasedGateway`, `complexGateway`

**Subprocesses:**
- `subProcess`, `callActivity`
- `transaction`, `adHocSubProcess`

### Valid Task Types (for service tasks)

- `research`, `design`, `code_generation`
- `testing`, `documentation`, `integration`
- `debugging`, `optimization`, `deployment`
- `monitoring`

## Use Cases

### CI/CD Integration

```bash
#!/bin/bash
# Validate all workflows in CI pipeline

abcdodaf-cli list workflows/ --validate --recursive --json > validation_report.json

if [ $? -eq 0 ]; then
  echo "All workflows valid!"
  exit 0
else
  echo "Workflow validation failed!"
  cat validation_report.json
  exit 1
fi
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Find all changed JSON files
CHANGED_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep '\.json$')

if [ -z "$CHANGED_FILES" ]; then
  exit 0
fi

# Validate each file
for file in $CHANGED_FILES; do
  if [[ $file == *workflow*.json ]] || [[ $file == workflows/* ]]; then
    echo "Validating $file..."
    abcdodaf-cli validate "$file" --warn-as-success
    if [ $? -ne 0 ]; then
      echo "Validation failed for $file"
      exit 1
    fi
  fi
done

exit 0
```

### Batch Processing

```bash
# Find and fix all workflows in a directory
for file in workflows/**/*.json; do
  echo "Processing $file..."
  abcdodaf-cli fix "$file" --in-place
  abcdodaf-cli validate "$file"
done
```

### Workflow Analysis

```bash
# Generate a report of all workflows
abcdodaf-cli list workflows/ --validate --json | \
  jq '{
    total: .total_files,
    valid: .valid_files,
    errors: .files_with_errors,
    files: [.files[] | select(.status == "invalid") | .path]
  }'
```

## Development

### Building

```bash
# Development build
cargo build --bin abcdodaf-cli

# Release build (optimized)
cargo build --release --bin abcdodaf-cli
```

### Testing

```bash
# Run unit tests
cargo test --bin abcdodaf-cli

# Test all commands
./target/debug/abcdodaf-cli validate test_workflow.json
./target/debug/abcdodaf-cli info test_workflow.json --detailed
./target/debug/abcdodaf-cli fix broken_workflow.json --in-place
./target/debug/abcdodaf-cli list workflows/ --validate
```

### Adding New Validation Rules

Edit `src/bpmn/json_validation.rs` to add new validation rules. The CLI automatically uses the validation functions from the library.

## Troubleshooting

### "Failed to parse JSON"

Ensure your JSON is well-formed. Use a JSON validator or:

```bash
cat workflow.json | jq .
```

### "Missing field 'bpmn_process'"

The workflow file doesn't match the expected BPMN JSON schema. Ensure it has the required top-level structure.

### Exit code 1 even with valid workflow

Check for warnings. Use `--warn-as-success` if you want to ignore warnings:

```bash
abcdodaf-cli validate workflow.json --warn-as-success
```

## Examples

See the `workflows/examples/` directory for sample BPMN JSON workflows.

## License

MIT

## See Also

- [BPMN 2.0 Specification](https://www.omg.org/spec/BPMN/2.0/)
- [DoDAF 2.02 Framework](https://dodcio.defense.gov/Library/DoD-Architecture-Framework/)
- Main library documentation: `cargo doc --open`
