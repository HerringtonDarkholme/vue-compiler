use super::{
    Name, Namespace, SourceLocation,
    error::{ErrorHandler, CompilationError},
    tokenizer::{Attribute, Tokenizer, Token, FlagCDataNs}
};
use std::rc::Rc;

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

pub trait ParseOption: ErrorHandler {
    fn whitespace_strategy()->  WhitespaceStrategy {
        WhitespaceStrategy::default()
    }
    fn get_namespace(_: &Vec<Element<'_>>) -> Namespace {
        Namespace::Html
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
        let ctx = Rc::new(option);
        let tokens = self.tokenizer.scan(source, ctx.clone());
        AstBuilder::new(ctx, tokens).build_ast()
    }
}

struct AstBuilder<'a, E, Ts>
where
    E: ErrorHandler,
    Ts: Iterator<Item=Token<'a>> + FlagCDataNs,
{
    ctx: Rc<E>,
    tokens: Ts,
    open_elems: Vec<Element<'a>>,
    in_pre: bool,
    in_v_pre: bool,
}

impl<'a, E, Ts> AstBuilder<'a, E, Ts>
where
    E: ErrorHandler,
    Ts: Iterator<Item=Token<'a>> + FlagCDataNs,
{
    fn new(ctx: Rc<E>, tokens: Ts) -> Self {
        Self {
            ctx,
            tokens,
            open_elems: vec![],
            in_pre: false,
            in_v_pre: false
        }
    }

    fn build_ast(mut self) -> AstRoot<'a> {
        todo!()
    }

    // must call this when handle CDATA
    fn set_tokenizer_flag(&mut self) {
        self.tokens.set_is_in_html(todo!())
    }
}
