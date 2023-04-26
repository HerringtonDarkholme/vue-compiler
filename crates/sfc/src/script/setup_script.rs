/// process setup script
/// If both setup script and normal script blocks exist, we will merge them into one script block.
use smallvec::SmallVec;
use super::parse_script::parse_ts;
use crate::{SfcDescriptor, SfcScriptBlock};
use super::{SfcScriptCompileOptions, inject_css_vars, apply_ref_transform};
use super::analysis::{
    collect_normal_import, collect_setup_assets, process_normal_script, process_setup_script,
};
use super::setup_context::SetupScriptContext;
use rustc_hash::FxHashMap;

pub fn compile_setup_scripts<'a, 'b>(
    scripts: &'b mut SmallVec<[SfcScriptBlock<'a>; 1]>,
    sfc: &'b SfcDescriptor<'a>,
    options: &'b SfcScriptCompileOptions<'a>,
) -> SfcScriptBlock<'a> {
    let mut context = SetupScriptContext::new(sfc, options);
    let (script, script_setup) = split_script(scripts);
    // 0. parse both <script> and <script setup> blocks
    let script_ast = script.map(|s| parse_ts(s.block.source));
    let script_setup_ast = script_setup
        .map(|s| parse_ts(s.block.source))
        .expect("should always have script setup");
    // 1.1 walk import delcarations of <script>
    if let Some(script_ast) = &script_ast {
        collect_normal_import(&mut context, script_ast.root());
    }
    // 1.2 walk import declarations of <script setup>
    collect_setup_assets(&mut context, script_setup_ast.root());
    // 1.3 resolve possible user import alias of `ref` and `reactive`
    let _vue_import_aliases: FxHashMap<_, _> = context
        .analysis
        .user_imports
        .values()
        .filter_map(|import| {
            if import.source == "vue" {
                Some((import.imported, import.local))
            } else {
                None
            }
        })
        .collect();

    // 2.1 process normal <script> body
    if let Some(script_ast) = &script_ast {
        process_normal_script(script_ast.root());
    }
    // 2.2 process <script setup> body
    process_setup_script(script_setup_ast.root());

    apply_ref_transform();
    extract_runtime_code();
    check_invalid_scope_refs();
    remove_non_script_content();
    analyze_binding_metadata();
    inject_css_vars(&mut scripts[0], &sfc.css_vars, options);
    finalize_setup_arg();
    generate_return_stmt();
    finalize_default_export();
    todo!()
}

fn split_script<'a, 'b>(
    scripts: &'b mut SmallVec<[SfcScriptBlock<'a>; 1]>,
) -> (
    Option<&'b SfcScriptBlock<'a>>,
    Option<&'b SfcScriptBlock<'a>>,
) {
    debug_assert!(scripts.len() <= 2);
    let normal = scripts.iter().find(|s| !s.is_setup());
    let setup = scripts.iter().find(|s| s.is_setup());
    (normal, setup)
}

// props and emits
fn extract_runtime_code() {}
// check useOptions does not refer to setup scipe
fn check_invalid_scope_refs() {}
fn remove_non_script_content() {}
fn analyze_binding_metadata() {}
fn finalize_setup_arg() {}
fn generate_return_stmt() {}
fn finalize_default_export() {}
