use anyhow::Result;
use compiler::{converter::ErrorHandler, error::PrettyErrorHandler, parser, tokenizer::{self, TokenizeOption}};
use std::fs::read_to_string;
struct ErrorHandle {

}

impl ErrorHandle {
    fn new() -> Self { Self {  } }
}
impl ErrorHandler for ErrorHandle {
    fn on_error(&self, err: compiler::converter::CompilationError) {
        println!("{}", err);
    }
}
// impl
fn main() -> Result<()> {
    let file = read_to_string("test.vue")?;
    let lexer = tokenizer::Tokenizer::new(TokenizeOption::default());
    let tokens = lexer.scan(&file, PrettyErrorHandler::new(&file));
    let parser = parser::Parser::new(parser::ParseOption::default());
    let res = parser.parse(tokens, PrettyErrorHandler::new(&file));
    // println!("{}", );
    Ok(())
}
