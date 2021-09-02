use super::ir_converter::IRNode;

pub trait CodeGenerator {
    type IRNode;
    type Output;
    /// generate will take optimized ir node and output
    /// desired code format, either String or Binary code
    fn generate(&self, node: Self::IRNode) -> Self::Output;
}

pub fn generate(node: IRNode) {
    use IRNode::*;
    match node {
        Text => generate_text(),
        Interpolation => generate_interpolation(),
        Simple => generate_simple(),
        Compound => generate_compound(),
        Comment => generate_comment(),
        VNode => generate_vnode(),
        Call => generate_call(),
        Object => generate_object(),
        Array => generate_array(),
        Function => generate_function(),
        Conditional => generate_conditional(),
        Cache => generate_cache(),
        Block => generate_block(),
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
fn generate_function() {}
fn generate_conditional() {}
fn generate_cache() {}
fn generate_block() {}
