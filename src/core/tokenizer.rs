//! Vue template tokenization.
//! The canonical parsing strategy should adhere to the spec below.
//! https://html.spec.whatwg.org/multipage/parsing.html#tokenization

use super::{
    error::{
        CompilationError, ErrorHandler,
        CompilationErrorKind as ErrorKind,
    },
    Name, Position, SourceLocation,
};
use std::{
    borrow::Cow, ops::Add, str::Chars,
    iter::FusedIterator,
};
use rustc_hash::FxHashSet;
use smallvec::{smallvec, SmallVec};

/// DecodedStr represents text after decoding html entities.
/// SmallVec and Cow are used internally for less allocation.
#[derive(Debug)]
pub struct DecodedStr<'a>(SmallVec<[Cow<'a, str>; 1]>);

impl<'a> DecodedStr<'a> {
    pub fn is_all_whitespace(&self) -> bool {
        self.0.iter().all(|s| {
            !s.chars().any(non_whitespace)
        })
    }
    pub fn contains(&self, p: &[char]) -> bool {
        self.0.iter().any(|s| s.contains(p))
    }
    pub fn trim_leading_newline(&mut self) {
        if self.0.is_empty() {
            return;
        }
        let first = &self.0[0];
        let offset = if first.starts_with('\n') {
            1
        } else if first.starts_with("\r\n") {
            2
        } else {
            return;
        };
        self.0[0] = match first {
            Cow::Borrowed(s) => Cow::Borrowed(&s[offset..]),
            Cow::Owned(s) => Cow::Owned(s[offset..].to_owned()),
        };
    }
}

impl<'a> Add for DecodedStr<'a> {
    type Output = Self;
    fn add(mut self, rhs: Self) -> Self {
        self.0.extend(rhs.0.into_iter());
        self
    }
}

impl<'a> From<&'a str> for DecodedStr<'a> {
    fn from(decoded: &'a str) -> Self {
        debug_assert!(!decoded.is_empty());
        Self(smallvec![Cow::Borrowed(decoded)])
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

pub type EntityDecoder = fn(&str, bool) -> DecodedStr<'_>;

/// TokenizeOption defined a list of methods used in scanning
#[derive(Clone)]
pub struct TokenizeOption {
    delimiters: (String, String),
    get_text_mode: fn(&str) -> TextMode,
    decode_entities: EntityDecoder,
}

impl Default for TokenizeOption {
    fn default() -> Self {
        Self {
            delimiters: ("{{".into(), "}}".into()),
            get_text_mode: |_| TextMode::Data,
            decode_entities: |t, _| DecodedStr::from(t),
        }
    }
}

/// A tokenizer needs to implement this trait to know if it is_in_html_namespace.
/// A parser tells tokenizer the current namespace through the trait's method.
// Because parsing CDATA requires tokenizer to know the parser's state.
// The trait decouples parser state from tokenizer state.
// The logic is somewhat convoluted in that the parser must handle logic belonging to
// tokenizer. A parser can skip flagging namespace if need_flag_hint returns false.
// Alternative is wrap Parser in a RefCell to appease Rust borrow check
// minimal case https://play.rust-lang.org/?gist=c5cb2658afbebceacdfc6d387c72e1ab
// but it is either too hard to bypass brrwchk or using too many Rc/RefCell
// Another alternative in Servo's parser:
// https://github.com/servo/html5ever/blob/57eb334c0ffccc6f88d563419f0fbeef6ff5741c/html5ever/src/tokenizer/interface.rs#L98
pub trait FlagCDataNs {
    /// Sets the tokenizer's is_in_html_namespace flag for CDATA.
    /// NB: Parser should call this method if necessary. See trait comment for details.
    /// https://html.spec.whatwg.org/multipage/parsing.html#markup-declaration-open-state
    fn set_is_in_html(&mut self, flag: bool);
    /// hint the parser if flagging is needed. Hint must be conservative.
    /// False alarm is acceptable but miss detection is not.
    fn need_flag_hint(&self) -> bool;
}

/// This trait produces a compiler's current position and selects a range.
pub trait Locatable {
    /// Returns the tokenizer's current position in the source.
    fn current_position(&self) -> Position;
    fn last_position(&self) -> Position;
    /// Returns the tokenizer's source location from the start position.
    fn get_location_from(&self, start: Position) -> SourceLocation;
}

/// TextMode represents different text scanning strategy.
/// e.g. Scanning in script/textarea/div are different.
#[derive(PartialEq, Eq)]
pub enum TextMode {
    //         | Elements | Entities | End sign              | Inside of
    // DATA    | ✔        | ✔        | End tags of ancestors |
    // RCDATA  | ✘        | ✔        | End tag of the parent | <textarea>
    // RAWTEXT | ✘        | ✘        | End tag of the parent | <style>,<script>
    Data,
    RcData,
    RawText,
}

pub struct Tokenizer {
    option: TokenizeOption,
    delimiter_first_char: char,
}

// builder methods
impl Tokenizer {
    pub fn new(option: TokenizeOption) -> Self {
        let delimiters = &option.delimiters;
        let delimiter_first_char = delimiters
            .0
            .chars()
            .next()
            .expect("interpolation delimiter cannot be empty");
        Self {
            option,
            delimiter_first_char,
        }
    }
    pub fn scan<'a, E>(&self, source: &'a str, err_handle: E) -> Tokens<'a, E>
    where
        E: ErrorHandler,
    {
        Tokens {
            source,
            err_handle,
            position: Default::default(),
            last_pos: Default::default(),
            mode: TextMode::Data,
            option: self.option.clone(),
            last_start_tag_name: None,
            is_in_html_namespace: true,
            delimiter_first_char: self.delimiter_first_char,
        }
    }
}

