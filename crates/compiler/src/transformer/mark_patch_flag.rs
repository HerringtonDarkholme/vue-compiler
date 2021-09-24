// mark patch flag and is_block for runtime
// it should happen after process_expression
use super::{BaseFor, BaseIf, BaseInfo, BaseText, CorePass};
use crate::converter::{BaseIR, IRNode as IR, JsExpr as Js, Prop};
use crate::flags::{PatchFlag, RuntimeHelper as RH, StaticLevel};

pub struct PatchFlagMarker;

impl<'a> CorePass<BaseInfo<'a>> for PatchFlagMarker {
    fn enter_if(&mut self, i: &mut BaseIf<'a>) {
        for branch in i.branches.iter_mut() {
            // TODO: handle v-memo/v-once
            if let IR::VNodeCall(vn) = &mut *branch.child {
                if !matches!(vn.tag, Js::Symbol(RH::Fragment)) {
                    vn.is_block = true;
                }
            }
            let props = match &mut *branch.child {
                IR::VNodeCall(v) => &mut v.props,
                IR::RenderSlotCall(r) => &mut r.slot_props,
                IR::For(f) => return f.key = Some(Js::Num(branch.info)),
                _ => return,
            };
            // already has key
            if props.as_ref().map_or(false, find_key_on_js) {
                return;
            }
            // inject default key
            let key = Js::str_lit("key");
            let val = Js::Num(branch.info);
            let key_prop = (key, val);

            if let Some(ps) = props {
                inject_prop(ps, key_prop);
            } else {
                *props = Some(Js::Props(vec![key_prop]));
            }
        }
    }
    fn exit_for(&mut self, f: &mut BaseFor<'a>) {
        let is_stable_fragment = f.source.static_level() > StaticLevel::NotStatic;
        let has_key = find_key(&f.child);
        f.fragment_flag = if is_stable_fragment {
            PatchFlag::STABLE_FRAGMENT
        } else if has_key {
            PatchFlag::KEYED_FRAGMENT
        } else {
            PatchFlag::UNKEYED_FRAGMENT
        };
        f.is_stable = is_stable_fragment;
    }

    fn exit_text(&mut self, t: &mut BaseText<'a>) {
        if !t.fast_path {
            return;
        }
        let level = t
            .texts
            .iter()
            .map(Js::static_level)
            .min()
            .unwrap_or(StaticLevel::CanStringify);
        if level == StaticLevel::NotStatic {
            t.need_patch = true;
        }
    }
}

fn find_key(t: &BaseIR) -> bool {
    let props = match t {
        IR::VNodeCall(v) => &v.props,
        IR::RenderSlotCall(r) => &r.slot_props,
        IR::AlterableSlot(..) => return false,
        IR::VSlotUse(_) => {
            panic!("v-slot with v-for must be alterable slots")
        }
        IR::TextCall(_) | IR::For(_) | IR::If(_) | IR::CommentCall(_) => {
            panic!("v-for child must be vnode/renderSlot/slotfn")
        }
    };
    if let Some(prop) = props {
        find_key_on_js(prop)
    } else {
        false
    }
}

fn find_key_on_js(e: &Js) -> bool {
    match e {
        Js::Call(RH::MergeProps, args) => args.iter().any(find_key_on_js),
        Js::Props(ps) => ps.iter().any(|(k, _)| match k {
            Js::StrLit(s) => s.raw == "key",
            _ => false,
        }),
        _ => false,
    }
}

// 1. mergeProps(...)
// 2. toHandlers(...)
fn inject_prop<'a>(props: &mut Js<'a>, key: Prop<'a>) {
    debug_assert!(!find_key_on_js(props));
    match props {
        Js::Call(RH::MergeProps, args) => {
            for arg in args.iter_mut() {
                if let Js::Props(ps) = arg {
                    ps.push(key);
                    return;
                }
            }
            args.push(Js::Props(vec![key]));
        }
        Js::Props(ps) => ps.push(key),
        // should not inject props to any other expression
        obj => {
            return {
                let mut temp = Js::Src("");
                std::mem::swap(obj, &mut temp);
                let p = Js::Props(vec![key]);
                *obj = Js::Call(RH::MergeProps, vec![temp, p]);
            }
        }
    }
}
