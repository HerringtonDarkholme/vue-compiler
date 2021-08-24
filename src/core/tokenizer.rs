use super::{
    Name, SourceLocation,
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
    /// for search optimization: only compare delimiters' first char
    cached_first_char: Option<char>,
}

impl TokenizerOption {
    fn delimiter_first_char(&mut self) -> char {
        if let Some(c) =  self.cached_first_char {
            return c
        }
        let c = self.delimiters.0.chars().next().expect("empty delimiter is invalid");
        self.cached_first_char.replace(c);
        c
    }
}

impl Default for TokenizerOption {
    fn default() -> Self {
        Self {
            delimiters: ("{{".into(), "}}".into()),
            cached_first_char: None,
        }
    }
}

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
    current: usize,
    option: TokenizerOption,
    state: TokenizerState,
    errors: Vec<CompilationError>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            current: 0,
            option: Default::default(),
            state: TokenizerState::RawText,
            errors: Vec::new(),
        }
    }
    pub fn with_option<'b>(&'b mut self, option: TokenizerOption) -> &'b mut Tokenizer<'a> {
        self.option = option;
        self
    }

    fn scan_open_tag(&mut self) -> Token<'a> {
        todo!()
    }

    fn scan_rawtext(&mut self) -> Token<'a> {
        let start = self.current;
        let d = self.option.delimiter_first_char();
        let index = self.source[start..]
            .find(|c| c == '<' || c == d);
        // no tag or interpolation found
        if index.is_none() {
            self.current = self.source.len();
            return Token::Text(&self.source[start..])
        }
        let i = index.unwrap();
        if i != start {
            self.current = i;
            return Token::Text(&self.source[start..i])
        }
        let next_source = &self.source[i..];
        if next_source.starts_with(&self.option.delimiters.0) {
            return self.scan_interpolation()
        }
        if next_source.starts_with("</") {
            return Token::LeftBracketSlash
        }
        if next_source.starts_with("<!") {
            return self.scan_comment_and_like()
        }
        self.state = TokenizerState::OpenTag;
        Token::LeftBracket
    }

    fn scan_interpolation(&mut self) -> Token<'a> {
        let delimiters = &self.option.delimiters;
        debug_assert!(self.current_str().starts_with(&delimiters.0));
        let start = self.current;
        let index =  self.current_str().find(&delimiters.1);
        if index.is_none() {
            self.emit_error(CompilationErrorKind::MissingInterpolationEnd);
            self.current = self.source.len();
            return Token::Interpolation(&self.source[start..])
        }
        let index = index.unwrap();
        self.current = index + 1;
        Token::Interpolation(&self.source[start..=index])
    }

    fn scan_comment_and_like(&mut self) -> Token<'a> {
        let s = self.current_str();
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
        debug_assert!(self.current_str().starts_with("<!--"));
        let start = self.current;
        while let Some(end) = self.current_str().find("--") {
            let s = &self.source[end..];
            let offset = if s.starts_with("!>") {
                2
            } else if s.starts_with('>') {
                1
            } else {
                0
            };
            if offset > 0 {
                self.current = end + offset + 1;
                return Token::Comment(&self.source[start..self.current])
            }
        }
        self.emit_error(CompilationErrorKind::EofInComment);
        Token::Comment(self.current_str())
    }
    #[cold]
    fn scan_bogus_comment(&mut self) -> Token<'a> {
        todo!()
    }
    #[cold]
    fn scan_cdata(&mut self) -> Token<'a> {
        todo!()
    }
    fn scan_in_tag(&mut self) -> Token<'a> {
        todo!()
    }
    fn scan_in_attr(&mut self) -> Token<'a> {
        todo!()
    }
    fn scan_attr_equal(&mut self) -> Token<'a> {
        todo!()
    }

    fn emit_error(&mut self, error_kind: CompilationErrorKind) {
        let loc = self.current_location();
        let err = CompilationError::new(error_kind).with_location(loc);
        self.errors.push(err);
    }

    fn current_str(&self) -> &'a str {
        &self.source[self.current..]
    }
    fn current_location(&self) -> SourceLocation {
        todo!()
    }
}

fn get_line_count(s: &str) -> usize {
    s.lines().count()
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.source.len() {
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
