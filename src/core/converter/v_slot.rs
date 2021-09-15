use super::{
    super::error::{CompilationError, CompilationErrorKind as ErrorKind},
    AstNode, BaseConverter as BC, BaseIR, CoreConverter, Directive, Element, IRNode, JsExpr as Js,
};
use crate::core::{
    flags::RuntimeHelper,
    parser::{DirectiveArg, ElementType},
    util::{dir_finder, VStr},
};
use std::mem;

// TODO: add has_dynamic_slot
// we have three forms of slot:
// 1. On component slot: <comp v-slot="">
// 2. Full template slot: <template v-slot>
// 3. implicit default with named: hybrid of 1 and 2
pub fn convert_v_slot<'a>(bc: &BC, e: &mut Element<'a>) -> BaseIR<'a> {
    // 0. Check dynamic identifier usage
    // 1. Check for slot with slotProps on component itself. <Comp v-slot="{ prop }"/>
    if let Some(ret) = convert_on_component_slot(bc, &mut *e) {
        return ret;
    }
    let (implicit_default, explicit_slots) = split_implicit_and_explit(&mut *e);
    // 2. traverse children and check template slots (v-if/v-for)
    //   2.a. v-if
    //   2.b. v-for (need dup name check)
    //   2.c. check dup static name
    //   2.d. check nested v-slot
    //   output static slots and dynamic ones

    // 3. create slot properties with default slot props
    // 4. merge static slot and dynamic ones if available
    todo!()
}

fn get_slot_name<'a>(arg: &Option<DirectiveArg<'a>>) -> Js<'a> {
    match arg {
        None => Js::StrLit(VStr::raw("default")),
        Some(DirectiveArg::Static(s)) => Js::StrLit(VStr::raw(s)),
        Some(DirectiveArg::Dynamic(s)) => Js::simple(*s),
    }
}

fn convert_on_component_slot<'a>(bc: &BC, e: &mut Element<'a>) -> Option<BaseIR<'a>> {
    let dir = dir_finder(&mut *e, "slot").allow_empty().find()?.take();
    let Directive {
        argument,
        expression,
        ..
    } = dir;
    let slot_name = get_slot_name(&argument);
    let expr = expression.map(|v| Js::simple(v.content));
    let children = mem::take(&mut e.children);
    //  check nested <template v-slot/>
    let children = children.into_iter().filter(|n| {
        if let AstNode::Element(e) = n {
            e.tag_type != ElementType::Template
                || !check_wrong_slot(bc, e, ErrorKind::VSlotMixedSlotUsage)
        } else {
            true
        }
    });
    Some(IRNode::VSlotExpression(Js::Props(vec![(
        slot_name,
        build_slot_fn(expr, children),
    )])))
}

fn is_template_slot<'a>(e: &Element<'a>) -> bool {
    if e.tag_type != ElementType::Template {
        return false;
    }
    dir_finder(e, "slot").allow_empty().find().is_some()
}

fn split_implicit_and_explit<'a>(e: &mut Element<'a>) -> (Vec<AstNode<'a>>, Vec<Element<'a>>) {
    let children = mem::take(&mut e.children);
    let mut implicit_default = vec![];
    let explicit_slots = children
        .into_iter()
        .filter_map(|n| match n {
            AstNode::Element(e) if is_template_slot(&e) => return Some(e),
            _ => {
                implicit_default.push(n);
                None
            }
        })
        .collect();
    (implicit_default, explicit_slots)
}

fn build_slot_fn<'a, C>(exp: Option<Js<'a>>, children: C) -> Js<'a>
where
    C: IntoIterator<Item = AstNode<'a>>,
{
    todo!()
}

pub fn check_wrong_slot(bc: &BC, e: &Element, kind: ErrorKind) -> bool {
    if let Some(found) = dir_finder(e, "slot").allow_empty().find() {
        let dir = found.get_ref();
        let error = CompilationError::new(kind).with_location(dir.location.clone());
        bc.emit_error(error);
        true
    } else {
        false
    }
}

pub fn check_build_as_slot(bc: &BC, e: &Element, tag: &Js) -> bool {
    debug_assert!(e.tag_type != ElementType::Template);
    use RuntimeHelper::{KeepAlive, Teleport};
    match tag {
        Js::Symbol(KeepAlive) => true,
        Js::Symbol(Teleport) => true,
        _ => e.is_component(),
    }
}
