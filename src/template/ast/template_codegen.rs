use super::{
    element_node::TemplateChildNode,
    js_expression::{
        JsChildNode, BlockStatement,
        SimpleExpression, CacheExpression,
        MemoExpression, IfConditionalExpression,
        CallExpression, ExpressionNode, ObjectExpression,
    },
};

pub enum RootCodegen<'a> {
    TemplateChild(TemplateChildNode<'a>),
    JsChild(JsChildNode<'a>),
    JsBlock(BlockStatement),
}

// ElementNode Codegen
pub enum PlainElementCodegen<'a> {
    VNodeCall(),
    Simple(SimpleExpression<'a>), // when hoisted
    Cache(CacheExpression), // when cached by v-once
    Memo(MemoExpression), // when cached by v-memo
}

pub enum ComponentCodegen {
    VNodeCall(),
    Cache(CacheExpression), // when cached by v-once
    Memo(MemoExpression), // when cached by v-memo
}

pub enum SlotOutletCodegen {
    RenderSlot(),
    Cache(CacheExpression), // when cached by v-once
}
// end of ElementNode Codegen

pub struct ForCodegen {
}

pub enum IfNodeCodegen {
    IfConditional(IfConditionalExpression),
    Cache(CacheExpression), // <div v-if v-once>
}

pub enum TextCallCodegen<'a> {
    Call(CallExpression),
    Simple(SimpleExpression<'a>),
}

pub struct DirectiveArguments<'a> {
    elements: Vec<DirectiveArgument<'a>>,
}
// dir, exp, arg, modifiers
// v-dir:arg.modifier=exp
pub struct DirectiveArgument<'a> {
    dir: &'a str,
    exp: Option<ExpressionNode<'a>>,
    arg: Option<ExpressionNode<'a>>,
    modifiers: Option<ObjectExpression<'a>>,
}
