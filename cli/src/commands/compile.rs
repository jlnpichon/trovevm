use trove_core::CompiledFunction;
use trovec::{CompileError, ParseErrorWithContext, codegen::compiler::CompiledProgram};

use crate::{InputSource, input::InputError};

#[derive(Debug, thiserror::Error)]
pub enum CompileCommandError {
    #[error(transparent)]
    Input(#[from] InputError),
    #[error(transparent)]
    Parse(#[from] Box<ParseErrorWithContext>),
    #[error(transparent)]
    Compile(#[from] Box<CompileError>),
}

pub fn compile(input: InputSource) -> Result<CompiledProgram, CompileCommandError> {
    let source = input.read_to_string()?;
    let program = trovec::parser::parse_program(input.source_name(), &source)
        .map_err(CompileCommandError::from)?;
    trovec::codegen::compile(&program.statements)
        .map_err(|e| CompileCommandError::Compile(e.into()))
}
