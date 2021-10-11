use super::{
    BaseInfo, BaseRenderSlot, BaseSlotFn, BaseVNode, CorePassExt, IRNode as IR, BaseCache, Scope,
};
use crate::{
    converter::v_on::get_handler_type,
    ir::{JsExpr as Js, CacheKind, HandlerType},
};

// 1. cache handler
// 2. hoist static
pub struct HoistStatic {
    in_v_once: bool,
    is_component: bool,
    cache_handlers: bool,
}

impl<'a> CorePassExt<BaseInfo<'a>, Scope<'a>> for HoistStatic {
    fn enter_cache(&mut self, cn: &mut BaseCache<'a>, _: &mut Scope<'a>) {
        if matches!(cn.kind, CacheKind::Once) {
            self.in_v_once = true;
        }
    }
    fn exit_cache(&mut self, cn: &mut BaseCache<'a>, _: &mut Scope<'a>) {
        if matches!(cn.kind, CacheKind::Once) {
            self.in_v_once = false;
        }
    }
    fn enter_vnode(&mut self, vn: &mut BaseVNode<'a>, _: &mut Scope<'a>) {
        self.is_component = vn.is_component;
    }
    fn enter_js_expr(&mut self, exp: &mut Js<'a>, scope: &mut Scope<'a>) {
        let (src, lvl) = match exp {
            Js::FuncSimple(src, lvl) => (src, lvl),
            _ => return,
        };
        let ty = get_handler_type(*src);
        let is_member_exp = matches!(ty, HandlerType::MemberExpr);
        let should_cache = self.cache_handlers &&
            // unnecessary to cache inside v-once
            !self.in_v_once &&
            // #1541 bail if this is a member exp handler passed to a component -
            // we need to use the original function to preserve arity,
            // e.g. <transition> relies on checking cb.length to determine
            // transition end handling. Inline function is ok since its arity
            // is preserved even when cached.
            !(is_member_exp && self.is_component);
        // bail if the function references closure variables (v-for, v-slot)
        // it must be passed fresh to avoid stale values.
        // && !hasScopeRef(exp, context.identifiers) &&
        // runtime constants don't need to be cached
        // (this is analyzed by compileScript in SFC <script setup>)
        // !(exp.type === NodeTypes.SIMPLE_EXPRESSION && exp.constType > 0) &&
    }
}
