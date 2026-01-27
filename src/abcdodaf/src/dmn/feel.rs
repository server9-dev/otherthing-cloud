//! FEEL (Friendly Enough Expression Language) implementation
//!
//! FEEL is a simple, friendly expression language defined in DMN specification.
//! This module implements a parser and evaluator for FEEL expressions.
//!
//! ## Supported Features
//!
//! - Literals: numbers, strings, booleans, null, dates, times
//! - Variables: reference context values
//! - Comparisons: =, !=, <, >, <=, >=
//! - Arithmetic: +, -, *, /, **
//! - Logic: and, or, not
//! - Functions: built-in FEEL functions
//! - Lists and contexts (basic)

use crate::dmn::errors::{DmnError, DmnResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// FEEL value types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FeelValue {
    /// Null value
    Null,
    /// Boolean value
    Boolean(bool),
    /// Numeric value (stored as f64 for simplicity)
    Number(f64),
    /// String value
    String(String),
    /// Date value (YYYY-MM-DD format)
    Date(String),
    /// Time value (HH:MM:SS format)
    Time(String),
    /// DateTime value
    DateTime(String),
    /// List of values
    List(Vec<FeelValue>),
    /// Context (map) of key-value pairs
    Context(HashMap<String, FeelValue>),
}

impl FeelValue {
    /// Convert to boolean
    pub fn to_bool(&self) -> DmnResult<bool> {
        match self {
            FeelValue::Boolean(b) => Ok(*b),
            FeelValue::Number(n) => Ok(*n != 0.0),
            FeelValue::String(s) => Ok(!s.is_empty()),
            FeelValue::Null => Ok(false),
            _ => Err(DmnError::TypeMismatch {
                expected: "Boolean".to_string(),
                actual: self.type_name().to_string(),
            }),
        }
    }

    /// Convert to number
    pub fn to_number(&self) -> DmnResult<f64> {
        match self {
            FeelValue::Number(n) => Ok(*n),
            FeelValue::String(s) => s.parse::<f64>().map_err(|_| {
                DmnError::TypeMismatch {
                    expected: "Number".to_string(),
                    actual: format!("String({})", s),
                }
            }),
            FeelValue::Boolean(b) => Ok(if *b { 1.0 } else { 0.0 }),
            _ => Err(DmnError::TypeMismatch {
                expected: "Number".to_string(),
                actual: self.type_name().to_string(),
            }),
        }
    }

    /// Convert to string
    pub fn to_string(&self) -> String {
        match self {
            FeelValue::Null => "null".to_string(),
            FeelValue::Boolean(b) => b.to_string(),
            FeelValue::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{:.0}", n)
                } else {
                    n.to_string()
                }
            }
            FeelValue::String(s) => s.clone(),
            FeelValue::Date(d) => d.clone(),
            FeelValue::Time(t) => t.clone(),
            FeelValue::DateTime(dt) => dt.clone(),
            FeelValue::List(items) => {
                let items_str = items
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{}]", items_str)
            }
            FeelValue::Context(map) => {
                let items_str = map
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_string()))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{{}}}", items_str)
            }
        }
    }

    /// Get type name for error messages
    pub fn type_name(&self) -> &str {
        match self {
            FeelValue::Null => "Null",
            FeelValue::Boolean(_) => "Boolean",
            FeelValue::Number(_) => "Number",
            FeelValue::String(_) => "String",
            FeelValue::Date(_) => "Date",
            FeelValue::Time(_) => "Time",
            FeelValue::DateTime(_) => "DateTime",
            FeelValue::List(_) => "List",
            FeelValue::Context(_) => "Context",
        }
    }

    /// Check equality with FEEL semantics
    pub fn equals(&self, other: &FeelValue) -> DmnResult<bool> {
        match (self, other) {
            (FeelValue::Null, FeelValue::Null) => Ok(true),
            (FeelValue::Boolean(a), FeelValue::Boolean(b)) => Ok(a == b),
            (FeelValue::Number(a), FeelValue::Number(b)) => Ok((a - b).abs() < 1e-10),
            (FeelValue::String(a), FeelValue::String(b)) => Ok(a == b),
            (FeelValue::Date(a), FeelValue::Date(b)) => Ok(a == b),
            _ => {
                // Try numeric comparison if both can be converted
                if let (Ok(a), Ok(b)) = (self.to_number(), other.to_number()) {
                    return Ok((a - b).abs() < 1e-10);
                }
                Ok(false)
            }
        }
    }
}

impl Display for FeelValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

