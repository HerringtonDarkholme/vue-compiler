use super::{BaseConverter as BC, CoreConverter, Element, JsExpr as Js, Prop, VStr};
use crate::{
    flags::{self, PatchFlag, RuntimeHelper},
    parser::{Directive, ElemProp},
    tokenizer::Attribute,
    util::{self, is_bind_key, is_component_tag, is_reserved_prop},
};
use rustc_hash::{FxHashMap, FxHashSet};
use std::iter::IntoIterator;
use std::mem;

pub struct BuildProps<'a> {
    pub props: Option<Js<'a>>,
    pub directives: Dirs<'a>,
    pub patch_flag: PatchFlag,
    pub dynamic_props: FxHashSet<VStr<'a>>,
}

#[derive(Default)]
struct PropFlags {
    is_component: bool,
    has_ref: bool,
    has_class_binding: bool,
    has_style_binding: bool,
    has_hydration_event_binding: bool,
    has_dynamic_keys: bool,
    has_vnode_hook: bool,
}

#[derive(Default)]
/// collecting props object for vnode call. e.g:
/// <:prop="val" v-bind="obj"/> becomes {prop: val, ...obj}
struct PropArgs<'a> {
    /// pending properties, e.g. (prop, val)
    pending_props: Props<'a>,
    /// merged prop argument, e.g. obj
    merge_args: Args<'a>,
}

#[derive(Default)]
struct CollectProps<'a> {
    prop_args: PropArgs<'a>,
    runtime_dirs: Dirs<'a>,
    dynamic_props: FxHashSet<VStr<'a>>,
    prop_flags: PropFlags,
}

impl<'a> CollectProps<'a> {
    fn new(e: &Element<'a>) -> Self {
        let mut s = Self::default();
        s.prop_flags.is_component = e.is_component();
        s
    }
}

type Props<'a> = Vec<Prop<'a>>;
type Args<'a> = Vec<Js<'a>>;
type Dir<'a> = (Directive<'a>, Option<RuntimeHelper>);
type Dirs<'a> = Vec<Dir<'a>>;

pub fn build_props<'a, T>(bc: &BC, e: &mut Element<'a>, elm_props: T) -> BuildProps<'a>
where
    T: IntoIterator<Item = ElemProp<'a>>,
{
    let mut cp = CollectProps::new(e);
    elm_props.into_iter().for_each(|prop| match prop {
        ElemProp::Dir(dir) => collect_dir(bc, e, dir, &mut cp),
        ElemProp::Attr(attr) => collect_attr(bc, e, attr, &mut cp),
    });
    let prop_expr = compute_prop_expr(cp.prop_args);
    let CollectProps {
        runtime_dirs,
        dynamic_props,
        ..
    } = cp;
    let patch_flag = build_patch_flag(cp.prop_flags, &runtime_dirs, &dynamic_props);
    // let prop_expr = pre_normalize_prop(prop_expr);
    BuildProps {
        props: prop_expr,
        directives: runtime_dirs,
        patch_flag,
        dynamic_props,
    }
}

fn collect_attr<'a>(bc: &BC, e: &Element<'a>, attr: Attribute<'a>, cp: &mut CollectProps<'a>) {
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
    cp.prop_args
        .pending_props
        .push((Js::StrLit(VStr::raw(name)), value_expr));
}

#[inline]
fn is_pre_convert_dir(s: &str) -> bool {
    match s.len() {
        2 => s == "if" || s == "is",
        4 => ["slot", "memo", "once"].contains(&s),
        _ => s == "for",
    }
}

// by abstracting DirConvRet we can fully extract out v-on/v-bind!
fn collect_dir<'a>(bc: &BC, e: &mut Element<'a>, mut dir: Directive<'a>, cp: &mut CollectProps<'a>) {
    use super::DirectiveConvertResult as DirConv;
    let Directive { name, argument, .. } = &dir;
    let name = *name;
    if is_pre_convert_dir(name) {
        return;
    }
    if is_bind_key(argument, "is") && is_component_tag(e.tag_name) {
        return; // skip <component :is="c"/>
    }
    let (value, runtime) = match bc.convert_directive(&mut dir, e) {
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
        props.iter().for_each(|p| analyze_patch_flag(p, cp));
        cp.prop_args.pending_props.extend(props);
        return;
    }
    flush_pending_props(&mut cp.prop_args);
    // if dir returns an object, dynamic key must be true
    cp.prop_flags.has_dynamic_keys = true;
    cp.prop_args.merge_args.push(value);
}

