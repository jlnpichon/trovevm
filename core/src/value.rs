use std::{collections::HashMap, fmt};

use serde::{Deserialize, Serialize};

use super::{contract::ContractInstance, function::CompiledFunction};
use crate::{contract::CompiledContract, error::RuntimeError};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
    Function(CompiledFunction),
    ContractInstance(ContractInstance),
    ContractDef(CompiledContract),
    Map(HashMap<String, Value>),
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
        match (self, rhs, op.clone()) {
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

            _ => Err(RuntimeError::TypeError(format!(
                "Cant {op:?} {self:?} and {rhs:?}"
            ))),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Number(n) => n.abs() >= f64::EPSILON,
            Value::String(s) => !s.is_empty(),
            Value::Bool(b) => *b,
            Value::Function(_) => true,
            Value::ContractInstance(_) => true,
            Value::Map { .. } => true,
            Value::Null => false,
            Value::ContractDef(_) => unreachable!(),
        }
    }

    pub fn as_string(&self) -> Option<&String> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_instance(&self) -> Option<&ContractInstance> {
        match self {
            Value::ContractInstance(instance) => Some(instance),
            _ => None,
        }
    }

    pub fn as_instance_mut(&mut self) -> Option<&mut ContractInstance> {
        match self {
            Value::ContractInstance(instance) => Some(instance),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Value::Map(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_map_mut(&mut self) -> Option<&mut HashMap<String, Value>> {
        match self {
            Value::Map(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_function(&self) -> Option<&CompiledFunction> {
        match self {
            Value::Function(f) => Some(f),
            _ => None,
        }
    }

    pub fn to_function(self) -> Option<CompiledFunction> {
        match self {
            Value::Function(f) => Some(f),
            _ => None,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Number(_) => "Number",
            Value::String(_) => "String",
            Value::Bool(_) => "Bool",
            Value::Null => "Null",
            Value::Map(_) => "Map",
            Value::Function(_) => "Function",
            Value::ContractInstance(_) => "ContractInstance",
            Value::ContractDef(_) => "ContractDef",
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

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        if let Ok(n) = value.parse::<f64>() {
            Value::Number(n)
        } else if value.to_lowercase() == "true" {
            Value::Bool(true)
        } else if value.to_lowercase() == "false" {
            Value::Bool(false)
        } else if value.to_lowercase() == "null" {
            Value::Null
        } else {
            Value::String(value.to_string())
        }
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::from(value.as_str())
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{n}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Null => write!(f, "Null"),
            Value::Map(map) => write!(f, "{:?}", map),
            Value::Function(function) => {
                write!(
                    f,
                    "<{} {}/{}>",
                    function.kind, function.name, function.arity
                )
            }
            Value::ContractInstance(instance) => write!(
                f,
                "<instance {}@{:?}>",
                instance.contract.name, instance.address
            ),
            Value::ContractDef(contract) => {
                write!(f, "<contract {}>", contract.name)
            }
        }
    }
}
