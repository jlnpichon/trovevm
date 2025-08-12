use crate::cli::input::{InputError, InputSource};
use crate::core::vm::Program;
use crate::core::{self, codegen::CompileError, parser::error::ParseErrorWithContext};

#[derive(Debug, thiserror::Error)]
pub enum CompileCommandError {
    #[error(transparent)]
    Input(#[from] InputError),
    #[error(transparent)]
    Parse(#[from] Box<ParseErrorWithContext>),
    #[error(transparent)]
    Compile(#[from] Box<CompileError>),
}

pub fn compile(input: InputSource) -> Result<Program, CompileCommandError> {
    let source = input.read_to_string()?;
    let program = core::parser::parse_program(input.source_name(), &source)
        .map_err(CompileCommandError::from)?;
    core::codegen::compile(&program.statements).map_err(|e| CompileCommandError::Compile(e.into()))
}
