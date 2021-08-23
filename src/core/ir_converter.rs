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

pub trait IRConverter {
    type IRNode;
    /// convert template ast node to intermediate representation
    /// the IR format is implementation specific
    /// e.g SSR Codegen and DOM Codegen can have different IR
    fn convert_ir(&self, ast: AstRoot) -> Self::IRNode;
}

pub fn convert_ast_to_ir(ast: AstRoot) -> IRNode {
    unimplemented!("TODO")
}
