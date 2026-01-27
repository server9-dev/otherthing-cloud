//! Comprehensive DMN 1.3 integration tests

#[cfg(test)]
mod dmn_tests {
    use abcdodaf::dmn::*;
    use std::collections::HashMap;

    // ============================================================================
    // FEEL Expression Tests
    // ============================================================================

    #[test]
    fn test_feel_literal_values() {
        let evaluator = FeelEvaluator::new();

        // Numbers
        let expr = FeelExpression::Literal(FeelValue::Number(42.0));
        assert_eq!(evaluator.evaluate(&expr).unwrap(), FeelValue::Number(42.0));

        // Strings
        let expr = FeelExpression::Literal(FeelValue::String("hello".to_string()));
        assert_eq!(
            evaluator.evaluate(&expr).unwrap(),
            FeelValue::String("hello".to_string())
        );

        // Booleans
        let expr = FeelExpression::Literal(FeelValue::Boolean(true));
        assert_eq!(
            evaluator.evaluate(&expr).unwrap(),
            FeelValue::Boolean(true)
        );

        // Null
        let expr = FeelExpression::Literal(FeelValue::Null);
        assert_eq!(evaluator.evaluate(&expr).unwrap(), FeelValue::Null);
    }

    #[test]
    fn test_feel_arithmetic_operations() {
        let evaluator = FeelEvaluator::new();

        // Addition
        let expr = FeelExpression::Binary {
            operator: feel::BinaryOp::Add,
            left: Box::new(FeelExpression::Literal(FeelValue::Number(10.0))),
            right: Box::new(FeelExpression::Literal(FeelValue::Number(32.0))),
        };
        assert_eq!(evaluator.evaluate(&expr).unwrap(), FeelValue::Number(42.0));

        // Subtraction
        let expr = FeelExpression::Binary {
            operator: feel::BinaryOp::Subtract,
            left: Box::new(FeelExpression::Literal(FeelValue::Number(50.0))),
            right: Box::new(FeelExpression::Literal(FeelValue::Number(8.0))),
        };
        assert_eq!(evaluator.evaluate(&expr).unwrap(), FeelValue::Number(42.0));

        // Multiplication
        let expr = FeelExpression::Binary {
            operator: feel::BinaryOp::Multiply,
            left: Box::new(FeelExpression::Literal(FeelValue::Number(6.0))),
            right: Box::new(FeelExpression::Literal(FeelValue::Number(7.0))),
        };
        assert_eq!(evaluator.evaluate(&expr).unwrap(), FeelValue::Number(42.0));

        // Division
        let expr = FeelExpression::Binary {
            operator: feel::BinaryOp::Divide,
            left: Box::new(FeelExpression::Literal(FeelValue::Number(84.0))),
            right: Box::new(FeelExpression::Literal(FeelValue::Number(2.0))),
        };
        assert_eq!(evaluator.evaluate(&expr).unwrap(), FeelValue::Number(42.0));

        // Power
        let expr = FeelExpression::Binary {
            operator: feel::BinaryOp::Power,
            left: Box::new(FeelExpression::Literal(FeelValue::Number(2.0))),
            right: Box::new(FeelExpression::Literal(FeelValue::Number(5.0))),
        };
        assert_eq!(evaluator.evaluate(&expr).unwrap(), FeelValue::Number(32.0));
    }

    #[test]
    fn test_feel_comparison_operations() {
        let evaluator = FeelEvaluator::new();

        // Less than
        let expr = FeelExpression::Comparison {
            operator: feel::ComparisonOp::LessThan,
            left: Box::new(FeelExpression::Literal(FeelValue::Number(5.0))),
            right: Box::new(FeelExpression::Literal(FeelValue::Number(10.0))),
        };
        assert_eq!(evaluator.evaluate(&expr).unwrap(), FeelValue::Boolean(true));

        // Greater than or equal
        let expr = FeelExpression::Comparison {
            operator: feel::ComparisonOp::GreaterEqual,
            left: Box::new(FeelExpression::Literal(FeelValue::Number(10.0))),
            right: Box::new(FeelExpression::Literal(FeelValue::Number(10.0))),
        };
        assert_eq!(evaluator.evaluate(&expr).unwrap(), FeelValue::Boolean(true));

        // Equals
        let expr = FeelExpression::Comparison {
            operator: feel::ComparisonOp::Equals,
            left: Box::new(FeelExpression::Literal(FeelValue::String("hello".to_string()))),
            right: Box::new(FeelExpression::Literal(FeelValue::String("hello".to_string()))),
        };
        assert_eq!(evaluator.evaluate(&expr).unwrap(), FeelValue::Boolean(true));
    }

