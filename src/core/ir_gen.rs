use super::parser::AstRoot;

pub enum IRNode {
    Text,
    Interpolation,
    // expression
    Simple,
    Compound,
    Comment,
    VNode,
    Call,
    Object,
    Array,
    Function,
    Conditional,
    Cache,
    Block,
}

pub fn convert_ast_to_ir(ast: AstRoot) -> IRNode {
    unimplemented!("TODO")
}
