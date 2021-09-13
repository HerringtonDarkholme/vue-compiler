use super::{
    super::error::{CompilationError, CompilationErrorKind as ErrorKind},
    build_props::{build_props, BuildProps},
    BaseConverter as BC, BaseIR, BindingMetadata, BindingTypes, CoreConverter, Element, IRNode,
    JsExpr as Js, VNodeIR, VStr,
};
use crate::core::{
    flags::{PatchFlag, RuntimeHelper},
    parser::{Directive, ElemProp, ElementType},
    tokenizer::Attribute,
    util::{find_dir, get_core_component, is_component_tag, prop_finder},
};
use rustc_hash::FxHashSet;
use std::mem;

pub fn convert_element<'a>(bc: &mut BC, mut e: Element<'a>) -> BaseIR<'a> {
    debug_assert!(matches!(
        e.tag_type,
        ElementType::Plain | ElementType::Component
    ));
    let tag = resolve_element_tag(bc, &e);
    let is_block = should_use_block(&e, &tag);
    // curiously, we should first build children instead of props
    // since we will pre-convert and consume v-slot here.
    let (children, more_flags) = build_children(bc, &e);
    let properties = mem::take(&mut e.properties);
    let BuildProps {
        props,
        directives,
        mut patch_flag,
        dynamic_prop_names,
    } = build_props(bc, &e, properties);
    let directives = build_directive_args(directives);
    patch_flag |= more_flags;
    let dynamic_props = stringify_dynamic_prop_names(dynamic_prop_names);
    let vnode = VNodeIR {
        tag,
        props,
        directives,
        dynamic_props,
        children,
        patch_flag,
        is_block,
        disable_tracking: false,
        is_component: e.is_component(),
    };
    IRNode::VNodeCall(vnode)
}

pub fn convert_template<'a>(bc: &BC, e: Element<'a>) -> BaseIR<'a> {
    todo!()
}

/// Returns a expression for createVnode's first argument. It can be
/// 1. Js::Call for dynamic component or user component.
/// 2. Js::Symbol for builtin component
/// 3. Js::StrLit for plain element or component
pub fn resolve_element_tag<'a>(bc: &mut BC, e: &Element<'a>) -> Js<'a> {
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
        // TODO: ensure SSR does not collect this. since built-ins are simply fallthroughs
        // or have special handling during compilation so we don't need to import their runtime
        return Js::Symbol(builtin);
    }
    // 3. user component (from setup bindings)
    if let Some(from_setup) = resolve_setup_component(bc, tag) {
        return from_setup;
    }
    // 4. User component or Self referencing component (inferred from filename)
    let mut comp = VStr::raw(tag);
    if VStr::raw(tag).camelize().capitalize().into_string() == bc.self_name {
        // codegen special checks for __self postfix when generating component imports,
        // which will pass additional `maybeSelfReference` flag to `resolveComponent`.
        comp.suffix_self();
    }
    // 5. user component (resolve)
    bc.add_component(comp);
    Js::StrLit(*comp.clone().be_asset()) // use clone to avoid mutating comp
}

const MUST_NON_EMPTY: &str = "find_prop must return prop with non-empty value";
/// Returns Ok if resolved as dynamic component call, Err if resolved as static string tag
fn resolve_dynamic_component<'a>(
    e: &Element<'a>,
    is_explicit_dynamic: bool,
) -> Result<Js<'a>, &'a str> {
    let is_prop = prop_finder(e, "is").find();
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
            }) => Js::simple(exp.content),
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
        vec![Js::simple(exp)],
    ))
}

