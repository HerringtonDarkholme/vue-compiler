use vue_compiler_core as compiler;
use super::common::get_compiler;
use compiler::compiler::TemplateCompiler;
use insta::assert_snapshot;
use rslint_parser::parse_text;

fn test_codegen(case: &str) {
    let name = insta::_macro_support::AutoName;
    let val = base_compile(case);
    // `function target have return outside function
    let wrap_in_func = format!("function () {{ {} }}", val);
    let parsed = parse_text(&wrap_in_func, 0);
    assert!(parsed.errors().is_empty());
    assert_snapshot!(name, val, case);
}

pub fn base_compile(source: &str) -> String {
    let sfc_info = Default::default();
    let compiler = get_compiler();
    let ret = compiler.compile(source, &sfc_info).unwrap();
    String::from_utf8(ret).unwrap()
}

#[test]
fn test_text_codegen() {
    let cases = [
        "Hello world",
        "Hello {{world}}",
        "<p>Hello {{world}}</p>",
        "<comp>Hello {{world}}</comp>",
    ];
    for case in cases {
        test_codegen(case);
    }
}
