mod parse_script;
mod vanilla_script;

use smallvec::SmallVec;
use compiler::SFCInfo;
use parse_script::parse_ts;
use vanilla_script::process_single_script;
use crate::{SfcDescriptor, SfcScriptBlock, SfcTemplateCompileOptions};
use crate::rewrite_default;
use crate::style::css_vars::gen_normal_script_css_vars_code;

#[derive(Default)]
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
    pub reactivity_transform: bool,
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

impl<'a> SfcScriptCompileOptions<'a> {
    pub fn new(s: &str) -> Self {
        Self {
            id: s.into(),
            ..Default::default()
        }
    }
}

// struct ImportBinding<'a> {
//     is_type: bool,
//     imported: &'a str,
//     source: &'a str,
//     is_from_setup: bool,
//     is_used_in_template: bool,
// }

pub fn compile_script<'a>(
    sfc: &SfcDescriptor<'a>,
    options: SfcScriptCompileOptions<'a>,
) -> Option<SfcScriptBlock<'a>> {
    let mut scripts = sfc.scripts.clone();
    debug_assert!(scripts.len() <= 2);
    if scripts.is_empty() {
        return None;
    }
    debug_assert!(
        !options.id.is_empty(),
        "compileScript requires `id` option."
    );
    // let id = options.id;
    // let scope_id = id.strip_prefix("data-v").unwrap_or(&id);
    // let css_vars = &sfc.css_vars;
    let has_uniform_lang = scripts.len() == 1 || scripts[0].get_lang() == scripts[1].get_lang();
    if !has_uniform_lang {
        // TODO: report error
        return None;
    }
    if !scripts.iter().any(|s| s.is_setup()) {
        Some(process_single_script(&mut scripts, sfc, options))
    } else {
        process_setup_scripts(&mut scripts, sfc, options)
    }
}

fn process_setup_scripts<'a>(
    scripts: &mut SmallVec<[SfcScriptBlock<'a>; 1]>,
    sfc: &SfcDescriptor<'a>,
    options: SfcScriptCompileOptions<'a>,
) -> Option<SfcScriptBlock<'a>> {
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

const DEFAULT_VAR: &str = "__default__";

fn parse_script_setup() {}
fn apply_ref_transform() {
    // nothing! ref transform is deprecated!
}
// props and emits
fn extract_runtime_code() {}
// check useOptions does not refer to setup scipe
fn check_invalid_scope_refs() {}
fn remove_non_script_content() {}
fn analyze_binding_metadata() {}
fn inject_css_vars<'a>(
    script: &mut SfcScriptBlock<'a>,
    css_vars: &[&'a str],
    options: &SfcScriptCompileOptions<'a>,
) {
    let content = &script.block.compiled_content;
    let content = rewrite_default(content.to_string(), DEFAULT_VAR);
    let sfc_info = SFCInfo {
        inline: true,
        slotted: true, // TODO
        binding_metadata: script.bindings.clone().unwrap(),
        scope_id: None,
        self_name: "".into(),
    };
    let css_vars_code = gen_normal_script_css_vars_code(
        css_vars,
        &sfc_info,
        &options.id,
        options.is_prod,
        /* is_ssr*/ false,
    );
    script.block.compiled_content =
        format!("{content}{css_vars_code}\nexport default {DEFAULT_VAR}");
}
fn finalize_setup_arg() {}
fn generate_return_stmt() {}
fn finalize_default_export() {}
