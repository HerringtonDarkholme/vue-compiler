use super::{
    Name, Namespace, SourceLocation,
    error::{
        ErrorHandleOption, CompilationError,
    },
    tokenizer::{
        ParseContext,
        Tokenizer, Token, Attribute
    },
};
use std::rc::Rc;
use std::cell::RefCell;

pub enum AstNode<'a> {
    Plain(Element<'a>),
    Template(Element<'a>),
    Component(Element<'a>),
    Slot(Element<'a>),
    Interpolation(&'a str),
    Text(&'a str),
}

pub struct Element<'a> {
    pub tag_name: Name<'a>,
    pub namespace: Namespace,
    pub attributes: Vec<Attribute<'a>>,
    pub directives: Vec<Directive<'a>>,
    pub children: Vec<AstNode<'a>>,
    pub loc: SourceLocation,
}

/// Directive supports two forms
/// static and dynamic
enum DirectiveArg<'a> {
    // :static="val"
    Static(Name<'a>),
    Dynamic(Name<'a>), // :[dynamic]="val"
}

/// Directive has
/// v-name:arg.modifier="expr"
pub struct Directive<'a> {
    name: Name<'a>,
    arg: DirectiveArg<'a>,
    modifiers: Vec<&'a str>,
    loc: SourceLocation,
}

pub struct AstRoot<'a> {
    children: Vec<AstNode<'a>>,
    loc: SourceLocation,
}

#[derive(Debug, Clone)]
pub enum WhitespaceStrategy {
    Preserve,
    Condense,
}
impl Default for WhitespaceStrategy {
    fn default() -> Self {
        WhitespaceStrategy::Condense
    }
}

pub trait ParseOption: ErrorHandleOption {
    fn whitespace_strategy()->  WhitespaceStrategy {
        WhitespaceStrategy::default()
    }
    fn get_namespace(_: &Vec<Element<'_>>) -> Namespace {
        Namespace::Html
    }
}

struct ParseCtxImpl<'a, O: ParseOption> {
    // if RefCell is too slow, UnsafeCell can be used.
    // prefer safety for now.
    open_elems: RefCell<Vec<Element<'a>>>,
    option: O,
}

impl<'a, O> ParseCtxImpl<'a, O>
where O: ParseOption
{
    fn new(option: O) -> Self {
        Self {
            open_elems: RefCell::new(vec![]),
            option,
        }
    }
}

impl<'a, O> ParseContext for ParseCtxImpl<'a, O>
where O: ParseOption
{
    fn on_error(&self, err: CompilationError) {
        self.option.on_error(err);
    }
}

pub struct Parser {
    tokenizer: Tokenizer,
}

impl Parser {
    pub fn new(tokenizer: Tokenizer) -> Self {
        Self {
            tokenizer,
        }
    }

    pub fn parse<'a, O>(
        &self, source: &'a str, option: O
    ) -> AstRoot<'a>
    where O: ParseOption + 'a
    {
        let ctx = Rc::new(ParseCtxImpl::new(option));
        let tokens = self.tokenizer.scan(source, ctx.clone());
        AstBuilder::new(ctx, tokens).build_ast()
    }
}

struct AstBuilder<'a, Ctx, Ts>
where
    Ctx: ParseContext,
    Ts: Iterator<Item=Token<'a>>,
{
    ctx: Rc<Ctx>,
    tokens: Ts,
    in_pre: bool,
    in_v_pre: bool,
}

impl<'a, Ctx, Ts> AstBuilder<'a, Ctx, Ts>
where
    Ctx: ParseContext,
    Ts: Iterator<Item=Token<'a>>,
{
    fn new(ctx: Rc<Ctx>, tokens: Ts) -> Self {
        Self {
            ctx,
            tokens,
            in_pre: false,
            in_v_pre: false
        }
    }

    fn build_ast(mut self) -> AstRoot<'a> {
        todo!()
    }
}
