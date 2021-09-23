use smallvec::SmallVec;

use super::{BaseInfo, BaseRenderSlot, BaseSlotFn, BaseVNode, CorePass, IRNode as IR};
use crate::converter::{BaseIR, BaseRoot, JsExpr as Js};

pub struct TextOptimizer;

impl<'a> CorePass<BaseInfo<'a>> for TextOptimizer {
    fn enter_root(&mut self, r: &mut BaseRoot<'a>) {
        merge_consecutive_calls(&mut r.body);
        optimize_away_call(&mut r.body);
    }
    fn enter_vnode(&mut self, v: &mut BaseVNode<'a>) {
        merge_consecutive_calls(&mut v.children);
        // #3756 custom directives can mutate DOM arbitrarily so set no textContent
        if v.is_component || has_custom_dir(v) {
            return;
        }
        // if this is a plain element with a single text child,
        // leave it as is since the runtime has dedicated fast path for this
        // by directly setting textContent of the element
        optimize_away_call(&mut v.children);
    }
    fn enter_slot_outlet(&mut self, r: &mut BaseRenderSlot<'a>) {
        merge_consecutive_calls(&mut r.fallbacks);
    }
    fn enter_slot_fn(&mut self, s: &mut BaseSlotFn<'a>) {
        merge_consecutive_calls(&mut s.body);
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

fn optimize_away_call(cs: &mut Vec<BaseIR>) {
    if cs.len() != 1 {
        return;
    }
    if let IR::TextCall(t) = &mut cs[0] {
        t.fast_path = true;
    }
}

fn must_text<'a, 'b>(a: &'b mut BaseIR<'a>) -> &'b mut SmallVec<[Js<'a>; 1]> {
    if let IR::TextCall(t) = a {
        return &mut t.texts;
    }
    panic!("impossible")
}

#[cfg(test)]
mod test {
    use super::super::test::{base_convert, get_transformer};
    use super::super::{BaseText, Transformer};
    use super::*;
    use crate::converter::RenderSlotIR;

    fn must_render_slot<'a, 'b>(a: &'b mut BaseIR<'a>) -> &'b mut RenderSlotIR<BaseInfo<'a>> {
        if let IR::RenderSlotCall(t) = a {
            return t;
        }
        panic!("impossible")
    }
    fn must_ir<'a, 'b>(a: &'b BaseIR<'a>) -> &'b BaseText<'a> {
        if let IR::TextCall(t) = a {
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
        let ir = must_ir(&mut ir.body[0]);
        assert!(ir.fast_path);
        assert!(!ir.need_patch);
    }

    #[test]
    fn test_merge_text_with_element() {
        let mut transformer = get_transformer(TextOptimizer);
        let mut ir = base_convert("hello <p/> {{world}}");
        assert_eq!(ir.body.len(), 4);
        transformer.transform(&mut ir);
        assert_eq!(ir.body.len(), 3);
        assert_eq!(must_text(&mut ir.body[2]).len(), 2);
        assert!(!must_ir(&mut ir.body[2]).fast_path);
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
        assert_eq!(text.len(), 2);
        let ir = must_ir(&mut slot.fallbacks[0]);
        assert!(!ir.fast_path);
    }
}