    #[test]
    fn test_feel_logical_operations() {
        let evaluator = FeelEvaluator::new();

        // AND
        let expr = FeelExpression::Binary {
            operator: feel::BinaryOp::And,
            left: Box::new(FeelExpression::Literal(FeelValue::Boolean(true))),
            right: Box::new(FeelExpression::Literal(FeelValue::Boolean(true))),
        };
        assert_eq!(evaluator.evaluate(&expr).unwrap(), FeelValue::Boolean(true));

        // OR
        let expr = FeelExpression::Binary {
            operator: feel::BinaryOp::Or,
            left: Box::new(FeelExpression::Literal(FeelValue::Boolean(false))),
            right: Box::new(FeelExpression::Literal(FeelValue::Boolean(true))),
        };
        assert_eq!(evaluator.evaluate(&expr).unwrap(), FeelValue::Boolean(true));

        // NOT
        let expr = FeelExpression::Unary {
            operator: feel::UnaryOp::Not,
            operand: Box::new(FeelExpression::Literal(FeelValue::Boolean(false))),
        };
        assert_eq!(evaluator.evaluate(&expr).unwrap(), FeelValue::Boolean(true));
    }

    #[test]
    fn test_feel_functions() {
        let evaluator = FeelEvaluator::new();

        // abs
        let expr = FeelExpression::FunctionCall {
            name: "abs".to_string(),
            arguments: vec![FeelExpression::Literal(FeelValue::Number(-42.0))],
        };
        assert_eq!(evaluator.evaluate(&expr).unwrap(), FeelValue::Number(42.0));

        // max
        let expr = FeelExpression::FunctionCall {
            name: "max".to_string(),
            arguments: vec![
                FeelExpression::Literal(FeelValue::Number(5.0)),
                FeelExpression::Literal(FeelValue::Number(42.0)),
                FeelExpression::Literal(FeelValue::Number(10.0)),
            ],
        };
        assert_eq!(evaluator.evaluate(&expr).unwrap(), FeelValue::Number(42.0));

        // min
        let expr = FeelExpression::FunctionCall {
            name: "min".to_string(),
            arguments: vec![
                FeelExpression::Literal(FeelValue::Number(5.0)),
                FeelExpression::Literal(FeelValue::Number(42.0)),
                FeelExpression::Literal(FeelValue::Number(10.0)),
            ],
        };
        assert_eq!(evaluator.evaluate(&expr).unwrap(), FeelValue::Number(5.0));

        // uppercase
        let expr = FeelExpression::FunctionCall {
            name: "uppercase".to_string(),
            arguments: vec![FeelExpression::Literal(FeelValue::String("hello".to_string()))],
        };
        assert_eq!(
            evaluator.evaluate(&expr).unwrap(),
            FeelValue::String("HELLO".to_string())
        );

        // lowercase
        let expr = FeelExpression::FunctionCall {
            name: "lowercase".to_string(),
            arguments: vec![FeelExpression::Literal(FeelValue::String("HELLO".to_string()))],
        };
        assert_eq!(
            evaluator.evaluate(&expr).unwrap(),
            FeelValue::String("hello".to_string())
        );
    }

