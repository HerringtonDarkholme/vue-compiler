//! Vue template tokenization.
//! The canonical parsing strategy should adhere to the spec below.
//! https://html.spec.whatwg.org/multipage/parsing.html#tokenization

use std::{borrow::Cow, str::Chars};
use super::{
    Name, SourceLocation, Position,
    error::{CompilationError, CompilationErrorKind as ErrorKind},
};
use smallvec::{smallvec, SmallVec};

/// DecodedStr represents text after decoding html entities.
/// SmallVec and Cow are used internally for less allocation.
#[derive(Debug)]
pub struct DecodedStr<'a>(SmallVec<[Cow<'a, str>; 1]>);

impl<'a> From<&'a str> for DecodedStr<'a> {
    fn from(s: &'a str) -> Self {
        Self(smallvec![Cow::Borrowed(s)])
    }
}

#[derive(Debug)]
pub struct Attribute<'a> {
    pub name: Name<'a>,
    pub value: Option<DecodedStr<'a>>,
}

/// Tag is used only for start tag since end tag is bare
#[derive(Debug)]
pub struct Tag<'a> {
    pub name: Name<'a>,
    pub attributes: Vec<Attribute<'a>>,
    pub self_closing: bool,
}

/// html token definition is tailored for convenience.
/// https://html.spec.whatwg.org/multipage/parsing.html#tokenization
#[derive(Debug)]
pub enum Token<'a> {
    StartTag(Tag<'a>),
    EndTag(Name<'a>), // with no attrs or self_closing flag
    // TODO: investigate if we can postpone decoding to codegen
    // 1. in SSR we don't need to output decoded entities
    // 2. in DOM we can output decoded text during transform
    // 3. parser/IRConverter does not read text content
    Text(DecodedStr<'a>), // merges chars to one str
    Comment(&'a str),
    Interpolation(&'a str), // Vue specific token
}

// NB: Token::from only takes decoded str
impl<'a> From<&'a str> for Token<'a> {
    fn from(decoded: &'a str) -> Self {
        Token::Text(DecodedStr::from(decoded))
    }
}


// equivalent HRTB form is
// for<'a> fn(&'a str, bool) -> DecodedStr<'a>
/// source: &str, is_attr: bool
pub type EntityDecoder = fn(&str, bool) -> DecodedStr<'_>;

/// Note: TokenizerOption is not thread safe.
/// due to `cached_first_char` is shared.
pub struct TokenizerOption {
    pub delimiters: (String, String),
    pub get_text_mode: fn(&str) -> TextMode,
    pub decode_entities: EntityDecoder,
    // for search optimization: only compare delimiters' first char
    cached_first_char: Option<char>,
}

impl TokenizerOption {
    fn delimiter_first_char(&mut self) -> char {
        if let Some(c) =  self.cached_first_char {
            return c
        }
        let c = self.delimiters.0.chars().next()
            .expect("interpolation delimiter cannot be empty");
        self.cached_first_char.replace(c);
        c
    }
}

impl Default for TokenizerOption {
    fn default() -> Self {
        Self {
            delimiters: ("{{".into(), "}}".into()),
            cached_first_char: Some('{'),
            get_text_mode: |_| TextMode::DATA,
            decode_entities: |s, _| DecodedStr::from(s),
        }
    }
}

/// TextMode represents different text scanning strategy.
/// e.g. Scannings in script/textarea/div are different.
#[derive(PartialEq, Eq)]
pub enum TextMode {
  //          | Elements | Entities | End sign              | Inside of
  DATA, //    | ✔        | ✔        | End tags of ancestors |
  RCDATA, //  | ✘        | ✔        | End tag of the parent | <textarea>
  RAWTEXT, // | ✘        | ✘        | End tag of the parent | <style>,<script>
  CDATA,
}

pub struct Tokenizer<'a> {
    source: &'a str,
    position: Position,
    mode: TextMode,
    option: TokenizerOption,
    errors: Vec<CompilationError>,
}

// builder methods
impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            position: Default::default(),
            mode: TextMode::DATA,
            option: Default::default(),
            errors: Vec::new(),
        }
    }
    pub fn with_option<'b>(&'b mut self, option: TokenizerOption) -> &'b mut Tokenizer<'a> {
        self.option = option;
        self
    }
}

