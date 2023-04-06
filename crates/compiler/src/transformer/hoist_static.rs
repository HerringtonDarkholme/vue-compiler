/// hoist static element like `<div class="static">static text</div>`
/// to top level. This improves runtime performance by reducing dom diffiing.
use super::{BaseInfo, BaseVNode, BaseRoot, CorePass};
use crate::converter::BaseIR;
use crate::ir::IRNode;
use crate::flags::{StaticLevel, PatchFlag};

pub struct HoistStatic<'a> {
    statics: Vec<BaseVNode<'a>>,
    /// a mark to skip hoisting due to parent/global context.
    /// e.g. root node is not hoistable. Single v-for is not hoistable.
    bail_out_hoist: Vec<bool>,
}

impl<'a> CorePass<BaseInfo<'a>> for HoistStatic<'a> {
    fn enter_root(&mut self, r: &mut BaseRoot<'a>) {
        debug_assert!(self.bail_out_hoist.is_empty());
        self.bail_out_hoist.push(is_single_element_root(r));
    }
    fn exit_root(&mut self, _r: &mut crate::ir::IRRoot<BaseInfo<'a>>) {
        self.bail_out_hoist.pop();
        debug_assert!(self.bail_out_hoist.is_empty());
    }
}
impl<'a> HoistStatic<'a> {
    fn walk_root(&mut self, r: &mut BaseRoot<'a>) {}

    fn walk_chilren(&mut self, children: &mut [BaseIR<'a>], bail_out_hoist: bool) -> usize {
        let mut hoist_count = 0;
        for child in children {
            if let IRNode::VNodeCall(e) = child {
                let static_level = if bail_out_hoist {
                    StaticLevel::NotStatic
                } else {
                    get_static_level(e)
                };
                if static_level > StaticLevel::NotStatic {
                    if static_level >= StaticLevel::CanHoist {
                        e.patch_flag = PatchFlag::HOISTED;
                        let e = std::mem::take(e);
                        *child = self.hoist(e);
                        hoist_count += 1;
                        continue;
                    }
                } else {
                    todo!()
                }
            }
        }
        hoist_count
    }

    fn hoist(&mut self, _expr: BaseVNode<'a>) -> BaseIR<'a> {
        todo!()
        // if (isString(exp)) exp = createSimpleExpression(exp)
        // context.hoists.push(exp)
        // const identifier = createSimpleExpression(
        //   `_hoisted_${context.hoists.length}`,
        //   false,
        //   exp.loc,
        //   ConstantTypes.CAN_HOIST
        // )
        // identifier.hoisted = exp
        // return identifier
    }
}

fn is_single_element_root(r: &BaseRoot) -> bool {
    if r.body.len() != 1 {
        return false;
    }
    let first = r.body.first().unwrap();
    matches!(first, IRNode::VNodeCall(a) if !a.is_component)
}

fn get_static_level(_node: &BaseVNode) -> StaticLevel {
    todo!()
}
