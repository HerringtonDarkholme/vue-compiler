use super::{
    codegen::CodeGenerator,
    converter::Converter,
    error::ErrorHandler,
    parser::{ParseOption, Parser},
    tokenizer::{TokenizeOption, Tokenizer},
    transformer::Transformer,
};

pub struct CompileOption<E: ErrorHandler> {
    tokenization: TokenizeOption,
    parsing: ParseOption,
    error_handler: E,
}

pub trait TemplateCompiler<'a> {
    type IR;
    type Output;
    type Eh: ErrorHandler;
    type Conv: Converter<'a, IR = Self::IR>;
    type Trans: Transformer<IR = Self::IR>;
    type Gen: CodeGenerator<IR = Self::IR, Output = Self::Output>;

    fn get_tokenizer() -> Tokenizer;
    fn get_parser() -> Parser;
    fn get_converter() -> Self::Conv;
    fn get_transformer() -> Self::Trans;
    fn get_code_generator() -> Self::Gen;
    fn get_error_handler() -> Self::Eh;

    fn compile(source: &'a str) -> Self::Output {
        let tokenizer = Self::get_tokenizer();
        let parser = Self::get_parser();
        let eh = Self::get_error_handler();
        let tokens = tokenizer.scan(source, eh);
        let eh = Self::get_error_handler();
        let ast = parser.parse(tokens, eh);
        let mut ir = Self::get_converter().convert_ir(ast);
        Self::get_transformer().transform(&mut ir);
        Self::get_code_generator().generate(ir)
    }
}

pub fn base_compile<'a, C: TemplateCompiler<'a>>(source: &'a str) -> C::Output {
    C::compile(source)
}
