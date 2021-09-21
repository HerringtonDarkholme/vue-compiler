use super::{ConvertInfo, IRRoot, C};

pub trait CorePass<T: ConvertInfo> {
    fn enter_root(&mut self, r: &mut IRRoot<T>) {}
    fn exit_root(&mut self, r: &mut IRRoot<T>) {}
    fn enter_text(&mut self, t: &mut T::TextType) {}
    fn exit_text(&mut self, t: &mut T::TextType) {}
    fn enter_if(&mut self, i: &mut C::IfNodeIR<T>) {}
    fn exit_if(&mut self, i: &mut C::IfNodeIR<T>) {}
    fn enter_for(&mut self, f: &mut C::ForNodeIR<T>) {}
    fn exit_for(&mut self, f: &mut C::ForNodeIR<T>) {}
    fn enter_vnode(&mut self, v: &mut C::VNodeIR<T>) {}
    fn exit_vnode(&mut self, v: &mut C::VNodeIR<T>) {}
    fn enter_slot_outlet(&mut self, r: &mut C::RenderSlotIR<T>) {}
    fn exit_slot_outlet(&mut self, r: &mut C::RenderSlotIR<T>) {}
    fn enter_v_slot(&mut self, s: &mut C::VSlotIR<T>) {}
    fn exit_v_slot(&mut self, s: &mut C::VSlotIR<T>) {}
    fn enter_slot_fn(&mut self, s: &mut C::Slot<T>) {}
    fn exit_slot_fn(&mut self, s: &mut C::Slot<T>) {}
    fn enter_js_expr(&mut self, e: &mut T::JsExpression) {}
    fn exit_js_expr(&mut self, e: &mut T::JsExpression) {}
    fn enter_comment(&mut self, c: &mut T::CommentType) {}
    fn exit_comment(&mut self, c: &mut T::CommentType) {}
}

pub struct MergedPass<P, const N: usize> {
    passes: [P; N],
}

impl<P, const N: usize> MergedPass<P, N> {
    fn enter<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut P),
    {
        for p in &mut self.passes {
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

impl<T, Pass, const N: usize> CorePass<T> for MergedPass<Pass, N>
where
    T: ConvertInfo,
    Pass: CorePass<T>,
{
    fn enter_root(&mut self, r: &mut IRRoot<T>) {
        self.enter(|p| p.enter_root(r))
    }
    fn exit_root(&mut self, r: &mut IRRoot<T>) {
        self.exit(|p| p.exit_root(r))
    }
    fn enter_text(&mut self, t: &mut T::TextType) {
        self.enter(|p| p.enter_text(t))
    }
    fn exit_text(&mut self, t: &mut T::TextType) {
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
    fn enter_comment(&mut self, c: &mut T::CommentType) {
        self.enter(|p| p.enter_comment(c))
    }
    fn exit_comment(&mut self, c: &mut T::CommentType) {
        self.exit(|p| p.exit_comment(c))
    }
}

pub trait CorePassExt<T: ConvertInfo, Shared> {
    fn enter_root(&mut self, r: &mut IRRoot<T>, shared: &mut Shared) {}
    fn exit_root(&mut self, r: &mut IRRoot<T>, shared: &mut Shared) {}
}

pub struct SharedInfoPasses<Pass, Shared, const N: usize> {
    passes: MergedPass<Pass, N>,
    shared_info: Shared,
}
// TODO: add transform used
impl<T, Pass, Shared, const N: usize> CorePass<T> for SharedInfoPasses<Pass, Shared, N>
where
    T: ConvertInfo,
    Pass: CorePassExt<T, Shared>,
{
    fn enter_root(&mut self, r: &mut IRRoot<T>) {
        let shared = &mut self.shared_info;
        self.passes.enter(|p| p.enter_root(r, shared));
    }
    fn exit_root(&mut self, r: &mut IRRoot<T>) {
        let shared = &mut self.shared_info;
        self.passes.exit(|p| p.exit_root(r, shared));
    }
}
