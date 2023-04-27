use super::{TsNode, TsPattern, TypeScript};
use crate::script::{apply_ref_transform};
use lazy_static::lazy_static;
use ast_grep_core::{Pattern, Matcher, matcher::KindMatcher};

pub fn process_normal_script(ast: TsNode) {
    rewrite_export_or_walk_declaration(ast);
    // apply reactivity transform
    apply_ref_transform();
    move_script_before_setup();
}

fn move_script_before_setup() {
    // TODO: do we really need this?
}

// <script> after <script setup>
// we need to move the block up so that `const __default__` is
// declared before being used in the actual component definition
pub fn process_setup_script(ast: TsNode) {}

fn rewrite_export_or_walk_declaration(ast: TsNode) {
    for node in ast.children() {
        match &*node.kind() {
            "export_statement" => rewrite_export(node),
            _ => try_walk_declaration(node),
        }
    }
}

lazy_static! {
    static ref EXPORT_DEFAULT_PATTERN: TsPattern = Pattern::new("export default $EXP", TypeScript);
    static ref EXPORT_NS_PATTERN: TsPattern = Pattern::contextual(
        "export * as default from 'a'",
        "namespace_export",
        TypeScript
    )
    .unwrap();
    static ref EXPORT_SPECIFIER: KindMatcher<TypeScript> =
        KindMatcher::new("export_specifier", TypeScript);
}

fn rewrite_export(node: TsNode) {
    if let Some(nm) = EXPORT_DEFAULT_PATTERN.match_node(node.clone()) {
        rewrite_default_export(nm.into());
        return;
    }
    if node.find(&*EXPORT_NS_PATTERN).is_some() {
        // who wrote these weird stuffs???
        todo!()
    }
    let specifiers: Vec<_> = node.find_all(&*EXPORT_SPECIFIER).collect();
    if let Some(nm) = specifiers.iter().find(|n| {
        if let Some(name) = n.field("alias").or_else(|| n.field("name")) {
            name.text() == "default"
        } else {
            false
        }
    }) {
        //  defaultExport = node // TODO: add defaultNode

        // 1. remove specifier
        let edit = if specifiers.len() > 1 {
            nm.remove()
        } else {
            node.remove()
        };
        // export { x as default } from './x'
        // rewrite to `import { x as __default__ } from './x'` and
        // add to top
        if let Some(src) = node.field("source") {
            // ctx.s.prepend(
            //   `import { ${defaultSpecifier.local.name} as ${normalScriptDefaultVar} } from '${node.source.value}'\n`
            // )
        } else {
            // export { x as default }
            // rewrite to `const __default__ = x` and move to end

            //    ctx.s.appendLeft(
            //      scriptEndOffset!,
            //      `\nconst ${normalScriptDefaultVar} = ${defaultSpecifier.local.name}\n`
            //    )
        }
    }

    if let Some(decl) = node.field("declaration") {
        try_walk_declaration(decl);
    }
    //   if (node.type === 'ExportNamedDeclaration') {
    //     if (node.declaration) {
    //       walkDeclaration(
    //         'script',
    //         node.declaration,
    //         scriptBindings,
    //         vueImportAliases,
    //         hoistStatic
    //       )
    //     }
    //   }
}

fn rewrite_default_export(_node: TsNode) {
    //     // export default
    //     defaultExport = node

    //     // check if user has manually specified `name` or 'render` option in
    //     // export default
    //     // if has name, skip name inference
    //     // if has render and no template, generate return object instead of
    //     // empty render function (#4980)
    //     let optionProperties
    //     if (defaultExport.declaration.type === 'ObjectExpression') {
    //       optionProperties = defaultExport.declaration.properties
    //     } else if (
    //       defaultExport.declaration.type === 'CallExpression' &&
    //       defaultExport.declaration.arguments[0] &&
    //       defaultExport.declaration.arguments[0].type === 'ObjectExpression'
    //     ) {
    //       optionProperties = defaultExport.declaration.arguments[0].properties
    //     }
    //     if (optionProperties) {
    //       for (const p of optionProperties) {
    //         if (
    //           p.type === 'ObjectProperty' &&
    //           p.key.type === 'Identifier' &&
    //           p.key.name === 'name'
    //         ) {
    //           ctx.hasDefaultExportName = true
    //         }
    //         if (
    //           (p.type === 'ObjectMethod' || p.type === 'ObjectProperty') &&
    //           p.key.type === 'Identifier' &&
    //           p.key.name === 'render'
    //         ) {
    //           // TODO warn when we provide a better way to do it?
    //           ctx.hasDefaultExportRender = true
    //         }
    //       }
    //     }

    //     // export default { ... } --> const __default__ = { ... }
    //     const start = node.start! + scriptStartOffset!
    //     const end = node.declaration.start! + scriptStartOffset!
    //     ctx.s.overwrite(start, end, `const ${normalScriptDefaultVar} = `)
}

// NOTE: declare xx is ambient_declaration in tree-sitter
fn try_walk_declaration(node: TsNode) {
    match &*node.kind() {
        "variable_declaration" | "lexical_declaration" => todo!(),
        "function_declaration" => todo!(),
        "class_declaration" => todo!(),
        "enum_declaration" => todo!(),
        _ => (), // passs
    }
    //     walkDeclaration(
    //       'script',
    //       node,
    //       scriptBindings,
    //       vueImportAliases,
    //       hoistStatic
    //     )
    //   }
}
