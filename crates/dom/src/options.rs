use compiler::{
    Namespace, codegen::ScriptMode, compiler::CompileOption, converter::RcErrHandle,
    flags::RuntimeHelper, parser::Element, scanner::TextMode,
};
use crate::{converter::DOM_DIR_CONVERTERS, extension::dom_helper};

fn is_html_tag(s: &str) -> bool {
    todo!()
}

fn is_svg_tag(s: &str) -> bool {
    todo!()
}

fn is_native_tag(s: &str) -> bool {
    is_html_tag(s) || is_svg_tag(s)
}

fn is_pre_tag(s: &str) -> bool {
    s.eq_ignore_ascii_case("pre")
}

fn is_void_tag(s: &str) -> bool {
    todo!()
}

fn get_builtin_component(s: &str) -> Option<RuntimeHelper> {
    todo!()
}

fn get_text_mode(s: &str) -> TextMode {
    match s {
        "style" | "script" | "iframe" | "noscript" => TextMode::RawText,
        "textarea" | "title" => TextMode::RcData,
        _ => TextMode::Data,
    }
}

fn get_namespace(s: &str, ancestors: &[Element]) -> Namespace {
    todo!()
}

pub fn compile_option(error_handler: RcErrHandle) -> CompileOption {
    CompileOption {
        is_native_tag,
        get_text_mode,
        is_pre_tag,
        is_void_tag,
        get_builtin_component,
        get_namespace,
        delimiters: ("{{".to_string(), "}}".to_string()),
        directive_converters: DOM_DIR_CONVERTERS.iter().copied().collect(),
        helper_strs: dom_helper::DOM_HELPER_MAP,
        error_handler,
        mode: ScriptMode::Function {
            prefix_identifier: false,
            runtime_global_name: "Vue".into(),
        },
        ..Default::default()
    }
}
