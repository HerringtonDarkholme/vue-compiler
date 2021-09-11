//! This module defines a collection of flags used for Vue's runtime.
//! Currently it includes preamble helper and vnode patch flags.
//! Ideally we can make flags extensible by extracting them to trait.
//! But currently it works well enough and adding traits makes compiler
//! bloated with too many generic parameters.

use bitflags::bitflags;

bitflags! {
    #[derive(Default)]
    pub struct PatchFlag: u32 {
    }
}

pub enum RuntimeHelper {
    Fragment,
    Teleport,
    Suspense,
    KeepAlive,
    BaseTransition,
    OpenBlock,
    CreateBlock,
    CreateElementBlock,
    CreateVnode,
    CreateElementVnode,
    CreateComment,
    CreateText,
    CreateStatic,
    ResolveComponent,
    ResolveDynamicComponent,
    ResolveDirective,
    ResolveFilter,
    WithDirectives,
    RenderList,
    RenderSlot,
    CreateSlots,
    ToDisplayString,
    MergeProps,
    NormalizeClass,
    NormalizeStyle,
    NormalizeProps,
    GuardReactiveProps,
    ToHandlers,
    Camelize,
    Capitalize,
    ToHandlerKey,
    SetBlockTracking,
    PushScopeId,
    PopScopeId,
    WithScopeId,
    WithCtx,
    Unref,
    IsRef,
    WithMemo,
    IsMemoSame,
}

/// PreambleHelper is a collection of JavaScript imports at the head of output
/// e.g. v-for needs a list looping helper to make vdom
/// preamble helper needs collect helper when traversing template ast
/// and generates corresponding JavaScript imports in compilation output
impl RuntimeHelper {
    pub fn generate_imports(&self) -> String {
        todo!()
    }
    pub fn helper_str(&self) -> &'static str {
        use RuntimeHelper::*;
        match *self {
            Fragment => "Fragment",
            Teleport => "Teleport",
            Suspense => "Suspense",
            KeepAlive => "KeepAlive",
            BaseTransition => "BaseTransition",
            OpenBlock => "openBlock",
            CreateBlock => "createBlock",
            CreateElementBlock => "createElementBlock",
            CreateVnode => "createVnode",
            CreateElementVnode => "createElementVnode",
            CreateComment => "createComment",
            CreateText => "createText",
            CreateStatic => "createStatic",
            ResolveComponent => "resolveComponent",
            ResolveDynamicComponent => "resolveDynamicComponent",
            ResolveDirective => "resolveDirective",
            ResolveFilter => "resolveFilter",
            WithDirectives => "withDirectives",
            RenderList => "renderList",
            RenderSlot => "renderSlot",
            CreateSlots => "createSlots",
            ToDisplayString => "toDisplayString",
            MergeProps => "mergeProps",
            NormalizeClass => "normalizeClass",
            NormalizeStyle => "normalizeStyle",
            NormalizeProps => "normalizeProps",
            GuardReactiveProps => "guardReactiveProps",
            ToHandlers => "toHandlers",
            Camelize => "camelize",
            Capitalize => "capitalize",
            ToHandlerKey => "toHandlerKey",
            SetBlockTracking => "setBlockTracking",
            PushScopeId => "pushScopeId",
            PopScopeId => "popScopeId",
            WithScopeId => "withScopeId",
            WithCtx => "withCtx",
            Unref => "unref",
            IsRef => "isRef",
            WithMemo => "withMemo",
            IsMemoSame => "isMemoSame",
        }
    }
}

/*
// we can extend helper by extracting trait like below.
// but it does not pay off now.
pub trait PreambleHelper {
    fn generate_imports(&self) -> String;
    fn helper_str(&self) -> &'static str;
}
*/
