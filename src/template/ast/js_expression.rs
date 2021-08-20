use super::{SourceLocation};
use super::element_node::{
    InterpolationNode, TextNode,
};
pub enum JsChildNode<'a> {
}
pub struct ImportItem<'a> {
    exp: &'a str,
    path: &'a str,
}


pub enum ExpressionNode<'a> {
    Simple(SimpleExpression<'a>),
    Compound(CompoundExpression<'a>),
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
    hoisted: Option<JsChildNode<'a>>,
    /// an expression parsed as the params of a function will track
    /// the identifiers declared inside the function body.
    identifiers: Vec<&'a str>,
    // is_handler_key: bool, // looks like not used
    loc: SourceLocation
}


pub enum CompoundChildren<'a> {
    Simple(SimpleExpression<'a>),
    // TODO: enable compound
    // Compound(CompoundChildren<'a>),
    Interpolation(InterpolationNode<'a>),
    Text(TextNode<'a>),
    Str(&'a str),
    // Symbol,
}

pub struct CompoundExpression<'a> {
    children: Vec<CompoundChildren<'a>>,
    /// an expression parsed as the params of a function will track
    /// the identifiers declared inside the function body.
    identifiers: Vec<&'a str>,
    is_handler_key: bool,
    loc: SourceLocation,
}

pub struct CallExpression {
    loc: SourceLocation,
}
pub struct CacheExpression {
    loc: SourceLocation,
}
pub struct MemoExpression {
    loc: SourceLocation,
}
pub struct TemplateLiteral {
    loc: SourceLocation,
}
pub struct IfConditionalExpression {
    loc: SourceLocation,
}

pub struct PropsExpression {
    loc: SourceLocation,
}
pub enum SlotsExpression {
}

pub struct Property<'a> {
    key: ExpressionNode<'a>,
    value: JsChildNode<'a>,
    loc: SourceLocation,
}
pub struct ObjectExpression<'a> {
    properties: Vec<Property<'a>>,
    loc: SourceLocation,
}
pub struct BlockStatement {
}
