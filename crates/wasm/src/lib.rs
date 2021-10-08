use wasm_bindgen::prelude::*;
use compiler::compiler::{BaseCompiler, TemplateCompiler, get_base_passes};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn base_compile(source: &str) -> String {
    let sfc_info = Default::default();
    let option = Default::default();
    let passes = get_base_passes(&sfc_info, &option);
    let mut ret = vec![];
    let mut compiler = BaseCompiler::new(&mut ret, passes, option);
    compiler.compile(source).unwrap();
    String::from_utf8(ret).unwrap()
}
