// this module collects following entities:
// runtime helper
// component/directive asset
// temporary variable
// static hoist
use super::{
    BaseConvertInfo, BaseFor, BaseIf, BaseRenderSlot, BaseVNode, BaseVSlot, CoreTransformPass,
    IRNode as IR,
};
use crate::converter::{BaseIR, BaseRoot, JsExpr as Js};
use crate::flags::{HelperCollector, RuntimeHelper as RH};

pub struct EntityCollector<'a> {
    helper: HelperCollector,
    components: Vec<&'a str>,
    directives: Vec<&'a str>,
}

impl<'a> CoreTransformPass<BaseConvertInfo<'a>> for EntityCollector<'a> {
    fn exit_js_expr(&mut self, e: &mut Js) {
        match e {
            Js::Call(h, ..) | Js::Symbol(h) => {
                self.helper.collect_helper(*h);
            }
            _ => {}
        }
    }
    fn exit_if(&mut self, i: &mut BaseIf) {
        if i.branches.iter().all(|b| b.condition.is_some()) {
            self.helper.collect_helper(RH::CreateComment);
        }
    }
    fn exit_for(&mut self, f: &mut BaseFor<'a>) {
        self.helper.collect_helper(RH::OpenBlock);
        self.helper.collect_helper(RH::CreateElementBlock);
    }
    fn exit_slot_outlet(&mut self, r: &mut BaseRenderSlot<'a>) {
        self.helper.collect_helper(RH::RenderSlot);
    }
    fn exit_v_slot(&mut self, s: &mut BaseVSlot<'a>) {
        if !s.alterable_slots.is_empty() {
            self.helper.collect_helper(RH::CreateSlots);
        }
    }
}
