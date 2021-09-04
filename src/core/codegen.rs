use super::ir_converter::IRNode;
use std::fmt::{Result, Write};

pub trait CodeGenerator {
    type IRNode;
    type Output;
    /// generate will take optimized ir node and output
    /// desired code format, either String or Binary code
    fn generate(&self, node: Self::IRNode) -> Self::Output;
}

pub fn generate(node: IRNode) {
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
        IR::Function => generate_function(),
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
fn generate_function() {}
fn generate_conditional() {}
fn generate_cache() {}
fn generate_block() {}

pub trait CodeGenWrite: Write {
    fn write_hyphenated(&mut self, s: &str) -> Result {
        // JS word boundary is `\w`: `[a-zA-Z0-9-]`.
        // https://javascript.info/regexp-boundary
        // str.replace(/\B([A-Z])/g, '-$1').toLowerCase()
        let mut is_boundary = true;
        for c in s.chars() {
            if !is_boundary && c.is_ascii_uppercase() {
                self.write_char('-')?;
                self.write_char((c as u8 - b'A' + b'a') as char)?;
            } else {
                self.write_char(c)?;
            }
            is_boundary = !c.is_ascii_alphanumeric() && c != '_';
        }
        Ok(())
    }
}
