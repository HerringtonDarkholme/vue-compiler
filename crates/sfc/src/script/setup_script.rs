/// process setup script
/// If both setup script and normal script blocks exist, we will merge them into one script block.
use smallvec::SmallVec;
use super::parse_script::{parse_ts, TsNode, TsPattern, TypeScript};
use crate::{SfcDescriptor, SfcScriptBlock};
use super::{SfcScriptCompileOptions, inject_css_vars, apply_ref_transform};
use super::setup_context::SetupScriptContext;
use ast_grep_core::{Pattern, matcher::KindMatcher};
use lazy_static::lazy_static;

pub fn process_setup_scripts<'a, 'b>(
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
    collect_setup_assets(script_setup_ast.root());
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

lazy_static! {
    static ref DEFAULT_PAT: TsPattern =
        Pattern::contextual("import $LOCAL from 'a'", "import_clause", TypeScript).unwrap();
    static ref NAMESPACE_PAT: TsPattern = Pattern::contextual(
        "import * as $LOCAL from 'a'",
        "namespace_imports",
        TypeScript
    )
    .unwrap();
}

fn collect_normal_import(ctx: &mut SetupScriptContext, script_ast: TsNode) {
    for import in script_ast.find_all(KindMatcher::new("import_statement", TypeScript)) {
        for default in import.find_all(&*DEFAULT_PAT) {
            let source = ctx.script_text(&default);
            let local_node = default.get_env().get_match("LOCAL").unwrap();
            let local = ctx.script_text(local_node);
            let is_type = local_node
                .prev()
                .map(|n| n.text() == "type")
                .unwrap_or(false);
            ctx.register_user_import(source, local, "default", is_type, false);
        }
        // import { type A } from 'xxx' or import type {A} from 'xxx'
        for named in import.find_all(KindMatcher::new("import_specifier", TypeScript)) {
            let source = ctx.script_text(&named);
            let imported_node = named.field("name").unwrap();
            let local = ctx.script_text(&named.field("alias").unwrap_or(imported_node.clone()));
            let imported = ctx.script_text(&imported_node);
            let is_type =
                // { type A } from 'xxx'
                imported_node.prev().map(|n| n.text() == "type").or_else(|| {
                    // type { A } from 'xxx'
                    let named_imports = named.parent()?;
                    Some(named_imports.prev()?.text() == "type")
                }).unwrap_or(false);
            ctx.register_user_import(source, local, imported, is_type, false);
        }
        for ns in import.find_all(&*NAMESPACE_PAT) {
            let source = ctx.script_text(&ns);
            let local = ctx.script_text(ns.get_env().get_match("LOCAL").unwrap());
            // TODO: babel does not support `import type * as ns from 'bb'`
            let is_type = ns.prev().map(|n| n.text() == "type").unwrap_or(false);
            ctx.register_user_import(source, local, "*", is_type, false);
        }
    }
}

fn collect_setup_assets(setup_ast: TsNode) {}
// props and emits
fn extract_runtime_code() {}
// check useOptions does not refer to setup scipe
fn check_invalid_scope_refs() {}
fn remove_non_script_content() {}
fn analyze_binding_metadata() {}
fn finalize_setup_arg() {}
fn generate_return_stmt() {}
fn finalize_default_export() {}

#[cfg(test)]
mod test {
    use super::*;
}
