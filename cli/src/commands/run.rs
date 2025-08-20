use anyhow::Result;
use trove_core::{RuntimeError, Value};
use trovec::{CompileError, ParseErrorWithContext};

use crate::{InputSource, input::InputError};

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
    let mut ast =
        trovec::parser::parse_program(input.source_name(), &source).map_err(RunError::from)?;
    let compiled_program =
        trovec::codegen::compile(&mut ast.statements).map_err(|e| RunError::Compile(e.into()))?;

    let mut vm = trove_vm::VM::default();
    vm.run(compiled_program.script)
        .map_err(|e| RunError::Runtime(e.into()))
}
