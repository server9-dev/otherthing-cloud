# DMN Quick Start Guide

**Quick reference for using the DMN 1.3 engine in ABCDODAF**

---

## Installation

The DMN module is built into ABCDODAF. Just import from prelude:

```rust
use abcdodaf::prelude::*;
```

---

## 1. Simple Decision Table (5 minutes)

```rust
// Create decision table
let mut table = DecisionTable::new("age_category");
table.hit_policy = HitPolicy::First;

// Add input column
table.add_input(DecisionTableInput {
    id: "age".into(),
    label: "Age".into(),
    expression: Expression::Literal(FeelValue::Null),
    input_type: Some("number".into()),
});

// Add output column
table.add_output(DecisionTableOutput {
    id: "category".into(),
    label: "Category".into(),
    output_type: Some("string".into()),
    default_value: None,
});

// Add rules
let mut rule1 = RuleEntry::new(1, 1);
rule1.set_input(0, Some("< 18".into())).unwrap();
rule1.set_output(0, "\"Minor\"".into()).unwrap();
table.add_rule(rule1).unwrap();

let mut rule2 = RuleEntry::new(1, 1);
rule2.set_input(0, Some(">= 18".into())).unwrap();
rule2.set_output(0, "\"Adult\"".into()).unwrap();
table.add_rule(rule2).unwrap();

// Execute
let mut executor = DecisionExecutor::new();
executor.set_input("age".into(), FeelValue::Number(25.0));
let result = executor.execute_table(&table)?;

println!("Category: {:?}", result.final_output.get("category"));
// Output: Category: Some(String("Adult"))
```

---

## 2. FEEL Expressions (5 minutes)

### Simple Arithmetic
```rust
let evaluator = FeelEvaluator::new();
let expr = FeelExpression::Binary {
    operator: BinaryOp::Add,
    left: Box::new(FeelExpression::Literal(FeelValue::Number(10.0))),
    right: Box::new(FeelExpression::Literal(FeelValue::Number(32.0))),
};
let result = evaluator.evaluate(&expr)?;
// result: Number(42.0)
```

### With Variables
```rust
let mut evaluator = FeelEvaluator::new();
evaluator.set_variable("x".into(), FeelValue::Number(10.0));
evaluator.set_variable("y".into(), FeelValue::Number(20.0));

let expr = FeelExpression::Comparison {
    operator: ComparisonOp::GreaterThan,
    left: Box::new(FeelExpression::Binary {
        operator: BinaryOp::Add,
        left: Box::new(FeelExpression::Variable("x".into())),
        right: Box::new(FeelExpression::Variable("y".into())),
    }),
    right: Box::new(FeelExpression::Literal(FeelValue::Number(25.0))),
};
let result = evaluator.evaluate(&expr)?;
// result: Boolean(true)
```

### Built-in Functions
```rust
// abs(-42) -> 42
// max(1, 2, 3) -> 3
// min(1, 2, 3) -> 1
// count([1, 2, 3]) -> 3
// uppercase("hello") -> "HELLO"
// lowercase("HELLO") -> "hello"
// substring("hello", 1, 3) -> "hel"
```

---

## 3. Decision Graph (5 minutes)

```rust
let mut graph = DecisionGraph::new("lending_system");

// Add input node
graph.add_node(DecisionNode::InputData {
    id: "credit_score".into(),
    name: "Credit Score".into(),
    description: None,
}).unwrap();

// Add decision node
graph.add_node(DecisionNode::Decision {
    id: "approval".into(),
    name: "Loan Approval".into(),
    description: None,
}).unwrap();

// Add requirement (credit_score -> approval)
graph.add_requirement("credit_score".into(), "approval".into()).unwrap();

// Get execution order (topological sort)
let order = graph.topological_sort()?;
// order: ["credit_score", "approval"]

// Validate graph
graph.validate()?;
```

---

## 4. Common Patterns

