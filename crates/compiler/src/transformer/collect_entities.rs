// this module collects following entities:
// runtime helpers
// component/directive asset
use super::{
    BaseFor, BaseIf, BaseInfo, BaseRenderSlot, BaseText, BaseVNode, BaseVSlot, BaseCache, CorePass,
};
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
            self.helpers.collect(RH::FRAGMENT);
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
            self.helpers.collect(RH::CREATE_COMMENT);
        }
    }
    fn exit_for(&mut self, f: &mut BaseFor<'a>) {
        if let IR::AlterableSlot(_) = &*f.child {
            // v-for in slot only need renderList
            return self.helpers.collect(RH::RENDER_LIST);
        }
        self.helpers.collect(RH::OPEN_BLOCK);
        self.helpers.collect(RH::CREATE_ELEMENT_BLOCK);
        self.helpers.collect(RH::RENDER_LIST);
        self.helpers.collect(RH::FRAGMENT);
    }
    fn exit_vnode(&mut self, v: &mut BaseVNode<'a>) {
        if !v.directives.is_empty() {
            self.helpers.collect(RH::WITH_DIRECTIVES);
            // dir with Js::Symbol is collected in js_expr
            for dir in v.directives.iter() {
                if let Js::StrLit(d) = dir.name {
                    self.directives.insert(d);
                }
            }
        }
        if v.is_block {
            self.helpers.collect(RH::OPEN_BLOCK);
        }
        let h = get_vnode_call_helper(v);
        self.helpers.collect(h);
        if !v.is_component {
            return;
        }
        // only hoisted asset needs handling, Js::Symbol is collected in js_expr
        // see [resolve_element_tag] in convert_element
        if let Some(tag) = is_hoisted_asset(&v.tag) {
            self.helpers.collect(RH::RESOLVE_COMPONENT);
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
            self.helpers.collect(RH::RESOLVE_DIRECTIVE);
        }
        for dir_name in hoisted_dir_names {
            self.directives.insert(*dir_name);
        }
    }
    fn exit_slot_outlet(&mut self, _: &mut BaseRenderSlot<'a>) {
        self.helpers.collect(RH::RENDER_SLOT);
    }
    fn exit_v_slot(&mut self, s: &mut BaseVSlot<'a>) {
        if !s.alterable_slots.is_empty() {
            self.helpers.collect(RH::CREATE_SLOTS);
        }
        self.helpers.collect(RH::WITH_CTX);
    }
    fn exit_comment(&mut self, _: &mut &str) {
        self.helpers.collect(RH::CREATE_COMMENT);
    }

    fn exit_text(&mut self, t: &mut BaseText<'a>) {
        if !t.fast_path {
            self.helpers.collect(RH::CREATE_TEXT);
        }
    }
    fn enter_cache(&mut self, r: &mut BaseCache<'a>) {
        use crate::ir::CacheKind::{Once, Memo, MemoInVFor};
        match r.kind {
            Once => self.helpers.collect(RH::SET_BLOCK_TRACKING),
            Memo(_) => self.helpers.collect(RH::WITH_MEMO),
            MemoInVFor { .. } => {
                self.helpers.collect(RH::IS_MEMO_SAME);
            }
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
    use super::super::test::base_convert;
    use super::*;
    use crate::transformer::{Transformer, BaseTransformer};
    fn transform(s: &str) -> BaseRoot {
        let mut ir = base_convert(s);
        BaseTransformer::transform(&mut ir, EntityCollector::default());
        ir
    }
    #[test]
    fn test_v_if_helper() {
        let ir = transform("<p v-if='a'/>");
        let helpers = ir.top_scope.helpers;
        assert!(helpers.contains(RH::CREATE_COMMENT));
    }
    #[test]
    fn test_v_for_helper() {
        let ir = transform("<p v-for='a in b'/>");
        let helpers = ir.top_scope.helpers;
        assert!(helpers.contains(RH::FRAGMENT));
        assert!(helpers.contains(RH::OPEN_BLOCK));
        assert!(helpers.contains(RH::RENDER_LIST));
        assert!(!helpers.contains(RH::CREATE_COMMENT));
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
        assert!(!helpers.contains(RH::FRAGMENT));
        assert!(!helpers.contains(RH::CREATE_ELEMENT_BLOCK));
        assert!(helpers.contains(RH::RENDER_LIST));
        assert!(helpers.contains(RH::WITH_CTX));
    }
}
