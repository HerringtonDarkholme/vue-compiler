use super::{
    build_props::{build_props, BuildProps},
    BaseConverter, BaseIR, Element, IRNode, JsExpr as Js, VNodeIR, VStr,
};
use crate::core::{
    flags::{PatchFlag, RuntimeHelper},
    parser::{Directive, ElemProp, ElementType},
    tokenizer::Attribute,
    util::{find_prop, get_core_component},
};

pub fn convert_element<'a>(bc: &BaseConverter, e: Element<'a>) -> BaseIR<'a> {
    let tag = resolve_element_tag(&e, bc);
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
        is_component: e.tag_type == ElementType::Component,
    };
    IRNode::VNodeCall(vnode)
}

pub fn convert_template<'a>(bc: &BaseConverter, e: Element<'a>) -> BaseIR<'a> {
    todo!()
}

/// Returns a expression for createVnode's first argument. It can be
/// 1. Js::Call for dynamic component or user component.
/// 2. Js::Symbol for builtin component
/// 3. Js::StrLit for plain element
pub fn resolve_element_tag<'a>(e: &Element<'a>, bc: &BaseConverter) -> Js<'a> {
    if e.tag_type == ElementType::Plain {
        return Js::StrLit(VStr::raw(e.tag_name));
    }
    let is_explicit_dynamic = is_component_tag(e.tag_name);
    // 1. resolve dynamic component
    let tag = match resolve_dynamic_component(e, is_explicit_dynamic) {
        Ok(call_expr) => return call_expr,
        Err(tag_name) => tag_name,
    };
    // 1.5 v-is (deprecated)
    if let Some(call_expr) = resolve_v_is_component(e, is_explicit_dynamic) {
        return call_expr;
    }
    // 2. built-in components (Teleport, Transition, KeepAlive, Suspense...)
    // if is_core_component(tag) || bc.is_builtin_component(tag) {
    //     // TODO: make sure SSR helper does nothing since
    //     // built-ins are simply fallthroughs / have special handling during ssr
    //     // so we don't need to import their runtime equivalents

    // }
    // 3. user component (from setup bindings)
    // 4. Self referencing component (inferred from filename)
    // 5. user component (resolve)
    todo!()
}

const NON_EMPTY_ASSERTION: &str = "find_prop must return prop with non-empty value";
/// Returns Ok if resolved as dynamic component call, Err if resolved as static string tag
fn resolve_dynamic_component<'a>(
    e: &Element<'a>,
    is_explicit_dynamic: bool,
) -> Result<Js<'a>, &'a str> {
    let is_prop = find_prop(e, "is");
    let prop = match is_prop {
        Some(prop) => prop,
        None => return Err(e.tag_name),
    };
    if is_explicit_dynamic {
        let exp = match prop.get_ref() {
            ElemProp::Attr(Attribute {
                value: Some(val), ..
            }) => Js::StrLit(val.content),
            ElemProp::Dir(Directive {
                expression: Some(exp),
                ..
            }) => Js::Simple(exp.content),
            _ => panic!("{}", NON_EMPTY_ASSERTION),
        };
        return Ok(Js::Call(
            RuntimeHelper::ResolveDynamicComponent.helper_str(),
            vec![exp],
        ));
    }
    if let ElemProp::Attr(Attribute {
        value: Some(val), ..
    }) = prop.get_ref()
    {
        // if not <component>, e.g. <button is="vue:xxx">
        // only `is` value that starts with "vue:" will be
        // treated as component by the parse phase and reach here
        debug_assert!(val.content.starts_with("vue:"));
        return Err(&val.content.raw[4..]); // strip vue:
    }
    Err(e.tag_name)
}

/// Returns dynamic component call if we found v-is, otherwise None
fn resolve_v_is_component<'a>(e: &Element<'a>, is_explicit_dynamic: bool) -> Option<Js<'a>> {
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

fn is_component_tag(tag: &str) -> bool {
    tag == "component" || tag == "Component"
}
