use super::common::serialize_yaml;
use super::common::TestErrorHandler;
use super::parser_test::base_parse;
use compiler::converter::{self as C, BaseConverter, ConvertOption, Converter};
use insta::assert_snapshot;
use serde::Serialize;

#[derive(Serialize)]
struct BaseRoot;

impl<'a> From<C::BaseRoot<'a>> for BaseRoot {
    fn from(_: C::BaseRoot) -> Self {
        Self
    }
}

fn test_ir(case: &str) {
    let name = insta::_macro_support::AutoName;
    let ir = BaseRoot::from(base_convert(case));
    let val = serialize_yaml(ir);
    assert_snapshot!(name, val, case);
}

pub fn base_convert(s: &str) -> C::BaseRoot {
    let ast = base_parse(s);
    let converter = BaseConverter {
        option: ConvertOption::default(),
        err_handle: Box::new(TestErrorHandler),
    };
    converter.convert_ir(ast)
}
