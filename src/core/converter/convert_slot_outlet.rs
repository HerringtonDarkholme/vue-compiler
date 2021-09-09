use super::{BaseConverter, BaseIR, Element, IRNode, JsExpr as Js, RenderSlotIR};

pub fn convert_slot_outlet<'a>(bc: &BaseConverter, e: Element<'a>) -> BaseIR<'a> {
    let info = process_slot_outlet(&e);
    IRNode::RenderSlotCall(RenderSlotIR { slot_args: vec![] })
}

struct SlotOutletInfo<'a> {
    slot_name: Js<'a>,
    slot_props: Js<'a>,
}

fn process_slot_outlet<'a>(e: &Element<'a>) -> SlotOutletInfo<'a> {
    todo!()
}
