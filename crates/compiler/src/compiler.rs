use super::{
    codegen::{
        CodeGenerateOption, CodeGenerator, CodeWriter, DecodedStr, EntityDecoder, ScriptMode,
    },
    converter::{
        no_op_directive_convert, BaseConvertInfo as BaseInfo, BaseConverter, BaseRoot,
        ConvertOption, Converter, DirConvertFn, V_BIND, V_MODEL,
    },
    error::{NoopErrorHandler, RcErrHandle},
    flags::RuntimeHelper,
    parser::{Element, ParseOption, Parser, WhitespaceStrategy, AstRoot},
    scanner::{ScanOption, Scanner, TextMode, Tokens},
    transformer::{BaseTransformer, CorePass, TransformOption, Transformer},
    util::{no, yes},
    Namespace,
};

use rustc_hash::FxHashMap;
use std::{io, rc::Rc};

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
    pub get_builtin_component: fn(&str) -> Option<RuntimeHelper>,

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
    pub preserve_comments: Option<bool>,
    /// Whether the output is dev build which includes v-if comment and dev patch flags.
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
    /// Whether the output JS needs re-rendering when Vue runtime data change.
    /// e.g. SSR can set it to false since SSR is executed only once per request.
    /// @default true
    pub need_reactivity: bool,
    /// Custom error reporter. Default is noop.
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

    // moved to SSR or need_reactivity
    // ssr: bool // will be false in fallback node
    // inSSR?: bool // always true in ssr build
    // ssrCssVars?: string
    // ssrRuntimeModuleName?: string
}

impl Default for CompileOption {
    fn default() -> Self {
        let mut directive_converters = FxHashMap::default();
        directive_converters.insert(V_BIND.0, V_BIND.1);
        directive_converters.insert(V_MODEL.0, V_MODEL.1);
        directive_converters.insert("on", no_op_directive_convert);
        Self {
            is_native_tag: yes,
            is_void_tag: no,
            is_pre_tag: no,
            get_builtin_component: |_| None,
            is_custom_element: no,
            get_namespace: |_, _| Namespace::Html,
            get_text_mode: |_| TextMode::Data,
            delimiters: ("{{".into(), "}}".into()),
            whitespace: WhitespaceStrategy::Preserve,
            decode_entities: |s, _| DecodedStr::from(s),
            preserve_comments: None,
            is_dev: true,
            directive_converters,
            hoist_static: false,
            cache_handlers: false,
            mode: ScriptMode::Function {
                prefix_identifier: false,
                runtime_global_name: "Vue".into(),
            },
            source_map: false,
            need_reactivity: true,
            error_handler: Rc::new(NoopErrorHandler),
        }
    }
}

impl CompileOption {
    pub fn scanning(&self) -> ScanOption {
        ScanOption {
            delimiters: self.delimiters.clone(),
            get_text_mode: self.get_text_mode,
        }
    }
    pub fn parsing(&self) -> ParseOption {
        ParseOption {
            whitespace: self.whitespace.clone(),
            preserve_comment: self.preserve_comments.unwrap_or(self.is_dev),
            get_namespace: self.get_namespace,
            get_text_mode: self.get_text_mode,
            is_native_element: self.is_native_tag,
            is_void_tag: self.is_void_tag,
            is_pre_tag: self.is_pre_tag,
            get_builtin_component: self.get_builtin_component,
            is_custom_element: self.is_custom_element,
        }
    }
    pub fn converting(&self) -> ConvertOption {
        ConvertOption {
            get_builtin_component: self.get_builtin_component,
            is_dev: self.is_dev,
            directive_converters: self.directive_converters.clone(),
            need_reactivity: self.need_reactivity,
        }
    }
    pub fn transforming(&self) -> TransformOption {
        let prefix = match self.mode {
            ScriptMode::Function {
                prefix_identifier, ..
            } => prefix_identifier,
            ScriptMode::Module { .. } => true,
        };
        TransformOption {
            prefix_identifier: prefix,
            is_dev: self.is_dev,
        }
    }
    pub fn codegen(&self) -> CodeGenerateOption {
        CodeGenerateOption {
            is_dev: self.is_dev,
            mode: self.mode.clone(),
            source_map: self.source_map,
            decode_entities: self.decode_entities,
        }
    }
}

// TODO: refactor this ownership usage
pub trait TemplateCompiler<'a> {
    type IR;
    type Output;

    fn scan(&self, source: &'a str) -> Tokens<'a>;
    fn parse(&self, tokens: Tokens<'a>) -> AstRoot<'a>;
    fn convert(&self, ast: AstRoot<'a>) -> Self::IR;
    fn transform(&mut self, ir: &mut Self::IR);
    fn generate(&mut self, ir: Self::IR) -> Self::Output;
    fn get_error_handler(&self) -> RcErrHandle;

    fn compile(&mut self, source: &'a str) -> Self::Output {
        let tokens = self.scan(source);
        let ast = self.parse(tokens);
        let mut ir = self.convert(ast);
        self.transform(&mut ir);
        self.generate(ir)
    }
}

pub struct BaseCompiler<'a, P, W>
where
    W: io::Write,
    P: CorePass<BaseInfo<'a>>,
{
    writer: Option<W>,
    passes: Option<P>,
    option: CompileOption,
    scanner: Scanner,
    parser: Parser,
    pd: std::marker::PhantomData<&'a ()>,
}

impl<'a, P, W> BaseCompiler<'a, P, W>
where
    W: io::Write,
    P: CorePass<BaseInfo<'a>>,
{
    pub fn new(writer: W, passes: P, option: CompileOption) -> Self {
        Self {
            writer: Some(writer),
            passes: Some(passes),
            scanner: Scanner::new(option.scanning()),
            parser: Parser::new(option.parsing()),
            option,
            pd: std::marker::PhantomData,
        }
    }
    fn get_converter(&self) -> BaseConverter<'a> {
        let eh = self.get_error_handler();
        let option = self.option.converting();
        BaseConverter {
            err_handle: eh,
            sfc_info: Default::default(),
            option,
        }
    }
    fn get_transformer(&mut self, pass: P) -> BaseTransformer<'a, P> {
        BaseTransformer::new(pass)
    }
    fn get_code_generator(&mut self) -> CodeWriter<'a, W> {
        let option = self.option.codegen();
        let writer = self.writer.take().unwrap();
        CodeWriter::new(writer, option, Default::default())
    }
}

impl<'a, P, W> TemplateCompiler<'a> for BaseCompiler<'a, P, W>
where
    W: io::Write,
    P: CorePass<BaseInfo<'a>>,
{
    type IR = BaseRoot<'a>;
    type Output = io::Result<()>;

    fn scan(&self, source: &'a str) -> Tokens<'a> {
        self.scanner.scan(source, self.get_error_handler())
    }

    fn parse(&self, tokens: Tokens<'a>) -> AstRoot<'a> {
        self.parser.parse(tokens, self.get_error_handler())
    }
    fn convert(&self, ast: AstRoot<'a>) -> Self::IR {
        self.get_converter().convert_ir(ast)
    }
    fn transform(&mut self, ir: &mut Self::IR) {
        let pass = self.passes.take().unwrap();
        self.get_transformer(pass).transform(ir);
    }
    fn generate(&mut self, ir: Self::IR) -> Self::Output {
        self.get_code_generator().generate(ir)
    }
    fn get_error_handler(&self) -> RcErrHandle {
        self.option.error_handler.clone()
    }
}
