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
pub mod scanner;
pub mod transformer;
#[macro_use]
pub mod util;

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
#[derive(Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
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
        if let $pat(a, ..) = $target {
            a
        } else {
            panic!("mismatch variant when cast to {}", stringify!($pat));
        }
    }};
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_source_size() {
        assert_eq!(std::mem::size_of::<Position>(), 16);
    }
}
