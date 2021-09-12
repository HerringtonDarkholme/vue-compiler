use super::{Element, JsExpr as Js, Prop, VStr};
use crate::core::{
    flags::PatchFlag,
    parser::{Directive, ElemProp},
};
use std::iter::IntoIterator;

pub struct BuildProps<'a> {
    pub props: Option<Js<'a>>,
    pub directives: Vec<Directive<'a>>,
    pub patch_flag: PatchFlag,
    pub dynamic_prop_names: Vec<VStr<'a>>,
}

struct PropFlags {
    has_ref: bool,
    has_class_binding: bool,
    has_style_binding: bool,
    has_hydration_event_binding: bool,
    has_dynamic_keys: bool,
    has_vnode_hook: bool,
}

struct CollectProps<'a> {
    props: Props<'a>,
    merge_args: Args<'a>,
    runtime_dirs: Dirs<'a>,
    dynamic_prop_names: Vec<VStr<'a>>,
    prop_flags: PropFlags,
}

type Props<'a> = Vec<Prop<'a>>;
type Args<'a> = Vec<Js<'a>>;
type Dirs<'a> = Vec<Directive<'a>>;

pub fn build_props<'a, T>(e: &Element<'a>, elm_props: T) -> BuildProps<'a>
where
    T: IntoIterator<Item = ElemProp<'a>>,
{
    let CollectProps {
        props,
        merge_args,
        runtime_dirs,
        dynamic_prop_names,
        prop_flags,
    } = collect_props(elm_props);
    let prop_expr = compute_prop_expr(props, merge_args);
    let patch_flag = build_patch_flag(prop_flags);
    let prop_expr = pre_normalize_prop(prop_expr);
    BuildProps {
        props: prop_expr,
        directives: runtime_dirs,
        patch_flag,
        dynamic_prop_names,
    }
}

fn collect_props<'a, T>(t: T) -> CollectProps<'a>
where
    T: IntoIterator<Item = ElemProp<'a>>,
{
    todo!()
}

fn compute_prop_expr<'a>(props: Props, args: Args) -> Option<Js<'a>> {
    todo!()
}

fn analyze_patch_flag() -> PatchFlag {
    todo!()
}

fn build_patch_flag(info: PropFlags) -> PatchFlag {
    todo!()
}

fn pre_normalize_prop(prop_expr: Option<Js>) -> Option<Js> {
    todo!("pre-normalize props, SSR should be skipped")
}
