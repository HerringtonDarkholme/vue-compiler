// TODO: reorg pub
pub mod codegen;
pub mod converter;
pub mod error;
pub mod parser;
pub mod runtime_helper;
pub mod tokenizer;
pub mod transformer;
pub mod util;

pub use codegen::CodeGenerator;
pub use converter::Converter;
use error::{CompilationError, ErrorHandler};
use parser::{ParseOption, Parser};
use tokenizer::{TokenizeOption, Tokenizer};
pub use transformer::Transformer;

#[cfg(test)]
use serde::Serialize;

// use plain &str here for now
// may change to tendril
pub type Name<'a> = &'a str;

#[derive(PartialEq, Eq, Clone)]
#[cfg_attr(test, derive(Serialize))]
pub struct Position {
    /// the 0-indexed offset in the source str modulo newline
    pub offset: usize,
    /// the line number in the source code
    pub line: usize,
    /// the column number in the source code
    pub column: usize,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            offset: 0,
            line: 1,
            column: 1,
        }
    }
}

#[derive(Default, PartialEq, Eq, Clone)]
#[cfg_attr(test, derive(Serialize))]
pub struct SourceLocation {
    pub start: Position,
    pub end: Position,
}

/// namespace for HTML/SVG/MathML tag
#[non_exhaustive]
#[derive(Eq, PartialEq)]
#[cfg_attr(test, derive(Serialize))]
pub enum Namespace {
    Html,
    Svg,
    MathMl,
    UserDefined(&'static str),
}

pub trait TemplateCompiler {}

/// PreambleHelper is a collection of JavaScript imports at the head of output
/// e.g. v-for needs a list looping helper to make vdom
/// preamble helper needs collect helper when traversing template ast
/// and generates corresponding JavaScript imports in compilation output
pub trait PreambleHelper<Helper> {
    fn collect_helper(&mut self, helper: Helper);
    fn generate_imports(&self) -> String;
    fn helper_str(&self) -> &'static str;
}

pub struct CompileOption<E: ErrorHandler> {
    tokenization: TokenizeOption,
    parsing: ParseOption,
    error_handler: E,
}

pub fn base_compile<'a, IR, O, E, Conv, Trans, Gen>(
    source: &'a str,
    opt: CompileOption<E>,
    conv: Conv,
    trans: Trans,
    gen: Gen,
) -> Result<O, CompilationError>
where
    E: ErrorHandler + Clone,
    Conv: Converter<'a, IR = IR>,
    Trans: Transformer<IR = IR>,
    Gen: CodeGenerator<IR = IR, Output = O>,
{
    let eh = opt.error_handler;
    let tokenizer = Tokenizer::new(opt.tokenization);
    let parser = Parser::new(opt.parsing);
    let tokens = tokenizer.scan(source, eh.clone());
    let ast = parser.parse(tokens, eh);
    let mut ir = conv.convert_ir(ast);
    trans.transform(&mut ir);
    Ok(gen.generate(ir))
}
