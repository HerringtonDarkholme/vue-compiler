// mark patch flag and is_block for runtime
// it should happen after process_expression
use super::{BaseFor, BaseIf, BaseInfo, BaseVNode, CorePassExt, Scope};
use crate::converter::{BaseIR, IRNode, JsExpr as Js};
use crate::flags::{PatchFlag, RuntimeHelper as RH, StaticLevel};

pub struct PatchFlagMarker;

impl<'a> CorePassExt<BaseInfo<'a>, Scope<'a>> for PatchFlagMarker {
    fn enter_if(&mut self, i: &mut BaseIf<'a>, _: &mut Scope<'a>) {
        for branch in i.branches.iter_mut() {
            // TODO: handle v-memo/v-once
            if let IRNode::VNodeCall(vn) = &mut *branch.child {
                if !matches!(vn.tag, Js::Symbol(RH::Fragment)) {
                    vn.is_block = true;
                }
            }
        }
    }
    fn exit_for(&mut self, f: &mut BaseFor<'a>, shared: &mut Scope<'a>) {
        let is_stable_fragment = f.source.static_level() > StaticLevel::NotStatic;
        let has_key = find_key(&f.child);
        let fragment_flag = if is_stable_fragment {
            PatchFlag::STABLE_FRAGMENT
        } else if has_key {
            PatchFlag::KEYED_FRAGMENT
        } else {
            PatchFlag::UNKEYED_FRAGMENT
        };
        f.is_stable = is_stable_fragment;
    }
    fn exit_vnode(&mut self, v: &mut BaseVNode<'a>, shared: &mut Scope<'a>) {}
}

fn find_key(t: &BaseIR) -> bool {
    use IRNode as IR;
    match t {
        IR::VNodeCall(..) => todo!("read props"),
        IR::RenderSlotCall(..) => todo!("read props"),
        IR::AlterableSlot(..) => false,
        IR::VSlotUse(_) => {
            panic!("v-slot with v-for must be alterable slots")
        }
        IR::TextCall(_) | IR::For(_) | IR::If(_) | IR::CommentCall(_) => {
            panic!("v-for child must be vnode/renderSlot/slotfn")
        }
    }
}
