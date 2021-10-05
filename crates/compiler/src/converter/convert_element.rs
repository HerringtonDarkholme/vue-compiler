use super::{
    build_props::{build_props, BuildProps},
    v_slot, BaseConvertInfo, BaseConverter as BC, BaseIR, CoreConverter, Element, VStr,
};
use crate::{
    converter::v_slot::check_wrong_slot,
    error::{CompilationError, CompilationErrorKind as ErrorKind},
    flags::{PatchFlag, RuntimeHelper, StaticLevel},
    ir::{IRNode, JsExpr as Js, RuntimeDir, VNodeIR},
    parser::{AstNode, Directive, ElemProp, ElementType},
    scanner::Attribute,
    util::{find_dir, get_core_component, is_builtin_symbol, is_component_tag, prop_finder},
    BindingMetadata, BindingTypes, SourceLocation,
};
use std::{iter, mem};

pub fn convert_element<'a>(bc: &BC<'a>, mut e: Element<'a>) -> BaseIR<'a> {
    debug_assert!(matches!(
        e.tag_type,
        ElementType::Plain | ElementType::Component
    ));
    let tag = resolve_element_tag(bc, &e);
    let is_block = should_use_block(&e, &tag);
    // curiously, we should first build children instead of props
    // since we will pre-convert and consume v-slot here.
    let (children, more_flags) = build_children(bc, &mut e, &tag);
    let properties = mem::take(&mut e.properties);
    let BuildProps {
        props,
        directives,
        mut patch_flag,
        dynamic_props,
    } = build_props(bc, &mut e, properties);
    let directives = build_directive_args(bc, directives);
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
        is_component: e.is_component(),
    };
    IRNode::VNodeCall(vnode)
}

// is_slot indicates if the template should be compiled to dynamic slot expr
pub fn convert_template<'a>(bc: &BC<'a>, mut e: Element<'a>, is_slot: bool) -> BaseIR<'a> {
    debug_assert!(e.tag_type == ElementType::Template);
    check_wrong_slot(bc, &e, ErrorKind::VSlotTemplateMisplaced);
    // TODO: optimize away template if it has one stable element child
    // TODO: pass key property to the direct element child
    // template here is purely a fragment that groups element.
    let mut patch_flag = PatchFlag::STABLE_FRAGMENT;
    let child_count = e
        .children
        .iter()
        .filter(|c| !matches!(c, AstNode::Comment(_)))
        .count();
    // only build key for props
    let props = |e: &mut Element<'a>| {
        let p = prop_finder(&mut *e, "key").find()?;
        let key_prop_iter = iter::once(p.take());
        build_props(bc, &mut *e, key_prop_iter).props
    };
    let props = props(&mut e);
    if child_count == 1 && bc.option.is_dev {
        patch_flag |= PatchFlag::DEV_ROOT_FRAGMENT;
    }
    IRNode::VNodeCall(VNodeIR {
        tag: Js::Symbol(RuntimeHelper::Fragment),
        children: bc.convert_children(e.children),
        patch_flag,
        props,
        is_block: true, // only v-if/v-for(always block) or v-slot(as wrapper)
        ..VNodeIR::default()
    })
}

/// Returns a expression for createVnode's first argument. It can be
/// 1. Js::Call for dynamic component or user component.
/// 2. Js::Symbol for builtin component
/// 3. Js::StrLit for plain element or component
pub fn resolve_element_tag<'a>(bc: &BC<'a>, e: &Element<'a>) -> Js<'a> {
    if e.tag_type == ElementType::Plain {
        return Js::str_lit(e.tag_name);
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
    if !bc.sfc_info.self_name.is_empty()
        && VStr::raw(tag).pascalize().into_string() == bc.sfc_info.self_name
    {
        // codegen special checks for __self postfix when generating component imports,
        // which will pass additional `maybeSelfReference` flag to `resolveComponent`.
        comp.suffix_self();
    }
    // 5. user component (resolve)
    let comp_name = *comp.clone().be_component(); // use clone to avoid mutating
                                                  // comp will be collected by collect_entities transform pass
    Js::Simple(comp_name, StaticLevel::CanHoist)
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
            }) => Js::StrLit(val.content), // TODO: return Err(val.content) ?
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

// Builtin component is compiled with raw children instead of slot functions
// so that it can be used inside Transition or other Transition-wrapping HOCs.
// To ensure correct updates with block optimizations, we need to handle Builtin Block
fn should_use_block<'a>(e: &Element<'a>, tag: &Js<'a>) -> bool {
    use RuntimeHelper as H;
    match tag {
        // dynamic component may resolve to plain element
        Js::Call(H::ResolveDynamicComponent, _) => return true,
        // Builtin Block 1. Force keep-alive/teleport into a block.
        // This avoids its children being collected by a parent block.
        Js::Symbol(H::Teleport) | Js::Symbol(H::Suspense) => return true,
        Js::Symbol(H::KeepAlive) => return !e.children.is_empty(),
        _ => {
            if e.is_component() {
                return false;
            }
        }
    }
    // <svg> and <foreignObject> must be forced into blocks so that block
    // updates inside get proper isSVG flag at runtime. (vue-next/#639, #643)
    // Technically web-specific, but splitting out of core is too complex
    e.tag_name == "svg" || e.tag_name == "foreignObject" ||
    // vue-next/#938: elements with dynamic keys should be forced into blocks
    prop_finder(e, "key").dynamic_only().find().is_some()
}

