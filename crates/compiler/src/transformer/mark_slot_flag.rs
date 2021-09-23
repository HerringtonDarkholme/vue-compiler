use super::{BaseInfo, BaseVNode, CorePassExt, Scope};
use crate::converter::BaseIR;

pub struct SlotFlagMarker;

impl<'a> CorePassExt<BaseInfo<'a>, Scope<'a>> for SlotFlagMarker {
    fn enter_vnode(&mut self, v: &mut BaseVNode<'a>, shared: &mut Scope<'a>) {
        if !v.is_component {
            return;
        }
        let has_dynamic_slots = has_scope_in_vnode(v, shared);
    }
}

fn has_scope_in_vnode(v: &BaseVNode, scope: &Scope) -> bool {
    todo!()
}

fn has_scope_ref(v: &BaseIR, scope: &Scope) -> bool {
    todo!()
}

#[cfg(test)]
mod test {
    use super::super::test::{base_convert, transformer_ext};
    use super::super::Transformer;
    use super::*;

    #[test]
    fn test_dynamic_slot() {
        let mut transformer = transformer_ext(SlotFlagMarker);
        let mut ir = base_convert(
            r"
<component v-for='upper in 123'>
  <template v-slot='test' :test='upper'>
  </template>
</component>
",
        );
        transformer.transform(&mut ir);
    }
}
