// track variables introduced in template
// currently only v-for and v-slot
use super::{
    BaseConvertInfo, BaseFor, BaseIf, BaseRenderSlot, BaseVNode, BaseVSlot, CoreTransformPass,
};
use crate::converter::{BaseRoot, JsExpr as Js};
use crate::util::VStr;
use rustc_hash::FxHashMap;

pub struct ScopeTracker<'a> {}
