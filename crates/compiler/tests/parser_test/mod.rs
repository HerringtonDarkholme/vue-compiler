use vue_compiler_core as compiler;
mod dir;
use super::common::{serialize_yaml, TestErrorHandler};
use super::scanner_test::scan_with_opt;
use compiler::parser::{self as p, ParseOption, Parser};
use compiler::scanner::{TextMode, ScanOption};
use insta::assert_snapshot;

fn test_ast(case: &str) {
    let name = insta::_macro_support::AutoName;
    let root = base_parse(case);
    let val = serialize_yaml(root);
    assert_snapshot!(name, val, case);
}

#[test]
fn test_base_parse() {
    let cases = ["<p/>", "<p></p>", "<p>123</p>"];
    for case in cases {
        test_ast(case);
    }
}

#[test]
fn test_script() {
    let cases = [
        // "<script>abc", position is not correct
        "<script><div/></script>",
        "<script>let a = 123</scrip></script>",
    ];
    for case in cases {
        test_ast(case);
    }
}

pub fn base_parse(s: &str) -> p::AstRoot {
    let tokens = scan_with_opt(
        s,
        ScanOption {
            get_text_mode: |s| match s {
                "script" => TextMode::RawText,
                "textarea" => TextMode::RcData,
                _ => TextMode::Data,
            },
            ..Default::default()
        },
    );
    let parser = Parser::new(ParseOption {
        get_text_mode: |s| match s {
            "script" => TextMode::RawText,
            "textarea" => TextMode::RcData,
            _ => TextMode::Data,
        },
        is_native_element: |s| s != "comp",
        ..ParseOption::default()
    });
    let eh = std::rc::Rc::new(TestErrorHandler);
    parser.parse(tokens, eh)
}
