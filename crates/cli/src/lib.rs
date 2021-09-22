use std::{env, io, path::{Path, PathBuf}};

use anyhow::bail;
use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFiles,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream}, },
};
use compiler::converter::{CompilationError, ErrorHandler};
use path_clean::PathClean;
use anyhow::Result;

pub mod ast_print;
pub struct PrettyErrorHandler<'a> {
    source: &'a str,
}

impl<'a> PrettyErrorHandler<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source }
    }
}
impl<'a> ErrorHandler for PrettyErrorHandler<'a> {
    fn on_error(&self, err: CompilationError) {
        let mut files = SimpleFiles::new();
        let default_vue = files.add("default.vue", self.source);
        let diagnostic = Diagnostic::error().with_labels(vec![Label::primary(
            default_vue,
            err.location.clone(),
        )
        .with_message(format!("{}", err))]);

        let writer = StandardStream::stderr(ColorChoice::Always);
        let config = codespan_reporting::term::Config::default();

        term::emit(&mut writer.lock(), &config, &files, &diagnostic)
            .expect("unable to generate codespan diagnostic");
    }
}


pub fn absolute_path(path: impl AsRef<Path>) -> io::Result<PathBuf> {
    let path = path.as_ref();

    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()?.join(path)
    }
    .clean();

    Ok(absolute_path)


}
pub fn get_delimiters(delimiters: String) -> Result<(String, String)> {
    let split_delimiter = delimiters.split_once(" ");
    if let Some((a, b)) = split_delimiter {
        Ok((a.to_string(), b.to_string()))
    } else {
        bail!("The delimiter argument should be split by one whitespace")
    }
}
