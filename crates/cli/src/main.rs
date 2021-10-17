mod cli;
mod pretty_error;

use std::{
    env, fs,
    io::{self, Read},
    path::{Path, PathBuf},
    rc::Rc,
};

use anyhow::{bail, Result};
use clap::{AppSettings, Clap};

use compiler::compiler::CompileOption;
use dom::compile_option;

use cli::compile_to_stdout;
use path_clean::PathClean;

use pretty_error::PrettyErrorHandler;

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
    #[clap(long, number_of_values = 2)]
    delimiters: Option<Vec<String>>,

    /// Display the token stream produced by scanner
    #[clap(short = 's', long)]
    dump_scan: bool,
    /// Display the AST produced by parser
    #[clap(short = 'p', long)]
    dump_parse: bool,
    /// Display the IR produced by converter
    #[clap(short = 'c', long)]
    dump_convert: bool,
    /// Display the optimized IR after transformation
    #[clap(short = 't', long)]
    dump_transform: bool,
}

struct ShowOption {
    dump_scan: bool,
    dump_parse: bool,
    dump_convert: bool,
    dump_transform: bool,
}

fn main() -> Result<()> {
    let opts = Opts::parse();
    let cli_input = process(opts)?;
    compile_to_stdout(cli_input)
}

type CliInput = (String, CompileOption, ShowOption);
fn process(opts: Opts) -> Result<CliInput> {
    let (name, source) = get_file(opts.input_file_name)?;
    let err_handle = PrettyErrorHandler::new(name, source.clone());
    let delimiters = get_delimiters(opts.delimiters)?;
    let option = CompileOption {
        delimiters,
        ..compile_option(Rc::new(err_handle))
    };
    let show = ShowOption {
        dump_scan: opts.dump_scan,
        dump_parse: opts.dump_parse,
        dump_convert: opts.dump_convert,
        dump_transform: opts.dump_transform,
    };
    Ok((source, option, show))
}

fn get_file(input: Option<String>) -> Result<(String, String)> {
    if let Some(file_name) = input {
        let ab_path = absolute_path(file_name.clone())?;
        Ok((file_name, fs::read_to_string(ab_path)?))
    } else {
        let mut s = String::new();
        io::stdin().read_to_string(&mut s)?;
        Ok(("App.vue".to_owned(), s))
    }
}

fn get_delimiters(delimiters: Option<Vec<String>>) -> Result<(String, String)> {
    let mut delimiters = match delimiters {
        Some(d) => d,
        None => return Ok(("{{".into(), "}}".into())),
    };
    if delimiters.len() != 2 {
        bail!("The delimiters have exactly two parts.");
    }
    let end = delimiters.pop().unwrap();
    let start = delimiters.pop().unwrap();
    if start.is_empty() || end.is_empty() {
        bail!("Delimiter cannot be empty.")
    }
    Ok((start, end))
}

fn absolute_path(path: impl AsRef<Path>) -> io::Result<PathBuf> {
    let path = path.as_ref();

    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()?.join(path)
    };

    Ok(PathClean::clean(&absolute_path))
}
