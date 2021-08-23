use super::parser::ParseError;

pub enum CompilationError {
    InvaliToken(),
    InvalidSyntax(ParseError),
}

impl From<ParseError> for CompilationError {
    fn from(e: ParseError) -> Self {
        CompilationError::InvalidSyntax(e)
    }
}
