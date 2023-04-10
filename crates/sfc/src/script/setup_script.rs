use smallvec::SmallVec;
use super::parse_script::{parse_ts, TsNode};
use crate::{SfcDescriptor, SfcScriptBlock};
use super::{SfcScriptCompileOptions, inject_css_vars, apply_ref_transform};

pub fn process_setup_scripts<'a>(
    scripts: &mut SmallVec<[SfcScriptBlock<'a>; 1]>,
    sfc: &SfcDescriptor<'a>,
    options: SfcScriptCompileOptions<'a>,
) -> SfcScriptBlock<'a> {
    let (script, script_setup) = split_script(scripts);
    // 0. parse both <script> and <script setup> blocks
    let script_ast = script.map(|s| parse_ts(s.block.source));
    let script_setup_ast = script_setup.map(|s| parse_ts(s.block.source));
    // 1.1 walk import delcarations of <script>
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

fn process_normal_script(script_ast: TsNode) {

    // let content = parse_ts(normal.block.source);
    // for _item in module.items() {
    //     // import declration
    //     // export default
    //     // export named
    //     // declaration
    // }
}

fn parse_setup_script() {}
// props and emits
fn extract_runtime_code() {}
// check useOptions does not refer to setup scipe
fn check_invalid_scope_refs() {}
fn remove_non_script_content() {}
fn analyze_binding_metadata() {}
fn finalize_setup_arg() {}
fn generate_return_stmt() {}
fn finalize_default_export() {}

use std::collections::{HashSet, HashMap};

struct ImportBinding {
    // define fields for ImportBinding struct
}

struct BindingMetadata {
    // define fields for BindingMetadata struct
}

enum BindingTypes {
    // define variants for BindingTypes enum
}

enum PropsDeclType {
    // define variants for PropsDeclType enum
}

enum EmitsDeclType {
    // define variants for EmitsDeclType enum
}

struct PropTypeData {
    // define fields for PropTypeData struct
}

struct SetupBindings {
    binding_metadata: BindingMetadata,
    helper_imports: HashSet<String>,
    user_imports: HashMap<String, ImportBinding>,
    script_bindings: HashMap<String, BindingTypes>,
    setup_bindings: HashMap<String, BindingTypes>,
}

struct ExportRelated {
    default_export: Option<Node>,
    has_default_export_name: bool,
    has_default_export_render: bool,
}

type Node = ();
type ObjectExpression = ();

struct Misc {
    has_define_expose_call: bool,
    has_await: bool,
    has_inlined_ssr_render_fn: bool,
    declared_types: HashMap<String, Vec<String>>,
}

struct PropRelated {
    has_define_props_call: bool,
    type_declared_props: HashMap<String, PropTypeData>,
    props_runtime_decl: Option<Node>,
    props_runtime_defaults: Option<ObjectExpression>,
    props_destructure_decl: Option<Node>,
    props_destructure_rest_id: Option<String>,
    props_type_decl: Option<PropsDeclType>,
    props_type_decl_raw: Option<Node>,
    props_identifier: Option<String>,
    props_destructured_bindings: HashMap<String, HashMap<String, bool>>,
}

struct EmitRelated {
    has_define_emit_call: bool,
    emits_runtime_decl: Option<Node>,
    emits_type_decl: Option<EmitsDeclType>,
    emits_type_decl_raw: Option<Node>,
    emit_identifier: Option<String>,
    type_declared_emits: HashSet<String>,
}

pub struct SetupScriptData {
    bindings: SetupBindings,
    props: PropRelated,
    emits: EmitRelated,
    exports: ExportRelated,
}
