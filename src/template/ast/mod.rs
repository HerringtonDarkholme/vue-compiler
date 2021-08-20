mod element_node;
mod js_expression;
mod template_codegen;

pub struct Position {
    offset: usize,
    line: usize,
    column: usize,
}

pub struct SourceLocation {
    start: Position,
    end: Position,
}

pub struct ForParseResult {
}

pub enum PatchFlag {
}
