use super::parser::AstRoot;
use super::ir_gen::{
    IRNode, convert_ast_to_ir
};

pub trait CodeGenerator {
    type IRNode;
    type Output;
    /// convert template ast node to intermediate representation
    /// the IR format is implementation specific
    /// e.g SSR Codegen and DOM Codegen can have different IR
    fn convert_ir(&self, ast: AstRoot) -> Self::IRNode;
    /// transform will change ir node inplace
    /// usually transform will have multiple passes
    fn transform(&self, node: &mut Self::IRNode);
    /// generate will take optimized ir node and output
    /// desired code format, either String or Binary code
    fn genrate(&self, node: Self::IRNode) -> Self::Output;
}

pub struct CodeGeneratorImpl {
}

impl CodeGenerator for CodeGeneratorImpl {
    type IRNode = IRNode;
    type Output = String;
    fn convert_ir(&self, ast: AstRoot) -> Self::IRNode {
        convert_ast_to_ir(ast)
    }
    fn transform(&self, node: &mut Self::IRNode) {
    }
    fn genrate(&self, node: Self::IRNode) -> Self::Output {
        generate(node);
        unimplemented!("TODO")
    }
}

fn generate(node: IRNode) {
    use IRNode as IR;
    match node {
        IR::Text => generate_text(),
        IR::Interpolation => generate_interpolation(),
        IR::Simple => generate_simple(),
        IR::Compound => generate_compound(),
        IR::Comment => generate_comment(),
        IR::VNode => generate_vnode(),
        IR::Call => generate_call(),
        IR::Object => generate_object(),
        IR::Array => generate_array(),
        IR::Function => genrate_function(),
        IR::Conditional => generate_conditional(),
        IR::Cache => generate_cache(),
        IR::Block => generate_block(),
    }
}

// TODO: implement code gen
fn generate_text() {}
fn generate_interpolation() {}
fn generate_simple() {}
fn generate_compound() {}
fn generate_comment() {}
fn generate_vnode() {}
fn generate_call() {}
fn generate_object() {}
fn generate_array() {}
fn genrate_function() {}
fn generate_conditional() {}
fn generate_cache() {}
fn generate_block() {}