use std::fmt::Display;

/// FEEL expression AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FeelExpression {
    /// Literal value
    Literal(FeelValue),
    /// Variable reference
    Variable(String),
    /// Unary operation
    Unary {
        operator: UnaryOp,
        operand: Box<FeelExpression>,
    },
    /// Binary operation
    Binary {
        operator: BinaryOp,
        left: Box<FeelExpression>,
        right: Box<FeelExpression>,
    },
    /// Comparison operation
    Comparison {
        operator: ComparisonOp,
        left: Box<FeelExpression>,
        right: Box<FeelExpression>,
    },
    /// Function call
    FunctionCall {
        name: String,
        arguments: Vec<FeelExpression>,
    },
    /// List construction
    List(Vec<FeelExpression>),
    /// Context construction
    Context(Vec<(String, FeelExpression)>),
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    /// Logical NOT
    Not,
    /// Negation
    Negate,
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    /// Addition
    Add,
    /// Subtraction
    Subtract,
    /// Multiplication
    Multiply,
    /// Division
    Divide,
    /// Power/Exponentiation
    Power,
    /// Logical AND
    And,
    /// Logical OR
    Or,
}

/// Comparison operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonOp {
    /// Equality
    Equals,
    /// Inequality
    NotEquals,
    /// Less than
    LessThan,
    /// Less than or equal
    LessEqual,
    /// Greater than
    GreaterThan,
    /// Greater than or equal
    GreaterEqual,
}

/// FEEL expression evaluator
pub struct FeelEvaluator {
    context: HashMap<String, FeelValue>,
}

impl FeelEvaluator {
    /// Create a new evaluator with empty context
    pub fn new() -> Self {
        Self {
            context: HashMap::new(),
        }
    }

    /// Create evaluator with initial context
    pub fn with_context(context: HashMap<String, FeelValue>) -> Self {
        Self { context }
    }

    /// Set a context variable
    pub fn set_variable(&mut self, name: String, value: FeelValue) {
        self.context.insert(name, value);
    }

    /// Get a context variable
    pub fn get_variable(&self, name: &str) -> Option<&FeelValue> {
        self.context.get(name)
    }

    /// Evaluate a FEEL expression
    pub fn evaluate(&self, expr: &FeelExpression) -> DmnResult<FeelValue> {
        match expr {
            FeelExpression::Literal(val) => Ok(val.clone()),

            FeelExpression::Variable(name) => {
                self.context
                    .get(name)
                    .cloned()
                    .ok_or_else(|| DmnError::MissingInput(name.clone()))
            }

            FeelExpression::Unary { operator, operand } => {
                let val = self.evaluate(operand)?;
                self.evaluate_unary(*operator, val)
            }

            FeelExpression::Binary { operator, left, right } => {
                let left_val = self.evaluate(left)?;
                let right_val = self.evaluate(right)?;
                self.evaluate_binary(*operator, left_val, right_val)
            }

            FeelExpression::Comparison { operator, left, right } => {
                let left_val = self.evaluate(left)?;
                let right_val = self.evaluate(right)?;
                let result = self.evaluate_comparison(*operator, &left_val, &right_val)?;
                Ok(FeelValue::Boolean(result))
            }

            FeelExpression::FunctionCall { name, arguments } => {
                let args: DmnResult<Vec<_>> =
                    arguments.iter().map(|arg| self.evaluate(arg)).collect();
                self.evaluate_function(name, args?)
            }

            FeelExpression::List(items) => {
                let values: DmnResult<Vec<_>> =
                    items.iter().map(|item| self.evaluate(item)).collect();
                Ok(FeelValue::List(values?))
            }

            FeelExpression::Context(items) => {
                let mut map = HashMap::new();
                for (key, expr) in items {
                    let value = self.evaluate(expr)?;
                    map.insert(key.clone(), value);
                }
                Ok(FeelValue::Context(map))
            }
        }
    }

    fn evaluate_unary(&self, op: UnaryOp, val: FeelValue) -> DmnResult<FeelValue> {
        match op {
            UnaryOp::Not => Ok(FeelValue::Boolean(!val.to_bool()?)),
            UnaryOp::Negate => Ok(FeelValue::Number(-val.to_number()?)),
        }
    }

