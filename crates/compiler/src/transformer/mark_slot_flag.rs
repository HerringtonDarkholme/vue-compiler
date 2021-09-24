use super::{BaseInfo, BaseVNode, CorePassExt, IRNode, Scope};
use crate::{
    converter::BaseIR,
    flags::{PatchFlag, SlotFlag},
};

pub struct SlotFlagMarker;

impl<'a> CorePassExt<BaseInfo<'a>, Scope<'a>> for SlotFlagMarker {
    fn enter_vnode(&mut self, v: &mut BaseVNode<'a>, shared: &mut Scope<'a>) {
        if !v.is_component || v.children.is_empty() {
            return;
        }
        debug_assert_eq!(v.children.len(), 1);

        let has_dynamic_slots = has_scope_in_vnode(v, shared);
        // has dynamic stable slot key
        // TODO: add dynamic_slots
        let v_slot = match &mut v.children[0] {
            IRNode::VSlotUse(v_slot) => v_slot,
            _ => panic!("impossible"),
        };
        let has_dynamic_slots = has_dynamic_slots
            || has_dynamic_slot_name(&v_slot)
            || !v_slot.alterable_slots.is_empty();
        v_slot.slot_flag = if has_dynamic_slots {
            SlotFlag::Dynamic
        } else if has_forwarded_slots(&v_slot.stable_slots) {
            SlotFlag::Forwarded
        } else {
            SlotFlag::Stable
        };
        if has_dynamic_slots {
            v.patch_flag |= PatchFlag::DYNAMIC_SLOTS;
        }
    }
}
fn has_dynamic_slot_name<T>(t: T) -> bool {
    todo!()
}

fn has_forwarded_slots<T>(t: T) -> bool {
    todo!()
}

fn has_scope_in_vnode(v: &BaseVNode, scope: &Scope) -> bool {
    todo!()
}

fn has_scope_ref(v: &BaseIR, scope: &Scope) -> bool {
    todo!()
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
