use ast_grep_core::{AstGrep, Language, language::TSLanguage, Node};
use tree_sitter_typescript::language_typescript;

pub type TsNode<'r> = Node<'r, TypeScript>;

#[derive(Clone)]
pub struct TypeScript;

impl Language for TypeScript {
    fn get_ts_language(&self) -> TSLanguage {
        language_typescript().into()
    }
}

pub fn parse_ts(text: &str) -> AstGrep<TypeScript> {
    TypeScript.ast_grep(text)
}
