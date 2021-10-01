use super::{
    codegen::{CodeGenerateOption, CodeGenerator, CodeWriter},
    converter::{BaseConvertInfo as BaseInfo, BaseConverter, BaseRoot, ConvertOption, Converter},
    error::ErrorHandler,
    parser::{ParseOption, Parser},
    scanner::{ScanOption, Scanner},
    transformer::{BaseTransformer, CorePass, MergedPass, TransformOption, Transformer},
};

use std::io;

// TODO: we have internal option that diverges from vue's option
// CompileOption should behave like Vue option and be normalized to internal option
pub struct CompileOption<'a, E: ErrorHandler + Clone> {
    pub scanning: ScanOption,
    pub parsing: ParseOption,
    pub conversion: ConvertOption,
    pub transformation: TransformOption<'a>,
    pub codegen: CodeGenerateOption,
    pub error_handler: E,
}

// TODO: refactor this ownership usage
pub trait TemplateCompiler<'a> {
    type IR;
    type Output;
    type Eh: ErrorHandler;
    type Conv: Converter<'a, IR = Self::IR>;
    type Trans: Transformer<IR = Self::IR>;
    type Gen: CodeGenerator<IR = Self::IR, Output = Self::Output>;

    fn get_scanner(&self) -> Scanner;
    fn get_parser(&self) -> Parser;
    fn get_converter(&self) -> Self::Conv;
    fn get_transformer(&mut self) -> Self::Trans;
    fn get_code_generator(&mut self) -> Self::Gen;
    fn get_error_handler(&self) -> Self::Eh;

    fn compile(&mut self, source: &'a str) -> Self::Output {
        let scanner = self.get_scanner();
        let parser = self.get_parser();
        let eh = self.get_error_handler();
        let tokens = scanner.scan(source, eh);
        let eh = self.get_error_handler();
        let ast = parser.parse(tokens, eh);
        let mut ir = self.get_converter().convert_ir(ast);
        self.get_transformer().transform(&mut ir);
        self.get_code_generator().generate(ir)
    }
}

type Passes<'a, 'b> = &'b mut dyn CorePass<BaseInfo<'a>>;

pub struct BaseCompiler<'a, 'b, Eh: ErrorHandler + Clone, W: io::Write> {
    writer: Option<W>,
    passes: Option<&'b mut [Passes<'a, 'b>]>,
    option: CompileOption<'a, Eh>,
}

impl<'a, 'b, Eh, W> BaseCompiler<'a, 'b, Eh, W>
where
    W: io::Write,
    Eh: ErrorHandler + Clone + 'static,
{
    pub fn new(writer: W, passes: &'b mut [Passes<'a, 'b>], option: CompileOption<'a, Eh>) -> Self {
        Self {
            writer: Some(writer),
            passes: Some(passes),
            option,
        }
    }
}

impl<'a, 'b, Eh, W> TemplateCompiler<'a> for BaseCompiler<'a, 'b, Eh, W>
where
    W: io::Write,
    Eh: ErrorHandler + Clone + 'static,
{
    type IR = BaseRoot<'a>;
    type Eh = Eh;
    type Output = io::Result<()>;
    type Conv = BaseConverter<'a>;
    type Trans = BaseTransformer<'a, MergedPass<'b, Passes<'a, 'b>>>;
    type Gen = CodeWriter<'a, W>;

    fn get_scanner(&self) -> Scanner {
        Scanner::new(self.option.scanning.clone())
    }

    fn get_parser(&self) -> Parser {
        Parser::new(self.option.parsing.clone())
    }
    fn get_converter(&self) -> Self::Conv {
        let eh = self.get_error_handler();
        let option = self.option.conversion.clone();
        BaseConverter {
            err_handle: Box::new(eh),
            sfc_info: Default::default(),
            option,
        }
    }
    fn get_transformer(&mut self) -> Self::Trans {
        let pass = MergedPass::new(self.passes.take().unwrap());
        BaseTransformer::new(pass)
    }
    fn get_code_generator(&mut self) -> Self::Gen {
        let option = self.option.codegen.clone();
        let writer = self.writer.take().unwrap();
        CodeWriter::new(writer, option)
    }
    fn get_error_handler(&self) -> Self::Eh {
        self.option.error_handler.clone()
    }
}
