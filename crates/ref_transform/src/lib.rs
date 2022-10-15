use ast_grep_core::{
    AstGrep,
    language::{Language, TSLanguage},
};

#[derive(Clone)]
pub struct Ts;
impl Language for Ts {
    fn get_ts_language(&self) -> TSLanguage {
        tree_sitter_typescript::language_tsx().into()
    }
}

pub fn test() {
    AstGrep::new("console.log(123)", Ts);
}
