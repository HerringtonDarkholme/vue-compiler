use super::{BaseConverter as BC, CoreConverter, Element, JsExpr as Js, Prop, VStr};
use crate::core::{
    flags::{PatchFlag, RuntimeHelper},
    parser::{Directive, ElemProp},
    tokenizer::Attribute,
    util::{is_bind_key, is_component_tag},
};
use std::iter::IntoIterator;

pub struct BuildProps<'a> {
    pub props: Option<Js<'a>>,
    pub directives: Dirs<'a>,
    pub patch_flag: PatchFlag,
    pub dynamic_prop_names: Vec<VStr<'a>>,
}

#[derive(Default)]
struct PropFlags {
    has_ref: bool,
    has_class_binding: bool,
    has_style_binding: bool,
    has_hydration_event_binding: bool,
    has_dynamic_keys: bool,
    has_vnode_hook: bool,
}

#[derive(Default)]
struct CollectProps<'a> {
    props: Props<'a>,
    merge_args: Args<'a>,
    runtime_dirs: Dirs<'a>,
    dynamic_prop_names: Vec<VStr<'a>>,
    prop_flags: PropFlags,
}

type Props<'a> = Vec<Prop<'a>>;
type Args<'a> = Vec<Js<'a>>;
type Dirs<'a> = Vec<(Directive<'a>, Option<RuntimeHelper>)>;

pub fn build_props<'a, T>(bc: &mut BC, e: &Element<'a>, elm_props: T) -> BuildProps<'a>
where
    T: IntoIterator<Item = ElemProp<'a>>,
{
    let mut cp = CollectProps::default();
    elm_props.into_iter().for_each(|prop| match prop {
        ElemProp::Dir(dir) => collect_dir(bc, e, dir, &mut cp),
        ElemProp::Attr(attr) => collect_attr(bc, e, attr, &mut cp),
    });
    let prop_expr = compute_prop_expr(cp.props, cp.merge_args);
    let patch_flag = build_patch_flag(cp.prop_flags);
    let prop_expr = pre_normalize_prop(prop_expr);
    BuildProps {
        props: prop_expr,
        directives: cp.runtime_dirs,
        patch_flag,
        dynamic_prop_names: cp.dynamic_prop_names,
    }
}

fn collect_attr<'a>(bc: &mut BC, e: &Element<'a>, attr: Attribute<'a>, cp: &mut CollectProps<'a>) {
    let Attribute { name, value, .. } = attr;
    let val = match value {
        Some(v) => v.content,
        None => VStr::raw(""),
    };
    // skip dynamic component is
    if name == "is" && (is_component_tag(e.tag_name) || val.starts_with("vue:")) {
        return;
    }
    let mut value_expr = Js::StrLit(val);
    if name == "ref" {
        cp.prop_flags.has_ref = true;
        if bc.inline && !val.is_empty() {
            value_expr = process_inline_ref(val);
        }
    }
    cp.props.push((Js::StrLit(val), value_expr));
}

#[inline]
fn is_pre_convert_dir(s: &str) -> bool {
    match s.len() {
        2 => s == "if" || s == "is",
        4 => ["slot", "memo", "once"].contains(&s),
        _ => s == "for",
    }
}

fn collect_dir<'a>(
    bc: &mut BC,
    e: &Element<'a>,
    mut dir: Directive<'a>,
    cp: &mut CollectProps<'a>,
) {
    use super::DirectiveConvertResult as DirConv;
    let Directive { name, argument, .. } = &dir;
    let name = *name;
    if is_pre_convert_dir(name) {
        return;
    }
    if is_bind_key(&argument, "is") && is_component_tag(e.tag_name) {
        return;
    }
    if (name == "bind" || name == "on") && argument.is_none() {
        cp.prop_flags.has_dynamic_keys = true;
    }
    let (value, runtime) = match bc.convert_directive(&mut dir) {
        DirConv::Converted { value, runtime } => (value, runtime),
        DirConv::Preserve => return cp.runtime_dirs.push((dir, None)),
        DirConv::Dropped => return,
    };
    match runtime {
        Ok(helper) => cp.runtime_dirs.push((dir, Some(helper))),
        Err(true) => cp.runtime_dirs.push((dir, None)),
        Err(false) => (),
    }
    if let Js::Props(props) = value {
        props.iter().for_each(|p| analyze_patch_flag(p));
        cp.props.extend(props);
        return;
    }
    // TODO flush properties
}

fn process_inline_ref(val: VStr) -> Js {
    todo!("setup binding is pending")
}

fn compute_prop_expr<'a>(props: Props, args: Args) -> Option<Js<'a>> {
    todo!()
}

fn analyze_patch_flag(p: &Prop) {
    todo!()
}

fn build_patch_flag(info: PropFlags) -> PatchFlag {
    todo!()
}

fn pre_normalize_prop(prop_expr: Option<Js>) -> Option<Js> {
    todo!("pre-normalize props, SSR should be skipped")
}
