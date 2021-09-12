use super::{Element, JsExpr as Js};
use crate::core::{
    flags::PatchFlag,
    parser::{Directive, ElemProp},
};
use std::iter::IntoIterator;

pub struct BuildProps<'a> {
    pub props: Option<Js<'a>>,
    pub directives: Vec<Directive<'a>>,
    pub patch_flag: PatchFlag,
    pub dynamic_props: Option<Js<'a>>,
}

pub fn build_props<'a, T>(e: &Element<'a>, props: T) -> BuildProps<'a>
where
    T: IntoIterator<Item = ElemProp<'a>>,
{
    todo!()
}
