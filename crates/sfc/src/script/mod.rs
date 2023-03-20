mod parse_script;

use smallvec::SmallVec;
use compiler::{BindingMetadata, BindingTypes};
use parse_script::{parse_ts, TsAst, TsNode, TypeScript};
use ast_grep_core::{Pattern, Matcher};
use rustc_hash::FxHashMap;

use crate::{SfcDescriptor, SfcScriptBlock, SfcTemplateCompileOptions};
use crate::rewrite_default;
use crate::style::css_vars::gen_normal_script_css_vars_code;

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

// struct ImportBinding<'a> {
//     is_type: bool,
//     imported: &'a str,
//     source: &'a str,
//     is_from_wsetup: bool,
//     is_used_in_template: bool,
// }

pub fn compile_script<'a>(
    sfc: SfcDescriptor<'a>,
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
        process_single_script(&mut scripts, sfc, options)
    } else {
        process_setup_scripts(&mut scripts, sfc, options)
    }
}

fn process_single_script<'a>(
    scripts: &mut SmallVec<[SfcScriptBlock<'a>; 1]>,
    sfc: SfcDescriptor<'a>,
    options: SfcScriptCompileOptions<'a>,
) -> Option<SfcScriptBlock<'a>> {
    debug_assert!(scripts.len() == 1);
    let is_ts = scripts
        .iter()
        .any(|s| s.get_lang() == "ts" || s.get_lang() == "tsx");
    let mut script = scripts.pop().unwrap();
    // do not process no-js script blocks
    if script.get_lang() != "jsx" && !is_ts {
        return Some(script);
    }
    // 1. parse ast
    let module = parse_ts(&script.block.content);
    // 2. build bindingMetadata
    let bindings = analyze_script_bindings(module);
    script.bindings = Some(bindings);
    // 3. transform ref
    apply_ref_transform();
    // 4. inject css vars
    inject_css_vars(&mut script, &sfc.css_vars, &options);
    Some(script)
}

fn analyze_script_bindings(ast: TsAst) -> BindingMetadata<'static> {
    let pattern = Pattern::new("export default { $$$ }", TypeScript);
    let root = ast.root();
    let mut children = root.children();
    if let Some(node_match) = children.find_map(|n| pattern.match_node(n)) {
        let object = node_match
            .get_node()
            .field("value")
            .expect("should have value");
        analyze_bindings_from_options(object)
    } else {
        BindingMetadata::default()
    }
}

fn analyze_bindings_from_options(node: TsNode) -> BindingMetadata<'static> {
    let mut map = FxHashMap::default();
    let props_pattern = Pattern::contextual("{props: $P}", "pair", TypeScript).unwrap();
    let inject_pattern = Pattern::contextual("{inject: $I}", "pair", TypeScript).unwrap();
    let method_pattern = Pattern::contextual("{methods: $M}", "pair", TypeScript).unwrap();
    let computed_pattern = Pattern::contextual("{computed: $C}", "pair", TypeScript).unwrap();
    for child in node.children() {
        if let Some(n) = props_pattern.match_node(child.clone()) {
            let key = get_keys(&n);
            map.insert(key, BindingTypes::Props);
        } else if let Some(n) = inject_pattern.match_node(child.clone()) {
            let key = get_keys(&n);
            map.insert(key, BindingTypes::Options);
        } else if let Some(n) = method_pattern.match_node(child.clone()) {
            let key = get_keys(&n);
            map.insert(key, BindingTypes::Options);
        } else if let Some(n) = computed_pattern.match_node(child.clone()) {
            let key = get_keys(&n);
            map.insert(key, BindingTypes::Options);
        }
    }
    // #3270, #3275
    // mark non-script-setup so we don't resolve components/directives from these
    BindingMetadata::new_option(map)
}

fn get_keys(_n: &TsNode) -> &'static str {
    todo!()
}

fn process_setup_scripts<'a>(
    scripts: &mut SmallVec<[SfcScriptBlock<'a>; 1]>,
    sfc: SfcDescriptor<'a>,
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
    let _content = parse_ts(&normal.block.content);
    // for _item in module.items() {
    //     // import declration
    //     // export default
    //     // export named
    //     // declaration
    // }
}

const DEFAULT_VAR: &str = "__default__";

fn parse_script_setup() {}
fn apply_ref_transform() {}
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
    let content = &script.block.content;
    let content = rewrite_default(content.to_string(), DEFAULT_VAR);
    let css_vars_code = gen_normal_script_css_vars_code(
        css_vars,
        script.bindings.clone().unwrap(),
        &options.id,
        options.is_prod,
    );
    script.block.content = format!("{content}{css_vars_code}\nexport default {DEFAULT_VAR}");
}
fn finalize_setup_arg() {}
fn generate_return_stmt() {}
fn finalize_default_export() {}
