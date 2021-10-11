/// extract class/style for faster runtime patching
use crate::ir::JsExpr as Js;
use crate::flags::RuntimeHelper as RH;
use super::{BaseInfo, BaseVNode, CorePass};

use std::mem;

pub struct NormalizeProp;

impl<'a> CorePass<BaseInfo<'a>> for NormalizeProp {
    fn enter_vnode(&mut self, v: &mut BaseVNode<'a>) {
        let props = match &mut v.props {
            Some(p) => p,
            None => return,
        };
        match props {
            Js::Call(..) => (), // nothing! MergeProps/toHandlers call
            Js::Props(ps) => {
                let ps = mem::take(ps);
                *props = pre_normalize_prop(ps);
            }
            e => {
                let single_v_bind = mem::take(e);
                *e = Js::Call(
                    RH::NORMALIZE_PROPS,
                    vec![Js::Call(RH::GUARD_REACTIVE_PROPS, vec![single_v_bind])],
                );
            }
        }
    }
}

fn is_handler_expr(j: &Js) -> bool {
    matches!(j, Js::FuncSimple { .. }) || matches!(j, Js::FuncCompound { .. })
}

fn pre_normalize_prop<'a>(mut props: Vec<(Js<'a>, Js<'a>)>) -> Js<'a> {
    let mut class_val = None;
    let mut style_val = None;
    let mut has_dynamic_key = false;
    for (key, val) in props.iter_mut() {
        if let Js::StrLit(k) = key {
            if k.raw == "class" {
                class_val = Some(val);
            } else if k.raw == "style" {
                style_val = Some(val);
            }
        } else if !is_handler_expr(val) {
            has_dynamic_key = true;
        }
    }
    if has_dynamic_key {
        return Js::Call(RH::NORMALIZE_PROPS, vec![Js::Props(props)]);
    }
    if let Some(cls) = class_val {
        if !matches!(cls, Js::StrLit(..)) {
            let val = mem::take(cls);
            *cls = Js::Call(RH::NORMALIZE_CLASS, vec![val]);
        }
    }
    if let Some(stl) = style_val {
        // Props is parsed from literal style string
        if !matches!(stl, Js::Props(..)) {
            let val = mem::take(stl);
            *stl = Js::Call(RH::NORMALIZE_STYLE, vec![val]);
        }
    }
    Js::Props(props)
}
