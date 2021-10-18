use smallvec::SmallVec;
use rslint_parser::{SyntaxNodeExt, parse_module};

use crate::{SfcDescriptor, SfcScriptBlock, SfcTemplateCompileOptions};

pub struct SfcScriptCompileOptions<'a> {
    /// Scope ID for prefixing injected CSS varialbes.
    /// This must be consistent with the `id` passed to `compileStyle`.
    pub id: String,
    /// Production mode. Used to determine whether to generate hashed CSS variables
    pub is_prod: bool,
    /// Enable/disable source map. Defaults to true.
    pub source_map: bool,
    /// (Experimental) Enable syntax transform for using refs without `.value`
    /// https://github.com/vuejs/rfcs/discussions/369
    /// @default false
    pub ref_transform: bool,
    /// (Experimental) Enable syntax transform for destructuring from defineProps()
    /// https://github.com/vuejs/rfcs/discussions/394
    /// @default false
    pub props_destructure_transform: bool,
    /// Compile the template and inline the resulting render function
    /// directly inside setup().
    /// - Only affects `<script setup>`
    /// - This should only be used in production because it prevents the template
    /// from being hot-reloaded separately from component state.
    pub inline_template: bool,
    /// Options for template compilation when inlining. Note these are options that
    /// would normally be pased to `compiler-sfc`'s own `compileTemplate()`, not
    /// options passed to `compiler-dom`.
    pub template_options: Option<SfcTemplateCompileOptions<'a>>,
}

// struct ImportBinding<'a> {
//     is_type: bool,
//     imported: &'a str,
//     source: &'a str,
//     is_from_wsetup: bool,
//     is_used_in_template: bool,
// }

pub fn compile_script<'a>(
    mut sfc: SfcDescriptor<'a>,
    _options: SfcScriptCompileOptions<'a>,
) -> SfcScriptBlock<'a> {
    process_normal_script(&mut sfc.scripts);
    parse_script_setup();
    apply_ref_transform();
    extract_runtime_code();
    check_invalid_scope_refs();
    remove_non_script_content();
    analyze_binding_metadata();
    inject_css_vars();
    finalize_setup_arg();
    generate_return_stmt();
    finalize_default_export();
    sfc.scripts.pop().unwrap()
}

fn process_normal_script(scripts: &mut SmallVec<[SfcScriptBlock; 1]>) {
    debug_assert!(scripts.len() <= 2);
    let normal = match scripts.iter_mut().find(|s| !s.is_setup()) {
        Some(script) => script,
        None => return,
    };
    let content = parse_module(normal.block.content, 0);
    if !content.errors().is_empty() {
        todo!()
    }
    let module = content
        .syntax()
        .try_to::<rslint_parser::ast::Module>()
        .unwrap();
    for _item in module.items() {
        // import declration
        // export default
        // export named
        // declaration
        todo!()
    }
}
fn parse_script_setup() {}
fn apply_ref_transform() {}
// props and emits
fn extract_runtime_code() {}
// check useOptions does not refer to setup scipe
fn check_invalid_scope_refs() {}
fn remove_non_script_content() {}
fn analyze_binding_metadata() {}
fn inject_css_vars() {}
fn finalize_setup_arg() {}
fn generate_return_stmt() {}
fn finalize_default_export() {}
