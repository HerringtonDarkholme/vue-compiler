use super::{
    codegen::{CodeGenerateOption, CodeGenerator, CodeWriter},
    converter::{BaseConvertInfo as BaseInfo, BaseConverter, BaseRoot, ConvertOption, Converter},
    error::{ErrorHandler, VecErrorHandler},
    parser::{ParseOption, Parser},
    tokenizer::{TokenizeOption, Tokenizer},
    transformer::{BaseTransformer, CorePass, MergedPass, TransformOption, Transformer},
};

use std::io;

// TODO: we have internal option that diverges from vue's option
// CompileOption should behave like Vue option and be normalized to internal option
pub struct CompileOption<'a, E: ErrorHandler> {
    tokenization: TokenizeOption,
    parsing: ParseOption,
    conversion: ConvertOption<'a>,
    transformation: TransformOption<'a>,
    codegen: CodeGenerateOption,
    error_handler: E,
}

pub trait TemplateCompiler<'a> {
    type IR;
    type Output;
    type Result;
    type Eh: ErrorHandler;
    type Conv: Converter<'a, IR = Self::IR>;
    type Trans: Transformer<IR = Self::IR>;
    type Gen: CodeGenerator<IR = Self::IR, Output = Self::Output>;

    fn get_tokenizer(&self) -> Tokenizer;
    fn get_parser(&self) -> Parser;
    fn get_converter(&self) -> Self::Conv;
    fn get_transformer(&mut self) -> Self::Trans;
    fn get_code_generator(&self) -> Self::Gen;
    fn get_error_handler(&self) -> Self::Eh;
    fn get_result(&self, gen: Self::Gen) -> Self::Result;

    fn compile(&mut self, source: &'a str) -> Self::Result {
        let tokenizer = self.get_tokenizer();
        let parser = self.get_parser();
        let eh = self.get_error_handler();
        let tokens = tokenizer.scan(source, eh);
        let eh = self.get_error_handler();
        let ast = parser.parse(tokens, eh);
        let mut ir = self.get_converter().convert_ir(ast);
        self.get_transformer().transform(&mut ir);
        let mut gen = self.get_code_generator();
        gen.generate(ir);
        self.get_result(gen)
    }
}

type Passes<'a, 'b> = &'b mut dyn CorePass<BaseInfo<'a>>;

pub struct BaseCompiler<'a, 'b, const N: usize> {
    passes: Option<[Passes<'a, 'b>; N]>,
    option: CompileOption<'a, VecErrorHandler>,
    eh: VecErrorHandler,
}

impl<'a, 'b, const N: usize> TemplateCompiler<'a> for BaseCompiler<'a, 'b, N> {
    type IR = BaseRoot<'a>;
    type Result = String;
    type Eh = VecErrorHandler;
    type Output = io::Result<()>;
    type Conv = BaseConverter<'a>;
    type Trans = BaseTransformer<'a, MergedPass<Passes<'a, 'b>, N>>;
    type Gen = CodeWriter<'a, Vec<u8>>;

    fn get_tokenizer(&self) -> Tokenizer {
        Tokenizer::new(self.option.tokenization.clone())
    }

    fn get_parser(&self) -> Parser {
        Parser::new(self.option.parsing.clone())
    }
    fn get_converter(&self) -> Self::Conv {
        let eh = self.get_error_handler();
        let option = self.option.conversion.clone();
        BaseConverter {
            err_handle: Box::new(eh),
            option,
        }
    }
    fn get_transformer(&mut self) -> Self::Trans {
        let pass = MergedPass {
            passes: self.passes.take().unwrap(),
        };
        BaseTransformer::new(pass)
    }
    fn get_code_generator(&self) -> Self::Gen {
        let option = self.option.codegen.clone();
        CodeWriter::new(vec![], option)
    }
    fn get_result(&self, gen: Self::Gen) -> Self::Result {
        String::from_utf8(gen.writer).expect("compiler must produce valid string")
    }
    fn get_error_handler(&self) -> Self::Eh {
        self.eh.clone()
    }
}
