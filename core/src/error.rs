use crate::value::Op;

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("stack underflow")]
    StackUnderflow,
    #[error("invalid opcode")]
    InvalidOpcode,
    #[error("invalid constant index {0}")]
    InvalidConstantIndex(usize),
    #[error("stack index {0} is out of bound")]
    StackIndexOutOfBound(usize),
    #[error("type error: {0}")]
    TypeError(String),
    #[error("division by zero")]
    DivisionByZero,
    #[error("invalid operation '{0:?}' on string")]
    InvalidStringOperation(Op),
    #[error("undefined variable '{0:?}'")]
    UndefinedVariable(String),
    #[error("undefined native '{0:?}'")]
    UndefinedNative(String),
    #[error("No call frame in VM")]
    NoCallFrame,
    #[error("ip is out of bounds")]
    OutOfBoundsIp,
    #[error("wrong number of arguments: expected {0}, got {1}")]
    WrongArgCount(usize, usize),
    #[error("no contract definition found for the given address")]
    ContractNotFound,
}
