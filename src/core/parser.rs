use super::{Name, Namespace, Position, SourceLocation, error::{ErrorHandler, CompilationError}, tokenizer::{Attribute, Token, FlagCDataNs, DecodedStr, Locatable}};

pub enum AstNode<'a> {
    Plain(Element<'a>),
    Template(Element<'a>),
    Component(Element<'a>),
    Slot(Element<'a>),
    Text(TextNode<'a>),
    Interpolation(SourceNode<'a>),
    Comment(SourceNode<'a>),
}

pub struct SourceNode<'a> {
    pub source: &'a str,
    pub location: SourceLocation,
}

pub struct TextNode<'a> {
    pub text: DecodedStr<'a>,
    pub location: SourceLocation,
}

pub struct Element<'a> {
    pub tag_name: Name<'a>,
    pub namespace: Namespace,
    pub attributes: Vec<Attribute<'a>>,
    pub directives: Vec<Directive<'a>>,
    pub children: Vec<AstNode<'a>>,
    pub location: SourceLocation,
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
    location: SourceLocation,
}

pub struct AstRoot<'a> {
    children: Vec<AstNode<'a>>,
    location: SourceLocation,
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
        Ts: Iterator<Item=Token<'a>> + FlagCDataNs + Locatable,
        E: ErrorHandler
    {
        let need_flag_namespace = tokens.need_flag_hint();
        AstBuilder {
            tokens,
            err_handle,
            option: self.option.clone(),
            open_elems: vec![],
            in_pre: false,
            in_v_pre: false,
            need_flag_namespace,
        }.build_ast()
    }
}

struct AstBuilder<'a, Ts, Eh>
where
    Ts: Iterator<Item=Token<'a>> + FlagCDataNs + Locatable,
    Eh: ErrorHandler,
{
    tokens: Ts,
    err_handle: Eh,
    option: ParseOption,
    open_elems: Vec<Element<'a>>,
    in_pre: bool,
    in_v_pre: bool,
    need_flag_namespace: bool,
}

impl<'a, Ts, Eh> AstBuilder<'a, Ts, Eh>
where
    Ts: Iterator<Item=Token<'a>> + FlagCDataNs + Locatable,
    Eh: ErrorHandler,
{
    fn build_ast(mut self) -> AstRoot<'a> {
        let start = self.tokens.current_position();
        let children = self.parse_children();
        let location = self.tokens.get_location_from(start);
        AstRoot{ children, location }
    }

    fn parse_children(&mut self) -> Vec<AstNode<'a>> {
        let mut children = vec![];
        children
    }
    fn parse_element(&mut self) -> AstNode<'a> {
        todo!()
    }
    fn parse_text(&mut self) -> AstNode<'a> {
        todo!()
    }

    // must call this when handle CDATA
    fn set_tokenizer_flag(&mut self) {
        self.tokens.set_is_in_html(todo!())
    }
}
