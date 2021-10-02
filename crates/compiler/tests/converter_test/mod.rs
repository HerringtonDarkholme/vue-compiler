use super::common::serialize_yaml;
use super::common::TestErrorHandler;
use super::parser_test::base_parse;
use compiler::converter::{self as C, BaseConverter, ConvertOption, Converter};
use insta::assert_snapshot;
use vue_compiler_core as compiler;

fn test_ir(case: &str) {
    let name = insta::_macro_support::AutoName;
    let ir = base_convert(case);
    let val = serialize_yaml(ir.body);
    assert_snapshot!(name, val, case);
}

#[test]
fn test_text_call() {
    let cases = ["hello world", "hello {{world}}", "hello < world"];
    for case in cases {
        test_ir(case);
    }
}

pub fn base_convert(s: &str) -> C::BaseRoot {
    let ast = base_parse(s);
    let converter = BaseConverter {
        option: ConvertOption::default(),
        sfc_info: Default::default(),
        err_handle: Box::new(TestErrorHandler),
    };
    converter.convert_ir(ast)
}
