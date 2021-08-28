mod tokenizer;
mod parser;
mod runtime_helper;
mod ir_converter;
mod codegen;
mod transformer;
mod error;

pub use codegen::CodeGenerator;
pub use ir_converter::IRConverter;
pub use transformer::Transformer;
use error::CompilationError;
use tokenizer::{Tokenizer, TokenizeOption};
use parser::{Parser, ParseOption};

// use plain &str here for now
// may change to tendril
pub type Name<'a> = &'a str;

#[derive(Debug, PartialEq, Eq, Clone)]
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

#[derive(Default, Debug)]
pub struct SourceLocation {
    pub start: Position,
    pub end: Position,
}

/// namespace for HTML/SVG/MathML tag
#[non_exhaustive]
pub enum Namespace {
    Html,
    Svg,
    MathMl,
    UserDefined(&'static str),
}

pub trait TemplateCompiler {
}

/// PreambleHelper is a collection of JavaScript imports at the head of output
/// e.g. v-for needs a list looping helper to make vdom
/// preamble helper needs collect helper when traversing template ast
/// and generates corresponding JavaScript imports in compilation output
pub trait PreambleHelper<Helper> {
    fn collect_helper(&mut self, helper: Helper);
    fn generate_imports(&self) -> String;
}

pub struct CompileOption {
    tokenization: TokenizeOption,
    parsing: ParseOption,
}

pub fn base_compile<IR, O, Conv, Trans, Gen>(
    source: &str, opt: CompileOption, conv: Conv, trans: Trans, gen: Gen
) -> Result<O, CompilationError> where
    Conv: IRConverter<IRNode=IR>,
    Trans: Transformer<IRNode=IR>,
    Gen: CodeGenerator<IRNode=IR, Output=O>,
{
    let tokenizer = Tokenizer::new(opt.tokenization);
    let parser = Parser::new(tokenizer).with_option(opt.parsing);
    let ast = parser.parse(source)?;
    let mut ir = conv.convert_ir(ast);
    trans.transform(&mut ir);
    Ok(gen.genrate(ir))
}
