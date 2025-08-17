use anyhow::Result;
use trove_core::{RuntimeError, Value};
use trovec::{CompileError, ParseErrorWithContext};

use crate::{InputSource, input::InputError};

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
