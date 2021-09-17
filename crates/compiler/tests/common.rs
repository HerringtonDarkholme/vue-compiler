use compiler::error::ErrorHandler;
use serde::Serialize;

#[derive(Clone)]
pub struct TestErrorHandler;
impl ErrorHandler for TestErrorHandler {}

#[derive(Serialize)]
pub struct Position {
    pub offset: usize,
    pub line: usize,
    pub column: usize,
}

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
