use vue_compiler_core as compiler;
use compiler::{
    codegen::{CodeWriter, CodeGenerator, CodeGenerateOption},
    SFCInfo,
};
use insta::assert_snapshot;
use super::converter_test::base_convert;
use rslint_parser::parse_text;

fn test_codegen(case: &str) {
    let name = insta::_macro_support::AutoName;
    let val = base_compile(case);
    let parsed = parse_text(&val, 0);
    assert!(parsed.errors().is_empty());
    assert_snapshot!(name, val, case);
}

pub fn base_compile(s: &str) -> String {
    let ir = base_convert(s);
    let mut ret = vec![];
    let mut writer = CodeWriter::new(&mut ret, CodeGenerateOption::default(), SFCInfo::default());
    writer.generate(ir).unwrap();
    String::from_utf8(ret).unwrap()
}

#[test]
fn test_basic_cases() {
    // let cases = [
    //     "Hello world"
    // ];
    // for case in cases {
    //     test_codegen(case);
    // }
}
