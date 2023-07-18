use super::{TsNode, TsPattern, TypeScript};
use super::SetupScriptContext;
use ast_grep_core::{Pattern, matcher::KindMatcher};
use lazy_static::lazy_static;
use std::borrow::Cow;

struct ImportNodes<'a> {
    source: TsNode<'a>,
    local: TsNode<'a>,
    imported: Option<TsNode<'a>>,
    is_type: bool,
}

lazy_static! {
    static ref DEFAULT_PAT: TsPattern =
        Pattern::contextual("import $LOCAL from 'a'", "import_clause", TypeScript).unwrap();
    static ref NAMED_PAT: KindMatcher<TypeScript> =
        KindMatcher::new("import_specifier", TypeScript);
    static ref NAMESPACE_PAT: TsPattern = Pattern::contextual(
        "import * as $LOCAL from 'a'",
        "namespace_imports",
        TypeScript
    )
    .unwrap();
}

fn collect_one_import(import: TsNode) -> impl Iterator<Item = ImportNodes> {
    let src = import.field("source").unwrap().child(0).unwrap();
    let source = src.clone();
    let default_nodes = import.find_all(&*DEFAULT_PAT).flat_map(move |default| {
        let local = default.get_env().get_match("LOCAL")?.clone();
        let is_type = local.prev().map(|n| n.text() == "type").unwrap_or(false);
        Some(ImportNodes {
            source: source.clone(),
            local,
            imported: None, // default
            is_type,
        })
    });
    let source = src.clone();
    // import { type A } from 'xxx' or import type {A} from 'xxx'
    let named_nodes = import.find_all(&*NAMED_PAT).flat_map(move |named| {
        let imported = named.field("name")?;
        let local = named.field("alias").unwrap_or_else(|| imported.clone());
        let is_type =
            // { type A } from 'xxx'
            imported.prev().map(|n| n.text() == "type").or_else(|| {
                // type { A } from 'xxx'
                let named_imports = named.parent()?;
                Some(named_imports.prev()?.text() == "type")
            }).unwrap_or(false);
        Some(ImportNodes {
            source: source.clone(),
            local,
            imported: Some(imported),
            is_type,
        })
    });
    let source = src.clone();
    let namespace_nodes = import.find_all(&*NAMESPACE_PAT).flat_map(move |ns| {
        let local = ns.get_env().get_match("LOCAL")?.clone();
        let imported = local.prev()?.prev();
        // TODO: babel does not support `import type * as ns from 'bb'`
        let is_type = ns.prev().map(|n| n.text() == "type").unwrap_or(false);
        Some(ImportNodes {
            source: source.clone(),
            local,
            imported,
            is_type,
        })
    });
    default_nodes.chain(namespace_nodes).chain(named_nodes)
}

pub fn collect_normal_import(ctx: &mut SetupScriptContext, script_ast: TsNode) {
    for import in script_ast.find_all(KindMatcher::new("import_statement", TypeScript)) {
        for imports in collect_one_import(import.into()) {
            let ImportNodes {
                source,
                local,
                imported,
                is_type,
            } = imports;
            ctx.register_script_import(source, local, imported, is_type);
        }
    }
}

fn hoist_node() {
    // // import declarations are moved to top
    // hoistNode(node)
    // use magic string to move import statement to the top
    // magic-string has quite strange move method...
}
fn dedupe_imports() {
    // // dedupe imports
    // let removed = 0
    // const removeSpecifier = (i: number) => {
    //   const removeLeft = i > removed
    //   removed++
    //   const current = node.specifiers[i]
    //   const next = node.specifiers[i + 1]
    //   s.remove(
    //     removeLeft
    //       ? node.specifiers[i - 1].end! + startOffset
    //       : current.start! + startOffset,
    //     next && !removeLeft
    //       ? next.start! + startOffset
    //       : current.end! + startOffset
    //   )
    // }
}

const DEFINE_PROPS: &str = "defineProps";
const DEFINE_EMITS: &str = "defineEmits";
const DEFINE_EXPOSE: &str = "defineExpose";

fn is_macro_import<'a>(source: &TsNode<'a>, imported: &Option<TsNode<'a>>) -> Option<Cow<'a, str>> {
    let imported = imported.as_ref()?;
    let source = source.text();
    let imported = imported.text();
    if source == "vue"
        && (imported == DEFINE_PROPS || imported == DEFINE_EMITS || imported == DEFINE_EXPOSE)
    {
        Some(imported)
    } else {
        None
    }
}

fn register(ctx: &mut SetupScriptContext, import: TsNode) {
    for imports in collect_one_import(import) {
        let ImportNodes {
            source,
            local,
            imported,
            is_type,
        } = imports;
        if let Some(importee) = is_macro_import(&source, &imported) {
            ctx.warn(format!(
                "`{importee}`  is a compiler macro and no longer needs to be imported."
            ));
            // TODO: remove specifier
        } else if let Some(existing) = ctx.get_registered_import(&local.text()) {
            let is_duplicate = existing.source == source.text()
                && imported
                    .map(|n| n.text() == existing.imported)
                    .unwrap_or_else(|| existing.imported == "default");
            if is_duplicate {
                // TODO: remove is_duplicate
            } else {
                ctx.error(format!(
                    "different imports aliased to same local name `{}`",
                    local.text()
                ));
            }
        } else {
            ctx.register_setup_import(source, local, imported, is_type);
        }
    }
}

fn remove_node_if_dupe() {
    // if (node.specifiers.length && removed === node.specifiers.length) {
    //   s.remove(node.start! + startOffset, node.end! + startOffset)
    // }
}

pub fn collect_setup_assets(ctx: &mut SetupScriptContext, setup_ast: TsNode) {
    for import in setup_ast.find_all(KindMatcher::new("import_statement", TypeScript)) {
        hoist_node();
        dedupe_imports();
        register(ctx, import.into());
        remove_node_if_dupe();
    }
}

#[cfg(test)]
mod test {
    use super::*;
}
