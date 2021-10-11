use compiler::transformer::{CorePass, BaseVNode};
use compiler::converter::BaseConvertInfo as BaseInfo;

struct UsageWarner;

impl<'a> CorePass<BaseInfo<'a>> for UsageWarner {
    fn enter_vnode(&mut self, _r: &mut BaseVNode<'a>) {}
}
