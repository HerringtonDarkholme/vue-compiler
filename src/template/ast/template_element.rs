use super::super::runtime_helper::RuntimeHelper;
use super::js_expression::{
    JsChildNode, ImportItem, CallExpression,
    SimpleExpression, CacheExpression, MemoExpression,
    TemplateLiteral, CompoundExpression, ExpressionNode,
};
use super::{SourceLocation, CodegenNode, ForParseResult};

pub enum TemplateChildNode<'a> {
    Element(ElementNode<'a>),
    Interpolation(),
    Expression(CompoundExpression),
    Text(TextNode<'a>),
    Comment(CommentNode<'a>),
    If(),
    IfBranch(),
    For(),
    TextCall(),
}

pub enum ParentNode<'a> {
    Root(RootNode<'a>),
    Element(ElementNode<'a>),
    IfBranch(IfBranchNode),
    For(ForNode),
}

pub struct RootNode<'a> {
    children: Vec<TemplateChildNode<'a>>,
    hoists: Vec<JsChildNode>,
    cached: i32,
    temps: i32,
    codegen_node: Option<CodegenNode<'a>>,
    preambles: Preambles<'a>,
    loc: SourceLocation,
}

pub struct Preambles<'a> {
    helpers: RuntimeHelper,
    components: Vec<&'a str>,
    directives: Vec<&'a str>,
    imports: Vec<ImportItem<'a>>,
    // ssrHelpers?: SSRHelper,
}

pub enum ElementNode<'a> {
    PlainElement(PlainElementNode<'a>),
    Component(ComponentNode<'a>),
    SlotOutlet(SlotOutletNode<'a>),
    Template(TemplateNode<'a>),
}

pub struct BaseElement<'a, CodeGen, SsrCodegen=()> {
    // currently the only ns is HTML
    // ns: Namespace
    tag: &'a str,
    is_self_closing: bool,
    props: Vec<PropsNode<'a>>,
    children: Vec<TemplateChildNode<'a>>,
    codegen_node: CodeGen,
    ssr_codegen: Option<SsrCodegen>,
    loc: SourceLocation,
}

pub enum PropsNode<'a> {
    Attribute(AttributeNode<'a>),
    Directive(DirectiveNode<'a>),
}

enum PlainElementCodegen<'a> {
    VNodeCall(),
    Simple(SimpleExpression<'a>), // when hoisted
    Cache(CacheExpression), // when cached by v-once
    Memo(MemoExpression), // when cached by v-memo
}
type PlainElementNode<'a> = BaseElement<'a, PlainElementCodegen<'a>, TemplateLiteral>;

enum ComponentCodegen {
    VNodeCall(),
    Cache(), // when cached by v-once
    Memo(), // when cached by v-memo
}
type ComponentNode<'a> = BaseElement<'a, ComponentCodegen, CallExpression>;

enum SlotOutletCodegen {
    RenderSlot(),
    Cache(CacheExpression), // when cached by v-once
}
type SlotOutletNode<'a> = BaseElement<'a, SlotOutletCodegen, CallExpression>;

type TemplateNode<'a> = BaseElement<'a, ()>;

pub struct TextNode<'a> {
    content: &'a str,
    loc: SourceLocation,
}

pub struct CommentNode<'a> {
    content: &'a str,
    loc: SourceLocation,
}

pub struct AttributeNode<'a> {
    name: &'a str,
    value: TextNode<'a>,
}

// directive format:
// v-name:arg.modifier=exp
pub struct DirectiveNode<'a> {
    name: &'a str,
    arg: Option<ExpressionNode<'a>>,
    modifiers: Vec<&'a str>, // mod is less used
    exp: Option<ExpressionNode<'a>>,
    parse_results: Option<ForParseResult>,
}

pub struct IfBranchNode {
}

pub struct ForNode {
}
