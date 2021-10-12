use compiler::flags::RuntimeHelper;
use compiler::transformer::{CorePass, BaseVNode};
use compiler::converter::{BaseConvertInfo as BaseInfo, BaseIR, RcErrHandle};
use compiler::error::CompilationError;
use crate::extension::{DomHelper, DomError};
use compiler::ir::{JsExpr as Js, IRNode};

struct UsageWarner(RcErrHandle);

impl<'a> CorePass<BaseInfo<'a>> for UsageWarner {
    fn enter_vnode(&mut self, vn: &mut BaseVNode<'a>) {
        match vn.tag {
            Js::Symbol(DomHelper::TRANSITION) => {
                if has_multiple_children(&vn.children) != Multiplicity::Multi {
                    return;
                }
            }
            _ => return,
        }
        let error = CompilationError::extended(DomError::TransitionInvalidChildren);
        self.0.on_error(error);
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum Multiplicity {
    Zero,
    One,
    Multi,
}

fn has_multiple_children(children: &[BaseIR]) -> Multiplicity {
    use Multiplicity::*;
    let mut multi = Zero;
    for child in children.iter() {
        match ir_multilicity(child) {
            Zero => (),
            One => {
                if multi == One {
                    return Multi;
                } else {
                    multi = One;
                }
            }
            Multi => {
                return Multi;
            }
        }
    }
    multi
}

fn ir_multilicity(ir: &BaseIR) -> Multiplicity {
    use Multiplicity::*;
    match ir {
        IRNode::VSlotUse(slots) => slots
            .stable_slots
            .iter()
            .find_map(|slot| match slot.name {
                Js::StrLit(s) if s.raw == "default" => Some(has_multiple_children(&slot.body)),
                _ => None,
            })
            .unwrap_or(Zero),
        IRNode::For(..) => Multi,
        IRNode::CommentCall(..) => Zero,
        IRNode::CacheNode(cn) => ir_multilicity(&*cn.child),
        IRNode::If(i) => i
            .branches
            .iter()
            .map(|b| ir_multilicity(&*b.child))
            .max()
            .unwrap_or(Zero),
        IRNode::TextCall(text) => {
            let empty = text.texts.iter().all(|t| match t {
                Js::StrLit(t) => t.trim().is_empty(),
                _ => false,
            });
            if empty {
                Zero
            } else {
                One
            }
        }
        IRNode::RenderSlotCall(..) => One,
        IRNode::VNodeCall(vn) => {
            if let Js::Symbol(RuntimeHelper::FRAGMENT) = vn.tag {
                has_multiple_children(&vn.children)
            } else {
                One
            }
        }
        IRNode::AlterableSlot(..) => panic!("impossible"),
    }
}
