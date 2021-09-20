use smallvec::{smallvec, SmallVec};

use super::{
    BaseConvertInfo, BaseRenderSlot, BaseVNode, BaseVSlot, CoreTransformPass, IRNode as IR,
};
use crate::converter::{BaseIR, BaseRoot, JsExpr as Js};
use crate::flags::RuntimeHelper as RH;

pub struct TextOptimizer;

impl<'a> CoreTransformPass<BaseConvertInfo<'a>> for TextOptimizer {
    fn enter_root(&mut self, r: &mut BaseRoot<'a>) {
        merge_consecutive_calls(&mut r.body);
        if r.body.len() <= 1 {
            return;
        }
        add_create_text(&mut r.body);
    }
    fn enter_vnode(&mut self, v: &mut BaseVNode<'a>) {
        merge_consecutive_calls(&mut v.children);
        // #3756 custom directives can mutate DOM arbitrarily so set no textContent
        if v.is_component || v.children.len() > 1 || has_custom_dir(v) {
            add_create_text(&mut v.children);
        }
        // if this is a plain element with a single text child,
        // leave it as is since the runtime has dedicated fast path for this
        // by directly setting textContent of the element
    }
    fn enter_slot_outlet(&mut self, r: &mut BaseRenderSlot<'a>) {
        merge_consecutive_calls(&mut r.fallbacks);
        add_create_text(&mut r.fallbacks);
    }
    fn enter_v_slot(&mut self, s: &mut BaseVSlot<'a>) {
        for slot in s.stable_slots.iter_mut() {
            merge_consecutive_calls(&mut slot.body);
            add_create_text(&mut slot.body);
        }
    }
}

fn merge_consecutive_calls(cs: &mut Vec<BaseIR>) {
    let mut i = 0;
    while i < cs.len() {
        if !matches!(&cs[i], IR::TextCall(_)) {
            i += 1;
            continue;
        }
        let (left, right) = cs.split_at_mut(i + 1);
        let dest = must_text(&mut left[i]);
        let mut j = 0;
        while j < right.len() {
            if !matches!(&right[j], IR::TextCall(_)) {
                break;
            }
            let src = must_text(&mut right[j]);
            dest.extend(src.drain(..));
            j += 1;
        }
        drop(cs.drain(i + 1..i + 1 + j));
        i += 1;
    }
}

fn has_custom_dir(v: &BaseVNode) -> bool {
    !v.directives.is_empty()
}

pub fn concat_str<'a, I>(mut texts: I) -> Js<'a>
where
    I: ExactSizeIterator<Item = Js<'a>>,
{
    debug_assert!(texts.len() > 0);
    let mut v = Vec::with_capacity(2 * texts.len() - 1);
    v.push(texts.next().unwrap());
    for t in texts {
        v.push(Js::Src(" + "));
        v.push(t);
    }
    Js::Compound(v)
}

fn add_create_text(cs: &mut Vec<BaseIR>) {
    for child in cs.iter_mut() {
        if let IR::TextCall(t) = child {
            let texts = std::mem::take(t);
            // TODO: add patch flag
            let merged_args = vec![concat_str(texts.into_iter())];
            *t = smallvec![Js::Call(RH::CreateText, merged_args)];
        }
    }
}

fn must_text<'a, 'b>(a: &'b mut BaseIR<'a>) -> &'b mut SmallVec<[Js<'a>; 1]> {
    if let IR::TextCall(t) = a {
        return t;
    }
    panic!("impossible")
}

#[cfg(test)]
mod test {
    use super::super::test::{base_convert, get_transformer};
    use super::super::Transformer;
    use super::*;
    use crate::converter::RenderSlotIR;

    fn must_render_slot<'a, 'b>(
        a: &'b mut BaseIR<'a>,
    ) -> &'b mut RenderSlotIR<BaseConvertInfo<'a>> {
        if let IR::RenderSlotCall(t) = a {
            return t;
        }
        panic!("impossible")
    }

    #[test]
    fn test_merge_text() {
        let mut transformer = get_transformer(TextOptimizer);
        let mut ir = base_convert("hello {{world}}");
        assert_eq!(ir.body.len(), 2);
        assert_eq!(must_text(&mut ir.body[0]).len(), 1);
        assert_eq!(must_text(&mut ir.body[1]).len(), 1);
        transformer.transform(&mut ir);
        assert_eq!(ir.body.len(), 1);
        assert_eq!(must_text(&mut ir.body[0]).len(), 2);
    }

    #[test]
    fn test_merge_text_with_element() {
        let mut transformer = get_transformer(TextOptimizer);
        let mut ir = base_convert("hello <p/> {{world}}");
        assert_eq!(ir.body.len(), 4);
        transformer.transform(&mut ir);
        assert_eq!(ir.body.len(), 3);
        assert_eq!(must_text(&mut ir.body[2]).len(), 1);
        let mut ir = base_convert("a <p/> a {{f}} b<p/> e {{c}}<p/>");
        transformer.transform(&mut ir);
        assert_eq!(ir.body.len(), 6);
    }
    #[test]
    fn test_merge_text_with_slot() {
        let mut transformer = get_transformer(TextOptimizer);
        let mut ir = base_convert("<slot>hello {{world}}</slot>");
        transformer.transform(&mut ir);
        assert_eq!(ir.body.len(), 1);
        let slot = must_render_slot(&mut ir.body[0]);
        assert_eq!(slot.fallbacks.len(), 1);
        let text = must_text(&mut slot.fallbacks[0]);
        assert_eq!(text.len(), 1);
        assert!(matches!(text[0], Js::Call(..)));
    }
}
