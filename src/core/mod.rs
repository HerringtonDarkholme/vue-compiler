mod tokenizer;
mod parser;
mod runtime_helper;
mod ir_converter;
mod codegen;
mod transformer;

pub use codegen::CodeGenerator;
pub use ir_converter::IRConverter;
pub use transformer::Transformer;
use tokenizer::Tokenizer;
use parser::Parser;

// use plain &str here for now
// may change to tendril
pub type Name<'a> = &'a str;

pub struct Position {
    /// the 0-indexed offset in the source str modulo newline
    pub offset: usize,
    /// the line number in the source code
    pub line: usize,
    /// the column number in the source code
    pub column: usize,
}

pub struct SourceLocation {
    pub start: Position,
    pub end: Position,
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

pub fn base_compile<IR, O, Conv, Trans, Gen>(
    source: &str, conv: Conv, trans: Trans, gen: Gen
) -> O where
    Conv: IRConverter<IRNode=IR>,
    Trans: Transformer<IRNode=IR>,
    Gen: CodeGenerator<IRNode=IR, Output=O>,
{
    let tokenizer = Tokenizer::new(source);
    let mut parser = Parser::new(tokenizer);
    let ast = parser.parse();
    let mut ir = conv.convert_ir(ast);
    trans.transform(&mut ir);
    gen.genrate(ir)
}
