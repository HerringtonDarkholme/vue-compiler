use super::super::runtime_helper::RuntimeHelper;
use super::js_expression::{
    JsChildNode, ImportItem, CallExpression,
    TemplateLiteral, CompoundExpression, ExpressionNode,
    PropsExpression, SlotsExpression, SimpleExpression,
};
use super::template_codegen::{
    RootCodegen, PlainElementCodegen,
    ComponentCodegen, SlotOutletCodegen,
    ForCodegen, IfNodeCodegen, TextCallCodegen,
    DirectiveArguments,
};
use super::{SourceLocation, ForParseResult, PatchFlag};

pub enum TemplateChildNode<'a> {
    Element(ElementNode<'a>),
    Interpolation(InterpolationNode<'a>),
    Expression(CompoundExpression<'a>),
    Text(TextNode<'a>),
    Comment(CommentNode<'a>),
    If(IfNode<'a>),
    IfBranch(IfBranchNode<'a>),
    For(ForNode<'a>),
    TextCall(TextCallNode<'a>),
}

pub enum ParentNode<'a> {
    Root(RootNode<'a>),
    Element(ElementNode<'a>),
    IfBranch(IfBranchNode<'a>),
    For(ForNode<'a>),
}

pub struct RootNode<'a> {
    children: Vec<TemplateChildNode<'a>>,
    hoists: Vec<JsChildNode<'a>>,
    cached: i32,
    temps: i32,
    codegen_node: Option<RootCodegen<'a>>,
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
    props: Vec<PropNode<'a>>,
    children: Vec<TemplateChildNode<'a>>,
    codegen_node: CodeGen,
    ssr_codegen: Option<SsrCodegen>,
    loc: SourceLocation,
}

pub enum PropNode<'a> {
    Attribute(AttributeNode<'a>),
    Directive(DirectiveNode<'a>),
}

pub type PlainElementNode<'a> = BaseElement<'a, PlainElementCodegen<'a>, TemplateLiteral>;
pub type ComponentNode<'a> = BaseElement<'a, ComponentCodegen, CallExpression>;
pub type SlotOutletNode<'a> = BaseElement<'a, SlotOutletCodegen, CallExpression>;
pub type TemplateNode<'a> = BaseElement<'a, ()>;

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
    loc: SourceLocation,
}

// directive format:
// v-name:arg.modifier=exp
pub struct DirectiveNode<'a> {
    name: &'a str,
    arg: Option<ExpressionNode<'a>>,
    modifiers: Vec<&'a str>, // mod is less used
    exp: Option<ExpressionNode<'a>>,
    parse_results: Option<ForParseResult>,
    loc: SourceLocation,
}

pub struct InterpolationNode<'a> {
    content: ExpressionNode<'a>,
    loc: SourceLocation,
}

pub struct IfNode<'a> {
    branches: Vec<IfBranchNode<'a>>,
    codegen_node: Option<IfNodeCodegen>,
    loc: SourceLocation,
}

pub struct IfBranchNode<'a> {
    // None is for v-else that has no condition
    condition: Option<ExpressionNode<'a>>,
    children: Vec<TemplateChildNode<'a>>,
    user_key: Option<PropNode<'a>>,
    loc: SourceLocation,
}

pub struct ForNode<'a> {
    source: ExpressionNode<'a>,
    key_alias: Option<ExpressionNode<'a>>,
    value_alias: Option<ExpressionNode<'a>>,
    object_index_alias: Option<ExpressionNode<'a>>,
    for_parse_result: ForParseResult,
    children: Vec<TemplateChildNode<'a>>,
    codegen_node: ForCodegen,
    loc: SourceLocation,
}

pub enum TextCallContent<'a> {
    Text(TextNode<'a>),
    Interpolation(InterpolationNode<'a>),
    Compound(CompoundExpression<'a>),
}

pub struct TextCallNode<'a> {
    content: TextCallContent<'a>,
    loc: SourceLocation,
    codegen_node: TextCallCodegen<'a>,
}

pub enum TemplateTextChildNode<'a> {
    Text(TextNode<'a>),
    Interpolation(InterpolationNode<'a>),
    Compound(CompoundExpression<'a>),
}

pub enum VNodeCallTag<'a> {
    Str(&'a str),
    Call(CallExpression),
}

pub enum VNodeCallChildren<'a> {
    MultiChildren(Vec<TemplateChildNode<'a>>),
    SingleTextChild(TemplateTextChildNode<'a>),
    ComponentSlots(SlotsExpression),
    VForFragment(),
    Hoisted(SimpleExpression<'a>),
    Noop,
}

pub enum VNodeCallProps<'a> {
    Str(&'a str),
    Hoisted(SimpleExpression<'a>),
}

pub struct VNodeCall<'a> {
    tag: VNodeCallTag<'a>,
    props: PropsExpression,
    children: VNodeCallChildren<'a>,
    patch_flag: Option<PatchFlag>,
    dynamic_props: VNodeCallProps<'a>,
    directives: Option<DirectiveArguments<'a>>,
    is_block: bool,
    disable_tracking: bool,
    is_component: bool,
    loc: SourceLocation,
}
