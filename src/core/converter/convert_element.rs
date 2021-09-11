use super::{
    build_props::{build_props, BuildProps},
    BaseConverter, BaseIR, Element, IRNode, JsExpr as Js, VNodeIR,
};
use crate::core::flags::PatchFlag;

pub fn convert_element<'a>(bc: &BaseConverter, e: Element<'a>) -> BaseIR<'a> {
    convert_impl(bc, e)
}
pub fn convert_component<'a>(bc: &BaseConverter, e: Element<'a>) -> BaseIR<'a> {
    convert_impl(bc, e)
}
pub fn convert_template<'a>(bc: &BaseConverter, e: Element<'a>) -> BaseIR<'a> {
    todo!()
}

pub fn convert_impl<'a>(bc: &BaseConverter, e: Element<'a>) -> BaseIR<'a> {
    let tag = resolve_component_type(&e);
    let is_block = should_use_block();
    let BuildProps {
        props,
        directives,
        dynamic_props,
        mut patch_flag,
    } = build_props(&e, "TODO");
    let (children, more_flags) = build_children(&e);
    patch_flag |= more_flags;
    let vnode = VNodeIR {
        tag,
        props,
        directives,
        dynamic_props,
        children,
        patch_flag,
        is_block,
        disable_tracking: false,
        is_component: false,
    };
    IRNode::VNodeCall(vnode)
}

pub fn resolve_component_type<'a>(e: &Element<'a>) -> Js<'a> {
    // 1. resolve dynamic component
    // 1.5 v-is (deprecated)
    // 2. built-in components (Teleport, Transition, KeepAlive, Suspense...)
    // 3. user component (from setup bindings)
    // 4. Self referencing component (inferred from filename)
    // 5. user component (resolve)
    todo!()
}

fn should_use_block() -> bool {
    todo!()
}
fn build_children<'a>(e: &Element<'a>) -> (Vec<BaseIR<'a>>, PatchFlag) {
    todo!()
}
fn resolve_setup_reference() {
    todo!()
}
