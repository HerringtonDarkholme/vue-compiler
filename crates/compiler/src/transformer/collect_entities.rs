// this module collects following entities:
// runtime helpers
// component/directive asset
// temporary variable
// static hoist
use super::{
    BaseConvertInfo, BaseFor, BaseIf, BaseRenderSlot, BaseVNode, BaseVSlot, CoreTransformPass,
};
use crate::converter::{BaseRoot, JsExpr as Js};
use crate::flags::{HelperCollector, RuntimeHelper as RH};
use crate::util::{get_vnode_call_helper, VStr};
use rustc_hash::FxHashSet;

pub struct EntityCollector<'a> {
    helpers: HelperCollector,
    components: FxHashSet<VStr<'a>>,
    directives: FxHashSet<VStr<'a>>,
}

impl<'a> CoreTransformPass<BaseConvertInfo<'a>> for EntityCollector<'a> {
    fn exit_root(&mut self, r: &mut BaseRoot<'a>) {
        if r.body.len() > 1 {
            self.helpers.collect(RH::Fragment);
        }
    }
    fn exit_js_expr(&mut self, e: &mut Js) {
        match e {
            Js::Call(h, ..) | Js::Symbol(h) => {
                self.helpers.collect(*h);
            }
            _ => {}
        }
    }
    fn exit_if(&mut self, i: &mut BaseIf) {
        if i.branches.iter().all(|b| b.condition.is_some()) {
            self.helpers.collect(RH::CreateComment);
        }
    }
    fn exit_for(&mut self, _: &mut BaseFor<'a>) {
        self.helpers.collect(RH::OpenBlock);
        self.helpers.collect(RH::CreateElementBlock);
        self.helpers.collect(RH::RenderList);
        self.helpers.collect(RH::Fragment);
    }
    fn exit_vnode(&mut self, v: &mut BaseVNode<'a>) {
        if !v.directives.is_empty() {
            self.helpers.collect(RH::WithDirectives);
            // dir with Js::Symbol is collected in js_expr
            for dir in v.directives.iter() {
                if let Js::StrLit(d) = dir.name {
                    self.directives.insert(d);
                }
            }
        }
        if v.is_block {
            self.helpers.collect(RH::OpenBlock);
        }
        let h = get_vnode_call_helper(v);
        self.helpers.collect(h);
        if !v.is_component {
            return;
        }
        // only StrLit needs handling, see [resolve_element_tag] in convert_element
        // component with Js::Symbol is collected in js_expr
        if let Js::StrLit(tag) = v.tag {
            self.helpers.collect(RH::ResolveComponent);
            self.components.insert(tag);
        }
    }
    fn exit_slot_outlet(&mut self, _: &mut BaseRenderSlot<'a>) {
        self.helpers.collect(RH::RenderSlot);
    }
    fn exit_v_slot(&mut self, s: &mut BaseVSlot<'a>) {
        if !s.alterable_slots.is_empty() {
            self.helpers.collect(RH::CreateSlots);
        }
        self.helpers.collect(RH::WithCtx);
    }
    fn exit_comment(&mut self, _: &mut &str) {
        self.helpers.collect(RH::CreateComment);
    }
}
