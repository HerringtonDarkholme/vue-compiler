use smallvec::SmallVec;
use super::SfcScriptCompileOptions;
use compiler::{BindingMetadata, BindingTypes};
use super::parse_script::{parse_ts, TsNode, TypeScript, TsPattern};
use ast_grep_core::{Pattern, Matcher};
use rustc_hash::FxHashMap;
use lazy_static::lazy_static;

use crate::{SfcDescriptor, SfcScriptBlock};

use super::{inject_css_vars, apply_ref_transform};

use std::ops::Range;

pub fn compile_single_script<'a>(
    scripts: &mut SmallVec<[SfcScriptBlock<'a>; 1]>,
    sfc: &SfcDescriptor<'a>,
    options: SfcScriptCompileOptions<'a>,
) -> SfcScriptBlock<'a> {
    debug_assert!(scripts.len() == 1);
    let is_ts = scripts
        .iter()
        .any(|s| s.get_lang() == "ts" || s.get_lang() == "tsx");
    let mut script = scripts.pop().unwrap();
    // do not process no-js script blocks
    if script.get_lang() != "jsx" && !is_ts {
        return script;
    }
    // 1. build bindingMetadata
    let bindings = analyze_script_bindings(script.block.source);
    script.bindings = Some(bindings);
    // 2. transform ref
    apply_ref_transform();
    // 3. inject css vars
    inject_css_vars(&mut script, &sfc.css_vars, &options);
    script
}

fn analyze_script_bindings(src: &str) -> BindingMetadata {
    // 1. parse ast
    let module = parse_ts(src);
    let root = module.root();
    let pattern = Pattern::new("export default { $$$ }", TypeScript);
    let mut children = root.children();
    let Some(node_match) = children.find_map(|n| pattern.match_node(n)) else {
        return BindingMetadata::default();
    };
    let object = node_match
        .get_node()
        .field("value")
        .expect("should have value");
    analyze_bindings_from_options(object, src)
}

fn analyze_bindings_from_options<'a>(node: TsNode, src: &'a str) -> BindingMetadata<'a> {
    let mut map = FxHashMap::default();
    for child in node.children() {
        let Some((keys, tpe)) = collect_keys_from_option_property(child) else {
            continue;
        };
        for key_range in keys {
            map.insert(&src[key_range], tpe.clone());
        }
    }
    // #3270, #3275
    // mark non-script-setup so we don't resolve components/directives from these
    BindingMetadata::new_option(map)
}

lazy_static! {
    static ref PROPS_PATTERN: TsPattern =
        Pattern::contextual("{props: $P}", "pair", TypeScript).unwrap();
    static ref INJECT_PATTERN: TsPattern =
        Pattern::contextual("{inject: $I}", "pair", TypeScript).unwrap();
    static ref METHOD_PATTERN: TsPattern =
        Pattern::contextual("{methods: $M}", "pair", TypeScript).unwrap();
    static ref COMPUTED_PATTERN: TsPattern =
        Pattern::contextual("{computed: $C}", "pair", TypeScript).unwrap();
}

fn collect_keys_from_option_property(node: TsNode) -> Option<(Vec<Range<usize>>, BindingTypes)> {
    let (keys, tpe) = if let Some(n) = PROPS_PATTERN.match_node(node.clone()) {
        let keys = get_object_or_array_keys(n.field("value")?);
        (keys, BindingTypes::Props)
    } else if let Some(n) = INJECT_PATTERN.match_node(node.clone()) {
        let keys = get_object_or_array_keys(n.field("value")?);
        (keys, BindingTypes::Options)
    } else if let Some(n) = METHOD_PATTERN.match_node(node.clone()) {
        let keys = get_object_keys(n.field("value")?);
        (keys, BindingTypes::Options)
    } else if let Some(n) = COMPUTED_PATTERN.match_node(node.clone()) {
        let keys = get_object_keys(n.field("value")?);
        (keys, BindingTypes::Options)
    } else if node.kind() == "method_definition" {
        let name = node.field("name")?;
        let tpe = if name.text() == "setup" {
            BindingTypes::SetupMaybeRef
        } else if name.text() == "data" {
            BindingTypes::Data
        } else {
            return None;
        };
        let body = node.field("body")?;
        let return_statement = body.children().find(|s| s.kind() == "return_statement")?;
        let child = return_statement.child(1)?;
        if child.kind() != "object" {
            return None;
        }
        let keys = get_object_keys(return_statement.child(1)?);
        (keys, tpe)
    } else {
        return None;
    };
    Some((keys, tpe))
}

fn get_object_or_array_keys(n: TsNode) -> Vec<Range<usize>> {
    match &*n.kind() {
        "object" => get_object_keys(n),
        "array" => get_array_keys(n),
        _ => Vec::new(),
    }
}

fn get_object_keys(n: TsNode) -> Vec<Range<usize>> {
    debug_assert!(n.kind() == "object");
    let mut result = vec![];
    for child in n.children() {
        let kind = child.kind();
        let node = if kind == "pair" {
            child.field("key")
        } else if kind == "method_definition" {
            child.field("name")
        } else {
            None
        };
        let Some(n) = node else {
            continue;
        };
        if let Some(key) = resolve_key(n) {
            result.push(key);
        }
    }
    result
}

fn resolve_key(n: TsNode) -> Option<Range<usize>> {
    let kind = n.kind();
    if kind == "property_identifier" || kind == "number" {
        Some(n.range())
    } else if kind == "string" {
        Some(n.child(0)?.range())
    } else if kind == "computed_property_name" {
        resolve_key(n.child(0)?)
    } else {
        None
    }
}

fn get_array_keys(n: TsNode) -> Vec<Range<usize>> {
    debug_assert!(n.kind() == "array");
    n.children()
        .filter_map(|n| {
            if n.kind() == "string" {
                n.child(1).map(|n| n.range())
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_analyze_props() {
        let src = "export default { props: { msg: String } }";
        let bindings = analyze_script_bindings(src);
        assert!(!bindings.is_empty());
        assert!(bindings.get("msg") == Some(&BindingTypes::Props));
    }

    #[test]
    fn test_analyze_computed() {
        let src = "export default { computed: { msg() {}, test: () => 123 } }";
        let bindings = analyze_script_bindings(src);
        assert!(!bindings.is_empty());
        assert!(bindings.get("msg") == Some(&BindingTypes::Options));
        assert!(bindings.get("test") == Some(&BindingTypes::Options));
    }

    #[test]
    fn test_analyze_data() {
        let src = "export default { data() { return { msg: 123 } } }";
        let bindings = analyze_script_bindings(src);
        assert!(!bindings.is_empty());
        assert!(bindings.get("msg") == Some(&BindingTypes::Data));
    }

    #[test]
    fn test_analyze_setup() {
        let src = "export default { setup() { return { msg: 123 } } }";
        let bindings = analyze_script_bindings(src);
        assert!(!bindings.is_empty());
        assert!(bindings.get("msg") == Some(&BindingTypes::SetupMaybeRef));
    }

    #[test]
    fn test_analyze_inject() {
        let src = "export default { inject: ['msg'] }";
        let bindings = analyze_script_bindings(src);
        assert!(!bindings.is_empty());
        assert!(bindings.get("msg") == Some(&BindingTypes::Options));
    }
}