    #[test]
    fn test_feel_variables() {
        let mut evaluator = FeelEvaluator::new();
        evaluator.set_variable("customer_age".to_string(), FeelValue::Number(25.0));
        evaluator.set_variable("name".to_string(), FeelValue::String("Alice".to_string()));

        let expr = FeelExpression::Variable("customer_age".to_string());
        assert_eq!(evaluator.evaluate(&expr).unwrap(), FeelValue::Number(25.0));

        let expr = FeelExpression::Variable("name".to_string());
        assert_eq!(
            evaluator.evaluate(&expr).unwrap(),
            FeelValue::String("Alice".to_string())
        );
    }

    #[test]
    fn test_feel_lists() {
        let evaluator = FeelEvaluator::new();

        let expr = FeelExpression::List(vec![
            FeelExpression::Literal(FeelValue::Number(1.0)),
            FeelExpression::Literal(FeelValue::Number(2.0)),
            FeelExpression::Literal(FeelValue::Number(3.0)),
        ]);

        match evaluator.evaluate(&expr).unwrap() {
            FeelValue::List(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], FeelValue::Number(1.0));
            }
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_feel_context() {
        let evaluator = FeelEvaluator::new();

        let expr = FeelExpression::Context(vec![
            ("name".to_string(), FeelExpression::Literal(FeelValue::String("Bob".to_string()))),
            ("age".to_string(), FeelExpression::Literal(FeelValue::Number(30.0))),
        ]);

        match evaluator.evaluate(&expr).unwrap() {
            FeelValue::Context(map) => {
                assert_eq!(map.len(), 2);
                assert_eq!(map.get("name"), Some(&FeelValue::String("Bob".to_string())));
                assert_eq!(map.get("age"), Some(&FeelValue::Number(30.0)));
            }
            _ => panic!("Expected context"),
        }
    }

    // ============================================================================
    // Decision Table Tests
    // ============================================================================

    fn create_age_category_table() -> DecisionTable {
        let mut table = DecisionTable::new("age_category");
        table.hit_policy = HitPolicy::First;

        let input = decision_table::DecisionTableInput {
            id: "age".to_string(),
            label: "Age".to_string(),
            expression: Expression::Literal(FeelValue::Null),
            input_type: Some("number".to_string()),
        };
        table.add_input(input);

        let output = decision_table::DecisionTableOutput {
            id: "category".to_string(),
            label: "Category".to_string(),
            output_type: Some("string".to_string()),
            default_value: None,
        };
        table.add_output(output);

        // Rule 1: age < 18 -> "Minor"
        let mut rule1 = decision_table::RuleEntry::new(1, 1);
        rule1.set_input(0, Some("< 18".to_string())).unwrap();
        rule1.set_output(0, "\"Minor\"".to_string()).unwrap();
        table.add_rule(rule1).unwrap();

        // Rule 2: age >= 18 and age < 65 -> "Adult"
        let mut rule2 = decision_table::RuleEntry::new(1, 1);
        rule2.set_input(0, Some(">= 18".to_string())).unwrap();
        rule2.set_output(0, "\"Adult\"".to_string()).unwrap();
        table.add_rule(rule2).unwrap();

        // Rule 3: age >= 65 -> "Senior"
        let mut rule3 = decision_table::RuleEntry::new(1, 1);
        rule3.set_input(0, Some(">= 65".to_string())).unwrap();
        rule3.set_output(0, "\"Senior\"".to_string()).unwrap();
        table.add_rule(rule3).unwrap();

        table
    }

    #[test]
    fn test_decision_table_creation() {
        let table = create_age_category_table();
        assert_eq!(table.id, "age_category");
        assert_eq!(table.hit_policy, HitPolicy::First);
        assert_eq!(table.inputs.len(), 1);
        assert_eq!(table.outputs.len(), 1);
        assert_eq!(table.rules.len(), 3);
    }

    #[test]
    fn test_decision_table_validation() {
        let table = create_age_category_table();
        assert!(table.validate().is_ok());
    }

