use super::runtime_helper::RuntimeHelper;

pub struct Position {
    offset: usize,
    line: usize,
    column: usize,
}

pub struct SourceLocation {
    start: Position,
    end: Position,
}

enum ParentNode<'a> {
    Root(RootNode<'a>),
    Element(ElementNode),
    IfBranch(IfBranchNode),
    For(ForNode),
}

enum ExpressionNode {
    Simple(SimpleExpressionNode),
    Compound(CompoundExpressionNode),
}

enum TemplateChildNode {
    Element(),
    Interpolation(),
    Expression(),
    Text(),
    Comment(),
    If(),
    IfBranch(),
    For(),
    TextCall(),
}

enum JsChildNode {
}

struct BlockStatement {
}

enum CodegenNode {
    TemplateChild(TemplateChildNode),
    JsChild(JsChildNode),
    JsBlock(BlockStatement),
}

pub struct RootNode<'a> {
  children: Vec<TemplateChildNode>,
  hoists: Vec<JsChildNode>,
  cached: i32,
  temps: i32,
  codegen_node: Option<CodegenNode>,
  preambles: Preambles<'a>,
}

struct ImportItem<'a> {
  exp: &'a str,
  path: &'a str,
}

pub struct Preambles<'a> {
  helpers: RuntimeHelper,
  components: Vec<&'a str>,
  directives: Vec<&'a str>,
  imports: Vec<ImportItem<'a>>,
  // ssrHelpers?: SSRHelper,
}
