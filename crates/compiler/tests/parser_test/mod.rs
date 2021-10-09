use vue_compiler_core as compiler;
mod dir;
use super::common::{serialize_yaml, get_compiler};
use compiler::compiler::TemplateCompiler;
use compiler::parser::AstRoot;
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

pub fn base_parse(s: &str) -> AstRoot {
    let compiler = get_compiler();
    let tokens = compiler.scan(s);
    compiler.parse(tokens)
}
