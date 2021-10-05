/*!
Transform IRNode.
This module contains the canonical transformations from vue-next and
the original ones for the parity of features not implemented in Convert.

## Canonical
* hoistStatic
* transformExpression
* ~~vOnce (moved to convert)~~
* ~~vMemo (moved to convert)~~
* trackScopes

## Original
* collect_entities:
track all helpers/components/directives used in AST.
Vue track it by helper/helperString.
* optimize_text:
1. merge consecutive text call
2. wrap text in createTextVNode
* patch_flag:
seems patch flag can be extracted out
 */

pub mod collect_entities;
pub mod mark_patch_flag;
pub mod mark_slot_flag;
pub mod optimize_text;
pub mod pass;
pub mod process_expression;

use crate::{
    converter::{BaseConvertInfo as BaseInfo, BaseRoot},
    ir::{self as C, ConvertInfo, IRNode, IRRoot, JsExpr as Js, RuntimeDir},
};
pub use pass::{CorePass, CorePassExt, MergedPass, Scope};
use std::marker::PhantomData;

pub trait Transformer {
    type IR;
    /// transform will change ir node inplace
    /// usually transform will have multiple passes
    fn transform(&mut self, root: &mut Self::IR);
}

#[derive(Default)]
pub struct TransformOption {
    pub prefix_identifier: bool,
    pub is_dev: bool,
}

pub type BaseText<'a> = C::TextIR<BaseInfo<'a>>;
pub type BaseIf<'a> = C::IfNodeIR<BaseInfo<'a>>;
pub type BaseFor<'a> = C::ForNodeIR<BaseInfo<'a>>;
pub type BaseVNode<'a> = C::VNodeIR<BaseInfo<'a>>;
pub type BaseRenderSlot<'a> = C::RenderSlotIR<BaseInfo<'a>>;
pub type BaseVSlot<'a> = C::VSlotIR<BaseInfo<'a>>;
pub type BaseSlotFn<'a> = C::Slot<BaseInfo<'a>>;

struct NoopTransformer<T>(PhantomData<T>);

impl<T> Transformer for NoopTransformer<T> {
    type IR = T;
    fn transform(&mut self, _root: &mut Self::IR) {
        // noop
    }
}

trait CoreTransformer<T: ConvertInfo, P: CorePass<T>>: Transformer {
    fn transform_root(root: &mut IRRoot<T>, ps: &mut P);
    fn transform_js_expr(e: &mut T::JsExpression, ps: &mut P);

    fn transform_ir(ir: &mut IRNode<T>, ps: &mut P) {
        use IRNode as I;
        match ir {
            I::TextCall(t) => Self::transform_text(t, ps),
            I::If(i) => Self::transform_if(i, ps),
            I::For(f) => Self::transform_for(f, ps),
            I::VNodeCall(v) => Self::transform_vnode(v, ps),
            I::RenderSlotCall(r) => Self::transform_slot_outlet(r, ps),
            I::CommentCall(c) => Self::transform_comment(c, ps),
            I::VSlotUse(s) => Self::transform_v_slot(s, ps),
            I::AlterableSlot(a) => Self::transform_slot_fn(a, ps),
        }
    }
    fn transform_children(children: &mut Vec<IRNode<T>>, ps: &mut P) {
        for child in children.iter_mut() {
            Self::transform_ir(child, ps);
        }
    }
    fn transform_text(t: &mut C::TextIR<T>, ps: &mut P) {
        ps.enter_text(t);
        for text in t.texts.as_mut().iter_mut() {
            Self::transform_js_expr(text, ps);
        }
        ps.exit_text(t);
    }
    fn transform_if(i: &mut C::IfNodeIR<T>, ps: &mut P) {
        ps.enter_if(i);
        for branch in i.branches.iter_mut() {
            if let Some(c) = branch.condition.as_mut() {
                Self::transform_js_expr(c, ps);
            }
            Self::transform_ir(&mut branch.child, ps);
        }
        ps.exit_if(i);
    }
    fn transform_for(f: &mut C::ForNodeIR<T>, ps: &mut P) {
        // 1. first transform source in for node
        Self::transform_js_expr(&mut f.source, ps);
        use crate::ir::ForParseResult;
        // 2. process renderList param
        // val, key, index should counted as param
        let ForParseResult { value, key, index } = &mut f.parse_result;
        ps.enter_fn_param(value);
        if let Some(k) = key {
            ps.enter_fn_param(k);
        }
        if let Some(i) = index {
            ps.enter_fn_param(i);
        }

        // 3. the for itsel
        ps.enter_for(f);
        Self::transform_ir(&mut f.child, ps);
        ps.exit_for(f);

        let ForParseResult { value, key, index } = &mut f.parse_result;
        if let Some(i) = index {
            Self::transform_js_expr(i, ps);
            ps.exit_fn_param(i);
        }
        if let Some(k) = key {
            Self::transform_js_expr(k, ps);
            ps.exit_fn_param(k);
        }
        Self::transform_js_expr(value, ps);
        ps.exit_fn_param(value);
    }
    fn transform_vnode(v: &mut C::VNodeIR<T>, ps: &mut P) {
        ps.enter_vnode(v);
        Self::transform_js_expr(&mut v.tag, ps);
        if let Some(props) = v.props.as_mut() {
            Self::transform_js_expr(props, ps);
        }
        Self::transform_children(&mut v.children, ps);
        for dir in v.directives.iter_mut() {
            Self::transform_runtime_dir(dir, ps);
        }
        ps.exit_vnode(v);
    }
    fn transform_runtime_dir(dir: &mut RuntimeDir<T>, ps: &mut P) {
        Self::transform_js_expr(&mut dir.name, ps);
        if let Some(expr) = dir.expr.as_mut() {
            Self::transform_js_expr(expr, ps);
        }
        if let Some(arg) = dir.arg.as_mut() {
            Self::transform_js_expr(arg, ps);
        }
        if let Some(mods) = dir.mods.as_mut() {
            Self::transform_js_expr(mods, ps);
        }
    }
    fn transform_slot_outlet(r: &mut C::RenderSlotIR<T>, ps: &mut P) {
        ps.enter_slot_outlet(r);
        Self::transform_js_expr(&mut r.slot_name, ps);
        if let Some(props) = r.slot_props.as_mut() {
            Self::transform_js_expr(props, ps);
        }
        Self::transform_children(&mut r.fallbacks, ps);
        ps.exit_slot_outlet(r);
    }
    fn transform_v_slot(s: &mut C::VSlotIR<T>, ps: &mut P) {
        ps.enter_v_slot(s);
        for slot in s.stable_slots.iter_mut() {
            Self::transform_slot_fn(slot, ps);
        }
        for slot in s.alterable_slots.iter_mut() {
            Self::transform_ir(slot, ps);
        }
        ps.exit_v_slot(s);
    }
    fn transform_slot_fn(slot: &mut C::Slot<T>, ps: &mut P) {
        ps.enter_slot_fn(slot);
        Self::transform_js_expr(&mut slot.name, ps);
        // slot param as fn_param, note: visit param after slot_fn
        // since v-slot has no bind props that depend on slot param
        if let Some(p) = &mut slot.param {
            ps.enter_fn_param(p);
        }
        Self::transform_children(&mut slot.body, ps);
        if let Some(p) = &mut slot.param {
            Self::transform_js_expr(p, ps);
            ps.exit_fn_param(p);
        }
        ps.exit_slot_fn(slot);
    }
    fn transform_comment(c: &mut T::CommentType, ps: &mut P) {
        ps.enter_comment(c);
        ps.exit_comment(c);
    }
}

