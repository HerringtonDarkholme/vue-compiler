#![allow(dead_code, unused_variables)]
//! See README.md

use std::ops::Range;

// TODO: reorg pub
pub mod codegen;
pub mod compiler;
pub mod converter;
pub mod error;
pub mod flags;
pub mod parser;
pub mod tokenizer;
pub mod transformer;
pub mod util;

// use plain &str here for now
// may change to tendril
pub type Name<'a> = &'a str;

#[derive(PartialEq, Eq, Clone)]
pub struct Position {
    /// the 0-indexed offset in the source str modulo newline
    pub offset: usize,
    /// the line number in the source code
    pub line: usize,
    /// the column number in the source code
    pub column: usize,
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
#[derive(Eq, PartialEq)]
pub enum Namespace {
    Html,
    Svg,
    MathMl,
    UserDefined(&'static str),
}

#[cfg(test)]
#[macro_export]
macro_rules! cast {
    ($target: expr, $pat: path) => {{
        if let $pat(a) = $target {
            a
        } else {
            panic!("mismatch variant when cast to {}", stringify!($pat));
        }
    }};
}
