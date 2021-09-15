use super::{
    AstNode, BaseConvertInfo, BaseConverter as BC, BaseIR, CoreConverter, Directive, Element,
    IRNode, JsExpr as Js, Slot, VSlotIR,
};
use crate::core::{
    error::{CompilationError, CompilationErrorKind as ErrorKind},
    flags::RuntimeHelper,
    parser::{DirectiveArg, ElementType},
    util::{dir_finder, VStr},
};
use std::mem;

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

type BaseVSlot<'a> = VSlotIR<BaseConvertInfo<'a>>;

// TODO: add has_alterable_slot
// we have three forms of slot:
// 1. On component slot: <comp v-slot="">
// 2. Full template slot: <template v-slot>
// 3. implicit default with named: hybrid of 1 and 2
pub fn convert_v_slot<'a>(bc: &BC, e: &mut Element<'a>) -> BaseIR<'a> {
    // TODO: Check dynamic identifier usage
    // 1. Check for slot with slotProps on component itself. <Comp v-slot="{ prop }"/>
    if let Some(ret) = convert_on_component_slot(bc, &mut *e) {
        return ret;
    }
    let (implicit_default, explicit_slots) = split_implicit_and_explicit(&mut *e);
    // 2. traverse children and check template slots
    let mut v_slot_ir = build_explicit_slots(bc, explicit_slots);
    // 3. merge stable slot and alterable ones if available
    if !implicit_default.is_empty() {
        if has_named_default(&v_slot_ir) {
            let first_child = &implicit_default[0];
            let error = CompilationError::new(ErrorKind::VSlotExtraneousDefaultSlotChildren)
                .with_location(first_child.get_location().clone());
            bc.emit_error(error);
        } else {
            let name = Js::StrLit(VStr::raw("default"));
            let body = build_slot_fn(implicit_default);
            let slot = Slot {
                name,
                body,
                param: None,
            };
            v_slot_ir.stable_slots.push(slot);
        }
    }
    IRNode::VSlotUse(v_slot_ir)
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
    let slot = Slot {
        name: slot_name,
        param: expr,
        body: build_slot_fn(children),
    };
    let v_slot_ir = VSlotIR {
        stable_slots: vec![slot],
        alterable_slots: vec![],
    };
    Some(IRNode::VSlotUse(v_slot_ir))
}

fn split_implicit_and_explicit<'a>(e: &mut Element<'a>) -> (Vec<AstNode<'a>>, Vec<Element<'a>>) {
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

const ALTERABLE_DIRS: [&str; 4] = ["if", "else-if", "else", "for"];
// TODO reduce AstNode rematching overhead
fn build_explicit_slots<'a>(bc: &BC, templates: Vec<Element<'a>>) -> BaseVSlot<'a> {
    // 1. check dup static name
    // 2. rebuild alterable slots
    // output stable slots and alterable ones
    let mut stable_slots = vec![];
    let mut alterable = vec![];
    for t in templates {
        if dir_finder(&t, ALTERABLE_DIRS)
            .allow_empty()
            .find()
            .is_some()
        {
            alterable.push(t);
            continue;
        }
        stable_slots.push(build_stable_slot(bc, t));
    }
    let alterable_slots = build_alterable_slots(bc, alterable);
    VSlotIR {
        stable_slots,
        alterable_slots,
    }
}

fn build_stable_slot<'a>(bc: &BC, mut t: Element<'a>) -> Slot<BaseConvertInfo<'a>> {
    let dir = dir_finder(&mut t, "slot").allow_empty().find().unwrap();
    let Directive {
        argument,
        expression,
        ..
    } = dir.take();
    let name = get_slot_name(&argument);
    let param = expression.map(|v| Js::simple(v.content));
    let body = bc.convert_children(t.children);
    Slot { name, param, body }
}
fn build_alterable_slots<'a>(bc: &BC, t: Vec<Element<'a>>) -> Vec<BaseIR<'a>> {
    todo!()
}

fn build_slot_fn<'a, C>(children: C) -> Vec<BaseIR<'a>>
where
    C: IntoIterator<Item = AstNode<'a>>,
{
    todo!()
}

fn get_slot_name<'a>(arg: &Option<DirectiveArg<'a>>) -> Js<'a> {
    match arg {
        None => Js::StrLit(VStr::raw("default")),
        Some(DirectiveArg::Static(s)) => Js::StrLit(VStr::raw(s)),
        Some(DirectiveArg::Dynamic(s)) => Js::simple(*s),
    }
}

fn is_template_slot(e: &Element) -> bool {
    if e.tag_type != ElementType::Template {
        return false;
    }
    dir_finder(e, "slot").allow_empty().find().is_some()
}

fn has_named_default(v_slot_ir: &BaseVSlot) -> bool {
    v_slot_ir.stable_slots.iter().any(|p| match p.name {
        Js::StrLit(s) => s.raw == "default",
        _ => false,
    })
}
