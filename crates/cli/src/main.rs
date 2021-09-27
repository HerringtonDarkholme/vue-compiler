use anyhow::Result;
use clap::{AppSettings, Clap};
use std::fs;
use std::io::{self, Read};

use cli::{absolute_path, compile_to_stdout, get_delimiters, PrettyErrorHandler};

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
    compile_to_stdout(name, file)
}
