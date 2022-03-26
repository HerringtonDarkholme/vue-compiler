use vue_compiler_core as compiler;
use super::common::get_compiler;
use compiler::compiler::TemplateCompiler;
use crate::meta_macro;
use rslint_parser::parse_text;

fn assert_codegen(case: &str) -> String {
    let val = base_compile(case);
    // `function target have return outside function
    let wrap_in_func = format!("function () {{ {} }}", val);
    let parsed = parse_text(&wrap_in_func, 0);
    assert!(parsed.errors().is_empty());
    val
}
meta_macro!(assert_codegen);

pub fn base_compile(source: &str) -> String {
    let sfc_info = Default::default();
    let compiler = get_compiler();
    let ret = compiler.compile(source, &sfc_info).unwrap();
    String::from_utf8(ret).unwrap()
}

#[test]
fn test_text_codegen() {
    assert_codegen![[
        "Hello world",
        "Hello {{world}}",
        "<p>Hello {{world}}</p>",
        "<comp>Hello {{world}}</comp>",
    ]];
}
