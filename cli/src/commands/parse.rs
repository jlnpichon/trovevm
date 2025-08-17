use trovec::ParseErrorWithContext;

use crate::{InputSource, input::InputError};

#[derive(Debug, thiserror::Error)]
pub enum ParseCommandError {
    #[error(transparent)]
    Input(#[from] InputError),
    #[error(transparent)]
    Parse(#[from] Box<ParseErrorWithContext>),
}

pub fn parse(input: InputSource) -> Result<trovec::ast::Program, ParseCommandError> {
    let source = input.read_to_string()?;
    trovec::parser::parse_program(input.source_name(), &source).map_err(Into::into)
}
