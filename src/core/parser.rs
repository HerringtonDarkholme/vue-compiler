use std::cell::RefCell;
use super::{
    tokenizer::{Tokenizer, TokenizeOption, ParseContext},
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

#[derive(Debug)]
pub enum WhitespaceStrategy {
    Preserve,
    Condense,
}
impl Default for WhitespaceStrategy {
    fn default() -> Self {
        WhitespaceStrategy::Condense
    }
}

pub struct ParseOption {
    whitespace: WhitespaceStrategy,
    get_namespace: fn(&Vec<Element<'_>>) -> Namespace,
    tokenize_option: TokenizeOption
}
impl Default for ParseOption {
    fn default() -> Self {
        Self {
            get_namespace: |_| Namespace::Html,
            whitespace: WhitespaceStrategy::default(),
            tokenize_option: TokenizeOption::default(),
        }
    }
}


// We need a RefCell because Rust cannot prove vec at compile time
// minimal case https://play.rust-lang.org/?gist=c5cb2658afbebceacdfc6d387c72e1ab
// Alternatively we can inject a method like `process_token` to the tokenizer
// but this inversion makes logic convoluted, reference in Servo's parser:
// https://github.com/servo/html5ever/blob/57eb334c0ffccc6f88d563419f0fbeef6ff5741c/html5ever/src/tokenizer/interface.rs#L98
#[derive(Default)]
struct ParseCtxImpl<'a> {
    option: ParseOption,
    // if RefCell is too slow, UnsafeCell can be used.
    // prefer safety since borrow_mut is called only once.
    open_elems: RefCell<Vec<Element<'a>>>,
}

impl<'a> ParseCtxImpl<'a> {
    fn new() -> Self {
        Self::default()
    }
}

impl<'a> ParseContext for ParseCtxImpl<'a> {
    fn get_namespace(&self) -> Namespace {
        let elems = self.open_elems.borrow();
        let get_namespace = self.option.get_namespace;
        get_namespace(&elems)
    }
}

pub struct Parser {
    option: ParseOption,
    tokenizer: Tokenizer,
}

pub type ParseResult<'a> = Result<AstRoot<'a>, CompilationError>;

impl Parser {
    pub fn new(tokenizer: Tokenizer) -> Self {
        Self {
            tokenizer,
            option: ParseOption::default(),
        }
    }
    pub fn with_option(mut self, option: ParseOption) -> Self {
        self.option = option;
        self
    }

    pub fn parse<'a>(&self, source: &'a str) -> ParseResult<'a> {
        let ctx = ParseCtxImpl::new();
        let open_elems = ctx.open_elems.borrow_mut();
        let tokens = self.tokenizer.scan(source, &ctx);
        for token in tokens {
        }
        todo!()
    }
}
