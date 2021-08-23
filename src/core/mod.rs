mod tokenizer;
mod parser;
mod runtime_helper;
mod ir_gen;
mod codegen;

use codegen::CodeGenerator;
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

pub fn compile<C: CodeGenerator>(source: &str, gen: C) -> C::Output {
    let tokenizer = Tokenizer::new(source);
    let mut parser = Parser::new(tokenizer);
    let ast = parser.parse();
    let mut ir = gen.convert_ir(ast);
    gen.transform(&mut ir);
    gen.genrate(ir)
}