    fn evaluate_binary(&self, op: BinaryOp, left: FeelValue, right: FeelValue) -> DmnResult<FeelValue> {
        match op {
            BinaryOp::Add => {
                let l = left.to_number()?;
                let r = right.to_number()?;
                Ok(FeelValue::Number(l + r))
            }
            BinaryOp::Subtract => {
                let l = left.to_number()?;
                let r = right.to_number()?;
                Ok(FeelValue::Number(l - r))
            }
            BinaryOp::Multiply => {
                let l = left.to_number()?;
                let r = right.to_number()?;
                Ok(FeelValue::Number(l * r))
            }
            BinaryOp::Divide => {
                let l = left.to_number()?;
                let r = right.to_number()?;
                if r == 0.0 {
                    return Err(DmnError::OperationError(
                        "Division by zero".to_string(),
                    ));
                }
                Ok(FeelValue::Number(l / r))
            }
            BinaryOp::Power => {
                let l = left.to_number()?;
                let r = right.to_number()?;
                Ok(FeelValue::Number(l.powf(r)))
            }
            BinaryOp::And => {
                let l = left.to_bool()?;
                let r = right.to_bool()?;
                Ok(FeelValue::Boolean(l && r))
            }
            BinaryOp::Or => {
                let l = left.to_bool()?;
                let r = right.to_bool()?;
                Ok(FeelValue::Boolean(l || r))
            }
        }
    }

    fn evaluate_comparison(
        &self,
        op: ComparisonOp,
        left: &FeelValue,
        right: &FeelValue,
    ) -> DmnResult<bool> {
        match op {
            ComparisonOp::Equals => left.equals(right),
            ComparisonOp::NotEquals => Ok(!left.equals(right)?),
            ComparisonOp::LessThan => {
                let l = left.to_number()?;
                let r = right.to_number()?;
                Ok(l < r)
            }
            ComparisonOp::LessEqual => {
                let l = left.to_number()?;
                let r = right.to_number()?;
                Ok(l <= r)
            }
            ComparisonOp::GreaterThan => {
                let l = left.to_number()?;
                let r = right.to_number()?;
                Ok(l > r)
            }
            ComparisonOp::GreaterEqual => {
                let l = left.to_number()?;
                let r = right.to_number()?;
                Ok(l >= r)
            }
        }
    }

    fn evaluate_function(&self, name: &str, args: Vec<FeelValue>) -> DmnResult<FeelValue> {
        match name {
            // List functions
            "count" => {
                if args.len() != 1 {
                    return Err(DmnError::OperationError(
                        "count expects 1 argument".to_string(),
                    ));
                }
                match &args[0] {
                    FeelValue::List(items) => Ok(FeelValue::Number(items.len() as f64)),
                    _ => Err(DmnError::TypeMismatch {
                        expected: "List".to_string(),
                        actual: args[0].type_name().to_string(),
                    }),
                }
            }

            // Numeric functions
            "abs" => {
                if args.len() != 1 {
                    return Err(DmnError::OperationError(
                        "abs expects 1 argument".to_string(),
                    ));
                }
                let n = args[0].to_number()?;
                Ok(FeelValue::Number(n.abs()))
            }

            "max" => {
                if args.is_empty() {
                    return Err(DmnError::OperationError(
                        "max expects at least 1 argument".to_string(),
                    ));
                }
                let numbers: DmnResult<Vec<f64>> = args.iter().map(|a| a.to_number()).collect();
                let nums = numbers?;
                let max = nums
                    .into_iter()
                    .fold(f64::NEG_INFINITY, f64::max);
                Ok(FeelValue::Number(max))
            }

            "min" => {
                if args.is_empty() {
                    return Err(DmnError::OperationError(
                        "min expects at least 1 argument".to_string(),
                    ));
                }
                let numbers: DmnResult<Vec<f64>> = args.iter().map(|a| a.to_number()).collect();
                let nums = numbers?;
                let min = nums.into_iter().fold(f64::INFINITY, f64::min);
                Ok(FeelValue::Number(min))
            }

            // String functions
            "substring" => {
                if args.len() < 2 || args.len() > 3 {
                    return Err(DmnError::OperationError(
                        "substring expects 2 or 3 arguments".to_string(),
                    ));
                }
                let s = args[0].to_string();
                let start = args[1].to_number()? as usize;
                let length = if args.len() == 3 {
                    args[2].to_number()? as usize
                } else {
                    s.len() - start + 1
                };

                let result = s
                    .chars()
                    .skip(start - 1)
                    .take(length)
                    .collect::<String>();
                Ok(FeelValue::String(result))
            }

            "upper case" | "uppercase" => {
                if args.len() != 1 {
                    return Err(DmnError::OperationError(
                        "upper case expects 1 argument".to_string(),
                    ));
                }
                let s = args[0].to_string();
                Ok(FeelValue::String(s.to_uppercase()))
            }

            "lower case" | "lowercase" => {
                if args.len() != 1 {
                    return Err(DmnError::OperationError(
                        "lower case expects 1 argument".to_string(),
                    ));
                }
                let s = args[0].to_string();
                Ok(FeelValue::String(s.to_lowercase()))
            }

            _ => Err(DmnError::OperationError(format!(
                "Unknown function: {}",
                name
            ))),
        }
    }
}

