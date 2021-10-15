use compiler::compiler::CompileOption;
use compiler::{
    SourceLocation, BindingMetadata,
    scanner::{Scanner, TextMode},
    parser::{Parser, AstNode, AstRoot, Element},
    error::{VecErrorHandler, CompilationError, RcErrHandle, ErrorKind, CompilationErrorKind},
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
    pub attrs: FxHashMap<&'a str, &'a str>,
    pub loc: SourceLocation,
    pub lang: Option<&'a str>,
    pub src: Option<&'a str>,
    // pub map: Option<RawSourceMap>,
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
            ScrtipSrcWithScriptSetup => "",
            DuplicateBlock => "",
        }
    }
}

// TODO
pub type Ast = String;

pub struct SfcTemplateBlock<'a> {
    pub ast: Ast,
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
        let location = node.get_location().clone();
        let maybe_errror = assemble_descriptor(node, &mut descriptor, option.ignore_empty);
        if let Some(kind) = maybe_errror {
            let error = CompilationError::new(kind).with_location(location);
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
    descriptor: &mut SfcDescriptor<'a>,
    ignore_empty: bool,
) -> Option<CompilationErrorKind> {
    let element = match node {
        AstNode::Element(elem) => elem,
        _ => return None,
    };
    if ignore_empty && element.tag_name != "template" && is_empty(&element) && !has_src(&element) {
        return None;
    }
    let tag_name = element.tag_name;
    if tag_name == "template" {
        if descriptor.template.is_some() {
            let kind = CompilationErrorKind::extended(SfcError::DuplicateBlock);
            return Some(kind);
        }
    } else if tag_name == "script" {
        todo!("script");
    } else if tag_name == "style" {
        todo!("style");
    } else {
        todo!("custom");
    }
    None
}

fn is_empty(elem: &Element) -> bool {
    todo!()
}

fn has_src(elem: &Element) -> bool {
    todo!()
}
