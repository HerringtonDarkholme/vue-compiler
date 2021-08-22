use super::parser::AstRoot;

pub trait CodeGenerator {
    type IRNode;
    type Output;
    fn get_ir(&self, ast: AstRoot) -> Self::IRNode;
    fn transform(&self, nodes: &mut Self::IRNode);
    fn genrate(&self, nodes: Self::IRNode) -> Self::Output;
}
