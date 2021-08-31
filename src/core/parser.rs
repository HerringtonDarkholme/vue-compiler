// Vue Template Parser does not adhere to HTML spec.
// https://html.spec.whatwg.org/multipage/parsing.html#tree-construction
// According to the spec: tree construction has several points:
// 1. Tree Construction Dispatcher: N/A. We don't consider foreign content.
// 2. appropriate place for inserting a node: For table/template elements.
//    N/A.  We can't know the global tree in a component.
// 3. create an element for a token: For custom component
//    N/A. We don't handle JS execution for custom component.
// 4. adjust MathML/SVG attributes:
//    ?? Should we handle this? The original Vue compiler does not.
// 5. Inserting Text/Comment: N/A. We don't handle script/insertion location.
// 6. Parsing elements that contain only text: Already handled in tokenizer.
// 7. Closing elements that have implied end tags:
//    N/A: Rule is too complicated and requires non-local context.
// Instead, we use a simple stack to construct AST.

use super::{
    Name, Namespace, SourceLocation,
    error::{ErrorHandler, CompilationError, CompilationErrorKind as ErrorKind},
    tokenizer::{Attribute, Token, Tag, DecodedStr, TokenSource},
};

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
        Ts: TokenSource<'a>,
        E: ErrorHandler
    {
        let need_flag_namespace = tokens.need_flag_hint();
        AstBuilder {
            tokens,
            err_handle,
            option: self.option.clone(),
            open_elems: vec![],
            root_nodes: vec![],
            in_pre: false,
            in_v_pre: false,
            need_flag_namespace,
        }.build_ast()
    }
}

struct AstBuilder<'a, Ts, Eh>
where
    Ts: TokenSource<'a>,
    Eh: ErrorHandler,
{
    tokens: Ts,
    err_handle: Eh,
    option: ParseOption,
    open_elems: Vec<Element<'a>>,
    root_nodes: Vec<AstNode<'a>>,
    in_pre: bool,
    in_v_pre: bool,
    need_flag_namespace: bool,
}

// utility method
impl<'a, Ts, Eh> AstBuilder<'a, Ts, Eh>
where
    Ts: TokenSource<'a>,
    Eh: ErrorHandler,
{
    // Insert node into current insertion point.
    // It's the last open element's children if open_elems is not empty.
    // Otherwise it is root_nodes.
    fn insert_node(&mut self, node: AstNode<'a>) {
        if let Some(elem) = self.open_elems.last_mut() {
            elem.children.push(node);
        } else {
            self.root_nodes.push(node);
        }
    }

    fn emit_error(&self, kind: ErrorKind, loc: SourceLocation) {
        let error = CompilationError::new(kind).with_location(loc);
        self.err_handle.on_error(error)
    }
}

// parse logic
impl<'a, Ts, Eh> AstBuilder<'a, Ts, Eh>
where
    Ts: TokenSource<'a>,
    Eh: ErrorHandler,
{
    fn build_ast(mut self) -> AstRoot<'a> {
        let start = self.tokens.current_position();
        while let Some(token) = self.tokens.next() {
            self.parse_token(token);
        }
        let location = self.tokens.get_location_from(start);
        AstRoot{ children: self.root_nodes, location }
    }

    fn parse_token(&mut self, token: Token<'a>) {
        // https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inbody:current-node-26
        match token {
            Token::EndTag(s) => self.parse_end_tag(s),
            Token::Text(text) => self.parse_text(text),
            Token::StartTag(tag) => self.parse_open_tag(tag),
            Token::Comment(c) => self.parse_comment(c),
            Token::Interpolation(i) => self.parse_interpolation(i),
        };
    }
    fn parse_open_tag(&mut self, tag: Tag<'a>) {
        todo!()
    }
    fn parse_end_tag(&mut self, s: &'a str) {
        // rfind is good since only mismatch will traverse stack
        let index = self.open_elems
            .iter()
            .enumerate()
            .rfind(|p| { element_matches_end_tag(p.1, s) })
            .map(|p| p.0);
        if let Some(i) = index {
            while self.open_elems.len() > i {
                self.close_element();
            }
        } else {
            let start = self.tokens.last_position();
            let loc = self.tokens.get_location_from(start);
            self.emit_error(ErrorKind::InvalidEndTag, loc);
        }
    }
    fn close_element(&mut self) {
        let mut elem = self.open_elems.pop().unwrap();
        let start = elem.location.start;
        let location = self.tokens.get_location_from(start);
        elem.location = location;
        self.insert_node(self.parse_element(elem));
    }
    fn parse_element(&self, elem: Element<'a>) -> AstNode<'a> {
        if self.in_v_pre {
            AstNode::Plain(elem)
        } else if elem.tag_name == "slot" {
            AstNode::Slot(elem)
        } else if is_template_element(&elem) {
            AstNode::Template(elem)
        } else if self.is_component(&elem) {
            AstNode::Component(elem)
        } else {
            AstNode::Plain(elem)
        }
    }
    fn parse_text(&mut self, mut text: DecodedStr<'a>) {
        while let Some(token) = self.tokens.next() {
            if matches!(&token, Token::Text(ref s)) {
                todo!("merge text node here")
            } else {
                // NB: token must not be dropped
                self.parse_token(token);
            }
        }
    }
    fn parse_comment(&mut self, c: &'a str) {
        let pos = self.tokens.last_position();
        let source_node = SourceNode{
            source: c,
            location: self.tokens.get_location_from(pos)
        };
        self.insert_node(AstNode::Comment(source_node));
    }
    fn parse_interpolation(&mut self, src: &'a str) {
        let pos = self.tokens.last_position();
        let source_node = SourceNode{
            source: src,
            location: self.tokens.get_location_from(pos)
        };
        self.insert_node(AstNode::Interpolation(source_node));
    }

    // must call this when handle CDATA
    fn set_tokenizer_flag(&mut self) {
        self.tokens.set_is_in_html(todo!())
    }

    fn is_component(&self, e: &Element) -> bool {
        todo!()
    }
}

fn is_special_template_directive(dir: &Directive) -> bool {
    todo!()
}

fn is_template_element(e: &Element) -> bool {
    e.tag_name == "template" && e.directives.iter().any(is_special_template_directive)
}

fn element_matches_end_tag(e: &Element, tag: &str) -> bool {
    e.tag_name.eq_ignore_ascii_case(tag)
}
