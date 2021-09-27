use std::{
    env, io,
    path::{Path, PathBuf},
};

use anyhow::bail;
use anyhow::Result;
use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFiles,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
    },
};
use compiler::{
    compiler::{BaseCompiler, CompileOption, TemplateCompiler},
    error::{CompilationError, ErrorHandler},
    transformer::base_passes,
};
use path_clean::PathClean;

pub mod ast_print;
#[derive(Clone)]
pub struct PrettyErrorHandler {
    name: String,
    source: String,
}

impl PrettyErrorHandler {
    pub fn new<S: ToOwned<Owned = String> + ?Sized>(name: &S, source: &S) -> Self {
        Self {
            name: name.to_owned(),
            source: source.to_owned(),
        }
    }
}
impl ErrorHandler for PrettyErrorHandler {
    fn on_error(&self, err: CompilationError) {
        let mut files = SimpleFiles::new();
        let default_vue = files.add(&self.name, &self.source);
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

pub fn compile_to_stdout<'a>(name: &'a str, source: &'a str) -> Result<(), anyhow::Error> {
    let mut passes = base_passes();
    let option = CompileOption {
        tokenization: Default::default(),
        parsing: Default::default(),
        conversion: Default::default(),
        transformation: Default::default(),
        codegen: Default::default(),
        error_handler: PrettyErrorHandler::new(name, source),
    };
    let mut compiler = BaseCompiler::new(io::stdout(), &mut [], option);
    compiler.compile(source)?;
    Ok(())
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
