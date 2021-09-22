use anyhow::Result;
use clap::{AppSettings, Clap};
use cli::ast_print::AstString;
use compiler::parser::{ParseOption, Parser};
use compiler::tokenizer::{self, TokenizeOption};

use std::fs;
use std::io::{self, Read};

use cli::{absolute_path, get_delimiters, PrettyErrorHandler};

/// A simple CLI app for quick debugging the compiler internal.
#[derive(Clap)]
#[clap(
    version = "0.1.0",
    author = "Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>"
)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// The file to parse. Stdin will be parsed as input if no file is provided.
    input_file_name: Option<String>,

    // pub delimiters: (String, String),
    #[clap(short, long, default_value = "{{ }}")]
    delimiters: String,
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let (name, file) = if let Some(file_name) = opts.input_file_name {
        let ab_path = absolute_path(file_name.clone())?;
        (file_name, fs::read_to_string(ab_path)?)
    } else {
        let mut s = String::new();
        io::stdin().read_to_string(&mut s)?;
        ("App.vue".to_owned(), s)
    };

    let tokenizer = tokenizer::Tokenizer::new(TokenizeOption {
        delimiters: get_delimiters(opts.delimiters)?,
        ..TokenizeOption::default()
    });
    let tokens = tokenizer.scan(&file, PrettyErrorHandler::new(&name, &file));
    let parser = Parser::new(ParseOption::default());
    let res = parser.parse(tokens, PrettyErrorHandler::new(&name, &file));
    println!("{}", res.ast_string(0));

    Ok(())
}
