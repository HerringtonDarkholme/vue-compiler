use super::{
    Name, Namespace, SourceLocation,
    error::{ErrorHandler, CompilationError},
    tokenizer::{Attribute, Token, FlagCDataNs}
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

#[derive(Clone)]
pub struct ParseOption {
    whitespace: WhitespaceStrategy,
    get_namespace: fn(_: &Vec<Element<'_>>) -> Namespace,
}

pub struct Parser {
    option: ParseOption,
}

impl Parser {
    pub fn new(option: ParseOption) -> Self {
        Self { option }
    }

    pub fn parse<'a, Ts, E>(
        &self, tokens: Ts, err_handle: E
    ) -> AstRoot<'a>
    where
        Ts: Iterator<Item=Token<'a>> + FlagCDataNs,
        E: ErrorHandler
    {
        AstBuilder {
            tokens,
            err_handle,
            option: self.option.clone(),
            open_elems: vec![],
            in_pre: false,
            in_v_pre: false,
        }.build_ast()
    }
}

struct AstBuilder<'a, Ts, Eh>
where
    Ts: Iterator<Item=Token<'a>> + FlagCDataNs,
    Eh: ErrorHandler,
{
    tokens: Ts,
    err_handle: Eh,
    option: ParseOption,
    open_elems: Vec<Element<'a>>,
    in_pre: bool,
    in_v_pre: bool,
}

impl<'a, Ts, Eh> AstBuilder<'a, Ts, Eh>
where
    Ts: Iterator<Item=Token<'a>> + FlagCDataNs,
    Eh: ErrorHandler,
{
    fn build_ast(mut self) -> AstRoot<'a> {
        todo!()
    }

    // must call this when handle CDATA
    fn set_tokenizer_flag(&mut self) {
        self.tokens.set_is_in_html(todo!())
    }
}
