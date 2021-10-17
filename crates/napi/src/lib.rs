#![deny(clippy::all)]

use napi_derive::napi;
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

// #[module_exports]
// fn init(mut exports: JsObject) -> Result<()> {
//   exports.create_named_method("sleep", sleep)?;
//   Ok(())
// }

#[napi]
fn compile_sync(source: String) -> String {
    let sfc_info = Default::default();
    let err_handler = VecErrorHandler::default();
    let option = compile_option(Rc::new(err_handler));
    let dest = Vec::new;
    let compiler = BaseCompiler::new(dest, get_dom_pass, option);
    let ret = compiler.compile(&source, &sfc_info).unwrap();
    String::from_utf8(ret).unwrap()
}

// #[js_function(1)]
// fn sleep(ctx: CallContext) -> Result<JsObject> {
//   let argument: u32 = ctx.get::<JsNumber>(0)?.try_into()?;
//   let task = AsyncTask(argument);
//   let async_task = ctx.env.spawn(task)?;
//   Ok(async_task.promise_object())
// }
