/// hoist static element like `<div class="static">static text</div>`
/// to a top level const. This improves runtime performance by reducing dom diffing.
use super::{BaseInfo, BaseVNode, BaseRoot, CorePass, Js};
use crate::converter::BaseIR;
use crate::ir::IRNode;
use crate::flags::{StaticLevel, PatchFlag};

use std::marker::PhantomData;

pub struct HoistStatic<'a> {
    lifetime: PhantomData<&'a ()>,
}

impl<'a> CorePass<BaseInfo<'a>> for HoistStatic<'a> {
    fn exit_root(&mut self, r: &mut BaseRoot<'a>) {
        // Root node is unfortunately non-hoistable due to potential parent
        // fallthrough attributes.
        let bail_out_hoist = is_single_element_root(r);
        self.walk_children(&mut r.body, bail_out_hoist);
    }
}

fn is_plain_element(node: &BaseVNode) -> bool {
    !node.is_component && matches!(node.tag, Js::StrLit(_))
}

impl<'a> HoistStatic<'a> {
    fn walk(&mut self, node: &mut BaseVNode<'a>, bail_out_hoist: bool) {
        let all_children_hoisted = self.walk_children(&mut node.children, bail_out_hoist);
        if all_children_hoisted && is_plain_element(node) {
            todo!()
        }
    }

    fn walk_children(&mut self, children: &mut [BaseIR<'a>], bail_out_hoist: bool) -> bool {
        let original_count = children.len();
        let mut hoist_count = 0;
        for child in children {
            hoist_count += if self.walk_child(child, bail_out_hoist) {
                1
            } else {
                0
            };
        }
        if hoist_count > 0 {
            // call additional transform hook
            // if (hoistedCount && context.transformHoist) {
            //     context.transformHoist(children, context, node)
            // }
        }
        hoist_count > 0 && hoist_count == original_count
    }

    fn walk_child(&mut self, child: &mut BaseIR<'a>, bail_out_hoist: bool) -> bool {
        if let IRNode::VNodeCall(e) = child {
            if e.is_component {}
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
                    return true;
                }
            } else {
                let patch_flag = e.patch_flag;
                if (patch_flag.is_empty()
                    || patch_flag == PatchFlag::NEED_PATCH
                    || patch_flag == PatchFlag::TEXT)
                    && get_generated_props_static_level(e) >= StaticLevel::CanHoist
                {
                    // if let Some(props) = get_node_props(e) {
                    //     *child = self.hoist(props);
                    // }
                }
                if !e.dynamic_props.is_empty() {
                    todo!()
                    // e.dynamic_props = self.hoist(e.dynamic_props);
                }
            }
        }

        // walk further
        match child {
            IRNode::VNodeCall(_) => {
                // visit child
            }
            IRNode::For(_) => {}
            IRNode::If(_) => {}
            _ => (),
        }
        false
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

fn get_generated_props_static_level(_node: &BaseVNode) -> StaticLevel {
    todo!()
}

fn get_node_props<'a>(_node: &BaseVNode<'a>) -> Option<BaseVNode<'a>> {
    todo!()
}
