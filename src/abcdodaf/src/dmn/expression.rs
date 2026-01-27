//! DMN Expression definitions and evaluation
//!
//! This module provides a unified expression interface that can wrap
//! FEEL expressions and other expression languages.

use crate::dmn::feel::{FeelExpression, FeelValue};
use crate::dmn::errors::{DmnError, DmnResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// DMN Expression variants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression {
    /// FEEL expression
    Feel(FeelExpression),
    /// Literal value
    Literal(FeelValue),
    /// Simple comparison shorthand (used in decision table inputs)
    /// e.g., "< 100", ">= 18", "in range [1..10]"
    Comparison(String),
}

impl Expression {
    /// Parse a string expression into an Expression
    pub fn parse(input: &str) -> DmnResult<Self> {
        let input = input.trim();

        // Try to parse as a number
        if let Ok(n) = input.parse::<f64>() {
            return Ok(Expression::Literal(FeelValue::Number(n)));
        }

        // Check if it's a quoted string
        if input.starts_with('"') && input.ends_with('"') {
            return Ok(Expression::Literal(FeelValue::String(
                input[1..input.len() - 1].to_string(),
            )));
        }

        // Check for boolean
        if input == "true" || input == "false" {
            return Ok(Expression::Literal(FeelValue::Boolean(input == "true")));
        }

        // Check for null
        if input == "null" {
            return Ok(Expression::Literal(FeelValue::Null));
        }

        // For now, treat as a comparison expression for decision table inputs
        // This will be extended with full FEEL parsing
        Ok(Expression::Comparison(input.to_string()))
    }
}

/// Expression evaluator
pub struct ExpressionEvaluator {
    context: HashMap<String, FeelValue>,
}

impl ExpressionEvaluator {
    /// Create a new evaluator
    pub fn new() -> Self {
        Self {
            context: HashMap::new(),
        }
    }

    /// Create with initial context
    pub fn with_context(context: HashMap<String, FeelValue>) -> Self {
        Self { context }
    }

    /// Set a context variable
    pub fn set_variable(&mut self, name: String, value: FeelValue) {
        self.context.insert(name, value);
    }

    /// Set multiple variables from a map
    pub fn set_variables(&mut self, vars: HashMap<String, FeelValue>) {
        self.context.extend(vars);
    }

    /// Get a variable
    pub fn get_variable(&self, name: &str) -> Option<&FeelValue> {
        self.context.get(name)
    }

    /// Evaluate an expression
    pub fn evaluate(&self, expr: &Expression) -> DmnResult<FeelValue> {
        match expr {
            Expression::Literal(val) => Ok(val.clone()),

            Expression::Feel(feel_expr) => {
                let evaluator = crate::dmn::feel::FeelEvaluator::with_context(
                    self.context.clone(),
                );
                evaluator.evaluate(feel_expr)
            }

            Expression::Comparison(comp) => {
                self.evaluate_comparison(comp)
            }
        }
    }

    /// Evaluate a comparison expression for decision table inputs
    fn evaluate_comparison(&self, comp: &str) -> DmnResult<FeelValue> {
        let comp = comp.trim();

        // Handle range expressions: "1..10" or "[1..10)"
        if comp.contains("..") {
            return self.evaluate_range(comp);
        }

        // Handle comparison operators
        for op in &["<=", ">=", "!=", "<", ">", "="] {
            if comp.starts_with(op) {
                let value_str = comp[op.len()..].trim();
                let comparison_expr = self.create_comparison_expr(*op, value_str)?;
                return self.evaluate(&Expression::Feel(comparison_expr));
            }
        }

        // Handle "in" operator
        if comp.starts_with("in") {
            return self.evaluate_in_expression(comp);
        }

        // Default: treat as a variable reference
        Ok(FeelValue::String(comp.to_string()))
    }

    fn create_comparison_expr(
        &self,
        operator: &str,
        value_str: &str,
    ) -> DmnResult<FeelExpression> {
        use crate::dmn::feel::{ComparisonOp, FeelExpression};

        let op = match operator {
            "=" | "==" => ComparisonOp::Equals,
            "!=" => ComparisonOp::NotEquals,
            "<" => ComparisonOp::LessThan,
            "<=" => ComparisonOp::LessEqual,
            ">" => ComparisonOp::GreaterThan,
            ">=" => ComparisonOp::GreaterEqual,
            _ => return Err(DmnError::OperationError(
                format!("Unknown operator: {}", operator)
            )),
        };

        // Parse the right-hand side value
        let rhs = Expression::parse(value_str)?;
        let rhs_feel = match rhs {
            Expression::Literal(val) => FeelExpression::Literal(val),
            Expression::Comparison(comp) => {
                return Err(DmnError::FeelParsingError(format!(
                    "Cannot parse nested comparison: {}",
                    comp
                )))
            }
            Expression::Feel(expr) => expr,
        };

        Ok(FeelExpression::Comparison {
            operator: op,
            left: Box::new(FeelExpression::Variable("input".to_string())),
            right: Box::new(rhs_feel),
        })
    }

    fn evaluate_range(&self, range: &str) -> DmnResult<FeelValue> {
        // Parse ranges like "1..10" or "[1..10]" or "(1..10)" or "[1..10)"
        // For now, return a string representation
        // Full implementation would return a Range value
        Ok(FeelValue::String(range.to_string()))
    }

    fn evaluate_in_expression(&self, expr: &str) -> DmnResult<FeelValue> {
        // Parse expressions like "in (1, 2, 3)" or "in [1..10]"
        Ok(FeelValue::String(expr.to_string()))
    }
}

impl Default for ExpressionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        let expr = Expression::parse("42").unwrap();
        match expr {
            Expression::Literal(FeelValue::Number(n)) => assert_eq!(n, 42.0),
            _ => panic!("Expected number literal"),
        }
    }

    #[test]
    fn test_parse_string() {
        let expr = Expression::parse("\"hello\"").unwrap();
        match expr {
            Expression::Literal(FeelValue::String(s)) => assert_eq!(s, "hello"),
            _ => panic!("Expected string literal"),
        }
    }

    #[test]
    fn test_parse_boolean() {
        let expr = Expression::parse("true").unwrap();
        match expr {
            Expression::Literal(FeelValue::Boolean(b)) => assert!(b),
            _ => panic!("Expected boolean literal"),
        }
    }

    #[test]
    fn test_parse_null() {
        let expr = Expression::parse("null").unwrap();
        match expr {
            Expression::Literal(FeelValue::Null) => {},
            _ => panic!("Expected null literal"),
        }
    }

    #[test]
    fn test_evaluator_with_context() {
        let mut eval = ExpressionEvaluator::new();
        eval.set_variable("x".to_string(), FeelValue::Number(42.0));

        let val = eval.get_variable("x");
        assert_eq!(val, Some(&FeelValue::Number(42.0)));
    }
}