impl Default for FeelEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feel_value_conversion() {
        let num = FeelValue::Number(42.0);
        assert_eq!(num.to_number().unwrap(), 42.0);
        assert_eq!(num.to_bool().unwrap(), true);
        assert_eq!(num.to_string(), "42");
    }

    #[test]
    fn test_feel_value_equality() {
        let a = FeelValue::Number(42.0);
        let b = FeelValue::Number(42.0);
        assert!(a.equals(&b).unwrap());
    }

    #[test]
    fn test_simple_literal_evaluation() {
        let evaluator = FeelEvaluator::new();
        let expr = FeelExpression::Literal(FeelValue::Number(42.0));
        let result = evaluator.evaluate(&expr).unwrap();
        assert_eq!(result, FeelValue::Number(42.0));
    }

    #[test]
    fn test_unary_not() {
        let evaluator = FeelEvaluator::new();
        let expr = FeelExpression::Unary {
            operator: UnaryOp::Not,
            operand: Box::new(FeelExpression::Literal(FeelValue::Boolean(true))),
        };
        let result = evaluator.evaluate(&expr).unwrap();
        assert_eq!(result, FeelValue::Boolean(false));
    }

    #[test]
    fn test_binary_arithmetic() {
        let evaluator = FeelEvaluator::new();
        let expr = FeelExpression::Binary {
            operator: BinaryOp::Add,
            left: Box::new(FeelExpression::Literal(FeelValue::Number(10.0))),
            right: Box::new(FeelExpression::Literal(FeelValue::Number(32.0))),
        };
        let result = evaluator.evaluate(&expr).unwrap();
        assert_eq!(result, FeelValue::Number(42.0));
    }

    #[test]
    fn test_comparison() {
        let evaluator = FeelEvaluator::new();
        let expr = FeelExpression::Comparison {
            operator: ComparisonOp::GreaterThan,
            left: Box::new(FeelExpression::Literal(FeelValue::Number(10.0))),
            right: Box::new(FeelExpression::Literal(FeelValue::Number(5.0))),
        };
        let result = evaluator.evaluate(&expr).unwrap();
        assert_eq!(result, FeelValue::Boolean(true));
    }

    #[test]
    fn test_variable_resolution() {
        let mut evaluator = FeelEvaluator::new();
        evaluator.set_variable("x".to_string(), FeelValue::Number(42.0));

        let expr = FeelExpression::Variable("x".to_string());
        let result = evaluator.evaluate(&expr).unwrap();
        assert_eq!(result, FeelValue::Number(42.0));
    }

    #[test]
    fn test_function_abs() {
        let evaluator = FeelEvaluator::new();
        let expr = FeelExpression::FunctionCall {
            name: "abs".to_string(),
            arguments: vec![FeelExpression::Literal(FeelValue::Number(-42.0))],
        };
        let result = evaluator.evaluate(&expr).unwrap();
        assert_eq!(result, FeelValue::Number(42.0));
    }

    #[test]
    fn test_function_count() {
        let evaluator = FeelEvaluator::new();
        let expr = FeelExpression::FunctionCall {
            name: "count".to_string(),
            arguments: vec![FeelExpression::List(vec![
                FeelExpression::Literal(FeelValue::Number(1.0)),
                FeelExpression::Literal(FeelValue::Number(2.0)),
                FeelExpression::Literal(FeelValue::Number(3.0)),
            ])],
        };
        let result = evaluator.evaluate(&expr).unwrap();
        assert_eq!(result, FeelValue::Number(3.0));
    }

    #[test]
    fn test_list_evaluation() {
        let evaluator = FeelEvaluator::new();
        let expr = FeelExpression::List(vec![
            FeelExpression::Literal(FeelValue::Number(1.0)),
            FeelExpression::Literal(FeelValue::Number(2.0)),
        ]);
        let result = evaluator.evaluate(&expr).unwrap();
        match result {
            FeelValue::List(items) => assert_eq!(items.len(), 2),
            _ => panic!("Expected list"),
        }
    }
}
