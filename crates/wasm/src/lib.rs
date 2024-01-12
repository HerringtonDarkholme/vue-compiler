#![allow(clippy::unused_unit)]
// https://github.com/rustwasm/wasm-bindgen/issues/2774

use wasm_bindgen::prelude::*;
use compiler::compiler::{BaseCompiler, TemplateCompiler};
use compiler::error::NoopErrorHandler;
use dom::{get_dom_pass, compile_option};

use std::rc::Rc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(js_name = baseCompile)]
pub fn base_compile(source: &str) -> String {
    let sfc_info = Default::default();
    let option = compile_option(Rc::new(NoopErrorHandler));
    let dest = Vec::new;
    let compiler = BaseCompiler::new(dest, get_dom_pass, option);
    let ret = compiler.compile(source, &sfc_info).unwrap();
    String::from_utf8(ret).unwrap()
}
