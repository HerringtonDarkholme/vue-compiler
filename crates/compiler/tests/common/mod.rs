use compiler::error::ErrorHandler;
pub use compiler::{Position, SourceLocation};
use serde::Serialize;
use vue_compiler_core as compiler;

#[derive(Clone)]
pub struct TestErrorHandler;
impl ErrorHandler for TestErrorHandler {}

// insta does not support yaml with customized expr :(
// https://github.com/mitsuhiko/insta/issues/177
// WARNING: insta private API usage.
/// serialize object to yaml string
pub fn serialize_yaml<T: Serialize>(t: T) -> String {
    use insta::_macro_support::{serialize_value, SerializationFormat, SnapshotLocation};
    serialize_value(&t, SerializationFormat::Yaml, SnapshotLocation::File)
}
