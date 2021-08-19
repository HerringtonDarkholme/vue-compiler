mod sfc;
mod template;
mod core;

use sfc::parse_sfc;
use template::compile_template;

use std::path::PathBuf;

fn compile_script() -> String {
    unimplemented!();
}

fn compile_style() -> String {
    unimplemented!();
}

fn bundle(template: String, script: String, style: String) -> String {
    unimplemented!();
}

fn compile_sfc(source: String) -> String {
    let sfc_descriptor = parse_sfc(source);
    let template = compile_template(sfc_descriptor.template);
    let script = compile_script();
    let style = compile_style();
    bundle(template, script, style)
}
