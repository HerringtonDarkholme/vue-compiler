use super::{BaseInfo, BaseTransformer, BaseVNode, ConvertInfo, CoreTransformer, Js, C};
use crate::ir::IRRoot;
use crate::Name;
use rustc_hash::FxHashMap;

macro_rules! impl_enter {
    ($impl: ident) => {
        $impl!(enter_root, IRRoot);
        $impl!(enter_text, TextIR);
        $impl!(enter_if, IfNodeIR);
        $impl!(enter_for, ForNodeIR);
        $impl!(enter_vnode, VNodeIR);
        $impl!(enter_slot_outlet, RenderSlotIR);
        $impl!(enter_v_slot, VSlotIR);
        $impl!(enter_slot_fn, Slot);
        $impl!(enter_js_expr, JsExpression);
        $impl!(enter_fn_param, JsExpression);
        $impl!(enter_comment, CommentType);
    };
}
macro_rules! impl_exit {
    ($impl: ident) => {
        $impl!(exit_root, IRRoot);
        $impl!(exit_text, TextIR);
        $impl!(exit_if, IfNodeIR);
        $impl!(exit_for, ForNodeIR);
        $impl!(exit_vnode, VNodeIR);
        $impl!(exit_slot_outlet, RenderSlotIR);
        $impl!(exit_v_slot, VSlotIR);
        $impl!(exit_slot_fn, Slot);
        $impl!(exit_js_expr, JsExpression);
        $impl!(exit_fn_param, JsExpression);
        $impl!(exit_comment, CommentType);
    };
}
macro_rules! noop_pass {
    ($method: ident, $ty: ident) => {
        #[inline]
        fn $method(&mut self, r: &mut C::$ty<T>) {}
    };
}

pub trait CorePass<T: ConvertInfo> {
    impl_enter!(noop_pass);
    impl_exit!(noop_pass);
    // macro output example:
    // fn enter_root(&mut self, _: &mut IRRoot<T>) {}
    // fn exit_root(&mut self, _: &mut IRRoot<T>) {}
}

macro_rules! chain_enter {
    ($method: ident, $ty: ident) => {
        #[inline]
        fn $method(&mut self, r: &mut C::$ty<T>) {
            self.first.$method(r);
            self.second.$method(r);
        }
    };
}
macro_rules! chain_exit {
    ($method: ident, $ty: ident) => {
        #[inline]
        fn $method(&mut self, r: &mut C::$ty<T>) {
            self.second.$method(r);
            self.first.$method(r);
        }
    };
}

pub struct Chain<A, B> {
    pub first: A,
    pub second: B,
}
impl<'b, T, A, B> CorePass<T> for Chain<A, B>
where
    T: ConvertInfo,
    A: CorePass<T>,
    B: CorePass<T>,
{
    impl_enter!(chain_enter);
    impl_exit!(chain_exit);
    // macro output example:
    // #[inline]
    // fn enter_root(&mut self, r: &mut IRRoot<T>) {
    //     self.first.enter_root(r);
    //     self.second.enter_root(r);
    // }
    // #[inline]
    // fn exit_root(&mut self, r: &mut IRRoot<T>) {
    //     self.second.exit_root(r);
    //     self.first.exit_root(r);
    // }
}

// TODO: refactor this to Future like chain
pub struct MergedPass<'b, P> {
    passes: &'b mut [P],
}

impl<'b, P> MergedPass<'b, P> {
    pub fn new(passes: &'b mut [P]) -> Self {
        Self { passes }
    }
    fn enter<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut P),
    {
        for p in &mut self.passes.iter_mut() {
            f(p)
        }
    }
    fn exit<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut P),
    {
        for p in self.passes.iter_mut().rev() {
            f(p)
        }
    }
}

impl<'b, T> CorePass<T> for MergedPass<'b, &'b mut dyn CorePass<T>>
where
    T: ConvertInfo,
{
    fn enter_root(&mut self, r: &mut IRRoot<T>) {
        self.enter(|p| p.enter_root(r))
    }
    fn exit_root(&mut self, r: &mut IRRoot<T>) {
        self.exit(|p| p.exit_root(r))
    }
    fn enter_text(&mut self, t: &mut C::TextIR<T>) {
        self.enter(|p| p.enter_text(t))
    }
    fn exit_text(&mut self, t: &mut C::TextIR<T>) {
        self.exit(|p| p.exit_text(t))
    }
    fn enter_if(&mut self, i: &mut C::IfNodeIR<T>) {
        self.enter(|p| p.enter_if(i))
    }
    fn exit_if(&mut self, i: &mut C::IfNodeIR<T>) {
        self.exit(|p| p.exit_if(i))
    }
    fn enter_for(&mut self, f: &mut C::ForNodeIR<T>) {
        self.enter(|p| p.enter_for(f))
    }
    fn exit_for(&mut self, f: &mut C::ForNodeIR<T>) {
        self.exit(|p| p.exit_for(f))
    }
    fn enter_vnode(&mut self, v: &mut C::VNodeIR<T>) {
        self.enter(|p| p.enter_vnode(v))
    }
    fn exit_vnode(&mut self, v: &mut C::VNodeIR<T>) {
        self.exit(|p| p.exit_vnode(v))
    }
    fn enter_slot_outlet(&mut self, r: &mut C::RenderSlotIR<T>) {
        self.enter(|p| p.enter_slot_outlet(r))
    }
    fn exit_slot_outlet(&mut self, r: &mut C::RenderSlotIR<T>) {
        self.exit(|p| p.exit_slot_outlet(r))
    }
    fn enter_v_slot(&mut self, s: &mut C::VSlotIR<T>) {
        self.enter(|p| p.enter_v_slot(s))
    }
    fn exit_v_slot(&mut self, s: &mut C::VSlotIR<T>) {
        self.exit(|p| p.exit_v_slot(s))
    }
    fn enter_slot_fn(&mut self, s: &mut C::Slot<T>) {
        self.enter(|p| p.enter_slot_fn(s))
    }
    fn exit_slot_fn(&mut self, s: &mut C::Slot<T>) {
        self.exit(|p| p.exit_slot_fn(s))
    }
    fn enter_js_expr(&mut self, e: &mut T::JsExpression) {
        self.enter(|p| p.enter_js_expr(e))
    }
    fn exit_js_expr(&mut self, e: &mut T::JsExpression) {
        self.exit(|p| p.exit_js_expr(e))
    }
    fn enter_fn_param(&mut self, m: &mut T::JsExpression) {
        self.enter(|p| p.enter_fn_param(m))
    }
    fn exit_fn_param(&mut self, m: &mut T::JsExpression) {
        self.exit(|p| p.exit_fn_param(m))
    }
    fn enter_comment(&mut self, c: &mut T::CommentType) {
        self.enter(|p| p.enter_comment(c))
    }
    fn exit_comment(&mut self, c: &mut T::CommentType) {
        self.exit(|p| p.exit_comment(c))
    }
}

