mod template_element;
mod js_expression;

pub struct Position {
    offset: usize,
    line: usize,
    column: usize,
}

pub struct SourceLocation {
    start: Position,
    end: Position,
}

pub enum CodegenNode<'a> {
    TemplateChild(template_element::TemplateChildNode<'a>),
    JsChild(js_expression::JsChildNode),
    JsBlock(js_expression::BlockStatement),
}

pub struct ForParseResult {
}
