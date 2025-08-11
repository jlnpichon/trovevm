use crate::cli::input::{InputError, InputSource};
use crate::core::{self, ast::Program, parser::error::ParseErrorWithContext};

#[derive(Debug, thiserror::Error)]
pub enum ParseCommandError {
    #[error(transparent)]
    Input(#[from] InputError),
    #[error(transparent)]
    Parse(#[from] Box<ParseErrorWithContext>),
}

pub fn parse(input: InputSource) -> Result<Program, ParseCommandError> {
    let source = input.read_to_string()?;
    core::parser::parse_program(input.source_name(), &source).map_err(Into::into)
}
