use super::{
    build_props::{build_props, BuildProps},
    BaseConverter as BC, BaseIR, BindingTypes, CoreConverter, Element, IRNode, JsExpr as Js,
    VNodeIR, VStr,
};
use crate::core::{
    flags::{PatchFlag, RuntimeHelper},
    parser::{Directive, ElemProp, ElementType},
    tokenizer::Attribute,
    util::{find_dir, find_prop, get_core_component},
};

pub fn convert_element<'a>(bc: &mut BC, e: Element<'a>) -> BaseIR<'a> {
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

pub fn convert_template<'a>(bc: &BC, e: Element<'a>) -> BaseIR<'a> {
    todo!()
}

/// Returns a expression for createVnode's first argument. It can be
/// 1. Js::Call for dynamic component or user component.
/// 2. Js::Symbol for builtin component
/// 3. Js::StrLit for plain element
pub fn resolve_element_tag<'a>(e: &Element<'a>, bc: &mut BC) -> Js<'a> {
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
    let builtin = bc
        .get_builtin_component(tag)
        .or_else(|| get_core_component(tag));
    if let Some(builtin) = builtin {
        // TODO: make sure SSR helper does nothing since
        // built-ins are simply fallthroughs / have special handling during ssr
        // so we don't need to import their runtime equivalents
        bc.collect_helper(builtin);
        return Js::Symbol(builtin);
    }
    // 3. user component (from setup bindings)
    // 4. Self referencing component (inferred from filename)
    // 5. user component (resolve)
    todo!()
}

const MUST_NON_EMPTY: &str = "find_prop must return prop with non-empty value";
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
            _ => panic!("{}", MUST_NON_EMPTY),
        };
        return Ok(Js::Call(RuntimeHelper::ResolveDynamicComponent, vec![exp]));
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
    if is_explicit_dynamic {
        return None;
    }
    let dir = find_dir(e, "is")?;
    let exp = dir
        .get_ref()
        .expression
        .as_ref()
        .expect(MUST_NON_EMPTY)
        .content;
    Some(Js::Call(
        RuntimeHelper::ResolveDynamicComponent,
        vec![Js::Simple(exp)],
    ))
}

fn should_use_block() -> bool {
    todo!()
}
fn build_children<'a>(e: &Element<'a>) -> (Vec<BaseIR<'a>>, PatchFlag) {
    todo!()
}

// TODO: externalize this into the CoreConverter trait
fn resolve_setup_reference<'a>(name: &'a str, bc: &BC) -> Option<VStr<'a>> {
    use crate::core::util::Lazy;
    let bindings = &bc.binding_metadata;
    if bindings.is_empty() || !bindings.is_setup() {
        return None;
    }
    let camel_name = *VStr::raw(name).camelize();
    let pascal_name = *VStr::raw(name).capitalize();
    let name = VStr::raw(name);
    // TODO: remove the lazy using a better VStr instead
    let camel = Lazy::new(|| camel_name.into_string());
    let pascal = Lazy::new(|| pascal_name.into_string());
    let check_type = |tpe: BindingTypes| {
        let is_match = |n: &str| Some(bindings.get(n)? == &tpe);
        if is_match(&name)? {
            Some(name)
        } else if is_match(&camel)? {
            Some(camel_name)
        } else if is_match(&pascal)? {
            Some(pascal_name)
        } else {
            None
        }
    };
    todo!()
}

fn is_component_tag(tag: &str) -> bool {
    tag == "component" || tag == "Component"
}