fn flush_pending_props(prop_args: &mut PropArgs) {
    // flush existing props to an object
    if prop_args.pending_props.is_empty() {
        return;
    }
    let arg = mem::take(&mut prop_args.pending_props);
    let arg = dedupe_properties(arg);
    prop_args.merge_args.push(Js::Props(arg));
}

fn process_inline_ref(val: VStr) -> Js {
    todo!("setup binding is pending")
}

fn dedupe_properties(props: Props) -> Props {
    let mut known_props = FxHashMap::default();
    let mut ret = vec![];
    for (key, val) in props {
        let name = match &key {
            Js::StrLit(name) => name,
            _ => {
                ret.push((key, val));
                continue;
            }
        };
        if let Some(&i) = known_props.get(name) {
            if util::is_mergeable_prop(name) {
                merge_as_array(&mut ret[i], val);
            }
            // TODO: should remove by parser
        } else {
            known_props.insert(*name, ret.len());
            ret.push((key, val));
        }
    }
    ret
}

fn merge_as_array<'a>(existing: &mut Prop<'a>, incoming: Js<'a>) {
    let val = &mut existing.1;
    if let Js::Array(arr) = val {
        arr.push(incoming);
    } else {
        let v = mem::replace(val, Js::Src(""));
        let mut arr = Js::Array(vec![v]);
        mem::swap(val, &mut arr);
    }
}

fn compute_prop_expr(mut prop_args: PropArgs) -> Option<Js> {
    flush_pending_props(&mut prop_args);
    let PropArgs {
        pending_props,
        merge_args,
    } = prop_args;
    debug_assert!(pending_props.is_empty());
    if merge_args.len() <= 1 {
        merge_args.into_iter().next()
    } else {
        Some(Js::Call(RuntimeHelper::MergeProps, merge_args))
    }
}

fn analyze_patch_flag<'a>(p: &Prop<'a>, cp: &mut CollectProps<'a>) {
    let is_component = cp.prop_flags.is_component;
    let flags = &mut cp.prop_flags;
    let (name, val) = match p {
        (Js::StrLit(k), val) => (k, val),
        _ => return flags.has_dynamic_keys = true,
    };
    let is_event_handler = VStr::is_handler(name);
    if !is_component &&
        is_event_handler &&
        // omit click because hydration gives click fast path
        !name.raw.eq_ignore_ascii_case("click") &&
        name.raw != "onUpdate:modelValue" && // omit v-model
        !is_reserved_prop(name)
    // vnode hooks
    {
        flags.has_hydration_event_binding = true;
    }
    if is_event_handler && is_reserved_prop(name) {
        flags.has_vnode_hook = true;
    }
    if val.static_level() > flags::StaticLevel::NotStatic {
        return;
    }
    match name.raw {
        "ref" => flags.has_ref = true,
        "class" => flags.has_class_binding = true,
        "style" => flags.has_style_binding = true,
        "key" => (),
        n => {
            cp.dynamic_props.insert(*name);
        }
    }
    if is_component && (["class", "style"].contains(&name.raw)) {
        cp.dynamic_props.insert(*name);
    }
}

fn build_patch_flag<'a>(
    f: PropFlags,
    runtime_dirs: &[Dir<'a>],
    dynamic_names: &FxHashSet<VStr<'a>>,
) -> PatchFlag {
    if f.has_dynamic_keys {
        return PatchFlag::FULL_PROPS;
    }
    let mut patch_flag = PatchFlag::empty();
    // actually element can also be slot
    let is_plain = !f.is_component;
    if f.has_class_binding && is_plain {
        patch_flag |= PatchFlag::CLASS;
    }
    if f.has_style_binding && is_plain {
        patch_flag |= PatchFlag::STYLE;
    }
    if !dynamic_names.is_empty() {
        patch_flag |= PatchFlag::PROPS;
    }
    if f.has_hydration_event_binding {
        patch_flag |= PatchFlag::HYDRATE_EVENTS;
    }
    let no_prop_patch = patch_flag == PatchFlag::empty() || patch_flag == PatchFlag::HYDRATE_EVENTS;
    if no_prop_patch && (f.has_ref || f.has_vnode_hook || !runtime_dirs.is_empty()) {
        patch_flag |= PatchFlag::NEED_PATCH;
    }
    patch_flag
}

/// extract class/style for faster runtime patching
pub fn pre_normalize_prop(prop_expr: Option<Js>) -> Option<Js> {
    todo!("pre-normalize props only in DOM for now. usable in any platform")
}
