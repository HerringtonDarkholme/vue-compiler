use super::{SourceLocation};
pub enum JsChildNode {
}
pub struct ImportItem<'a> {
  exp: &'a str,
  path: &'a str,
}

pub struct BlockStatement {
}

pub enum ExpressionNode<'a> {
  Simple(SimpleExpression<'a>),
  Compound(CompoundExpression),
}
/// Static types have several levels.
/// Higher levels implies lower levels.
/// e.g. a node that can be stringified
/// can always be hoisted and skipped for patch.
#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub enum ConstantTypes {
  NotConstant,
  CanSkipPatch,
  CanHoist,
  CanStringify,
}

pub struct SimpleExpression<'a> {
  content: &'a str,
  is_static: bool,
  const_type: ConstantTypes,
  /// if this is an identifier for a hoist vnode call
  /// and points to the hoisted node.
  hoisted: Option<JsChildNode>,
  loc: SourceLocation
}
pub struct CompoundExpression {
  loc: SourceLocation
}

pub struct CallExpression {
  loc: SourceLocation
}
pub struct CacheExpression {
  loc: SourceLocation
}
pub struct MemoExpression {
  loc: SourceLocation
}
pub struct TemplateLiteral {
  loc: SourceLocation
}
