use std::rc::Rc;
use super::{
    SourceLocation,
    element_node::{
        InterpolationNode, TextNode,
        VNodeCall, TemplateChildNode,
    },
    template_codegen::BlockCodegen,

};

pub enum JsChildNode<'a> {
    VNode(Rc<VNodeCall<'a>>),
    Call(CallExpression<'a>),
    Object(ObjectExpression<'a>),
    Array(ArrayExpression<'a>),
    Expression(ExpressionNode<'a>),
    Function(FunctionExpression<'a>),
    Conditional(ConditionalExpression<'a>),
    Assignment(AssignmentExpression<'a>),
    Sequence(SequenceExpression<'a>),
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

pub enum CallArguments<'a> {
    Str(&'a str),
    JsChild(JsChildNode<'a>),
    Template(Vec<TemplateChildNode<'a>>),
    // TODO
    // Ssr(SsrCodegenNode),
}

pub struct CallExpression<'a> {
    callee: &'a str,
    arguments: CallArguments<'a>,
    loc: SourceLocation,
}

pub struct ObjectExpression<'a> {
    properties: Vec<Property<'a>>,
    loc: SourceLocation,
}
pub struct Property<'a> {
    key: ExpressionNode<'a>,
    value: JsChildNode<'a>,
    loc: SourceLocation,
}

pub struct ArrayExpression<'a> {
    elements: Vec<&'a str>
    loc: SourceLocation,
}

enum FunctionParam<'a> {
    Str(&'a str),
    Expression(ExpressionNode<'a>),
}
enum FunctionReturn<'a> {
    Template(Vec<TemplateChildNode<'a>>),
    JsChild(JsChildNode<'a>),
}
pub enum FunctionBody<'a> {
    Block(BlockStatement<'a>),
    If(IfStatement<'a>),
}

pub struct FunctionExprBase<'a, R> {
    params: Vec<FunctionParam<'a>>,
    returns: R,
    body: FunctionBody<'a>,
    newline: bool,
    ///  This flag is for codegen to determine
    ///  to generate the withScopeId() wrapper
    is_slot: bool,
    loc: SourceLocation,
}

pub type FunctionExpression<'a> = FunctionExprBase<'a, FunctionReturn<'a>>;

pub struct ConditionalExpression<'a> {
    test: JsChildNode<'a>,
    consequent: JsChildNode<'a>,
    alternate: JsChildNode<'a>,
    newline: bool,
    loc: SourceLocation,
}

pub struct CacheExpression<'a> {
    index: usize,
    value: JsChildNode<'a>,
    is_vnode: bool,
    loc: SourceLocation,
}
pub struct MemoExpression<'a> {
    arguments: (ExpressionNode<'a>, MemoFactory<'a>, &'a str, &'a str),
    loc: SourceLocation,
}
pub type MemoFactory<'a> = FunctionExprBase<'a, BlockCodegen<'a>>;


pub enum BlockBody<'a> {
    JsChild(JsChildNode<'a>),
    If(IfStatement<'a>),
}
pub struct BlockStatement<'a> {
    body: Vec<BlockBody<'a>>,
    loc: SourceLocation,
}
pub enum TemplateLiteralElement<'a> {
    Literal(&'a str),
    Interpolation(JsChildNode<'a>),
}
pub struct TemplateLiteral<'a> {
    loc: SourceLocation,
    elements: Vec<TemplateLiteralElement<'a>>,
}

pub enum IfAlternate<'a> {
    If(IfAlternate<'a>),
    Block(BlockStatement<'a>),
    Return(ReturnStatement<'a>),
}

pub struct IfStatement<'a> {
    test: ExpressionNode<'a>,
    consequent: BlockStatement<'a>,
    alertnate: Option<IfAlternate<'a>>,
    loc: SourceLocation,
}
pub struct AssignmentExpression<'a> {
    left: SimpleExpression<'a>,
    right: JsChildNode<'a>,
    loc: SourceLocation,
}
pub struct SequenceExpression<'a> {
    expressions: Vec<JsChildNode<'a>>,
    loc: SourceLocation,
}
pub struct ReturnStatement<'a> {
    returns: FunctionReturn<'a>,
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
