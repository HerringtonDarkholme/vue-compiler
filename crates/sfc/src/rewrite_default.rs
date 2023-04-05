use crate::script::parse_script::{parse_ts, TsNode, TypeScript, TsPattern};
use ast_grep_core::{Pattern, Matcher, matcher::KindMatcher};
use lazy_static::lazy_static;

pub fn rewrite_default(input: String, as_var: &'static str) -> String {
    let mut root = parse_ts(&input);
    let node = root.root();
    if let Some(mt) = export_default(node.clone()) {
        if mt.kind() == "class_declaration" {
            // TODO: use pattern instead
            let edit = mt
                .replace(&*EXPORT_DEFAULT_PATTERN, "$EXP")
                .expect("should have exp");
            let class_name = mt.field("name").unwrap().text().to_string();
            root.edit(edit).expect("should work");
            let original = root.generate();
            format!("{original}\nconst {as_var} = {class_name}")
        } else {
            // TODO
            let edit = mt
                .replace(&*EXPORT_DEFAULT_PATTERN, &*format!("const {as_var} = $EXP"))
                .expect("should have exp");
            root.edit(edit).expect("should work");
            root.generate()
        }
    } else if let Some(_named_export) = named_export_default(node.clone()) {
        todo!()
    } else {
        format!("{input}\nconst {as_var} = {{}}")
    }
}

lazy_static! {
    static ref EXPORT_DEFAULT_PATTERN: TsPattern = Pattern::new("export default $EXP", TypeScript);
    static ref EXPORT_SPECIFIER: KindMatcher<TypeScript> =
        KindMatcher::new("export_specifier", TypeScript);
}

// `export default xxx`
fn export_default(ast: TsNode) -> Option<TsNode> {
    ast.children()
        .find_map(|n| EXPORT_DEFAULT_PATTERN.match_node(n))
        .map(|e| e.get_node().clone())
}
// export { a as default }
// export { default } from 'xxx'
fn named_export_default(ast: TsNode) -> Option<TsNode> {
    ast.find_all(&*EXPORT_SPECIFIER).find_map(|n| {
        let name = n.field("alias").or_else(|| n.field("name"))?;
        if name.text() == "default" {
            Some(n.get_node().clone())
        } else {
            None
        }
    })
}