pub struct Tokens<'a, E: ErrorHandler> {
    source: &'a str,
    err_handle: E,
    position: Position,
    last_pos: Position,
    mode: TextMode,
    pub option: TokenizeOption,
    // following fields are implementation details

    //  appropriate end tag token needs last start tag, if any
    // https://html.spec.whatwg.org/multipage/parsing.html#appropriate-end-tag-token
    last_start_tag_name: Option<&'a str>,
    // this flag is for handling CDATA in non HTML namespace.
    is_in_html_namespace: bool,
    delimiter_first_char: char,
}

// scanning methods
// NB: When storing self.source to a name, prefer using a ref.
// because Rust ownership can help us to prevent invalid state.
// e.g. `let src = self.source` causes a stale src after [`move_by`].
// while `let src= &self.source` forbids any src usage after a mut call.
impl<'a, C: ErrorHandler> Tokens<'a, C> {
    // https://html.spec.whatwg.org/multipage/parsing.html#data-state
    // NB: & is not handled here but instead in `decode_entities`
    fn scan_data(&mut self) -> Token<'a> {
        debug_assert!(self.mode == TextMode::Data);
        let d = self.delimiter_first_char;
        // process html entity & later
        let index = self.source.find(&['<', d][..]);
        // no tag or interpolation found
        if index.is_none() {
            return self.scan_text(self.source.len());
        }
        let i = index.unwrap();
        if i != 0 {
            return self.scan_text(i);
        }
        if self.source.starts_with(&self.option.delimiters.0) {
            return self.scan_interpolation();
        }
        self.scan_tag_open()
    }

    // produces an entity_decoded Text token.
    fn scan_text(&mut self, size: usize) -> Token<'a> {
        debug_assert!(matches!(self.mode, TextMode::Data | TextMode::RcData));
        let src = self.move_by(size);
        Token::Text(self.decode_text(src, false))
    }

    fn scan_interpolation(&mut self) -> Token<'a> {
        let delimiters = &self.option.delimiters;
        debug_assert!(self.source.starts_with(&delimiters.0));
        let index = self.source.find(&delimiters.1);
        if index.is_none() {
            let src = self.move_by(self.source.len());
            self.emit_error(ErrorKind::MissingInterpolationEnd);
            return Token::Interpolation(src);
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
            self.move_by(1);
            self.emit_error(ErrorKind::EofBeforeTagName);
            Token::from("<")
        } else if !source[1..].starts_with(ascii_alpha) {
            // we can indeed merge this standalone < char into surrounding text
            // but optimization for error is not worth the candle
            self.move_by(1);
            self.emit_error(ErrorKind::InvalidFirstCharacterOfTagName);
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
        // https://html.spec.whatwg.org/multipage/parsing.html#parsing-elements-that-contain-only-text
        // Parsing algorithms are always invoked in response to a start tag token.
        let parsing_algorithm = self.option.get_text_mode;
        self.mode = parsing_algorithm(tag.name);
        if self.mode != TextMode::Data {
            self.last_start_tag_name.replace(tag.name);
        }
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
        Tag {
            name,
            attributes,
            self_closing,
        }
    }
    // return attributes and if the tag is self closing
    // https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-name-state
    fn scan_attributes(&mut self) -> Vec<Attribute<'a>> {
        let mut attrs = vec![]; // TODO: size hint?
        let mut set = FxHashSet::default();
        loop {
            // TODO: forbid infinite loop
            self.skip_whitespace();
            if self.is_about_to_close_tag() {
                return attrs;
            }
            if self.did_skip_slash_in_tag() {
                continue;
            }
            let attr = self.scan_attribute();
            if set.contains(attr.name) {
                // new attribute must be removed from the token.
                // NB: original vue compiler does not remove it.
                self.emit_error(ErrorKind::DuplicateAttribute);
                continue;
            }
            set.insert(attr.name);
            attrs.push(attr);
        }
    }
    // https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-name-state
    fn scan_attribute(&mut self) -> Attribute<'a> {
        debug_assert!(!self.source.is_empty());
        let name = self.scan_attr_name();
        // 13.2.5.34 After attribute name state, ignore white spaces
        self.skip_whitespace();
        if self.is_about_to_close_tag()
            || self.did_skip_slash_in_tag()
            || !self.source.starts_with("=")
        {
            return Attribute { name, value: None };
        }
        self.move_by(1); // equal sign
        let value = self.scan_attr_value();
        Attribute { name, value }
    }
    fn is_about_to_close_tag(&self) -> bool {
        let source = &self.source; // must get fresh source
        source.is_empty() || source.starts_with("/>") || source.starts_with('>')
    }
    fn did_skip_slash_in_tag(&mut self) -> bool {
        debug_assert!(!self.source.is_empty());
        if self.source.starts_with('/') {
            self.move_by(1);
            self.emit_error(ErrorKind::UnexpectedSolidusInTag);
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
            let s = self.move_by(1);
            self.emit_error(ErrorKind::UnexpectedEqualsSignBeforeAttributeName);
            debug_assert!(s == "=");
            return s;
        }
        let count = self
            .source
            .chars()
            .take_while(|&c| semi_valid_attr_name(c))
            .count();
        let src = self.move_by(count);
        if src.contains(&['<', '"', '\''][..]) {
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
            return None;
        }
        if self.source.starts_with(&['"', '\''][..]) {
            let c = self.source.chars().next().unwrap();
            return Some(self.scan_quoted_attr_value(c));
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
        // https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-value-(quoted)-state
        if !self.is_about_to_close_tag()
            && !self.did_skip_slash_in_tag()
            && self.skip_whitespace() == 0
        {
            self.emit_error(ErrorKind::MissingWhitespaceBetweenAttributes);
        }
        self.decode_text(src, /*is_attr*/ true)
    }
    // https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(unquoted)-state
    fn scan_unquoted_attr_value(&mut self) -> Option<DecodedStr<'a>> {
        let val_len = self
            .source
            .chars()
            .take_while(semi_valid_unquoted_attr_value)
            .count();
        // unexpected EOF: <tag attr=
        if val_len == 0 {
            // whitespace or > is precluded in scan_attribute
            // so empty value must implies EOF
            debug_assert!(self.source.is_empty());
            return None;
        }
        let src = self.move_by(val_len);
        if src.contains(&['"', '\'', '<', '=', '`'][..]) {
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
            debug_assert!(self.source.starts_with('>'));
            self.move_by(1);
            false
        }
    }
    // https://html.spec.whatwg.org/multipage/parsing.html#end-tag-open-state
    fn scan_end_tag_open(&mut self) -> Token<'a> {
        debug_assert!(self.source.starts_with("</"));
        let source = &self.source;
        if source.len() == 2 {
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
        // reset text mode after tag close
        self.mode = TextMode::Data;
        Token::EndTag(tag.name)
    }

    // https://html.spec.whatwg.org/multipage/parsing.html#markup-declaration-open-state
    fn scan_comment_and_like(&mut self) -> Token<'a> {
        // TODO: investigate https://github.com/jneem/teddy
        // for simd string pattern matching
        let s = &self.source;
        if s.starts_with("<!--") {
            self.scan_comment()
        } else if s.starts_with("<!DOCTYPE") {
            self.scan_bogus_comment()
        } else if s.starts_with("<![CDATA[") {
            if self.is_in_html_namespace {
                self.emit_error(ErrorKind::CDataInHtmlContent);
                self.scan_bogus_comment()
            } else {
                self.scan_cdata()
            }
        } else {
            self.emit_error(ErrorKind::IncorrectlyOpenedComment);
            self.scan_bogus_comment()
        }
    }
    // https://html.spec.whatwg.org/multipage/parsing.html#comment-start-state
    fn scan_comment(&mut self) -> Token<'a> {
        debug_assert!(self.source.starts_with("<!--"));
        let comment_text = self.scan_comment_text();
        if self.source.is_empty() {
            self.emit_error(ErrorKind::EofInComment);
        } else if self.source.starts_with("--!>") {
            self.emit_error(ErrorKind::IncorrectlyClosedComment);
            self.move_by(4);
        } else {
            debug_assert!(self.source.starts_with("-->"));
            self.move_by(3);
        };
        Token::Comment(comment_text)
    }
    fn scan_comment_text(&mut self) -> &'a str {
        debug_assert!(self.source.starts_with("<!--"));
        let comment_end = self.source.find("--!>").or_else(|| self.source.find("-->"));
        // NB: we take &str here since we will call move_by later
        let text = if let Some(end) = comment_end {
            debug_assert!(end >= 2, "first two chars must be <!");
            // <!---> or <!-->
            if end <= 3 {
                self.emit_error(ErrorKind::AbruptClosingOfEmptyComment);
                self.move_by(end);
                return "";
            }
            self.move_by(4); // skip <!--
            &self.source[..end - 4] // must be exclusive
        } else {
            // no closing comment
            self.move_by(4);
            self.source
        };

        // report nested comment error
        let mut s = text;
        while let Some(i) = s.find("<!--") {
            self.move_by(i + 4);
            // spec does not emit the NestedComment error when EOF is met
            // #13.2.5.49 Comment less-than sign bang dash dash state
            if !self.source.is_empty() {
                self.emit_error(ErrorKind::NestedComment);
            }
            s = &s[i + 4..];
        }
        // consume remaining comment
        if !s.is_empty() {
            self.move_by(s.len());
        }
        text
    }
    #[cold]
    #[inline(never)]
    fn scan_bogus_comment(&mut self) -> Token<'a> {
        /* /^<(?:[\!\?]|\/[^a-z>])/i from Vue's parseBogusComment
        ^            // starts with
        <            // a < followed by
        (?:          // a non-capturing group of
         [\!\?]      // a char of ! or ?
         |           // or
         \/[^a-z>]   // a slash and non alpha or >
        )
        */
        let s = &self.source;
        debug_assert! {
            s.starts_with("<!") || s.starts_with("<?") ||
            (
                s.starts_with("</") &&
                s[2..].starts_with(|c| {
                    !matches!(c, 'a'..='z'|'A'..='Z'|'>')
                })
            )
        };
        let start = if s.starts_with("<?") { 1 } else { 2 };
        let text = if let Some(end) = s.find('>') {
            let t = &s[start..end];
            self.move_by(end + 1);
            t
        } else {
            let len = s.len();
            &self.move_by(len)[start..]
        };
        Token::Comment(text)
    }
    #[cold]
    #[inline(never)]
    fn scan_cdata(&mut self) -> Token<'a> {
        debug_assert!(self.source.starts_with("<![CDATA["));
        self.move_by(9);
        let i = self.source.find("]]>").unwrap_or(self.source.len());
        let text = if i > 0 { self.move_by(i) } else { "" };
        if self.source.is_empty() {
            self.emit_error(ErrorKind::EofInCdata);
        } else {
            debug_assert!(self.source.starts_with("]]>"));
            self.move_by(3);
        }
        // don't call scan_text since CDATA decodes nothing
        Token::from(text)
    }

    // https://html.spec.whatwg.org/multipage/parsing.html#rawtext-state
    fn scan_rawtext(&mut self) -> Token<'a> {
        debug_assert!(self.mode == TextMode::RawText);
        let end = self.find_appropriate_end();
        // NOTE: rawtext decodes no entity. Don't call scan_text
        let src = self.move_by(end);
        self.mode = TextMode::Data;
        Token::from(src)
    }

    fn scan_rcdata(&mut self) -> Token<'a> {
        debug_assert!(self.mode == TextMode::RcData);
        let delimiter = &self.option.delimiters.0;
        if self.source.starts_with(delimiter) {
            return self.scan_interpolation();
        }
        let end = self.find_appropriate_end();
        let interpolation_start = self.source.find(delimiter).unwrap_or(end);
        if interpolation_start < end {
            return self.scan_text(interpolation_start);
        }
        let ret = self.scan_text(end);
        self.mode = TextMode::Data;
        ret
    }

    /// find first </{last_start_tag_name}
    fn find_appropriate_end(&self) -> usize {
        debug_assert!(self.last_start_tag_name.is_some());
        let tag_name = self
            .last_start_tag_name
            .expect("RAWTEXT/RCDATA must appear inside a tag");
        let len = tag_name.len();
        let source = self.source; // no mut self, need no &&str
        for (i, _) in source.match_indices("</") {
            //  match point     non letter separator
            //      ￬   </  style ￬
            let e = i + 2 + len + 1;
            // emit text without error per spec
            if e > source.len() {
                break;
            }
            // https://html.spec.whatwg.org/multipage/parsing.html#rawtext-end-tag-name-state
            let is_appropriate_end = source[i + 2..e].eq_ignore_ascii_case(tag_name);
            let terminated = !source[e..].starts_with(is_valid_name_char);
            if is_appropriate_end && terminated {
                // found!
                return i;
            }
        }
        source.len()
    }
}

