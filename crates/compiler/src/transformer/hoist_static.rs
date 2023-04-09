/// hoist static element like `<div class="static">static text</div>`
/// to a top level const. This improves runtime performance by reducing dom diffing.
use super::{BaseInfo, BaseVNode, BaseRoot, CorePass, Js};
use crate::converter::{BaseIR, Hoist};
use crate::ir::IRNode;
use crate::flags::{StaticLevel, PatchFlag};

pub struct HoistStatic<'a> {
    hoists: Vec<Hoist<'a>>,
}

impl<'a> CorePass<BaseInfo<'a>> for HoistStatic<'a> {
    fn exit_root(&mut self, r: &mut BaseRoot<'a>) {
        // Root node is unfortunately non-hoistable due to potential parent
        // fallthrough attributes.
        let bail_out_hoist = is_single_element_root(r);
        self.walk_children(&mut r.body, bail_out_hoist);
        std::mem::swap(&mut r.top_scope.hoists, &mut self.hoists);
    }
}

fn is_plain_element(node: &BaseVNode) -> bool {
    !node.is_component && matches!(node.tag, Js::StrLit(_))
}

fn extract_plain_element<'a, 'b>(ir: &'a mut BaseIR<'b>) -> Option<&'a mut BaseVNode<'b>> {
    if let IRNode::VNodeCall(e) = ir {
        if is_plain_element(e) {
            return Some(e);
        }
    }
    None
}

impl<'a> HoistStatic<'a> {
    fn walk_vnode(&mut self, node: &mut BaseVNode<'a>, bail_out_hoist: bool) {
        let all_children_hoisted = self.walk_children(&mut node.children, bail_out_hoist);
        if all_children_hoisted && is_plain_element(node) {
            let children = std::mem::take(&mut node.children);
            let index = self.hoist(Hoist::ChildrenArray(children));
            node.hoisted.add_children(index);
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
        if let Some(e) = extract_plain_element(child) {
            let static_level = if bail_out_hoist {
                StaticLevel::NotStatic
            } else {
                get_static_level(e)
            };
            if static_level > StaticLevel::NotStatic {
                if static_level >= StaticLevel::CanHoist {
                    e.patch_flag = PatchFlag::HOISTED;
                    let e = Hoist::FullElement(std::mem::take(e));
                    *child = IRNode::Hoisted(self.hoist(e));
                    return true;
                }
            } else {
                let patch_flag = e.patch_flag;
                if (patch_flag.is_empty()
                    || patch_flag == PatchFlag::NEED_PATCH
                    || patch_flag == PatchFlag::TEXT)
                    && get_generated_props_static_level(e) >= StaticLevel::CanHoist
                {
                    if let Some(props) = take_node_props(e) {
                        let i = self.hoist(Hoist::StaticProps(props));
                        e.hoisted.add_props(i);
                    }
                }
                if !e.dynamic_props.is_empty() {
                    let dynamic = std::mem::take(&mut e.dynamic_props);
                    let index = self.hoist(Hoist::DynamicPropsHint(dynamic));
                    e.hoisted.add_dynamic_props(index);
                }
            }
        }

        // walk further
        match child {
            IRNode::VNodeCall(e) => {
                if e.is_component {
                    //context.scopes.vSlot++
                }
                self.walk_vnode(e, false);
                if e.is_component {
                    //context.scopes.vSlot--
                }
            }
            IRNode::For(ir) => {
                // Do not hoist v-for single child because it has to be a block
                let bail_out_hoist = match &*ir.child {
                    IRNode::VNodeCall(e) => e.children.len() == 1,
                    _ => false,
                };
                self.walk_child(&mut ir.child, bail_out_hoist);
            }
            IRNode::If(ir) => {
                for branch in &mut ir.branches {
                    // Do not hoist v-for single child because it has to be a block
                    let bail_out_hoist = match &*branch.child {
                        IRNode::VNodeCall(e) => e.children.len() == 1,
                        _ => false,
                    };
                    self.walk_child(&mut branch.child, bail_out_hoist);
                }
            }
            _ => (),
        }
        false
    }

    fn hoist(&mut self, expr: Hoist<'a>) -> usize {
        let len = self.hoists.len();
        self.hoists.push(expr);
        len
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

fn take_node_props<'a>(_node: &mut BaseVNode<'a>) -> Option<Js<'a>> {
    todo!()
}
