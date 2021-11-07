#![deny(clippy::all)]

use napi_derive::{napi};
use napi::bindgen_prelude::*;
use compiler::compiler::{BaseCompiler, TemplateCompiler};
use compiler::error::VecErrorHandler;
use dom::{get_dom_pass, compile_option};
use std::rc::Rc;

#[cfg(all(
    any(windows, unix),
    target_arch = "x86_64",
    not(target_env = "musl"),
    not(debug_assertions)
))]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

// struct AsyncTask(u32);

// impl Task for AsyncTask {
//   type Output = u32;
//   type JsValue = JsNumber;

//   fn compute(&mut self) -> Result<Self::Output> {
//     use std::thread::sleep;
//     use std::time::Duration;
//     sleep(Duration::from_millis(self.0 as u64));
//     Ok(self.0 * 2)
//   }

//   fn resolve(self, env: Env, output: Self::Output) -> Result<Self::JsValue> {
//     env.create_uint32(output)
//   }
// }

/// caller should guarantee buffer could convert to valid utf8 string
#[napi]
fn compile_sync_buffer(source: Buffer) -> String {
    let source = std::str::from_utf8(source.as_ref()).unwrap();
    compile(source)
}

#[napi]
fn compile_sync(source: String) -> String {
    compile(&source)
}

fn compile(source: &str) -> String {
    let sfc_info = Default::default();
    let err_handler = VecErrorHandler::default();
    let option = compile_option(Rc::new(err_handler));
    let dest = Vec::new;
    let compiler = BaseCompiler::new(dest, get_dom_pass, option);
    let ret = compiler.compile(source, &sfc_info).unwrap();
    String::from_utf8(ret).unwrap()
}