// scanning methods
// NB: When storing self.source to a name, prefer using a ref.
// because Rust ownership can help us to prevent invalid state.
// e.g. `let src = self.source` causes a stale src after `move_by`.
// while `let src= &self.source` forbids any src usage after a mut call.
impl<'a> Tokenizer<'a> {
    // https://html.spec.whatwg.org/multipage/parsing.html#data-state
    // NB: & is not handled here but instead in `decode_entities`
    fn scan_data(&mut self) -> Token<'a> {
        debug_assert!(self.mode == TextMode::DATA);
        let d = self.option.delimiter_first_char();
        // process html entity & later
        let index = self.source
            .find(|c| c == '<' || c == d);
        // no tag or interpolation found
        if index.is_none() {
            return self.scan_text(self.source.len())
        }
        let i = index.unwrap();
        if i != 0 {
            return self.scan_text(i)
        }
        if self.source.starts_with(&self.option.delimiters.0) {
            return self.scan_interpolation()
        }
        self.scan_tag_open()
    }

    fn scan_text(&mut self, size: usize) -> Token<'a> {
        let src = self.move_by(size);
        Token::Text(self.decode_text(src, false))
    }

    fn scan_interpolation(&mut self) -> Token<'a> {
        let delimiters = &self.option.delimiters;
        debug_assert!(self.source.starts_with(&delimiters.0));
        let index =  self.source.find(&delimiters.1);
        if index.is_none() {
            self.emit_error(ErrorKind::MissingInterpolationEnd);
            let src = self.move_by(self.source.len());
            return Token::Interpolation(src)
        }
        let step = index.unwrap() + delimiters.1.len();
        let src = self.move_by(step);
        Token::Interpolation(src)
    }

    // https://html.spec.whatwg.org/multipage/parsing.html#tag-open-state
    fn scan_tag_open(&mut self) -> Token<'a> {
        // use a ref to &str to ensure source is always valid
        // that is, source cannot be used after move_by
        let source = &self.source;
        if source.starts_with("</") {
            self.scan_end_tag_open()
        } else if source.starts_with("<!") {
            self.scan_comment_and_like()
        } else if source.starts_with("<?") {
            self.emit_error(ErrorKind::UnexpectedQuestionMarkInsteadOfTagName);
            self.scan_bogus_comment()
        } else if source.len() == 1 {
            self.emit_error(ErrorKind::EofBeforeTagName);
            self.move_by(1);
            Token::from("<")
        } else if !source[1..].starts_with(ascii_alpha) {
            // we can indeed merge this standalone < char into surrounding text
            // but optimization for error is not worth the candle
            self.emit_error(ErrorKind::InvalidFirstCharacterOfTagName);
            self.move_by(1);
            Token::from("<")
        } else {
            self.scan_start_tag()
        }
    }

    // https://html.spec.whatwg.org/multipage/parsing.html#tag-name-state
    fn scan_start_tag(&mut self) -> Token<'a> {
        debug_assert!(self.source.starts_with('<'));
        self.move_by(1);
        let tag = self.scan_tag_name();
        Token::StartTag(tag)
    }
    fn scan_tag_name(&mut self) -> Tag<'a> {
        debug_assert!(self.source.starts_with(ascii_alpha));
        let chars = self.source.chars();
        let l = scan_tag_name_length(chars);
        debug_assert!(l > 0);
        let name = self.move_by(l);
        let attributes = self.scan_attributes();
        let self_closing = if self.source.is_empty() {
            self.emit_error(ErrorKind::EofInTag);
            false
        } else {
            self.scan_close_start_tag()
        };
        Tag{
            name, attributes, self_closing,
        }
    }
    // return attributes and if the tag is self closing
    // https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-name-state
    fn scan_attributes(&mut self) -> Vec<Attribute<'a>> {
        let mut attrs = vec![]; // TODO: size hint?
        self.skip_whitespace();
        // TODO: forbid infinite loop
        loop {
            debug_assert!(self.source.starts_with(non_whitespace));
            if self.is_about_to_close_tag() {
                return attrs
            }
            if self.did_skip_slash_in_tag() {
                continue;
            }
            let attr = self.scan_attribute();
            attrs.push(attr);
        }
    }
    // https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-name-state
    fn scan_attribute(&mut self) -> Attribute<'a> {
        debug_assert!(!self.source.is_empty());
        let name = self.scan_attr_name();
        self.skip_whitespace();
        if self.is_about_to_close_tag() || self.did_skip_slash_in_tag() {
            return Attribute {
                name, value: None,
            }
        }
        debug_assert!(self.source.starts_with('='));
        self.move_by(1); // equal sign
        let value = self.scan_attr_value();
        Attribute {
            name, value
        }
    }
    fn is_about_to_close_tag(&self) -> bool {
        let source = &self.source; // must get fresh source
        source.is_empty() || source.starts_with("/>") || source.starts_with('>')
    }
    fn did_skip_slash_in_tag(&mut self) -> bool {
        if self.source.starts_with('/') {
            self.emit_error(ErrorKind::UnexpectedSolidusInTag);
            self.move_by(1);
            true
        } else {
            false
        }
    }
    // https://html.spec.whatwg.org/multipage/parsing.html#attribute-name-state
    fn scan_attr_name(&mut self) -> &'a str {
        debug_assert!(self.source.starts_with(is_valid_name_char));
        // case like <tag =="value"/>
        if self.source.starts_with('=') {
            self.emit_error(ErrorKind::UnexpectedEqualsSignBeforeAttributeName);
            let s = self.move_by(1);
            debug_assert!(s == "=");
            return s
        }
        let count = self.source.chars()
            .take_while(|&c| semi_valid_attr_name(c))
            .count();
        let src = self.move_by(count);
        if src.contains(|c| matches!(c, '<' | '"' | '\'')) {
            self.emit_error(ErrorKind::UnexpectedCharacterInAttributeName);
        }
        src
    }
    // https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-value-state
    fn scan_attr_value(&mut self) -> Option<DecodedStr<'a>> {
        self.skip_whitespace();
        let source = &self.source;
        if source.starts_with('>') {
            self.emit_error(ErrorKind::MissingAttributeValue);
            return None
        }
        for &c in ['"', '\''].iter() {
            if self.source.starts_with(c) {
                return Some(self.scan_quoted_attr_value(c))
            }
        }
        self.scan_unquoted_attr_value()
    }
    // https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(double-quoted)-state
    // https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(single-quoted)-state
    fn scan_quoted_attr_value(&mut self, quote: char) -> DecodedStr<'a> {
        debug_assert!(self.source.starts_with(quote));
        self.move_by(1);
        let src = if let Some(i) = self.source.find(quote) {
            let val = self.move_by(i);
            self.move_by(1); // consume quote char
            val
        } else {
            self.move_by(self.source.len())
        };
        self.decode_text(src, /*is_attr*/ true)
    }
    // https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(unquoted)-state
    fn scan_unquoted_attr_value(&mut self) -> Option<DecodedStr<'a>> {
        let val_len = self.source.chars()
            .take_while(semi_valid_unquoted_attr_value)
            .count();
        // unexpected EOF: <tag attr=
        if val_len == 0 {
            // whitespace or > are precluded in scan_attribute
            // so empty value must implies EOF
            debug_assert!(self.source.len() == 0);
            return None
        }
        let src = self.move_by(val_len);
        if src.contains(|c| matches!(c, '"' | '\'' | '<' | '=' | '`')) {
            self.emit_error(ErrorKind::UnexpectedCharacterInUnquotedAttributeValue);
        }
        Some(self.decode_text(src, /* is_attr */ true))
    }

    fn scan_close_start_tag(&mut self) -> bool {
        debug_assert!(!self.source.is_empty());
        if self.source.starts_with("/>") {
            self.move_by(2);
            true
        } else {
            debug_assert!(self.source.starts_with(">"));
            self.move_by(1);
            false
        }
    }
    // https://html.spec.whatwg.org/multipage/parsing.html#end-tag-open-state
    fn scan_end_tag_open(&mut self) -> Token<'a> {
        debug_assert!(self.source.starts_with("</"));
        let source = &self.source;
        if source.len() == 2{
            self.emit_error(ErrorKind::EofBeforeTagName);
            Token::from(self.move_by(2))
        } else if source.starts_with("</>") {
            self.emit_error(ErrorKind::MissingEndTagName);
            self.move_by(3);
            Token::from("")
        } else if !self.source[2..].starts_with(ascii_alpha) {
            self.emit_error(ErrorKind::InvalidFirstCharacterOfTagName);
            self.scan_bogus_comment()
        } else {
            self.scan_end_tag()
        }
    }
    // errors emit here is defined at the top of the tokenization spec
    fn scan_end_tag(&mut self) -> Token<'a> {
        debug_assert!(self.source.starts_with("</"));
        self.move_by(2);
        // indeed in end tag collecting attributes is useless
        // but, no, I don't want to opt for ill-formed input
        let tag = self.scan_tag_name();
        // When an end tag token is emitted with attributes
        if !tag.attributes.is_empty() {
            self.emit_error(ErrorKind::EndTagWithAttributes);
        }
        // When an end tag token is emitted with its self-closing flag set
        if tag.self_closing {
            self.emit_error(ErrorKind::EndTagWithTrailingSolidus);
        }
        Token::EndTag(tag.name)
    }

    fn scan_comment_and_like(&mut self) -> Token<'a> {
        // TODO: investigate https://github.com/jneem/teddy
        // for simd string pattern matching
        let s = &self.source;
        if s.starts_with("<!--") {
            self.scan_comment()
        } else if s.starts_with("<!DOCTYPE") {
            self.scan_bogus_comment()
        } else if s.starts_with("<!CDATA[") {
            self.scan_cdata()
        } else {
            self.emit_error(ErrorKind::IncorrectlyOpenedComment);
            self.scan_bogus_comment()
        }
    }
    fn scan_comment(&mut self) -> Token<'a> {
        debug_assert!(self.source.starts_with("<!--"));
        while let Some(end) = self.source.find("--") {
            let s = &self.source[end..];
            let offset = if s.starts_with("!>") {
                2
            } else if s.starts_with('>') {
                1
            } else {
                0
            };
            if offset > 0 {
                let src = self.move_by(end + offset + 2);
                return Token::Comment(src)
            }
        }
        self.emit_error(ErrorKind::EofInComment);
        Token::Comment(self.source)
    }
    #[cold]
    fn scan_bogus_comment(&mut self) -> Token<'a> {
        todo!()
    }
    #[cold]
    fn scan_cdata(&mut self) -> Token<'a> {
        todo!()
    }

    fn scan_rawtext(&mut self) -> Token<'a> {
        todo!()
    }

    fn scan_rcdata(&mut self) -> Token<'a> {
        todo!()
    }

}