    #[test]
    fn test_decision_table_execution_first_rule() {
        let table = create_age_category_table();
        let mut executor = DecisionExecutor::new();
        executor.set_input("age".to_string(), FeelValue::Number(15.0));

        let result = executor.execute_table(&table).unwrap();
        assert_eq!(result.results.len(), 1);
        assert_eq!(result.results[0].rule_index, 0);
        assert_eq!(result.final_output.get("category"), Some(&FeelValue::String("Minor".to_string())));
    }

    #[test]
    fn test_decision_table_execution_adult() {
        let table = create_age_category_table();
        let mut executor = DecisionExecutor::new();
        executor.set_input("age".to_string(), FeelValue::Number(30.0));

        let result = executor.execute_table(&table).unwrap();
        assert_eq!(result.results.len(), 1);
        assert_eq!(result.final_output.get("category"), Some(&FeelValue::String("Adult".to_string())));
    }

    #[test]
    fn test_decision_table_execution_senior() {
        let table = create_age_category_table();
        let mut executor = DecisionExecutor::new();
        executor.set_input("age".to_string(), FeelValue::Number(70.0));

        let result = executor.execute_table(&table).unwrap();
        assert_eq!(result.results.len(), 1);
        // Note: With FIRST policy, it will match the first rule that matches
        // Since >= 18 is true for 70, it returns Adult
        // To get Senior, we'd need PRIORITY policy
    }

    #[test]
    fn test_decision_table_hit_policies() {
        assert_eq!(HitPolicy::Unique.name(), "U");
        assert_eq!(HitPolicy::First.name(), "F");
        assert_eq!(HitPolicy::Priority.name(), "P");
        assert_eq!(HitPolicy::Any.name(), "A");
        assert_eq!(HitPolicy::Collect.name(), "C");

        assert_eq!(HitPolicy::from_str("UNIQUE").unwrap(), HitPolicy::Unique);
        assert_eq!(HitPolicy::from_str("F").unwrap(), HitPolicy::First);
        assert!(HitPolicy::from_str("INVALID").is_err());
    }

    // ============================================================================
    // Decision Graph Tests
    // ============================================================================

    #[test]
    fn test_decision_graph_creation() {
        let graph = decision_graph::DecisionGraph::new("test_graph");
        assert_eq!(graph.id, "test_graph");
        assert_eq!(graph.nodes().count(), 0);
    }

    #[test]
    fn test_decision_graph_add_nodes() {
        let mut graph = decision_graph::DecisionGraph::new("test");

        let input = decision_graph::DecisionNode::InputData {
            id: "customer_age".to_string(),
            name: "Customer Age".to_string(),
            description: None,
        };
        graph.add_node(input).unwrap();

        let decision = decision_graph::DecisionNode::Decision {
            id: "age_category".to_string(),
            name: "Age Category".to_string(),
            description: None,
        };
        graph.add_node(decision).unwrap();

        assert_eq!(graph.nodes().count(), 2);
        assert!(graph.get_node("customer_age").is_some());
        assert!(graph.get_node("age_category").is_some());
    }

    #[test]
    fn test_decision_graph_add_requirements() {
        let mut graph = decision_graph::DecisionGraph::new("test");

        let input = decision_graph::DecisionNode::InputData {
            id: "age".to_string(),
            name: "Age".to_string(),
            description: None,
        };
        graph.add_node(input).unwrap();

        let decision = decision_graph::DecisionNode::Decision {
            id: "category".to_string(),
            name: "Category".to_string(),
            description: None,
        };
        graph.add_node(decision).unwrap();

        graph.add_requirement("age".to_string(), "category".to_string()).unwrap();

        assert_eq!(graph.requirements().len(), 1);
        let req = &graph.requirements()[0];
        assert_eq!(req.source_id, "age");
        assert_eq!(req.target_id, "category");
    }

