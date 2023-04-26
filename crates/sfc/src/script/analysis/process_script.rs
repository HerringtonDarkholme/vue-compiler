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

lazy_static! {
    static ref EXPORT_DEFAULT_PATTERN: TsPattern = Pattern::new("export default $EXP", TypeScript);
    static ref EXPORT_SPECIFIER: KindMatcher<TypeScript> =
        KindMatcher::new("export_specifier", TypeScript);
}

// <script> after <script setup>
// we need to move the block up so that `const __default__` is
// declared before being used in the actual component definition
pub fn process_setup_script(ast: TsNode) {}

fn rewrite_export_or_walk_declaration(ast: TsNode) {
    for node in ast.children() {
        match &*node.kind() {
            "export_statement" => rewrite_export(),
            "variable_declaration"
            | "lexical_declaration"
            | "function_declaration"
            | "class_declaration"
            | "enum_declaration" => walk_declaration(),
            _ => (),
        }
    }
}

fn rewrite_export() {
    //   if (node.type === 'ExportDefaultDeclaration') {
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
    //   } else if (node.type === 'ExportNamedDeclaration') {
    //     const defaultSpecifier = node.specifiers.find(
    //       s => s.exported.type === 'Identifier' && s.exported.name === 'default'
    //     ) as ExportSpecifier
    //     if (defaultSpecifier) {
    //       defaultExport = node
    //       // 1. remove specifier
    //       if (node.specifiers.length > 1) {
    //         ctx.s.remove(
    //           defaultSpecifier.start! + scriptStartOffset!,
    //           defaultSpecifier.end! + scriptStartOffset!
    //         )
    //       } else {
    //         ctx.s.remove(
    //           node.start! + scriptStartOffset!,
    //           node.end! + scriptStartOffset!
    //         )
    //       }
    //       if (node.source) {
    //         // export { x as default } from './x'
    //         // rewrite to `import { x as __default__ } from './x'` and
    //         // add to top
    //         ctx.s.prepend(
    //           `import { ${defaultSpecifier.local.name} as ${normalScriptDefaultVar} } from '${node.source.value}'\n`
    //         )
    //       } else {
    //         // export { x as default }
    //         // rewrite to `const __default__ = x` and move to end
    //         ctx.s.appendLeft(
    //           scriptEndOffset!,
    //           `\nconst ${normalScriptDefaultVar} = ${defaultSpecifier.local.name}\n`
    //         )
    //       }
    //     }
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

// NOTE: declare xx is ambient_declaration in tree-sitter
fn walk_declaration() {
    //   } else if (
    //     (node.type === 'VariableDeclaration' ||
    //       node.type === 'FunctionDeclaration' ||
    //       node.type === 'ClassDeclaration' ||
    //       node.type === 'TSEnumDeclaration') &&
    //     !node.declare
    //   ) {
    //     walkDeclaration(
    //       'script',
    //       node,
    //       scriptBindings,
    //       vueImportAliases,
    //       hoistStatic
    //     )
    //   }
}
