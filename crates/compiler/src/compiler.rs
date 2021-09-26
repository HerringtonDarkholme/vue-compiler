use super::{
    codegen::{CodeGenerateOption, CodeGenerator, CodeWriter},
    converter::{BaseConvertInfo as BaseInfo, BaseConverter, BaseRoot, ConvertOption, Converter},
    error::{ErrorHandler, VecErrorHandler},
    parser::{ParseOption, Parser},
    tokenizer::{TokenizeOption, Tokenizer},
    transformer::{BaseTransformer, CorePass, MergedPass, TransformOption, Transformer},
};

use std::{io, marker::PhantomData};

// TODO: we have internal option that diverges from vue's option
// CompileOption should behave like Vue option and be normalized to internal option
pub struct CompileOption<E: ErrorHandler> {
    tokenization: TokenizeOption,
    parsing: ParseOption,
    conversion: ConvertOption,
    transformation: TransformOption,
    codegen: CodeGenerateOption,
    error_handler: E,
}

pub trait TemplateCompiler<'a> {
    type IR;
    type Output;
    type Eh: ErrorHandler;
    type Conv: Converter<'a, IR = Self::IR>;
    type Trans: Transformer<IR = Self::IR>;
    type Gen: CodeGenerator<IR = Self::IR, Output = Self::Output>;

    fn get_tokenizer(&self) -> Tokenizer;
    fn get_parser(&self) -> Parser;
    fn get_converter(&self) -> Self::Conv;
    fn get_transformer(&self) -> Self::Trans;
    fn get_code_generator(&self) -> Self::Gen;
    fn get_error_handler(&self) -> Self::Eh;

    fn compile(&self, source: &'a str) -> Self::Output {
        let tokenizer = self.get_tokenizer();
        let parser = self.get_parser();
        let eh = self.get_error_handler();
        let tokens = tokenizer.scan(source, eh);
        let eh = self.get_error_handler();
        let ast = parser.parse(tokens, eh);
        let mut ir = self.get_converter().convert_ir(ast);
        self.get_transformer().transform(&mut ir);
        self.get_code_generator().generate(ir)
    }
}

pub struct BaseCompiler<'a, P, const N: usize>
where
    P: CorePass<BaseInfo<'a>>,
{
    writer: Vec<u8>,
    passes: [P; N],
    option: CompileOption<VecErrorHandler>,
    eh: VecErrorHandler,
    pd: PhantomData<&'a ()>,
}
impl<'a, P, const N: usize> BaseCompiler<'a, P, N>
where
    P: CorePass<BaseInfo<'a>>,
{
    pub fn into_string(self) -> String {
        String::from_utf8(self.writer).expect("Compiler should ouput valid UTF8")
    }
}

impl<'a, P, const N: usize> TemplateCompiler<'a> for BaseCompiler<'a, P, N>
where
    P: CorePass<BaseInfo<'a>>,
{
    type IR = BaseRoot<'a>;
    type Eh = VecErrorHandler;
    type Output = io::Result<()>;
    type Conv = BaseConverter<'a>;
    type Trans = BaseTransformer<'a, P>;
    type Gen = CodeWriter<'a, Vec<u8>>;

    fn get_tokenizer(&self) -> Tokenizer {
        Tokenizer::new(self.option.tokenization.clone())
    }

    fn get_parser(&self) -> Parser {
        Parser::new(self.option.parsing.clone())
    }
    fn get_converter(&self) -> Self::Conv {
        let eh = self.get_error_handler();
        let option = self.option.conversion;
        BaseConverter {
            err_handle: Box::new(eh),
            option,
            pd: PhantomData,
        }
    }
    fn get_transformer(&self) -> Self::Trans {
        let pass = MergedPass {
            passes: self.passes,
        };
        BaseTransformer {
            pass,
            pd: PhantomData,
        }
    }
    fn get_code_generator(&self) -> Self::Gen {
        CodeWriter {
            writer: self.writer,
            indent_level: 0,
            closing_brackets: 0,
            helpers: Default::default(),
            option: self.option.codegen,
            pd: PhantomData,
        }
    }
    fn get_error_handler(&self) -> Self::Eh {
        self.eh.clone()
    }
}