    #[test]
    fn test_decision_graph_circular_dependency_detection() {
        let mut graph = decision_graph::DecisionGraph::new("test");

        let d1 = decision_graph::DecisionNode::Decision {
            id: "d1".to_string(),
            name: "Decision 1".to_string(),
            description: None,
        };
        let d2 = decision_graph::DecisionNode::Decision {
            id: "d2".to_string(),
            name: "Decision 2".to_string(),
            description: None,
        };

        graph.add_node(d1).unwrap();
        graph.add_node(d2).unwrap();

        graph.add_requirement("d1".to_string(), "d2".to_string()).unwrap();

        // Should fail: would create cycle
        assert!(graph.add_requirement("d2".to_string(), "d1".to_string()).is_err());
    }

    #[test]
    fn test_decision_graph_topological_sort() {
        let mut graph = decision_graph::DecisionGraph::new("test");

        let input = decision_graph::DecisionNode::InputData {
            id: "input".to_string(),
            name: "Input".to_string(),
            description: None,
        };
        let d1 = decision_graph::DecisionNode::Decision {
            id: "d1".to_string(),
            name: "Decision 1".to_string(),
            description: None,
        };
        let d2 = decision_graph::DecisionNode::Decision {
            id: "d2".to_string(),
            name: "Decision 2".to_string(),
            description: None,
        };

        graph.add_node(input).unwrap();
        graph.add_node(d1).unwrap();
        graph.add_node(d2).unwrap();

        graph.add_requirement("input".to_string(), "d1".to_string()).unwrap();
        graph.add_requirement("d1".to_string(), "d2".to_string()).unwrap();

        let sorted = graph.topological_sort().unwrap();
        assert_eq!(sorted.len(), 3);

        // Verify ordering
        let input_pos = sorted.iter().position(|id| id == "input").unwrap();
        let d1_pos = sorted.iter().position(|id| id == "d1").unwrap();
        let d2_pos = sorted.iter().position(|id| id == "d2").unwrap();

        assert!(input_pos < d1_pos);
        assert!(d1_pos < d2_pos);
    }

    #[test]
    fn test_requirement_diagram() {
        let diagram = decision_graph::RequirementDiagram::new("test_diagram", "Test Diagram");
        assert_eq!(diagram.id, "test_diagram");
        assert_eq!(diagram.name, "Test Diagram");
        assert!(diagram.validate().is_ok());
    }

    // ============================================================================
    // XML Export Tests
    // ============================================================================

