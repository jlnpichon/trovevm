use anyhow::Result;

use crate::cli::input::{InputError, InputSource};
use crate::core::vm::{RuntimeError, Value};
use crate::core::{self, codegen::CompileError, parser::error::ParseErrorWithContext};

#[derive(Debug, thiserror::Error)]
pub enum RunError {
    #[error(transparent)]
    Input(#[from] InputError),
    #[error(transparent)]
    Parse(#[from] Box<ParseErrorWithContext>),
    #[error(transparent)]
    Compile(#[from] Box<CompileError>),
    #[error(transparent)]
    Runtime(#[from] Box<RuntimeError>),
}

pub fn run(input: InputSource) -> Result<Value, RunError> {
    let source = input.read_to_string()?;
    let ast = core::parser::parse_program(input.source_name(), &source).map_err(RunError::from)?;
    let function =
        core::codegen::compile(&ast.statements).map_err(|e| RunError::Compile(e.into()))?;

    let mut vm = core::vm::VM::default();
    vm.run(function).map_err(|e| RunError::Runtime(e.into()))
}
