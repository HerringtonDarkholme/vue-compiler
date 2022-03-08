use super::common::serialize_yaml;
use super::common::TestErrorHandler;
use super::parser_test::base_parse;
use compiler::SFCInfo;
use compiler::converter::{self as C, BaseConverter, ConvertOption, Converter};
use insta::assert_snapshot;
use vue_compiler_core as compiler;

macro_rules! test_ir {
    ($case: expr) => {
        let name = insta::_macro_support::AutoName;
        let opt = SFCInfo::default();
        let ir = base_convert($case, &opt);
        let val = serialize_yaml(ir.body);
        assert_snapshot!(name, val, $case);
    };
}

#[test]
fn test_text_call() {
    let cases = ["hello world", "hello {{world}}", "hello < world"];
    for case in cases {
        test_ir!(case);
    }
}

pub fn base_convert<'a>(s: &'a str, opt: &'a SFCInfo<'a>) -> C::BaseRoot<'a> {
    let ast = base_parse(s);
    let converter =
        BaseConverter::new(std::rc::Rc::new(TestErrorHandler), ConvertOption::default());
    converter.convert_ir(ast, opt)
}