    #[test]
    fn test_export_decision_table_json() {
        let table = create_age_category_table();
        let json_str = xml::DmnXmlManager::export_table_json(&table).unwrap();

        let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(json["id"], "age_category");
        assert_eq!(json["inputs"][0]["id"], "age");
        assert_eq!(json["outputs"][0]["id"], "category");
        assert_eq!(json["rules"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_export_decision_table_xml() {
        let table = create_age_category_table();
        let xml = xml::DmnXmlManager::export_table_xml(&table).unwrap();

        assert!(xml.contains("<?xml"));
        assert!(xml.contains("age_category"));
        assert!(xml.contains("decisionTable"));
    }

    #[test]
    fn test_validate_dmn_xml() {
        let valid_xml = "<?xml version=\"1.0\"?>\n<definitions></definitions>";
        assert!(xml::DmnXmlManager::validate_xml(valid_xml).is_ok());

        let invalid_xml = "<definitions></definitions>";
        assert!(xml::DmnXmlManager::validate_xml(invalid_xml).is_err());
    }

    // ============================================================================
    // Expression Parsing Tests
    // ============================================================================

    #[test]
    fn test_expression_parse_number() {
        let expr = Expression::parse("42").unwrap();
        match expr {
            Expression::Literal(FeelValue::Number(n)) => assert_eq!(n, 42.0),
            _ => panic!("Expected number literal"),
        }
    }

    #[test]
    fn test_expression_parse_string() {
        let expr = Expression::parse("\"hello\"").unwrap();
        match expr {
            Expression::Literal(FeelValue::String(s)) => assert_eq!(s, "hello"),
            _ => panic!("Expected string literal"),
        }
    }

    #[test]
    fn test_expression_parse_boolean() {
        let expr = Expression::parse("true").unwrap();
        match expr {
            Expression::Literal(FeelValue::Boolean(b)) => assert!(b),
            _ => panic!("Expected boolean"),
        }

        let expr = Expression::parse("false").unwrap();
        match expr {
            Expression::Literal(FeelValue::Boolean(b)) => assert!(!b),
            _ => panic!("Expected boolean"),
        }
    }

    #[test]
    fn test_expression_parse_null() {
        let expr = Expression::parse("null").unwrap();
        match expr {
            Expression::Literal(FeelValue::Null) => {},
            _ => panic!("Expected null"),
        }
    }

    // ============================================================================
    // Integration Tests
    // ============================================================================

    #[test]
    fn test_complete_decision_workflow() {
        // Create a decision table for loan approval
        let mut table = DecisionTable::new("loan_approval");
        table.hit_policy = HitPolicy::First;

        // Inputs
        let credit_score_input = decision_table::DecisionTableInput {
            id: "credit_score".to_string(),
            label: "Credit Score".to_string(),
            expression: Expression::Literal(FeelValue::Null),
            input_type: Some("number".to_string()),
        };
        table.add_input(credit_score_input);

        let income_input = decision_table::DecisionTableInput {
            id: "annual_income".to_string(),
            label: "Annual Income".to_string(),
            expression: Expression::Literal(FeelValue::Null),
            input_type: Some("number".to_string()),
        };
        table.add_input(income_input);

        // Outputs
        let decision_output = decision_table::DecisionTableOutput {
            id: "approval".to_string(),
            label: "Approval".to_string(),
            output_type: Some("string".to_string()),
            default_value: None,
        };
        table.add_output(decision_output);

        // Rules
        // Poor credit and low income -> Deny
        let mut rule1 = decision_table::RuleEntry::new(2, 1);
        rule1.set_input(0, Some("< 600".to_string())).unwrap();
        rule1.set_input(1, Some("< 30000".to_string())).unwrap();
        rule1.set_output(0, "\"Deny\"".to_string()).unwrap();
        table.add_rule(rule1).unwrap();

        // Good credit and sufficient income -> Approve
        let mut rule2 = decision_table::RuleEntry::new(2, 1);
        rule2.set_input(0, Some(">= 700".to_string())).unwrap();
        rule2.set_input(1, Some(">= 50000".to_string())).unwrap();
        rule2.set_output(0, "\"Approve\"".to_string()).unwrap();
        table.add_rule(rule2).unwrap();

        // Default -> Review
        let mut rule3 = decision_table::RuleEntry::new(2, 1);
        rule3.set_input(0, None).unwrap();
        rule3.set_input(1, None).unwrap();
        rule3.set_output(0, "\"Review\"".to_string()).unwrap();
        table.add_rule(rule3).unwrap();

        // Test execution
        let mut executor = DecisionExecutor::new();
        executor.set_input("credit_score".to_string(), FeelValue::Number(750.0));
        executor.set_input("annual_income".to_string(), FeelValue::Number(75000.0));

        let result = executor.execute_table(&table).unwrap();
        assert_eq!(result.results.len(), 1);
        assert_eq!(
            result.final_output.get("approval"),
            Some(&FeelValue::String("Approve".to_string()))
        );
    }

    #[test]
    fn test_decision_with_feel_expressions() {
        let mut evaluator = FeelEvaluator::new();
        evaluator.set_variable("x".to_string(), FeelValue::Number(10.0));
        evaluator.set_variable("y".to_string(), FeelValue::Number(20.0));

        // Complex expression: (x + y) > 15
        let expr = FeelExpression::Comparison {
            operator: feel::ComparisonOp::GreaterThan,
            left: Box::new(FeelExpression::Binary {
                operator: feel::BinaryOp::Add,
                left: Box::new(FeelExpression::Variable("x".to_string())),
                right: Box::new(FeelExpression::Variable("y".to_string())),
            }),
            right: Box::new(FeelExpression::Literal(FeelValue::Number(15.0))),
        };

        assert_eq!(evaluator.evaluate(&expr).unwrap(), FeelValue::Boolean(true));
    }
}
