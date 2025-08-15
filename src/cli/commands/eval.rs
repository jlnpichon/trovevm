use anyhow::Result;

use crate::cli::input::{InputError, InputSource};
use crate::core::vm::{RuntimeError, Value};
use crate::core::{self, codegen::CompileError, parser::error::ParseErrorWithContext};

#[derive(Debug, thiserror::Error)]
pub enum EvalError {
    #[error(transparent)]
    Input(#[from] InputError),
    #[error(transparent)]
    Parse(#[from] Box<ParseErrorWithContext>),
    #[error(transparent)]
    Compile(#[from] Box<CompileError>),
    #[error(transparent)]
    Runtime(#[from] Box<RuntimeError>),
}

pub fn eval(input: InputSource) -> Result<Value, EvalError> {
    todo!()
}
