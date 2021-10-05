// this module collects following entities:
// runtime helpers
// component/directive asset
use super::{BaseFor, BaseIf, BaseInfo, BaseRenderSlot, BaseText, BaseVNode, BaseVSlot, CorePass};
use crate::converter::BaseRoot;
use crate::flags::{HelperCollector, RuntimeHelper as RH};
use crate::ir::{IRNode as IR, JsExpr as Js};
use crate::util::{get_vnode_call_helper, VStr};
use rustc_hash::FxHashSet;
use std::mem::swap;

#[derive(Default)]
pub struct EntityCollector<'a> {
    helpers: HelperCollector,
    components: FxHashSet<VStr<'a>>,
    directives: FxHashSet<VStr<'a>>,
}

impl<'a> CorePass<BaseInfo<'a>> for EntityCollector<'a> {
    fn exit_root(&mut self, r: &mut BaseRoot<'a>) {
        if r.body.len() > 1 {
            self.helpers.collect(RH::Fragment);
        }
        let scope = &mut r.top_scope;
        swap(&mut scope.helpers, &mut self.helpers);
        swap(&mut scope.components, &mut self.components);
        swap(&mut scope.directives, &mut self.directives);
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
    fn exit_for(&mut self, f: &mut BaseFor<'a>) {
        if let IR::AlterableSlot(_) = &*f.child {
            // v-for in slot only need renderList
            return self.helpers.collect(RH::RenderList);
        }
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
        // only hoisted asset needs handling, Js::Symbol is collected in js_expr
        // see [resolve_element_tag] in convert_element
        if let Some(tag) = is_hoisted_asset(&v.tag) {
            self.helpers.collect(RH::ResolveComponent);
            self.components.insert(*tag);
        }
        // only StrLit needs handling, see [build_directive_arg] in convert_element
        let mut hoisted_dir_names = v
            .directives
            .iter()
            .map(|dir| &dir.name)
            .filter_map(is_hoisted_asset)
            .peekable();
        if hoisted_dir_names.peek().is_some() {
            self.helpers.collect(RH::ResolveDirective);
        }
        for dir_name in hoisted_dir_names {
            self.directives.insert(*dir_name);
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

    fn exit_text(&mut self, t: &mut BaseText<'a>) {
        if !t.fast_path {
            self.helpers.collect(RH::CreateText);
        }
    }
}

pub fn is_hoisted_asset<'a, 'b>(expr: &'b Js<'a>) -> Option<&'b VStr<'a>> {
    match expr {
        Js::Simple(n, _) if VStr::is_asset(n) => Some(n),
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use super::super::test::{base_convert, get_transformer};
    use super::*;
    use crate::transformer::Transformer;
    fn transform(s: &str) -> BaseRoot {
        let mut transformer = get_transformer(EntityCollector::default());
        let mut ir = base_convert(s);
        transformer.transform(&mut ir);
        ir
    }
    #[test]
    fn test_v_if_helper() {
        let ir = transform("<p v-if='a'/>");
        let helpers = ir.top_scope.helpers;
        assert!(helpers.contains(RH::CreateComment));
    }
    #[test]
    fn test_v_for_helper() {
        let ir = transform("<p v-for='a in b'/>");
        let helpers = ir.top_scope.helpers;
        assert!(helpers.contains(RH::Fragment));
        assert!(helpers.contains(RH::OpenBlock));
        assert!(helpers.contains(RH::RenderList));
        assert!(!helpers.contains(RH::CreateComment));
    }
    #[test]
    fn test_v_for_alterable_helper() {
        let ir = transform(
            "
        <comp>
            <template #slot v-for='a in b'/>
        </comp>",
        );
        let helpers = ir.top_scope.helpers;
        assert!(!helpers.contains(RH::Fragment));
        assert!(!helpers.contains(RH::CreateElementBlock));
        assert!(helpers.contains(RH::RenderList));
        assert!(helpers.contains(RH::WithCtx));
    }
}
