use std::str::Chars;
use super::{
    Name, SourceLocation, Position,
    error::{CompilationError, CompilationErrorKind},
};

#[derive(PartialEq, Eq, Debug)]
pub enum Token<'a> {
    // bracket related token
    LeftBracket,
    RightBracket,
    LeftBracketSlash,
    SlashRightBracket,

    // in tag token
    TagName(Name<'a>),
    AttrName(Name<'a>),
    Equal,
    Value(Name<'a>),

    // content in raw text
    Text(&'a str),
    Interpolation(&'a str),
    Comment(&'a str),
}

// Note: TokenizerOption is not thread safe
// due to cached_first_char is shared
pub struct TokenizerOption {
    pub delimiters: (String, String),
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
        }
    }
}

#[derive(PartialEq, Eq)]
enum TokenizerState {
    /// match text between <tag> and </tag>
    RawText,
    /// matched left bracket <, expect TagName
    OpenTag,
    /// matched <tag, expect attr or right bracket >
    InTag,
    /// matched attribute name, expect = or right bracket >
    InAttr,
    /// matched attr=, expect attribute value
    AttrEqual,
}

pub struct Tokenizer<'a> {
    source: &'a str,
    position: Position,
    state: TokenizerState,
    option: TokenizerOption,
    errors: Vec<CompilationError>,
}

// builder methods
impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            position: Default::default(),
            state: TokenizerState::RawText,
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
impl<'a> Tokenizer<'a> {
    fn scan_rawtext(&mut self) -> Token<'a> {
        debug_assert!(self.state == TokenizerState::RawText);
        let d = self.option.delimiter_first_char();
        let index = self.source
            .find(|c| c == '<' || c == d);
        // no tag or interpolation found
        if index.is_none() {
            let src = self.move_by(self.source.len());
            return Token::Text(src)
        }
        let i = index.unwrap();
        if i != 0 {
            let src = self.move_by(i);
            return Token::Text(src)
        }
        let source = self.source;
        if source.starts_with(&self.option.delimiters.0) {
            return self.scan_interpolation()
        }
        if source.starts_with("</") {
            return Token::LeftBracketSlash
        }
        if source.starts_with("<!") {
            return self.scan_comment_and_like()
        }
        self.state = TokenizerState::OpenTag;
        self.move_by(1);
        Token::LeftBracket
    }

    fn scan_open_tag(&mut self) -> Token<'a> {
        debug_assert!(self.state == TokenizerState::OpenTag);
        let chars = self.source.chars();
        let l = scan_tag_name_length(chars);
        if l == 0 {
            self.emit_error(CompilationErrorKind::InvalidFirstCharacterOfTagName);
            self.state = TokenizerState::RawText;
            return self.scan_rawtext()
        }
        self.state = TokenizerState::InTag;
        let src = self.move_by(l);
        Token::TagName(src)
    }

    fn scan_in_tag(&mut self) -> Token<'a> {
        debug_assert!(self.state == TokenizerState::InTag);
        self.skip_whitespace();
        if self.source.starts_with("/>") {
            return Token::SlashRightBracket
        }
        if self.source.starts_with('>') {
            return Token::RightBracket
        }
        self.scan_attr_name()
    }
    fn scan_attr_name(&mut self) -> Token<'a> {
        debug_assert!(self.state == TokenizerState::InTag);
        debug_assert!(self.source.starts_with(is_valid_name_char));
        // state must transit to InAttr even if any error occurs
        self.state = TokenizerState::InAttr;
        if self.source.starts_with('=') {
            self.emit_error(CompilationErrorKind::MissingAttributeName);
            return Token::AttrName("")
        }
        let count = self.source.chars()
            .take_while(|&c| semi_valid_attr_name(c))
            .count();
        let src = self.move_by(count);
        if src.contains(|c| matches!(c, '<' | '"' | '\'')) {
            self.emit_error(CompilationErrorKind::UnexpectedCharacterInAttributeName);
        }
        Token::AttrName(src)
    }
    fn scan_in_attr(&mut self) -> Token<'a> {
        debug_assert!(self.state == TokenizerState::InAttr);
        todo!()
    }
    fn scan_attr_equal(&mut self) -> Token<'a> {
        debug_assert!(self.state == TokenizerState::AttrEqual);
        todo!()
    }

    fn scan_interpolation(&mut self) -> Token<'a> {
        let delimiters = &self.option.delimiters;
        debug_assert!(self.source.starts_with(&delimiters.0));
        let index =  self.source.find(&delimiters.1);
        if index.is_none() {
            self.emit_error(CompilationErrorKind::MissingInterpolationEnd);
            let src = self.move_by(self.source.len());
            return Token::Interpolation(src)
        }
        let step = index.unwrap() + delimiters.1.len();
        let src = self.move_by(step);
        Token::Interpolation(src)
    }

    fn scan_comment_and_like(&mut self) -> Token<'a> {
        let s = self.source;
        if s.starts_with("<!--") {
            self.scan_comment()
        } else if s.starts_with("<!DOCTYPE") {
            self.scan_bogus_comment()
        } else if s.starts_with("<!CDATA[") {
            self.scan_cdata()
        } else {
            self.emit_error(CompilationErrorKind::IncorrectlyOpenedComment);
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
        self.emit_error(CompilationErrorKind::EofInComment);
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
}

// util mtehods
impl<'a> Tokenizer<'a> {
    fn emit_error(&mut self, error_kind: CompilationErrorKind) {
        let loc = self.current_location();
        let err = CompilationError::new(error_kind).with_location(loc);
        self.errors.push(err);
    }

    fn current_location(&self) -> SourceLocation {
        todo!()
    }

    /// move tokenizer's interal position forward and return the range of movement
    /// tokenizer's line/column are also updated in the method
    /// note it only moves forward, not backward
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

    fn skip_whitespace(&mut self) {
        let idx = self.source.find(|c: char| !c.is_ascii_whitespace());
        if let Some(i) = idx {
            self.move_by(i);
        } else {
            self.move_by(self.source.len());
        }
    }
}

// `< ' "` are not valid but counted as semi valid
// to leniently recover from a parsing error
#[inline]
fn semi_valid_attr_name(c: char) -> bool {
    is_valid_name_char(c) && c != '='
}

#[inline]
fn is_valid_name_char(c: char) -> bool {
    !c.is_ascii_whitespace() && c != '/' && c != '>'
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
    fn next(&mut self) -> Option<Self::Item> {
        if self.source.is_empty() {
            return None
        }
        use TokenizerState as S;
        Some(match self.state {
            S::OpenTag => self.scan_open_tag(),
            S::RawText => self.scan_rawtext(),
            S::InTag => self.scan_in_tag(),
            S::InAttr => self.scan_in_attr(),
            S::AttrEqual => self.scan_attr_equal(),
        })
    }
}

#[cfg(test)]
mod test {
    fn test() {
        let cases = [
            r#"<a v-bind:['foo' + bar]="value">...</a>"#,
        ];
    }
}
