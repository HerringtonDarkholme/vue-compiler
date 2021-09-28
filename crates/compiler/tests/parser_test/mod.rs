mod dir;
use super::common::{serialize_yaml, TestErrorHandler};
use super::tokenizer_test::base_scan;
use compiler::parser::{self as p, ParseOption, Parser};
use compiler::tokenizer::TextMode;
use insta::assert_snapshot;
use serde::Serialize;

#[derive(Serialize)]
struct AstRoot;

impl<'a> From<p::AstRoot<'a>> for AstRoot {
    fn from(_: p::AstRoot<'a>) -> Self {
        Self
    }
}

fn test_ast(case: &str) {
    let name = insta::_macro_support::AutoName;
    let root = AstRoot::from(base_parse(case));
    let val = serialize_yaml(root);
    assert_snapshot!(name, val, case);
}

pub fn base_parse(s: &str) -> p::AstRoot {
    let tokens = base_scan(s);
    let parser = Parser::new(ParseOption {
        get_text_mode: |s| match s {
            "script" => TextMode::RawText,
            "textarea" => TextMode::RcData,
            _ => TextMode::Data,
        },
        is_native_element: |s| s != "comp",
        ..ParseOption::default()
    });
    let eh = TestErrorHandler;
    parser.parse(tokens, eh)
}