fn should_use_block<'a>(e: &Element<'a>, tag: &Js<'a>) -> bool {
    use RuntimeHelper as H;
    match tag {
        // dynamic component may resolve to plain element
        Js::Call(H::ResolveDynamicComponent, _) => return true,
        Js::Symbol(H::Teleport) | Js::Symbol(H::Suspense) => return true,
        _ => {
            if e.is_component() {
                return false;
            }
        }
    }
    // <svg> and <foreignObject> must be forced into blocks so that block
    // updates inside get proper isSVG flag at runtime. (vue-next/#639, #643)
    // Technically web-specific, but splitting out of core is too complex
    e.tag_name == "svg" || e.tag_name == "foreinObject" ||
    // vue-next/#938: elements with dynamic keys should be forced into blocks
    prop_finder(e, "key").dynamic_only().find().is_some()
}

fn build_directive_args(dirs: Vec<(Directive, Option<RuntimeHelper>)>) -> Option<Js> {
    todo!()
}

fn build_children<'a>(bc: &mut BC, e: &Element<'a>) -> (Vec<BaseIR<'a>>, PatchFlag) {
    if let Some(found) = find_dir(e, "slot") {
        debug_assert!(e.tag_type != ElementType::Template);
        let dir = found.get_ref();
        if !e.is_component() {
            let error = CompilationError::new(ErrorKind::VSlotMisplaced)
                .with_location(dir.location.clone());
            bc.emit_error(error);
        }
    }
    if e.tag_name == "slot" {}
    todo!()
}

fn stringify_dynamic_prop_names(prop_names: FxHashSet<VStr>) -> Option<Js> {
    todo!()
}

fn resolve_setup_component<'a>(bc: &BC, tag: &'a str) -> Option<Js<'a>> {
    if let Some(from_setup) = resolve_setup_reference(bc, tag) {
        return Some(from_setup);
    }
    // handle <obj.Tag/>
    let no_leading_trailing = |&i: &usize| i != 0 && i < tag.len() - 1;
    let dot_index = tag.find('.').filter(no_leading_trailing)?; // exclude .tag or obj.
    let (ns, access) = tag.split_at(dot_index);
    let ns = resolve_setup_reference(bc, ns)?;
    Some(Js::Compound(vec![ns, Js::Src(access)]))
}

// TODO: externalize this into the CoreConverter trait
/// returns the specific name created in script setup, modulo camel/pascal case
fn resolve_setup_reference<'a>(bc: &BC, name: &'a str) -> Option<Js<'a>> {
    let bindings = &bc.binding_metadata;
    if bindings.is_empty() || !bindings.is_setup() {
        return None;
    }
    // the returned closure will find the name modulo casing
    let varienty_by_type = get_variety_from_binding(name, bindings);
    if let Some(from_const) = varienty_by_type(BindingTypes::SetupConst) {
        return Some(if bc.inline {
            Js::simple(from_const)
        } else {
            Js::Compound(vec![
                Js::Src("$setup["),
                Js::StrLit(from_const),
                Js::Src("]"),
            ])
        });
    }
    let from_maybe_ref = varienty_by_type(BindingTypes::SetupLet)
        .or_else(|| varienty_by_type(BindingTypes::SetupRef))
        .or_else(|| varienty_by_type(BindingTypes::SetupMaybeRef));
    if let Some(maybe_ref) = from_maybe_ref {
        return Some(if bc.inline {
            Js::Call(RuntimeHelper::Unref, vec![Js::simple(maybe_ref)])
        } else {
            Js::Compound(vec![
                Js::Src("$setup["),
                Js::StrLit(maybe_ref),
                Js::Src("]"),
            ])
        });
    }
    None
}

#[inline(always)]
fn get_variety_from_binding<'a: 'b, 'b>(
    name: &'a str,
    bindings: &'b BindingMetadata,
) -> impl Fn(BindingTypes) -> Option<VStr<'a>> + 'b {
    use crate::core::util::Lazy;
    let camel_name = *VStr::raw(name).camelize();
    let pascal_name = *VStr::raw(name).capitalize();
    let name = VStr::raw(name);
    // TODO: remove the lazy using a better VStr instead
    let camel = Lazy::new(move || camel_name.into_string());
    let pascal = Lazy::new(move || pascal_name.into_string());
    move |tpe: BindingTypes| {
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
    }
}
