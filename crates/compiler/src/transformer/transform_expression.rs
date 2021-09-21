// 1. track variables introduced in template
// currently only v-for and v-slot
// 2. prefix expression
use super::{BaseInfo, CorePassExt};
use crate::converter::{BaseRoot, JsExpr as Js};
use crate::util::VStr;
use rustc_hash::FxHashMap;

pub struct Scope<'a> {
    identifiers: FxHashMap<VStr<'a>, usize>,
}

pub struct ExpressionProcessor;

impl<'a> CorePassExt<BaseInfo<'a>, Scope<'a>> for ExpressionProcessor {
    fn enter_root(&mut self, r: &mut BaseRoot<'a>, shared: &mut Scope<'a>) {}
    fn exit_root(&mut self, r: &mut BaseRoot<'a>, shared: &mut Scope<'a>) {}
    fn enter_fn_param(&mut self, p: &mut Js<'a>, shared: &mut Scope<'a>) {}
    fn exit_fn_param(&mut self, p: &mut Js<'a>, shared: &mut Scope<'a>) {}
}
