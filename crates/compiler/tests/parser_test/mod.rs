use vue_compiler_core as compiler;
mod dir;
use super::common::{serialize_yaml, get_compiler};
use compiler::compiler::TemplateCompiler;
use compiler::parser::AstRoot;
use crate::meta_macro;

fn assert_parse(case: &str) -> String {
    let root = base_parse(case);
    serialize_yaml(root)
}
meta_macro!(assert_parse);

#[test]
fn test_base_parse() {
    assert_parse![["<p/>", "<p></p>", "<p>123</p>"]];
}

#[test]
fn test_script() {
    assert_parse![[
        // "<script>abc", position is not correct
        "<script><div/></script>",
        "<script>let a = 123</scrip></script>",
    ]];
}

pub fn base_parse(s: &str) -> AstRoot {
    let compiler = get_compiler();
    let tokens = compiler.scan(s);
    compiler.parse(tokens)
}
