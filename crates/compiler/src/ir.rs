use crate::{
    flags::{PatchFlag, RuntimeHelper, SlotFlag, StaticLevel},
    util::VStr,
    Name,
};
use rustc_hash::FxHashSet;
use std::hash::Hash;

#[cfg(feature = "serde")]
use serde::Serialize;

#[cfg(feature = "serde")]
pub trait ConvertInfo {
    type TopType: Default + Serialize;
    // TextType should be a slice of JsExpressions
    type TextType: AsMut<[Self::JsExpression]> + Serialize;
    type IfBranchType: Serialize;
    type CommentType: Serialize;
    type JsExpression: Default + Serialize;
    type StrType: Serialize + Eq + Hash;
}
#[cfg(not(feature = "serde"))]
pub trait ConvertInfo {
    type TopType: Default;
    // TextType should be a slice of JsExpressions
    type TextType: AsMut<[Self::JsExpression]>;
    type IfBranchType;
    type CommentType;
    type JsExpression: Default;
    type StrType: Eq + Hash;
}

#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum IRNode<T: ConvertInfo> {
    /// interpolation or text node
    TextCall(TextIR<T>),
    /// v-if, else-if, else
    If(IfNodeIR<T>),
    /// v-for
    For(ForNodeIR<T>),
    /// component/template/plain element
    VNodeCall(VNodeIR<T>),
    /// <slot> slot outlet
    RenderSlotCall(RenderSlotIR<T>),
    /// v-slot used on component or template
    VSlotUse(VSlotIR<T>),
    // internal type for v-slot to reuse v-if/for
    AlterableSlot(Slot<T>),
    /// comment
    CommentCall(T::CommentType),
}

#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct TextIR<T: ConvertInfo> {
    pub fast_path: bool,  // without createTextCall
    pub need_patch: bool, // PatchFlag::TEXT
    pub texts: T::TextType,
}
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct IfNodeIR<T: ConvertInfo> {
    pub branches: Vec<IfBranch<T>>,
}
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct IfBranch<T: ConvertInfo> {
    pub condition: Option<T::JsExpression>,
    pub child: Box<IRNode<T>>,
    pub info: T::IfBranchType,
}
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct ForNodeIR<T: ConvertInfo> {
    pub source: T::JsExpression,
    pub parse_result: ForParseResult<T>,
    pub child: Box<IRNode<T>>,
    pub is_stable: bool,
    pub fragment_flag: PatchFlag,
    pub key: Option<T::JsExpression>,
}
// TODO: optimize as vec to save memory
// (value, key, index) in source
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct ForParseResult<T: ConvertInfo> {
    pub value: T::JsExpression,
    pub key: Option<T::JsExpression>,
    pub index: Option<T::JsExpression>,
}
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct RenderSlotIR<T: ConvertInfo> {
    pub slot_obj: T::JsExpression,
    pub slot_name: T::JsExpression,
    pub slot_props: Option<T::JsExpression>,
    pub fallbacks: Vec<IRNode<T>>,
    pub no_slotted: bool,
}
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct RuntimeDir<T: ConvertInfo> {
    pub name: T::JsExpression,
    pub expr: Option<T::JsExpression>,
    pub arg: Option<T::JsExpression>,
    pub mods: Option<T::JsExpression>,
}
#[derive(Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct VNodeIR<T: ConvertInfo> {
    pub tag: T::JsExpression,
    pub props: Option<T::JsExpression>,
    pub children: Vec<IRNode<T>>,
    pub patch_flag: PatchFlag,
    pub dynamic_props: FxHashSet<T::StrType>,
    pub directives: Vec<RuntimeDir<T>>,
    pub is_block: bool,
    pub disable_tracking: bool,
    pub is_component: bool,
}
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Slot<T: ConvertInfo> {
    pub name: T::JsExpression,
    pub param: Option<T::JsExpression>,
    pub body: Vec<IRNode<T>>,
}
// note the diffrence between stable and static, dynamic and alterable.
// static = static template name, capturing no identifier
// stable = no if nor for
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct VSlotIR<T: ConvertInfo> {
    /// stable v-slots declared statically in the template
    pub stable_slots: Vec<Slot<T>>,
    /// v-slots templates dynamically declared with v-if/v-for
    pub alterable_slots: Vec<IRNode<T>>,
    pub slot_flag: SlotFlag,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct IRRoot<T: ConvertInfo> {
    pub body: Vec<IRNode<T>>,
    /// entities to define/import in top level scope
    pub top_scope: T::TopType,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Clone)]
/// Records how v-on handler is written in the template.
/// Variants will be compiled differently (also depends on `cache_handlers`).
pub enum HandlerType {
    /// e.g. @click="c++"
    InlineStmt,
    /// e.g. @click="obj.method"
    MemberExpr,
    /// e.g. @click="() => func()"
    FuncExpr,
}

pub type Prop<'a> = (JsExpr<'a>, JsExpr<'a>);
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum JsExpr<'a> {
    /// Source. output to generated code as is.
    Src(&'a str),
    /// representing a number, either id or key
    Num(usize),
    /// String Literal. output after quoted, used by attr/static arg.
    // TODO: StaticLevel + Simple can mock StrLit?
    StrLit(VStr<'a>),
    /// non-string js expression, will be processed like prefixing
    Simple(VStr<'a>, StaticLevel),
    /// variable in parameter
    Param(Name<'a>),
    /// event handler function
    Func(VStr<'a>, HandlerType, StaticLevel),
    /// alternative to join string as JsExpr
    Compound(Vec<JsExpr<'a>>),
    Props(Vec<Prop<'a>>),
    /// for calling runtime helper, e.g. resolveComponent()
    Call(RuntimeHelper, Vec<JsExpr<'a>>),
    /// for builtin component called as symbol
    Symbol(RuntimeHelper),
    /// array of JsExpr
    Array(Vec<JsExpr<'a>>),
}

impl<'a> Default for JsExpr<'a> {
    fn default() -> Self {
        Self::Src("")
    }
}

impl<'a> JsExpr<'a> {
    /// a convenient util for creating JsExpr::Simple
    pub fn simple<V: Into<VStr<'a>>>(v: V) -> Self {
        JsExpr::Simple(v.into(), StaticLevel::NotStatic)
    }
    pub fn str_lit<V: Into<VStr<'a>>>(v: V) -> Self {
        JsExpr::StrLit(v.into())
    }
    pub fn static_level(&self) -> StaticLevel {
        use JsExpr::*;
        use StaticLevel as S;
        match self {
            Num(_) | StrLit(_) => S::CanStringify,
            Simple(_, level) => *level,
            Symbol(_) | Src(_) | Param(_) => S::CanHoist,
            Compound(v) | Array(v) | Call(_, v) => v
                .iter()
                .map(Self::static_level)
                .min()
                .unwrap_or(S::CanStringify),
            Props(ps) => {
                let prop_level = |prop: &Prop<'a>| {
                    let key_level = Self::static_level(&prop.0);
                    let val_level = Self::static_level(&prop.1);
                    key_level.min(val_level)
                };
                ps.iter().map(prop_level).min().unwrap_or(S::CanStringify)
            }
            Func(..) => {
                todo!()
            }
        }
    }
}
