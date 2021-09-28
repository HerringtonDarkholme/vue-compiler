use compiler::error::ErrorHandler;
use compiler::Position;
use serde::Serialize;

#[derive(Clone)]
pub struct TestErrorHandler;
impl ErrorHandler for TestErrorHandler {}

#[derive(Serialize)]
pub struct SourceLocation {
    pub start: Position,
    pub end: Position,
}

impl From<compiler::SourceLocation> for SourceLocation {
    fn from(loc: compiler::SourceLocation) -> Self {
        let s = loc.start;
        let e = loc.end;
        Self {
            start: Position {
                offset: s.offset,
                line: s.line,
                column: s.column,
            },
            end: Position {
                offset: e.offset,
                line: e.line,
                column: e.column,
            },
        }
    }
}

// insta does not support yaml with customized expr :(
// https://github.com/mitsuhiko/insta/issues/177
// WARNING: insta private API usage.
/// serialize object to yaml string
pub fn serialize_yaml<T: Serialize>(t: T) -> String {
    use insta::_macro_support::{serialize_value, SerializationFormat, SnapshotLocation};
    serialize_value(&t, SerializationFormat::Yaml, SnapshotLocation::File)
}
