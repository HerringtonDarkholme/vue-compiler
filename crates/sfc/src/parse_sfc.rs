use compiler::compiler::CompileOption;
use compiler::{
    SourceLocation, BindingMetadata,
    scanner::{Scanner, TextMode},
    parser::{Parser, AstNode},
    error::{VecErrorHandler, CompilationError},
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
    pub source: &'a str,
    pub template: Option<SfcTemplateBlock<'a>>,
    pub scripts: SmallVec<[SfcScriptBlock<'a>; 1]>,
    pub styles: SmallVec<[SfcStyleBlock<'a>; 1]>,
    pub custom_blocks: Vec<SfcCustomBlock<'a>>,
    pub css_vars: Vec<&'a str>,
    /// whether the SFC uses :slotted() modifier.
    /// this is used as a compiler optimization hint.
    pub slotted: bool,
}

pub enum SfcError {
    CompilerError(CompilationError),
    SyntaxError(&'static str),
}

pub struct SfcParseResult<'a> {
    pub descriptor: SfcDescriptor<'a>,
    pub errors: Vec<SfcError>,
}

pub fn parse_sfc(source: &str, option: SfcParseOptions) -> SfcParseResult<'_> {
    let err_handle = Rc::new(VecErrorHandler::default());
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
    let ast = parser.parse(tokens, err_handle.clone());
    let descriptor = SfcDescriptor {
        filename: option.filename,
        source,
        template: None,
        scripts: smallvec![],
        styles: smallvec![],
        custom_blocks: vec![],
        css_vars: vec![],
        slotted: false,
    };
    let errors = err_handle
        .error_mut()
        .drain(..)
        .map(SfcError::CompilerError)
        .collect::<Vec<_>>();
    for node in ast.children {
        let element = match node {
            AstNode::Element(elem) => elem,
            _ => continue,
        };
        match element.tag_name {
            "template" => {}
            "script" => {}
            "style" => {}
            _ => {}
        }
    }

    SfcParseResult { descriptor, errors }
}