### Pattern 1: Age Category Decision
```rust
fn create_age_table() -> DecisionTable {
    let mut table = DecisionTable::new("age_category");
    table.hit_policy = HitPolicy::First;

    // Add input
    table.add_input(DecisionTableInput {
        id: "age".into(),
        label: "Age".into(),
        expression: Expression::Literal(FeelValue::Null),
        input_type: Some("number".into()),
    });

    // Add output
    table.add_output(DecisionTableOutput {
        id: "category".into(),
        label: "Category".into(),
        output_type: Some("string".into()),
        default_value: Some(FeelValue::String("Unknown".into())),
    });

    // Add rules
    macro_rules! add_rule {
        ($table:expr, $condition:expr, $output:expr, $index:expr) => {
            let mut rule = RuleEntry::new(1, 1);
            rule.set_input(0, Some($condition.into())).unwrap();
            rule.set_output(0, $output.to_string()).unwrap();
            $table.add_rule(rule).unwrap();
        };
    }

    add_rule!(table, "< 13", "\"Child\"", 0);
    add_rule!(table, "< 18", "\"Teen\"", 1);
    add_rule!(table, "< 65", "\"Adult\"", 2);
    add_rule!(table, ">= 65", "\"Senior\"", 3);

    table
}
```

### Pattern 2: Loan Approval with Multiple Inputs
```rust
fn create_approval_table() -> DecisionTable {
    let mut table = DecisionTable::new("loan_approval");
    table.hit_policy = HitPolicy::First;

    // Inputs
    table.add_input(DecisionTableInput {
        id: "credit_score".into(),
        label: "Credit Score".into(),
        expression: Expression::Literal(FeelValue::Null),
        input_type: Some("number".into()),
    });

    table.add_input(DecisionTableInput {
        id: "income".into(),
        label: "Annual Income".into(),
        expression: Expression::Literal(FeelValue::Null),
        input_type: Some("number".into()),
    });

    // Output
    table.add_output(DecisionTableOutput {
        id: "decision".into(),
        label: "Decision".into(),
        output_type: Some("string".into()),
        default_value: None,
    });

    // Rules
    let mut rule1 = RuleEntry::new(2, 1);
    rule1.set_input(0, Some("< 600".into())).unwrap();
    rule1.set_input(1, Some("< 30000".into())).unwrap();
    rule1.set_output(0, "\"Deny\"".into()).unwrap();
    table.add_rule(rule1).unwrap();

    let mut rule2 = RuleEntry::new(2, 1);
    rule2.set_input(0, Some(">= 700".into())).unwrap();
    rule2.set_input(1, Some(">= 50000".into())).unwrap();
    rule2.set_output(0, "\"Approve\"".into()).unwrap();
    table.add_rule(rule2).unwrap();

    let mut rule3 = RuleEntry::new(2, 1);
    rule3.set_input(0, None).unwrap();  // Wildcard
    rule3.set_input(1, None).unwrap();  // Wildcard
    rule3.set_output(0, "\"Review\"".into()).unwrap();
    table.add_rule(rule3).unwrap();

    table
}
```

### Pattern 3: Executing a Decision
```rust
fn execute_decision(
    table: &DecisionTable,
    inputs: &[(&str, FeelValue)],
) -> DmnResult<HashMap<String, FeelValue>> {
    let mut executor = DecisionExecutor::new();

    for (name, value) in inputs {
        executor.set_input(name.to_string(), value.clone());
    }

    let result = executor.execute_table(table)?;
    Ok(result.final_output)
}

// Usage
let table = create_age_table();
let outputs = execute_decision(
    &table,
    &[("age", FeelValue::Number(25.0))],
)?;
```

---

## 5. Hit Policies

| Policy | Usage | When to Use |
|--------|-------|------------|
| **UNIQUE** | Exactly one rule must match | Validation, constraints |
| **FIRST** | First match wins | Sequential decisions |
| **PRIORITY** | Multiple matches, priority wins | Complex routing |
| **ANY** | All matches must be identical | Redundancy checking |
| **COLLECT** | Collect all outputs | Multi-result decisions |
| **RULE ORDER** | All in definition order | Process flows |
| **OUTPUT ORDER** | Ordered by priority | Preference ranking |

### Example: Using COLLECT Policy
```rust
let mut table = DecisionTable::new("discount_calculator");
table.hit_policy = HitPolicy::Collect;  // Collect all matching discounts

// Multiple rules can match, results are collected
table.add_rule(...);  // 10% discount
table.add_rule(...);  // 5% loyalty bonus
table.add_rule(...);  // 3% weekend offer

let result = executor.execute_table(&table)?;
// final_output will contain List of all matching discounts
```

---

## 6. Error Handling

```rust
match executor.execute_table(&table) {
    Ok(result) => {
        println!("Success: {:?}", result.final_output);
    }
    Err(DmnError::NoMatchingRules) => {
        println!("No rules matched");
    }
    Err(DmnError::MissingInput(name)) => {
        println!("Missing input: {}", name);
    }
    Err(DmnError::InvalidDecisionTable(msg)) => {
        println!("Invalid table: {}", msg);
    }
    Err(e) => {
        println!("Error: {}", e);
    }
}
```

