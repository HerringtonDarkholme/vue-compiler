use compiler::transformer::{CorePass, BaseVNode};
use compiler::converter::{BaseConvertInfo as BaseInfo, RcErrHandle};
use compiler::error::{CompilationError};
use crate::extension::{TRANSITION, DomError};
use compiler::ir::JsExpr as Js;

struct UsageWarner(RcErrHandle);

impl<'a> CorePass<BaseInfo<'a>> for UsageWarner {
    fn enter_vnode(&mut self, vn: &mut BaseVNode<'a>) {
        match vn.tag {
            Js::Symbol(TRANSITION) if has_multiple_children(vn) => (),
            _ => return,
        }
        let error = CompilationError::extended(DomError::TransitionInvalidChildren);
        self.0.on_error(error);
    }
}

fn has_multiple_children(vn: &BaseVNode) -> bool {
    todo!()
}
