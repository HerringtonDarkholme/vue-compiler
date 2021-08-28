use super::{
    tokenizer::{Tokenizer, TokenizerOption, ParseContext},
    Name, SourceLocation,
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

pub struct Parser<'a> {
    source: &'a str,
    tokenizer: Tokenizer,
}

pub type ParseResult<'a> = Result<AstRoot<'a>, CompilationError>;

impl<'a> Parser<'a> {
    pub fn new(source: &'a str, option: TokenizerOption) -> Self {
        Self {
            source,
            tokenizer: Tokenizer::new(option),
        }
    }
    pub fn parse(&mut self, source: &'a str) -> ParseResult<'a> {
        let mut tokens = self.tokenizer.scan(source, self);
        let opt = &mut tokens.option;
        for token in tokens {
            self.on_error(todo!())
        }
        todo!()
    }
}
impl<'a> ParseContext for Parser<'a> {
}
