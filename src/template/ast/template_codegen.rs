use super::{
    SourceLocation,
    element_node::{
        TemplateChildNode, VNodeCall,
    },
    js_expression::{
        JsChildNode, BlockStatement,
        SimpleExpression, CacheExpression,
        MemoExpression, IfConditionalExpression,
        CallExpression, ExpressionNode, ObjectExpression,
        TemplateLiteral, IfStatement, AssignmentExpression,
        ReturnStatement, SequenceExpression,
    },
};

pub enum RootCodegen<'a> {
    TemplateChild(TemplateChildNode<'a>),
    JsChild(JsChildNode<'a>),
    JsBlock(BlockStatement<'a>),
}

// ElementNode Codegen
pub enum PlainElementCodegen<'a> {
    VNode(VNodeCall<'a>),
    Simple(SimpleExpression<'a>), // when hoisted
    Cache(CacheExpression<'a>), // when cached by v-once
    Memo(MemoExpression<'a>), // when cached by v-memo
}

pub enum ComponentCodegen<'a> {
    VNode(VNodeCall<'a>),
    Cache(CacheExpression<'a>), // when cached by v-once
    Memo(MemoExpression<'a>), // when cached by v-memo
}

pub enum SlotOutletCodegen<'a> {
    RenderSlot(),
    Cache(CacheExpression<'a>), // when cached by v-once
}
// end of ElementNode Codegen

pub struct ForCodegen {
}

pub enum IfNodeCodegen<'a> {
    IfConditional(IfConditionalExpression),
    Cache(CacheExpression<'a>), // <div v-if v-once>
}

pub enum TextCallCodegen<'a> {
    Call(CallExpression<'a>),
    Simple(SimpleExpression<'a>),
}

pub struct DirectiveArguments<'a> {
    elements: Vec<DirectiveArgument<'a>>,
    loc: SourceLocation,
}
// dir, exp, arg, modifiers
// v-dir:arg.modifier=exp
pub struct DirectiveArgument<'a> {
    dir: &'a str,
    exp: Option<ExpressionNode<'a>>,
    arg: Option<ExpressionNode<'a>>,
    modifiers: Option<ObjectExpression<'a>>,
    loc: SourceLocation,
}

pub struct BlockCodegen<'a> {
    loc: SourceLocation,
}

pub enum SsrCodegen<'a> {
    Block(BlockStatement<'a>),
    TemplateLiteral(TemplateLiteral<'a>),
    If(IfStatement<'a>),
    Assignment(AssignmentExpression<'a>),
    Return(ReturnStatement<'a>),
    Sequence(SequenceExpression<'a>),
}
