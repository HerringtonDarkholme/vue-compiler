use super::common::serialize_yaml;
use super::common::TestErrorHandler;
use super::parser_test::base_parse;
use compiler::SFCInfo;
use compiler::converter::{self as C, BaseConverter, ConvertOption, Converter};
use crate::meta_macro;
use vue_compiler_core as compiler;

fn assert_ir(case: &str) -> String {
    let opt = SFCInfo::default();
    let ir = base_convert(case, &opt);
    serialize_yaml(ir.body)
}

meta_macro!(assert_ir);

#[test]
fn test_text_call() {
    assert_ir![["hello world", "hello {{world}}", "hello < world"]];
}

pub fn base_convert<'a>(s: &'a str, opt: &'a SFCInfo<'a>) -> C::BaseRoot<'a> {
    let ast = base_parse(s);
    let converter =
        BaseConverter::new(std::rc::Rc::new(TestErrorHandler), ConvertOption::default());
    converter.convert_ir(ast, opt)
}
