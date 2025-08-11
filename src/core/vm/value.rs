use crate::core::ast::Literal;

use super::RuntimeError;

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Neg,

    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Neq,

    And,
    Or,
}

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < f64::EPSILON
}
impl Value {
    pub fn try_apply(&self, op: Op, rhs: Option<&Value>) -> Result<Value, RuntimeError> {
        match (self, rhs, op) {
            (Value::Number(lhs), Some(Value::Number(rhs)), Op::Add) => Ok(Value::Number(lhs + rhs)),
            (Value::Number(lhs), Some(Value::Number(rhs)), Op::Sub) => Ok(Value::Number(lhs - rhs)),
            (Value::Number(lhs), Some(Value::Number(rhs)), Op::Mul) => Ok(Value::Number(lhs * rhs)),
            (Value::Number(lhs), Some(Value::Number(rhs)), Op::Div) => {
                if rhs.abs() <= f64::EPSILON {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Number(lhs / rhs))
                }
            }
            (Value::Number(lhs), Some(Value::Number(rhs)), Op::Mod) => Ok(Value::Number(lhs % rhs)),
            (Value::Number(lhs), Some(Value::Number(rhs)), Op::Lt) => Ok(Value::Bool(*lhs < *rhs)),
            (Value::Number(lhs), Some(Value::Number(rhs)), Op::Le) => Ok(Value::Bool(*lhs <= *rhs)),
            (Value::Number(lhs), Some(Value::Number(rhs)), Op::Gt) => Ok(Value::Bool(*lhs > *rhs)),
            (Value::Number(lhs), Some(Value::Number(rhs)), Op::Ge) => Ok(Value::Bool(*lhs >= *rhs)),
            (Value::Number(lhs), Some(Value::Number(rhs)), Op::Eq) => {
                Ok(Value::Bool(approx_eq(*lhs, *rhs)))
            }
            (Value::Number(lhs), Some(Value::Number(rhs)), Op::Neq) => {
                Ok(Value::Bool(!approx_eq(*lhs, *rhs)))
            }

            (Value::String(lhs), Some(Value::String(rhs)), Op::Add) => {
                Ok(Value::String(lhs.to_string() + rhs))
            }
            (Value::String(lhs), Some(Value::String(rhs)), Op::Lt) => {
                Ok(Value::Bool(lhs.len() < rhs.len()))
            }
            (Value::String(lhs), Some(Value::String(rhs)), Op::Le) => {
                Ok(Value::Bool(lhs.len() <= rhs.len()))
            }
            (Value::String(lhs), Some(Value::String(rhs)), Op::Gt) => {
                Ok(Value::Bool(lhs.len() > rhs.len()))
            }
            (Value::String(lhs), Some(Value::String(rhs)), Op::Ge) => {
                Ok(Value::Bool(lhs.len() >= rhs.len()))
            }
            (Value::String(_), Some(Value::String(_)), op) => {
                Err(RuntimeError::InvalidStringOperation(op))
            }

            (Value::Bool(lhs), Some(Value::Bool(rhs)), Op::Eq) => Ok(Value::Bool(*lhs == *rhs)),
            (Value::Bool(lhs), Some(Value::Bool(rhs)), Op::Neq) => Ok(Value::Bool(*lhs != *rhs)),

            (lhs, Some(rhs), Op::And) => Ok(Value::Bool(lhs.is_truthy() && rhs.is_truthy())),
            (lhs, Some(rhs), Op::Or) => Ok(Value::Bool(lhs.is_truthy() || rhs.is_truthy())),

            (Value::Number(lhs), None, Op::Neg) => Ok(Value::Number(-lhs)),

            _ => Err(RuntimeError::TypeMismatch),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Number(n) => n.abs() >= f64::EPSILON,
            Value::String(s) => !s.is_empty(),
            Value::Bool(b) => *b,
            Value::Null => false,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => (a - b).abs() < f64::EPSILON,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }
}

impl From<&Literal> for Value {
    fn from(literal: &Literal) -> Self {
        match literal {
            Literal::Number(n) => Value::Number(*n),
            Literal::String(s) => Value::String(s.to_string()),
            Literal::Bool(b) => Value::Bool(*b),
            Literal::Null => Value::Null,
        }
    }
}
