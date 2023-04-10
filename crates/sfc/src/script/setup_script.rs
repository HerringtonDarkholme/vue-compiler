use smallvec::SmallVec;
use super::parse_script::parse_ts;
use crate::{SfcDescriptor, SfcScriptBlock};
use super::{SfcScriptCompileOptions, inject_css_vars, apply_ref_transform};

pub fn process_setup_scripts<'a>(
    scripts: &mut SmallVec<[SfcScriptBlock<'a>; 1]>,
    sfc: &SfcDescriptor<'a>,
    options: SfcScriptCompileOptions<'a>,
) -> SfcScriptBlock<'a> {
    process_normal_script(scripts);
    parse_script_setup();
    apply_ref_transform();
    extract_runtime_code();
    check_invalid_scope_refs();
    remove_non_script_content();
    analyze_binding_metadata();
    inject_css_vars(&mut scripts[0], &sfc.css_vars, &options);
    finalize_setup_arg();
    generate_return_stmt();
    finalize_default_export();
    todo!()
}

fn process_normal_script(scripts: &mut SmallVec<[SfcScriptBlock; 1]>) {
    debug_assert!(scripts.len() <= 2);
    let normal = match scripts.iter_mut().find(|s| !s.is_setup()) {
        Some(script) => script,
        None => return,
    };
    let _content = parse_ts(normal.block.source);
    // for _item in module.items() {
    //     // import declration
    //     // export default
    //     // export named
    //     // declaration
    // }
}

fn parse_script_setup() {}
// props and emits
fn extract_runtime_code() {}
// check useOptions does not refer to setup scipe
fn check_invalid_scope_refs() {}
fn remove_non_script_content() {}
fn analyze_binding_metadata() {}
fn finalize_setup_arg() {}
fn generate_return_stmt() {}
fn finalize_default_export() {}
