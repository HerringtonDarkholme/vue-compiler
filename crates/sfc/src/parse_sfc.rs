use compiler::compiler::CompileOption;
use compiler::util::find_prop;
use compiler::{
    SourceLocation, BindingMetadata,
    scanner::{Scanner, TextMode},
    parser::{Parser, AstNode, AstRoot, Element, ElemProp},
    error::{VecErrorHandler, CompilationError, RcErrHandle, ErrorKind},
};
use smallvec::{smallvec, SmallVec};
use std::path::PathBuf;
use std::rc::Rc;
use rustc_hash::FxHashMap;

pub enum PadOption {
    Line,
    Space,
    NoPad,
}

pub struct SfcParseOptions {
    pub filename: String,
    pub source_map: bool,
    pub source_root: PathBuf,
    pub pad: PadOption,
    pub ignore_empty: bool,
}

impl Default for SfcParseOptions {
    fn default() -> Self {
        Self {
            filename: "anonymous.vue".into(),
            source_map: true,
            source_root: "".into(),
            pad: PadOption::NoPad,
            ignore_empty: true,
        }
    }
}

pub struct SfcBlock<'a> {
    pub content: &'a str,
    pub attrs: FxHashMap<&'a str, Option<&'a str>>,
    pub loc: SourceLocation,
    pub lang: Option<&'a str>,
    pub src: Option<&'a str>,
    // pub map: Option<RawSourceMap>,
}

fn create_block<'a>(element: Element<'a>, src: &'a str) -> SfcBlock<'a> {
    let loc = element.location;
    let content = &src[loc.start.offset..loc.end.offset];
    let attrs = element
        .properties
        .into_iter()
        .filter_map(|p| match p {
            ElemProp::Attr(attr) => Some((attr.name, attr.value.map(|v| v.content.raw))),
            _ => None,
        })
        .collect::<FxHashMap<_, _>>();
    let lang = attrs.get("lang").copied().flatten();
    let src = attrs.get("src").copied().flatten();
    SfcBlock {
        content,
        attrs,
        loc,
        lang,
        src,
    }
}

pub enum SfcError {
    DeprecatedFunctionalTemplate,
    DeprecatedStyleVars,
    SrcOnScriptSetup,
    ScrtipSrcWithScriptSetup,
    DuplicateBlock,
}

impl ErrorKind for SfcError {
    fn msg(&self) -> &'static str {
        use SfcError::*;
        match self {
            DeprecatedFunctionalTemplate => "<template functional> is no longer supported.",
            DeprecatedStyleVars => "<style vars> has been replaced by a new proposal.",
            SrcOnScriptSetup => "<script setup> cannot use the 'src' attribute because its syntax will be ambiguous outside of the component.",
            ScrtipSrcWithScriptSetup => "<script> cannot use the 'src' attribute when <script setup> is also present because they must be processed together.",
            DuplicateBlock => "Single file component can contain only one element: ",
        }
    }
}

// TODO
pub type Ast = String;

pub struct SfcTemplateBlock<'a> {
    // pub ast: Ast,
    pub block: SfcBlock<'a>,
}

pub struct SfcScriptBlock<'a> {
    pub ast: Option<Ast>,
    pub setup_ast: Option<Ast>,
    pub setup: Option<&'a str>,
    pub bindings: Option<BindingMetadata<'a>>,
    pub block: SfcBlock<'a>,
}

pub struct SfcStyleBlock<'a> {
    pub scoped: bool,
    pub module: Option<&'a str>,
    pub block: SfcBlock<'a>,
}
pub struct SfcCustomBlock<'a> {
    pub custom_type: &'a str,
    pub block: SfcBlock<'a>,
}

pub struct SfcDescriptor<'a> {
    pub filename: String,
    pub template: Option<SfcTemplateBlock<'a>>,
    pub scripts: SmallVec<[SfcScriptBlock<'a>; 1]>,
    pub styles: SmallVec<[SfcStyleBlock<'a>; 1]>,
    pub custom_blocks: Vec<SfcCustomBlock<'a>>,
    pub css_vars: Vec<&'a str>,
    /// whether the SFC uses :slotted() modifier.
    /// this is used as a compiler optimization hint.
    pub slotted: bool,
}

impl<'a> SfcDescriptor<'a> {
    fn new(filename: String) -> Self {
        Self {
            filename,
            template: None,
            scripts: smallvec![],
            styles: smallvec![],
            custom_blocks: vec![],
            css_vars: vec![],
            slotted: false,
        }
    }
}

pub struct SfcParseResult<'a> {
    pub descriptor: SfcDescriptor<'a>,
    pub errors: Vec<CompilationError>,
}

pub fn parse_sfc(source: &str, option: SfcParseOptions) -> SfcParseResult<'_> {
    let err_handle = Rc::new(VecErrorHandler::default());
    let ast = parse_ast(source, err_handle.clone());
    let mut descriptor = SfcDescriptor::new(option.filename);
    let mut errors = get_errors(err_handle);
    for node in ast.children {
        let maybe_errror = assemble_descriptor(node, source, &mut descriptor, option.ignore_empty);
        if let Some(error) = maybe_errror {
            errors.push(error);
        }
    }
    SfcParseResult { descriptor, errors }
}

fn parse_ast(source: &str, err_handle: RcErrHandle) -> AstRoot {
    let compile_opt = CompileOption {
        is_pre_tag: |_| true,
        is_native_tag: |_| true,
        get_text_mode: |tag| {
            if tag == "template" {
                TextMode::Data
            } else {
                TextMode::RawText
            }
        },
        error_handler: err_handle.clone(),
        ..Default::default()
    };
    let scanner = Scanner::new(compile_opt.scanning());
    let parser = Parser::new(compile_opt.parsing());
    let tokens = scanner.scan(source, err_handle.clone());
    parser.parse(tokens, err_handle.clone())
}

fn get_errors(err_handle: Rc<VecErrorHandler>) -> Vec<CompilationError> {
    err_handle.error_mut().drain(..).collect()
}

fn assemble_descriptor<'a>(
    node: AstNode<'a>,
    src: &'a str,
    descriptor: &mut SfcDescriptor<'a>,
    ignore_empty: bool,
) -> Option<CompilationError> {
    let element = match node {
        AstNode::Element(elem) => elem,
        _ => return None,
    };
    if ignore_empty && element.tag_name != "template" && is_empty(&element) && !has_src(&element) {
        return None;
    }
    let tag_name = element.tag_name;
    if tag_name == "template" {
        let has_functional =
            find_prop(&element, "functional").map(|func| func.get_ref().get_location().clone());
        if descriptor.template.is_some() {
            let error = CompilationError::extended(SfcError::DuplicateBlock)
                .with_additional_message("<template>")
                .with_location(element.location);
            return Some(error);
        }
        let block = SfcTemplateBlock {
            block: create_block(element, src),
        };
        descriptor.template = Some(block);
        has_functional.map(|loc| {
            CompilationError::extended(SfcError::DeprecatedFunctionalTemplate).with_location(loc)
        })
    } else if tag_name == "script" {
        todo!("script");
    } else if tag_name == "style" {
        todo!("style");
    } else {
        todo!("custom");
    }
}

fn is_empty(_elem: &Element) -> bool {
    todo!()
}

fn has_src(_elem: &Element) -> bool {
    todo!()
}
