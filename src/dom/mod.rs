use super::core::{
    base_compile, CodeGenerator,
}

struct DomCodeGenerator {}
impl CodeGenerator for DomCodeGenerator {
}

pub fn compile_dom(source: &str)
