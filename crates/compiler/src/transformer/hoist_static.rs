/// hoist static element like `<div class="static">static text</div>`
/// to a top level const. This improves runtime performance by reducing dom diffing.
use super::{BaseInfo, BaseVNode, BaseRoot, CorePass, Js, BaseText};
use crate::converter::{BaseIR, Hoist};
use crate::ir::IRNode;
use crate::flags::{StaticLevel, PatchFlag};

#[derive(Default)]
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
                get_vnode_static_level(e)
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
                    if let Some(props) = e.props.take() {
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

fn get_static_level(node: &BaseIR) -> StaticLevel {
    match node {
        IRNode::VNodeCall(e) => get_vnode_static_level(e),
        IRNode::TextCall(t) => get_text_call_static_level(t),
        IRNode::CommentCall(_) => StaticLevel::CanHoist,
        IRNode::CacheNode(_) => StaticLevel::NotStatic,
        IRNode::If(_) | IRNode::For(_) | IRNode::VSlotUse(_) => StaticLevel::NotStatic,
        IRNode::RenderSlotCall(_) | IRNode::AlterableSlot(_) => StaticLevel::NotStatic,
        IRNode::Hoisted(_) => StaticLevel::CanHoist,
    }
}

fn get_text_call_static_level(t: &BaseText) -> StaticLevel {
    t.texts
        .iter()
        .map(|t| t.static_level())
        .min()
        .unwrap_or(StaticLevel::CanStringify)
}

fn get_vnode_static_level(node: &BaseVNode) -> StaticLevel {
    if !is_plain_element(node) {
        return StaticLevel::NotStatic;
    }
    // TODO: add constantCache
    if node.is_block && !matches!(node.tag, Js::StrLit(v) if &*v == "svg" || &*v == "foreignObject")
    {
        return StaticLevel::NotStatic;
    }
    if !node.patch_flag.is_empty() {
        return StaticLevel::NotStatic;
    }
    let mut return_type = StaticLevel::CanStringify;

    // Element itself has no patch flag. However we still need to check:

    // 1. Even for a node with no patch flag, it is possible for it to contain
    // non-hoistable expressions that refers to scope variables, e.g. compiler
    // injected keys or cached event handlers. Therefore we need to always
    // check the codegenNode's props to be sure.
    let generated_props_level = get_generated_props_static_level(node);
    if generated_props_level == StaticLevel::NotStatic {
        return StaticLevel::NotStatic;
    }
    return_type = return_type.min(generated_props_level);
    // 2. its children.
    for child in &node.children {
        let child_level = get_static_level(child);
        if child_level == StaticLevel::NotStatic {
            return StaticLevel::NotStatic;
        }
        return_type = return_type.min(child_level);
    }
    // 3. if the type is not already CAN_SKIP_PATCH which is the lowest non-0
    // type, check if any of the props can cause the type to be lowered
    // we can skip can_patch because it's guaranteed by the absence of a
    // patchFlag.
    if return_type > StaticLevel::CanSkipPatch {
        if let Some(prop) = &node.props {
            if prop.static_level() == StaticLevel::NotStatic {
                return StaticLevel::NotStatic;
            }
            return_type = return_type.min(prop.static_level());
        }
    }
    // only svg/foreignObject could be block here, however if they are
    // static then they don't need to be blocks since there will be no
    // nested updates.
    if node.is_block {
        // except set custom directives.
        if !node.directives.is_empty() {
            return StaticLevel::NotStatic;
        }
        //   context.removeHelper(OPEN_BLOCK)
        //   context.removeHelper(
        //     getVNodeBlockHelper(context.inSSR, codegenNode.isComponent)
        //   )
        // node.is_block = false;
        //   context.helper(getVNodeHelper(context.inSSR, codegenNode.isComponent))
    }
    return_type
}

fn get_generated_props_static_level(node: &BaseVNode) -> StaticLevel {
    if let Some(prop) = &node.props {
        prop.static_level()
    } else {
        StaticLevel::CanStringify
    }
}