pub trait CorePassExt<T: ConvertInfo, Shared> {
    fn enter_js_expr(&mut self, _: &mut T::JsExpression, _: &mut Shared) {}
    fn exit_js_expr(&mut self, _: &mut T::JsExpression, _: &mut Shared) {}
    fn enter_fn_param(&mut self, _: &mut T::JsExpression, _: &mut Shared) {}
    fn exit_fn_param(&mut self, _: &mut T::JsExpression, _: &mut Shared) {}

    fn enter_vnode(&mut self, _: &mut C::VNodeIR<T>, _: &mut Shared) {}
    fn exit_vnode(&mut self, _: &mut C::VNodeIR<T>, _: &mut Shared) {}
}

type Identifiers<'a> = FxHashMap<Name<'a>, usize>;
#[derive(Default)]
pub struct Scope<'a> {
    pub identifiers: Identifiers<'a>,
}

/// Check if an IR contains expressions that reference current context scope ids
/// e.g. identifiers referenced in the scope can skip prefixing
// TODO: has_ref will repeatedly call on vnode regardless if new ids are introduced.
// So it's a O(d^2) complexity where d is the depth of nested v-slot component.
// we can optimize it by tracking how many IDs are introduced and skip unnecessary call
// in practice it isn't a problem because stack overflow happens way faster :/
impl<'a> Scope<'a> {
    pub fn has_identifier(&self, id: Name<'a>) -> bool {
        self.identifiers.contains_key(id)
    }
    pub fn add_identifier(&mut self, id: Name<'a>) {
        *self.identifiers.entry(id).or_default() += 1;
    }
    pub fn remove_identifier(&mut self, id: Name<'a>) {
        *self.identifiers.entry(id).or_default() -= 1;
    }
    pub fn has_ref_in_vnode(&self, node: &mut BaseVNode<'a>) -> bool {
        if self.identifiers.is_empty() {
            return false;
        }
        let mut ref_finder = RefFinder(&self.identifiers, false);
        BaseTransformer::transform_vnode(node, &mut ref_finder);
        ref_finder.1
    }
}
struct RefFinder<'a, 'b>(&'b Identifiers<'a>, bool);
// TODO: implement interruptible transformer for early return
// TODO: current implmentaion has false alarms in code like below
// <comp v-for="a in source">
//  <p v-for="a in s">{{a}}</p> <- expect stable, got dynamic
// </comp>
// but it is fine since ref_usage is only for optimization
impl<'a, 'b> CorePass<BaseInfo<'a>> for RefFinder<'a, 'b> {
    fn enter_js_expr(&mut self, e: &mut Js<'a>) {
        if let Js::Simple(e, _) = e {
            if self.0.contains_key(e.raw) {
                self.1 = true;
            }
        }
    }
}

pub struct SharedInfoPasses<'b, Pass, Shared> {
    pub passes: MergedPass<'b, Pass>,
    pub shared_info: Shared,
}
// TODO: add transform used
impl<'b, T, Shared> CorePass<T> for SharedInfoPasses<'b, &'b mut dyn CorePassExt<T, Shared>, Shared>
where
    T: ConvertInfo,
{
    fn enter_js_expr(&mut self, e: &mut T::JsExpression) {
        let shared = &mut self.shared_info;
        self.passes.enter(|p| p.enter_js_expr(e, shared));
    }
    fn exit_js_expr(&mut self, e: &mut T::JsExpression) {
        let shared = &mut self.shared_info;
        self.passes.exit(|p| p.exit_js_expr(e, shared));
    }
    fn enter_fn_param(&mut self, prm: &mut T::JsExpression) {
        let shared = &mut self.shared_info;
        self.passes.enter(|p| p.enter_fn_param(prm, shared));
    }
    fn exit_fn_param(&mut self, prm: &mut T::JsExpression) {
        let shared = &mut self.shared_info;
        self.passes.exit(|p| p.exit_fn_param(prm, shared));
    }
    fn enter_vnode(&mut self, v: &mut C::VNodeIR<T>) {
        let shared = &mut self.shared_info;
        self.passes.enter(|p| p.enter_vnode(v, shared))
    }
    fn exit_vnode(&mut self, v: &mut C::VNodeIR<T>) {
        let shared = &mut self.shared_info;
        self.passes.exit(|p| p.exit_vnode(v, shared))
    }
}