---

## 7. XML Export/Import

```rust
// Export to JSON
let json_str = DmnXmlManager::export_table_json(&table)?;
println!("{}", json_str);

// Export to XML
let xml_str = DmnXmlManager::export_table_xml(&table)?;
println!("{}", xml_str);

// Validate XML
if let Err(e) = DmnXmlManager::validate_xml(&xml_str) {
    println!("Invalid XML: {}", e);
}

// Import from JSON
let data = DmnXmlManager::import_from_json(&json_str)?;
println!("Imported: {:?}", data);
```

---

## 8. Integration with BPMN

```rust
// In BPMN process executor
pub fn execute_business_rule_task(
    process: &Process,
    task_id: &str,
    variables: &mut HashMap<String, FeelValue>,
) -> DmnResult<()> {
    // Find decision reference
    if let Some(decision_ref) = get_decision_reference(process, task_id) {
        // Load decision table
        let table = load_decision_table(&decision_ref)?;

        // Execute with process variables as inputs
        let mut executor = DecisionExecutor::new();
        for (key, value) in variables.iter() {
            executor.set_input(key.clone(), value.clone());
        }

        let result = executor.execute_table(&table)?;

        // Update process variables with outputs
        for (key, value) in result.final_output {
            variables.insert(key, value);
        }
    }

    Ok(())
}
```

---

## 9. Debugging

```rust
// Print decision table structure
fn print_table_structure(table: &DecisionTable) {
    println!("Table: {}", table.id);
    println!("Hit Policy: {:?}", table.hit_policy);
    println!("Inputs: {:?}", table.inputs.len());
    println!("Outputs: {:?}", table.outputs.len());
    println!("Rules: {:?}", table.rules.len());
}

// Print decision graph
fn print_graph_structure(graph: &DecisionGraph) {
    println!("Graph: {}", graph.id);
    println!("Nodes: {}", graph.nodes().count());
    println!("Requirements: {}", graph.requirements().len());
}

// Validate before execution
if let Err(e) = table.validate() {
    eprintln!("Table validation failed: {}", e);
}

if let Err(e) = graph.validate() {
    eprintln!("Graph validation failed: {}", e);
}
```

---

## 10. Testing

```rust
#[test]
fn test_age_category_table() {
    let table = create_age_table();
    assert!(table.validate().is_ok());

    let mut executor = DecisionExecutor::new();
    executor.set_input("age".into(), FeelValue::Number(15.0));

    let result = executor.execute_table(&table).unwrap();
    assert_eq!(
        result.final_output.get("category"),
        Some(&FeelValue::String("Teen".into()))
    );
}
```

---

## Troubleshooting

### Issue: "No matching rules"
**Solution**:
1. Check input values match condition syntax
2. Verify input IDs match column IDs
3. Add wildcard rule as fallback

### Issue: "Missing input"
**Solution**:
1. Set all required inputs before execute
2. Check input names match table definition
3. Use default values for optional inputs

### Issue: "Invalid decision table"
**Solution**:
1. Run `table.validate()` to get detailed error
2. Check inputs and outputs are not empty
3. Verify rules have correct number of entries

### Issue: Circular dependency in graph
**Solution**:
1. Review decision dependencies
2. Use topological sort to find cycle
3. Break cycle by removing one requirement

---

## Performance Tips

1. **Cache tables**: Load tables once, reuse for multiple executions
2. **Validate early**: Call validate() during setup, not execution
3. **Use FIRST policy**: Faster than COLLECT for single-result decisions
4. **Minimize conditions**: Fewer inputs = faster matching
5. **Batch operations**: Execute multiple times with one context

---

## Additional Resources

- **Full Documentation**: See `DMN_IMPLEMENTATION.md`
- **Examples**: See `tests/dmn_integration_tests.rs`
- **Module Source**: See `src/dmn/` directory
- **API Reference**: Use `cargo doc --lib --no-deps`

---

## Quick Command Reference

```bash
# Check DMN compiles
cargo check --lib dmn

# Run all DMN tests
cargo test --test dmn_integration_tests

# Run specific test
cargo test --test dmn_integration_tests test_feel_arithmetic

# View documentation
cargo doc --lib --no-deps --open

# Check code style
cargo fmt --check

# Lint code
cargo clippy --lib
```

---

**Happy deciding!** 🎯
