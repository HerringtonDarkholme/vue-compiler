use super::{
    tokenizer::{Tokenizer, TokenizerOption, ParseContext},
    Name, SourceLocation, Namespace,
    error::CompilationError,
};

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
    pub attributes: Vec<Attribute<'a>>,
    pub directives: Vec<Directive<'a>>,
    pub children: Vec<AstNode<'a>>,
    pub loc: SourceLocation,
}

pub struct Attribute<'a> {
    name: Name<'a>,
    value: &'a str,
}

/// Directive supports two forms
/// static and dynamic
enum DirectiveArg<'a> {
    // :static="val"
    Static(Name<'a>),
    Dynamic(Name<'a>), // :[dynamic]="val"
}

/// Directive has
/// v-name:arg.modifer="expr"
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

pub enum WhitespaceStrategy {
    Preserve,
    Condense,
}
pub trait ParseOption {
    fn whitespace_strategy() -> WhitespaceStrategy;
}

pub struct Parser {
    tokenizer: Tokenizer,
}

pub type ParseResult<'a> = Result<AstRoot<'a>, CompilationError>;

impl Parser {
    pub fn new(option: TokenizerOption) -> Self {
        Self {
            tokenizer: Tokenizer::new(option),
        }
    }
    pub fn parse<'a>(&mut self, source: &'a str) -> ParseResult<'a> {
        let mut tokens = self.tokenizer.scan(source, self);
        for token in tokens {
        }
        todo!()
    }
}
