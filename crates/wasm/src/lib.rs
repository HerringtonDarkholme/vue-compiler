use wasm_bindgen::prelude::*;
use compiler::compiler::{BaseCompiler, TemplateCompiler, get_base_passes};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(js_name = baseCompile)]
pub fn base_compile(source: &str) -> String {
    let sfc_info = Default::default();
    let option = Default::default();
    let dest = Vec::new;
    let mut compiler = BaseCompiler::new(dest, get_base_passes, option);
    let ret = compiler.compile(source, &sfc_info).unwrap();
    String::from_utf8(ret).unwrap()
}
