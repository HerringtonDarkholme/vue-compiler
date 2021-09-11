use super::{BaseConvertInfo, DirectiveArgument, Element, JsExpr as Js};
use crate::core::flags::PatchFlag;

pub struct BuildProps<'a> {
    pub props: Option<Js<'a>>,
    pub directives: Vec<DirectiveArgument<BaseConvertInfo<'a>>>,
    pub patch_flag: PatchFlag,
    pub dynamic_props: Option<Js<'a>>,
}

pub fn build_props<'a, T>(e: &Element<'a>, props: T) -> BuildProps<'a> {
    todo!()
}
