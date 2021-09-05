use super::ir_converter::{ConvertInfo, IRNode};
use std::fmt::{Result, Write};

pub trait CodeGenerator {
    type IR;
    type Output;
    /// generate will take optimized ir node and output
    /// desired code format, either String or Binary code
    fn generate(&self, node: Self::IR) -> Self::Output;
}

pub fn generate<T: ConvertInfo>(node: IRNode<T>) {
    use IRNode as IR;
    match node {
        IR::Text(..) => generate_text(),
        IR::Interpolation => generate_interpolation(),
    }
}

// TODO: implement code gen
fn generate_text() {}
fn generate_interpolation() {}

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
