use super::{BaseConverter, BaseIR, Element, IRNode, JsExpr as Js, RenderSlotIR, VStr};

pub fn convert_slot_outlet<'a>(bc: &BaseConverter, mut e: Element<'a>) -> BaseIR<'a> {
    let info = process_slot_outlet(&mut e);
    IRNode::RenderSlotCall(RenderSlotIR { slot_args: vec![] })
}

struct SlotOutletInfo<'a> {
    slot_name: Js<'a>,
    slot_props: Option<Js<'a>>,
}

fn process_slot_outlet<'a>(e: &mut Element<'a>) -> SlotOutletInfo<'a> {
    let mut slot_name = Js::StrLit(VStr::raw("default"));
    let mut slot_props = None;
    // TODO: I am not sure if I can take the name prop
    SlotOutletInfo {
        slot_name,
        slot_props,
    }
}
