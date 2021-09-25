use super::{BaseInfo, BaseVNode, BaseVSlot, CorePassExt, IRNode, Scope};
use crate::converter::BaseIR;
use crate::flags::{PatchFlag, SlotFlag, StaticLevel};

pub struct SlotFlagMarker;

impl<'a> CorePassExt<BaseInfo<'a>, Scope<'a>> for SlotFlagMarker {
    fn exit_vnode(&mut self, v: &mut BaseVNode<'a>, scope: &mut Scope<'a>) {
        if !v.is_component || v.children.is_empty() {
            return;
        }
        debug_assert_eq!(v.children.len(), 1);
        let has_dynamic_slots = scope.has_ref_in_vnode(v);
        // has dynamic stable slot key
        let v_slot = match &mut v.children[0] {
            IRNode::VSlotUse(v_slot) => v_slot,
            _ => panic!("impossible"),
        };
        let has_dynamic_slots = has_dynamic_slots
            || !v_slot.alterable_slots.is_empty()
            || has_dynamic_slot_name(v_slot);
        v_slot.slot_flag = if has_dynamic_slots {
            SlotFlag::Dynamic
        } else if has_forwarded_slots(v_slot) {
            SlotFlag::Forwarded
        } else {
            SlotFlag::Stable
        };
        if has_dynamic_slots {
            v.patch_flag |= PatchFlag::DYNAMIC_SLOTS;
        }
    }
}

fn has_dynamic_slot_name(v_slot: &BaseVSlot) -> bool {
    debug_assert!(v_slot.alterable_slots.is_empty());
    v_slot
        .stable_slots
        .iter()
        .any(|s| s.name.static_level() == StaticLevel::NotStatic)
}

fn has_forward_list(irs: &[BaseIR]) -> bool {
    irs.iter().any(has_forward_one)
}

fn has_forward_one(ir: &BaseIR) -> bool {
    use IRNode as IR;
    match ir {
        IR::RenderSlotCall(_) => true,
        IR::If(i) => i.branches.iter().map(|b| &*b.child).any(has_forward_one),
        IR::For(f) => has_forward_one(&f.child),
        IR::VNodeCall(vn) => has_forward_list(&vn.children),
        IR::VSlotUse(s) => has_forwarded_slots(s),
        IR::AlterableSlot(s) => has_forward_list(&s.body),
        IR::TextCall(_) => false,
        IR::CommentCall(_) => false,
    }
}

fn has_forwarded_slots(v_slot: &BaseVSlot) -> bool {
    let stable_forward = v_slot
        .stable_slots
        .iter()
        .any(|v| has_forward_list(&v.body));
    stable_forward || has_forward_list(&v_slot.alterable_slots)
}

#[cfg(test)]
mod test {
    // use super::super::test::{base_convert, transformer_ext};
    // use super::super::Transformer;
    // use super::*;

    // #[test]
    // fn test_dynamic_slot() {
    //     let mut transformer = transformer_ext(SlotFlagMarker);
    //     let mut ir = base_convert(
    //         r"
    // <component v-for='upper in 123'>
    // <template v-slot='test' :test='upper'>
    // </template>
    // </component>
    // ",
    //     );
    //     transformer.transform(&mut ir);
    // }
}