// utility methods
impl<'a> Tokenizer<'a> {
    fn emit_error(&mut self, error_kind: ErrorKind) {
        let loc = self.current_location();
        let err = CompilationError::new(error_kind).with_location(loc);
        self.errors.push(err);
    }

    fn current_location(&self) -> SourceLocation {
        todo!()
    }

    fn decode_text(&self, src: &'a str, is_attr: bool) -> DecodedStr<'a> {
        let decode = self.option.decode_entities;
        let decoded = decode(src, is_attr);
        decoded
    }

    /// move tokenizer's internal position forward and return &str
    /// tokenizer's line/column are also updated in the method
    /// NB: it only moves forward, not backward
    /// `advance_to` is a better name but it collides with iter
    fn move_by(&mut self, size: usize) -> &'a str {
        debug_assert!(size > 0, "tokenizer must move forward");
        let mut lines = 0;
        let mut last_new_line_pos = -1;
        for (i, c) in self.source[..size].chars().enumerate() {
            if c == '\n' {
                lines += 1;
                last_new_line_pos = i as i32;
            }
        }
        let old_source = self.source;
        self.source = &self.source[size..];
        let pos = &mut self.position;
        pos.offset += size;
        pos.line += lines;
        pos.column = if last_new_line_pos == -1 {
            pos.column + size
        } else {
            size - last_new_line_pos as usize
        };
        &old_source[..size]
    }

