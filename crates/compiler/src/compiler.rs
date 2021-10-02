use super::{
    codegen::{CodeGenerateOption, CodeGenerator, CodeWriter, EntityDecoder, ScriptMode},
    converter::{
        BaseConvertInfo as BaseInfo, BaseConverter, BaseRoot, ConvertOption, Converter,
        DirConvertFn,
    },
    error::RcErrHandle,
    flags::RuntimeHelper,
    parser::{Element, ParseOption, Parser, WhitespaceStrategy},
    scanner::{ScanOption, Scanner, TextMode},
    transformer::{BaseTransformer, CorePass, MergedPass, TransformOption, Transformer},
    Namespace,
};

use rustc_hash::FxHashMap;
use std::io;

pub struct CompileOption {
    /// e.g. platform native elements, e.g. `<div>` for browsers
    pub is_native_tag: fn(&str) -> bool,

    /// e.g. native elements that can self-close, e.g. `<img>`, `<br>`, `<hr>`
    pub is_void_tag: fn(&str) -> bool,

    /// e.g. elements that should preserve whitespace inside, e.g. `<pre>`
    pub is_pre_tag: fn(&str) -> bool,

    /// Platform-specific built-in components e.g. `<Transition>`
    /// The pairing runtime provides additional built-in elements,
    /// Platform developer can use this to mark them as built-in
    /// so the compiler will generate component vnodes for them.
    pub is_builtin_component: fn(&str) -> Option<RuntimeHelper>,

    /// Separate option for end users to extend the native elements list
    pub is_custom_element: fn(&str) -> bool,

    /// Get tag namespace
    pub get_namespace: fn(&str, &Vec<Element<'_>>) -> Namespace,

    /// Get text parsing mode for this element
    pub get_text_mode: fn(&str) -> TextMode,

    /// @default ['{{', '}}']
    pub delimiters: (String, String),

    /// Whitespace handling strategy
    pub whitespace: WhitespaceStrategy,

    /// Only needed for DOM compilers
    pub decode_entities: EntityDecoder,

    /// Whether to keep comments in the templates AST.
    /// This defaults to `true` in development and `false` in production builds.
    pub comments: bool,
    /// Whether the output is dev build
    pub is_dev: bool,

    /// An object of { name: transform } to be applied to every directive attribute
    /// node found on element nodes.
    pub directive_converters: FxHashMap<&'static str, DirConvertFn>,
    /// Hoist static VNodes and props objects to `_hoisted_x` constants
    /// @default false
    pub hoist_static: bool,
    /// Cache v-on handlers to avoid creating new inline functions on each render,
    /// also avoids the need for dynamically patching the handlers by wrapping it.
    /// e.g `@click="foo"` by default is compiled to `{ onClick: foo }`. With this
    /// option it's compiled to:
    /// ```js
    /// { onClick: _cache[0] || (_cache[0] = e => _ctx.foo(e)) }
    /// ```
    /// - Requires "prefixIdentifiers" to be enabled because it relies on scope
    /// analysis to determine if a handler is safe to cache.
    /// @default false
    pub cache_handlers: bool,

    /// - `module` mode will generate ES module import statements for helpers
    /// and export the render function as the default export.
    /// - `function` mode will generate a single `const { helpers... } = Vue`
    /// statement and return the render function. It expects `Vue` to be globally
    /// available (or passed by wrapping the code with an IIFE). It is meant to be
    /// used with `new Function(code)()` to generate a render function at runtime.
    /// @default 'function'
    pub mode: ScriptMode,
    /// Generate source map?
    /// @default false
    pub source_map: bool,
    pub error_handler: RcErrHandle,
    // deleted options
    // nodeTransforms?: NodeTransform[]
    // transformHoist?: HoistTransform | null
    // expressionPlugins?: ParserPlugin[]
    // prefix_identifiers: bool,
    // optimizeImports?: boolean // farewell, webpack optimization

    // moved to SFCInfo
    // bindingMetadata?: BindingMetadata
    // inline?: boolean
    // filename?: string
    // scopeId?: string | null
    // slotted?: boolean

    // moved to SSR
    // ssr: bool // will be false in fallback node
    // inSSR?: bool // always true in ssr build
    // ssrCssVars?: string
    // ssrRuntimeModuleName?: string
}

impl Default for CompileOption {
    fn default() -> Self {
        todo!()
    }
}

impl CompileOption {
    pub fn scanning(&self) -> ScanOption {
        todo!()
    }
    pub fn parsing(&self) -> ParseOption {
        todo!()
    }
    pub fn converting(&self) -> ConvertOption {
        todo!()
    }
    pub fn transforming(&self) -> TransformOption {
        todo!()
    }
    pub fn codegen(&self) -> CodeGenerateOption {
        todo!()
    }
}

// TODO: refactor this ownership usage
pub trait TemplateCompiler<'a> {
    type IR;
    type Output;
    type Conv: Converter<'a, IR = Self::IR>;
    type Trans: Transformer<IR = Self::IR>;
    type Gen: CodeGenerator<IR = Self::IR, Output = Self::Output>;

    fn get_scanner(&self) -> Scanner;
    fn get_parser(&self) -> Parser;
    fn get_converter(&self) -> Self::Conv;
    fn get_transformer(&mut self) -> Self::Trans;
    fn get_code_generator(&mut self) -> Self::Gen;
    fn get_error_handler(&self) -> RcErrHandle;

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

pub struct BaseCompiler<'a, 'b, W: io::Write> {
    writer: Option<W>,
    passes: Option<&'b mut [Passes<'a, 'b>]>,
    option: CompileOption,
}

impl<'a, 'b, W> BaseCompiler<'a, 'b, W>
where
    W: io::Write,
{
    pub fn new(writer: W, passes: &'b mut [Passes<'a, 'b>], option: CompileOption) -> Self {
        Self {
            writer: Some(writer),
            passes: Some(passes),
            option,
        }
    }
}

impl<'a, 'b, W> TemplateCompiler<'a> for BaseCompiler<'a, 'b, W>
where
    W: io::Write,
{
    type IR = BaseRoot<'a>;
    type Output = io::Result<()>;
    type Conv = BaseConverter<'a>;
    type Trans = BaseTransformer<'a, MergedPass<'b, Passes<'a, 'b>>>;
    type Gen = CodeWriter<'a, W>;

    fn get_scanner(&self) -> Scanner {
        Scanner::new(self.option.scanning())
    }

    fn get_parser(&self) -> Parser {
        Parser::new(self.option.parsing())
    }
    fn get_converter(&self) -> Self::Conv {
        let eh = self.get_error_handler();
        let option = self.option.converting();
        BaseConverter {
            err_handle: eh,
            sfc_info: Default::default(),
            option,
        }
    }
    fn get_transformer(&mut self) -> Self::Trans {
        let pass = MergedPass::new(self.passes.take().unwrap());
        BaseTransformer::new(pass)
    }
    fn get_code_generator(&mut self) -> Self::Gen {
        let option = self.option.codegen();
        let writer = self.writer.take().unwrap();
        CodeWriter::new(writer, option)
    }
    fn get_error_handler(&self) -> RcErrHandle {
        self.option.error_handler.clone()
    }
}
