use wasm_bindgen::prelude::*;
use compiler::compiler::{BaseCompiler, TemplateCompiler, get_base_passes};
use std::rc::Rc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(js_name = baseCompile)]
pub fn base_compile(source: &str) -> String {
    let sfc_info = Rc::new(Default::default());
    let option = Default::default();
    let passes = get_base_passes(&sfc_info, &option);
    let mut ret = vec![];
    let mut compiler = BaseCompiler::new(&mut ret, passes, option);
    compiler.compile(source, sfc_info.clone()).unwrap();
    String::from_utf8(ret).unwrap()
}