type BaseDir<'a> = RuntimeDir<BaseConvertInfo<'a>>;
fn build_directive_args<'a>(
    bc: &BC<'a>,
    dirs: Vec<(Directive<'a>, Option<RuntimeHelper>)>,
) -> Vec<BaseDir<'a>> {
    dirs.into_iter()
        .map(|(dir, rh)| build_directive_arg(bc, dir, rh))
        .collect()
}

fn build_directive_arg<'a>(
    bc: &BC<'a>,
    dir: Directive<'a>,
    helper: Option<RuntimeHelper>,
) -> BaseDir<'a> {
    let resolve_setup_dir = || {
        // TODO: should resolve "v-" + dir.name here
        resolve_setup_reference(bc, dir.name)
    };
    let name = if let Some(rh) = helper {
        Js::Symbol(rh)
    } else if let Some(from_setup) = resolve_setup_dir() {
        from_setup
    } else {
        // collected in collect_entities transform pass
        let dir_name = *VStr::raw(dir.name).be_directive();
        Js::Simple(dir_name, StaticLevel::CanHoist)
    };
    use crate::parser::DirectiveArg::{Dynamic, Static};
    let expr = dir.expression.map(|v| Js::simple(v.content));
    let arg = dir.argument.map(|a| match a {
        Static(v) => Js::str_lit(v),
        Dynamic(v) => Js::simple(v),
    });
    let mods = if dir.modifiers.is_empty() {
        None
    } else {
        let mapper = |v| (Js::simple(v), Js::Src("true"));
        let props = dir.modifiers.into_iter().map(mapper);
        Some(Js::Props(props.collect()))
    };
    RuntimeDir {
        name,
        expr,
        arg,
        mods,
    }
}

fn build_children<'a>(
    bc: &BC<'a>,
    e: &mut Element<'a>,
    tag: &Js<'a>,
) -> (Vec<BaseIR<'a>>, PatchFlag) {
    // check slot should precede return
    if !e.is_component() {
        v_slot::check_wrong_slot(bc, e, ErrorKind::VSlotMisplaced);
    }
    let mut more_flag = PatchFlag::empty();
    if e.children.is_empty() {
        return (vec![], more_flag);
    }
    let should_build_as_slot = v_slot::check_build_as_slot(bc, e, tag);
    if is_builtin_symbol(tag, RuntimeHelper::KeepAlive) {
        let children = &e.children;
        // Builtin Component: 2. Force keep-alive always be updated.
        more_flag |= PatchFlag::DYNAMIC_SLOTS;
        if children.len() > 1 {
            let start = children[0].get_location().start.clone();
            let end = children.last().unwrap().get_location().end.clone();
            let error = CompilationError::new(ErrorKind::KeepAliveInvalidChildren)
                .with_location(SourceLocation { start, end });
            bc.emit_error(error);
        }
    }
    // if is keep alive
    if should_build_as_slot {
        let slots = v_slot::convert_v_slot(bc, e);
        return (vec![slots], more_flag);
    }
    let children = std::mem::take(&mut e.children);
    let children = bc.convert_children(children);
    (children, more_flag)
}

fn resolve_setup_component<'a>(bc: &BC<'a>, tag: &'a str) -> Option<Js<'a>> {
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
fn resolve_setup_reference<'a>(bc: &BC<'a>, name: &'a str) -> Option<Js<'a>> {
    let bindings = &bc.sfc_info.binding_metadata;
    if bindings.is_empty() || !bindings.is_setup() {
        return None;
    }
    let is_inline = bc.sfc_info.inline;
    // the returned closure will find the name modulo casing
    let variety_by_type = get_variety_from_binding(name, bindings);
    if let Some(from_const) = variety_by_type(BindingTypes::SetupConst) {
        return Some(if is_inline {
            Js::simple(from_const)
        } else {
            Js::Compound(vec![
                Js::Src("$setup["),
                Js::StrLit(from_const),
                Js::Src("]"),
            ])
        });
    }
    let from_maybe_ref = variety_by_type(BindingTypes::SetupLet)
        .or_else(|| variety_by_type(BindingTypes::SetupRef))
        .or_else(|| variety_by_type(BindingTypes::SetupMaybeRef));
    if let Some(maybe_ref) = from_maybe_ref {
        return Some(if is_inline {
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
    use crate::util::Lazy;
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

#[cfg(test)]
mod test {
    use super::super::test::base_convert;
    use super::*;
    use crate::cast;
    #[test]
    fn test_component_basic() {
        let mut body = base_convert("<comp/>").body;
        assert_eq!(body.len(), 1);
        let vn = cast!(body.remove(0), IRNode::VNodeCall);
        let tag = cast!(vn.tag, Js::Simple);
        assert_eq!(tag.into_string(), "_component_comp");
        assert!(vn.is_component);
    }
}
