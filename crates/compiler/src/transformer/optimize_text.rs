use smallvec::SmallVec;

use super::{BaseConvertInfo, CoreTransformPass, IRNode};
use crate::converter::{BaseIR, JsExpr as Js};

pub struct OptimizeText;

impl<'a> CoreTransformPass<BaseConvertInfo<'a>> for OptimizeText {
    fn enter_children(&mut self, cs: &mut Vec<BaseIR<'a>>) {
        let mut i = 0;
        while i < cs.len() {
            if !matches!(&cs[i], IRNode::TextCall(_)) {
                i += 1;
                continue;
            }
            let (left, right) = cs.split_at_mut(i + 1);
            let dest = must_text(&mut left[i]);
            let mut j = 0;
            while j < right.len() {
                if !matches!(&right[j], IRNode::TextCall(_)) {
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
}

fn must_text<'a, 'b>(a: &'b mut BaseIR<'a>) -> &'b mut SmallVec<[Js<'a>; 1]> {
    if let IRNode::TextCall(t) = a {
        return t;
    }
    panic!("impossible")
}

#[cfg(test)]
mod test {
    use crate::Transformer;

    use super::super::test::{base_convert, get_transformer};
    use super::*;

    #[test]
    fn test_merge_text() {
        let mut transformer = get_transformer(OptimizeText);
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
        let mut transformer = get_transformer(OptimizeText);
        let mut ir = base_convert("hello <p/> {{world}}");
        assert_eq!(ir.body.len(), 4);
        transformer.transform(&mut ir);
        assert_eq!(ir.body.len(), 3);
        assert_eq!(must_text(&mut ir.body[2]).len(), 2);
    }
}
