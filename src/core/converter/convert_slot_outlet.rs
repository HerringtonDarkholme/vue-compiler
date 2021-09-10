use super::{
    BaseConverter, BaseIR, CoreConverter, Directive, Element, IRNode, JsExpr as Js, RenderSlotIR,
    VStr,
};
use crate::core::{
    parser::{DirectiveArg, ElemProp},
    tokenizer::Attribute,
    util::is_bind_key,
};
use std::mem::swap;

pub fn convert_slot_outlet<'a>(bc: &BaseConverter, mut e: Element<'a>) -> BaseIR<'a> {
    let info = process_slot_outlet(&mut e);
    let fallbacks = bc.convert_children(e.children);
    let no_slotted = bc.no_slotted();
    let slot_props = info.slot_props.or({
        if fallbacks.len() > 0 || no_slotted {
            Some(Js::Src("{}"))
        } else {
            None
        }
    });
    IRNode::RenderSlotCall(RenderSlotIR {
        slot_name: info.slot_name,
        slot_props,
        fallbacks,
        no_slotted,
    })
}

struct SlotOutletInfo<'a> {
    slot_name: Js<'a>,
    slot_props: Option<Js<'a>>,
}

fn process_slot_outlet<'a>(e: &mut Element<'a>) -> SlotOutletInfo<'a> {
    let mut slot_name = Js::StrLit(VStr::raw("default"));
    let mut slot_props = None;
    let mapper = |mut prop| {
        match &mut prop {
            ElemProp::Dir(dir @ Directive { name: "bind", .. })
                if is_bind_key(&dir.argument, "name") =>
            {
                if !dir.has_empty_expr() {
                    slot_name = Js::Simple(dir.expression.as_ref().unwrap().content);
                }
                None
            }
            ElemProp::Dir(Directive {
                name: "bind",
                argument: Some(arg),
                ..
            }) => {
                if let DirectiveArg::Static(_name) = arg {
                    // TODO: handle camelize
                    // name.camelize();
                }
                Some(prop)
            }
            ElemProp::Dir(_) => Some(prop),
            ElemProp::Attr(Attribute {
                name,
                value: Some(v),
                ..
            }) => {
                if v.content.is_empty() {
                    None
                } else if *name == "name" {
                    slot_name = Js::StrLit(v.content);
                    None
                } else {
                    // TODO: handle camelize
                    // name.camelize();
                    Some(prop)
                }
            }
            ElemProp::Attr(_) => None,
        }
    };

    let mut props = vec![];
    swap(&mut props, &mut e.properties);
    let mut non_name_props = props.into_iter().filter_map(mapper).peekable();
    if non_name_props.peek().is_some() {
        let (props, directives) = build_props(e, non_name_props);
        slot_props = Some(props);
    }
    SlotOutletInfo {
        slot_name,
        slot_props,
    }
}

fn build_props<'a, T>(e: &Element<'a>, props: T) -> (Js<'a>, Vec<Directive<'a>>) {
    todo!()
}