pub struct BaseTransformer<'a, P: CorePass<BaseInfo<'a>>> {
    pass: P,
    pd: PhantomData<&'a ()>,
}
impl<'a, P: CorePass<BaseInfo<'a>>> BaseTransformer<'a, P> {
    pub fn new(pass: P) -> Self {
        Self {
            pass,
            pd: PhantomData,
        }
    }
}

impl<'a, P: CorePass<BaseInfo<'a>>> Transformer for BaseTransformer<'a, P> {
    type IR = BaseRoot<'a>;
    fn transform(&mut self, node: &mut Self::IR) {
        Self::transform_root(node, &mut self.pass);
    }
}

impl<'a, P> CoreTransformer<BaseInfo<'a>, P> for BaseTransformer<'a, P>
where
    P: CorePass<BaseInfo<'a>>,
{
    fn transform_root(r: &mut IRRoot<BaseInfo<'a>>, ps: &mut P) {
        ps.enter_root(r);
        Self::transform_children(&mut r.body, ps);
        ps.exit_root(r);
    }

    fn transform_js_expr(e: &mut Js<'a>, ps: &mut P) {
        ps.enter_js_expr(e);
        match e {
            Js::Call(_, args) => {
                for arg in args.iter_mut() {
                    Self::transform_js_expr(arg, ps);
                }
            }
            Js::Compound(exprs) => {
                for expr in exprs.iter_mut() {
                    Self::transform_js_expr(expr, ps);
                }
            }
            Js::Array(arr) => {
                for item in arr.iter_mut() {
                    Self::transform_js_expr(item, ps);
                }
            }
            Js::Props(props) => {
                for (key, val) in props.iter_mut() {
                    Self::transform_js_expr(key, ps);
                    Self::transform_js_expr(val, ps);
                }
            }
            Js::FuncCompound(..) => {
                panic!("synthetic func expr should not be transformed")
            }
            Js::Src(_)
            | Js::Num(_)
            | Js::Simple(..)
            | Js::Param(_)
            | Js::FuncSimple(..)
            | Js::StrLit(_)
            | Js::Symbol(_) => {
                // no further recursion.
            }
        }
        ps.exit_js_expr(e);
    }
}

#[cfg(test)]
mod test {
    use super::pass::{Scope, SharedInfoPasses};
    use super::*;
    pub use crate::converter::test::base_convert;
    use rustc_hash::FxHashMap;
    pub fn get_transformer<'a, P>(pass: P) -> BaseTransformer<'a, P>
    where
        P: CorePass<BaseInfo<'a>> + 'static,
    {
        BaseTransformer {
            pass,
            pd: PhantomData,
        }
    }

    pub fn transformer_ext<'a, Ps: CorePassExt<BaseInfo<'a>, Scope<'a>>>(
        passes: Ps,
    ) -> BaseTransformer<'a, SharedInfoPasses<BaseInfo<'a>, Ps, Scope<'a>>> {
        let pass = SharedInfoPasses {
            passes,
            shared_info: Scope {
                identifiers: FxHashMap::default(),
            },
            pd: PhantomData,
        };
        BaseTransformer {
            pass,
            pd: PhantomData,
        }
    }
}