// utility methods
impl<'a, C: ErrorHandler> Tokens<'a, C> {
    fn emit_error(&self, error_kind: ErrorKind) {
        let start = self.current_position();
        let loc = self.get_location_from(start);
        let err = CompilationError::new(error_kind).with_location(loc);
        self.err_handle.on_error(err);
    }

    fn decode_text(&self, src: &'a str, is_attr: bool) -> DecodedStr<'a> {
        let decode = self.option.decode_entities;
        decode(src, is_attr)
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
        let len = idx.unwrap_or_else(|| self.source.len());
        if len != 0 {
            self.move_by(len);
        }
        len
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
    !c.is_ascii_whitespace()
}

// tag name should begin with [a-zA-Z]
// followed by chars except whitespace, / or >
fn scan_tag_name_length(mut chars: Chars<'_>) -> usize {
    let first_char = chars.next();
    debug_assert!(first_char.is_some());
    if !first_char.unwrap().is_ascii_alphabetic() {
        return 0;
    }
    let l = chars.take_while(|&c| is_valid_name_char(c)).count();
    l + 1
}

impl<'a, C: ErrorHandler> Iterator for Tokens<'a, C> {
    type Item = Token<'a>;
    // https://html.spec.whatwg.org/multipage/parsing.html#concept-frag-parse-context
    fn next(&mut self) -> Option<Self::Item> {
        if self.source.is_empty() {
            return None;
        }
        self.last_pos = self.current_position();
        Some(match self.mode {
            TextMode::Data => self.scan_data(),
            TextMode::RcData => self.scan_rcdata(),
            TextMode::RawText => self.scan_rawtext(),
        })
    }
}

// Parser requires Tokens always yield None when exhausted.
impl<'a, C: ErrorHandler> FusedIterator for Tokens<'a, C> {}

impl<'a, C: ErrorHandler> FlagCDataNs for Tokens<'a, C> {
    fn set_is_in_html(&mut self, in_html: bool) {
        self.is_in_html_namespace = in_html;
    }
    fn need_flag_hint(&self) -> bool {
        self.source.contains("<![CDATA[")
    }
}

impl <'a, C: ErrorHandler> Locatable for Tokens<'a, C> {
    fn current_position(&self) -> Position {
        self.position.clone()
    }
    fn last_position(&self) -> Position {
        debug_assert! {
            self.position.offset == 0 ||
            self.last_pos.offset < self.position.offset
        };
        self.last_pos.clone()
    }
    fn get_location_from(&self, start: Position) -> SourceLocation {
        let end = self.current_position();
        SourceLocation{start, end}
    }
}

pub trait TokenSource<'a>:
FusedIterator<Item=Token<'a>> + FlagCDataNs + Locatable {}
impl<'a, C> TokenSource<'a> for Tokens<'a, C>
where C: ErrorHandler {}

#[cfg(test)]
mod test {
    use super::*;
    #[derive(Clone)]
    struct TestCtx;
    impl ErrorHandler for TestCtx {}
    #[test]
    fn test() {
        let cases = [
            r#"{{test}}"#,
            r#"<a test="value">...</a>"#,
            r#"<a v-bind:['foo' + bar]="value">...</a>"#,
            r#"<tag =value />"#,
            r#"<a wrong-attr>=123 />"#,
            r#"<a></a < / attr attr=">" >"#,
            r#"<!-->"#,                      // abrupt closing
            r#"<!--->"#,                     // abrupt closing
            r#"<!---->"#,                    // ok
            r#"<!-- nested <!--> text -->"#, // ok
            r#"<textarea><div/></textareas>"#,
            r#"<textarea>{{test}}</textarea>"#,
            r#"<textarea>{{'</textarea>'}}</textarea>"#,
            r#"<style></style"#,
            r#"<style></styl"#,
            r#"<style></styles"#,
            r#"<style></style "#,
        ];
        let tokenizer = Tokenizer::new(TokenizeOption::default());
        let ctx = TestCtx;
        for &case in cases.iter() {
            let tokens = tokenizer.scan(case, ctx.clone());
            for t in tokens {
                println!("{:?}", t);
            }
        }
    }
}
