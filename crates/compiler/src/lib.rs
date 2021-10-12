#![allow(dead_code)]
#![feature(generic_associated_types, once_cell)]
//! See README.md

// TODO: reorg pub
#[macro_use]
pub mod util;
pub mod codegen;
pub mod compiler;
pub mod converter;
pub mod error;
pub mod flags;
pub mod ir;
pub mod parser;
pub mod scanner;
pub mod transformer;

use flags::StaticLevel;
use ir::JsExpr as Js;
use rustc_hash::FxHashMap;
use std::ops::Deref;
use std::ops::Range;
pub use transformer::pass::Chain;
use util::VStr;

#[cfg(feature = "serde")]
use serde::Serialize;

// use plain &str here for now
// may change to tendril
pub type Name<'a> = &'a str;

#[derive(PartialEq, Eq, Clone)]
pub struct Position {
    /// the 0-indexed offset in the source str modulo newline
    pub offset: usize,
    /// the line number in the source code
    pub line: u32,
    /// the column number in the source code
    pub column: u32,
}

#[cfg(feature = "serde")]
impl Serialize for Position {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = format!(
            // Position, Line, Column
            "Pos: {}, Ln: {}, Col: {}",
            self.offset,
            self.line,
            self.column,
        );
        serializer.serialize_str(&s)
    }
}

impl Default for Position {
    fn default() -> Self {
        Self {
            offset: 0,
            line: 1,
            column: 1,
        }
    }
}

#[derive(Default, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct SourceLocation {
    pub start: Position,
    pub end: Position,
}

impl From<SourceLocation> for Range<usize> {
    fn from(location: SourceLocation) -> Self {
        location.start.offset..location.end.offset
    }
}

/// namespace for HTML/SVG/MathML tag
#[non_exhaustive]
#[derive(Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum Namespace {
    Html,
    Svg,
    MathMl,
    UserDefined(&'static str),
}

#[derive(PartialEq, Eq)]
pub enum BindingTypes {
    /// returned from data()
    Data,
    /// declared as a prop
    Props,
    /// a let binding (may or may not be a ref)
    SetupLet,
    ///a const binding that can never be a ref.
    ///these bindings don't need `unref()` calls when processed in inlined
    ///template expressions.
    SetupConst,
    /// a const binding that may be a ref.
    SetupMaybeRef,
    /// bindings that are guaranteed to be refs
    SetupRef,
    /// declared by other options, e.g. computed, inject
    Options,
}

impl BindingTypes {
    pub fn get_js_prop<'a>(&self, name: VStr<'a>, lvl: StaticLevel) -> Js<'a> {
        use BindingTypes::*;
        let obj_dot = Js::Src(match self {
            Data => "$data.",
            Props => "$props.",
            Options => "$options.",
            _ => "$setup.",
        });
        let prop = Js::Simple(name, lvl);
        Js::Compound(vec![obj_dot, prop])
    }
}

/// stores binding variables exposed by data/prop/setup script.
/// also stores if the binding is from setup script.
#[derive(Default)]
pub struct BindingMetadata<'a>(FxHashMap<&'a str, BindingTypes>, bool);
impl<'a> BindingMetadata<'a> {
    pub fn new(map: FxHashMap<&'a str, BindingTypes>, from_setup: bool) -> Self {
        Self(map, from_setup)
    }
    pub fn is_setup(&self) -> bool {
        self.1
    }
}
impl<'a> Deref for BindingMetadata<'a> {
    type Target = FxHashMap<&'a str, BindingTypes>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// SFC info of the current template
pub struct SFCInfo<'a> {
    /// Compile the function for inlining inside setup().
    /// This allows the function to directly access setup() local bindings.
    pub inline: bool,
    /// Indicates this SFC template has used :slotted in its styles
    /// Defaults to `true` for backwards compatibility - SFC tooling should set it
    /// to `false` if no `:slotted` usage is detected in `<style>`
    pub slotted: bool,
    /// SFC scoped styles ID
    pub scope_id: Option<String>,
    /// Optional binding metadata analyzed from script - used to optimize
    /// binding access when `prefixIdentifiers` is enabled.
    pub binding_metadata: BindingMetadata<'a>,
    /// Filename for source map generation.
    /// Also used for self-recursive reference in templates
    /// @default 'template.vue.html'
    pub self_name: String,
}

impl<'a> Default for SFCInfo<'a> {
    fn default() -> Self {
        Self {
            scope_id: None,
            inline: false,
            slotted: true,
            binding_metadata: BindingMetadata::default(),
            self_name: "".into(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_source_size() {
        assert_eq!(std::mem::size_of::<Position>(), 16);
    }
}
