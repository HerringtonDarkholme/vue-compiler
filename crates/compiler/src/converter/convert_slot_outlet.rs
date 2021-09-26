use super::{
    build_props::{build_props, BuildProps},
    BaseConverter as BC, BaseIR, CoreConverter, Directive, Element, IRNode, JsExpr as Js,
    RenderSlotIR,
};
use crate::{
    error::{CompilationError, CompilationErrorKind::VSlotUnexpectedDirectiveOnSlotOutlet},
    parser::{DirectiveArg, ElemProp},
    tokenizer::Attribute,
    util::is_bind_key,
};
use std::mem;

pub fn convert_slot_outlet<'a>(bc: &BC<'a>, mut e: Element<'a>) -> BaseIR<'a> {
    let (slot_name, slot_props) = process_slot_outlet(bc, &mut e);
    let fallbacks = bc.convert_children(e.children);
    let no_slotted = bc.no_slotted();
    let slot_props = slot_props.or({
        if !fallbacks.is_empty() || no_slotted {
            Some(Js::Src("{}"))
        } else {
            None
        }
    });
    IRNode::RenderSlotCall(RenderSlotIR {
        slot_obj: Js::simple("$slots"),
        slot_name,
        slot_props,
        fallbacks,
        no_slotted,
    })
}

type NameAndProps<'a> = (Js<'a>, Option<Js<'a>>);

fn process_slot_outlet<'a>(bc: &BC<'a>, e: &mut Element<'a>) -> NameAndProps<'a> {
    let mut slot_name = Js::str_lit("default");
    let mapper = |mut prop| {
        match &mut prop {
            ElemProp::Dir(dir @ Directive { name: "bind", .. })
                if is_bind_key(&dir.argument, "name") =>
            {
                if !dir.has_empty_expr() {
                    let content = dir.expression.as_ref().unwrap().content;
                    slot_name = Js::simple(content);
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

    let props = mem::take(&mut e.properties);
    let mut non_name_props = props.into_iter().filter_map(mapper).peekable();
    if non_name_props.peek().is_none() {
        return (slot_name, None);
    }
    let BuildProps {
        props, directives, ..
    } = build_props(bc, e, non_name_props);
    if !directives.is_empty() {
        let error = CompilationError::new(VSlotUnexpectedDirectiveOnSlotOutlet)
            .with_location(directives[0].0.location.clone());
        bc.emit_error(error)
    }
    (slot_name, props)
}
