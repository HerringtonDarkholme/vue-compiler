use anyhow::Result;
use clap::{AppSettings, Clap};
use cli::ast_print::AstString;
use compiler::parser::{ParseOption, Parser};
use compiler::tokenizer::{self, TokenizeOption};

use std::fs::read_to_string;

use cli::{absolute_path, get_delimiters, PrettyErrorHandler};

/// This doc string acts as a help message when the user runs '--help'
/// as do all doc strings on fields
#[derive(Clap)]
#[clap(
    version = "0.1.0",
    author = "Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>"
)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// Sets a custom config file. Could have been an Option<T> with no default too
    input_file_name: String,

    // pub delimiters: (String, String),
    #[clap(short, long, default_value = "{{ }}")]
    delimiters: String,
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let ab_path = absolute_path(opts.input_file_name)?;
    let file = read_to_string(ab_path)?;

    let tokenizer = tokenizer::Tokenizer::new(TokenizeOption {
        delimiters: get_delimiters(opts.delimiters)?,
        ..TokenizeOption::default()
    });
    let tokens = tokenizer.scan(&file, PrettyErrorHandler::new(&file));
    let parser = Parser::new(ParseOption::default());
    let res = parser.parse(tokens, PrettyErrorHandler::new(&file));
    println!("{}", res.ast_string(0));

    Ok(())
}
