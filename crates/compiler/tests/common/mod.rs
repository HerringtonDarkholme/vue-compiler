use compiler::converter::BaseConvertInfo;
pub use compiler::error::NoopErrorHandler as TestErrorHandler;
use compiler::error::VecErrorHandler;
use compiler::transformer::CorePass;
use compiler::compiler::TemplateCompiler;
pub use compiler::{Position, SourceLocation};
use compiler::compiler::{BaseCompiler, CompileOption, get_base_passes};
use compiler::scanner::TextMode;
use serde::Serialize;
use std::rc::Rc;
use vue_compiler_core as compiler;

// insta does not support yaml with customized expr :(
// https://github.com/mitsuhiko/insta/issues/177
// WARNING: insta private API usage.
/// serialize object to yaml string
pub fn serialize_yaml<T: Serialize>(t: T) -> String {
    use insta::_macro_support::{serialize_value, SerializationFormat, SnapshotLocation};
    serialize_value(&t, SerializationFormat::Yaml, SnapshotLocation::File)
}

fn get_text_mode(s: &str) -> TextMode {
    match s {
        "style" | "script" | "iframe" | "noscript" => TextMode::RawText,
        "textarea" | "title" => TextMode::RcData,
        _ => TextMode::Data,
    }
}

fn get_compile_option() -> CompileOption {
    CompileOption {
        get_text_mode,
        is_native_tag: |s| s != "comp",
        error_handler: Rc::new(TestErrorHandler),
        ..Default::default()
    }
}

pub fn get_compiler<'a>() -> BaseCompiler<'a, impl CorePass<BaseConvertInfo<'a>>, Vec<u8>> {
    let dest = Vec::new;
    BaseCompiler::new(dest, get_base_passes, get_compile_option())
}

#[derive(Serialize)]
pub struct TestError {
    pub loc: SourceLocation,
    pub msg: String,
}

pub fn get_errors(source: &str) -> Vec<TestError> {
    let error_handler = Rc::new(VecErrorHandler::new());
    let option = CompileOption {
        get_text_mode,
        is_native_tag: |s| s != "comp",
        error_handler: error_handler.clone(),
        ..Default::default()
    };
    let dest = Vec::new;
    let sfc_info = Default::default();
    let compiler = BaseCompiler::new(dest, get_base_passes, option);
    let ret = compiler.compile(source, &sfc_info).unwrap();
    let errors = error_handler.errors();
    errors
        .iter()
        .map(|e| TestError {
            msg: e.msg().to_string(),
            loc: e.location.clone(),
        })
        .collect()
}