    fn skip_whitespace(&mut self) -> usize {
        let idx = self.source.find(non_whitespace);
        if let Some(i) = idx {
            if i != 0 {
                self.move_by(i);
            }
            i
        } else {
            let len = self.source.len();
            self.move_by(len);
            len
        }
    }
}

#[inline]
fn ascii_alpha(c: char) -> bool {
    c.is_ascii_alphabetic()
}

// `< ' "` are not valid but counted as semi valid
// to leniently recover from a parsing error
#[inline]
fn semi_valid_attr_name(c: char) -> bool {
    is_valid_name_char(c) && c != '='
}

// only whitespace and > terminates unquoted attr value
// other special char only emits error
#[inline]
fn semi_valid_unquoted_attr_value(&c: &char) -> bool {
    !c.is_ascii_whitespace() && c != '>'
}

#[inline]
fn is_valid_name_char(c: char) -> bool {
    !c.is_ascii_whitespace() && c != '/' && c != '>'
}

fn non_whitespace(c: char) -> bool {
    c.is_whitespace()
}

// tag name should begin with [a-zA-Z]
// followed by chars except whitespace, / or >
fn scan_tag_name_length(mut chars: Chars<'_>) -> usize {
    let first_char = chars.next();
    debug_assert!(first_char.is_some());
    if !first_char.unwrap().is_ascii_alphabetic() {
        return 0
    }
    let l = chars
        .take_while(|&c| is_valid_name_char(c))
        .count();
    l + 1
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token<'a>;
    // https://html.spec.whatwg.org/multipage/parsing.html#concept-frag-parse-context
    fn next(&mut self) -> Option<Self::Item> {
        if self.source.is_empty() {
            return None
        }
        Some(match self.mode {
            TextMode::DATA => self.scan_data(),
            TextMode::RCDATA => self.scan_rcdata(),
            TextMode::RAWTEXT => self.scan_rawtext(),
            TextMode::CDATA => self.scan_cdata(),
        })
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        let cases = [
            r#"<a v-bind:['foo' + bar]="value">...</a>"#,
            r#"<tag =value />"#,
            r#"<a wrong-attr>=123 />"#,
            r#"<a></a < / attr attr=">" >"#,
        ];
    }
}
