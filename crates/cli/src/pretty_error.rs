use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFiles,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
    },
};
use compiler::error::{CompilationError, ErrorHandler};

#[derive(Clone)]
pub struct PrettyErrorHandler {
    name: String,
    source: String,
}

impl PrettyErrorHandler {
    pub fn new(name: String, source: String) -> Self {
        Self { name, source }
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
