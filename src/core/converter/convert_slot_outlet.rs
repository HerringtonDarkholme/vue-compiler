use super::{BaseConverter, BaseIR, Directive, Element, IRNode, JsExpr as Js, RenderSlotIR, VStr};
use crate::core::{
    parser::{DirectiveArg, ElemProp},
    tokenizer::Attribute,
    util::is_bind_key,
};

pub fn convert_slot_outlet<'a>(bc: &BaseConverter, mut e: Element<'a>) -> BaseIR<'a> {
    let info = process_slot_outlet(e);
    IRNode::RenderSlotCall(RenderSlotIR { slot_args: vec![] })
}

struct SlotOutletInfo<'a> {
    slot_name: Js<'a>,
    slot_props: Option<Js<'a>>,
}

fn process_slot_outlet<'a>(mut e: Element<'a>) -> SlotOutletInfo<'a> {
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
    let mut non_name_props = e.properties.into_iter().filter_map(mapper).peekable();
    if non_name_props.peek().is_some() {}
    SlotOutletInfo {
        slot_name,
        slot_props,
    }
}
