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

/// Converts template ast node to intermediate representation.
/// All core template syntax conversion happens here.
/// the IR format can be platform specific.
/// e.g SSR Codegen and DOM Codegen can have different IR
pub trait IRConverter<'a> {
    type IRNode;
    fn convert_ir(&self, ast: AstRoot<'a>) -> Self::IRNode;

    // core template syntax conversion
    fn convert_once(&self) {}
    fn convert_if(&self) {}
    fn convert_memo(&self) {}
    fn convert_for(&self) {}
    fn convert_expression(&self) {}
    fn convert_slot_outlet(&self) {}
    fn convert_element(&self) {}
}

pub fn convert_ast_to_ir(_ast: AstRoot) -> IRNode {
    todo!()
}
