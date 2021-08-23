use super::Name;

#[derive(PartialEq, Eq, Debug)]
pub enum Token<'a> {
    LeftBracket,
    TagName(Name<'a>),
    AttrName(Name<'a>),
    Equal,
    Value(Name<'a>),
    RightBracket,
    RightSlashBracket,
    SlashRightBracket,
    Comment(&'a str),
    Text(&'a str),
    LeftInterpolation(&'a str),
    Interpolation(),
    RightInterpolation(&'a str),
}

pub enum WhitespaceStrategy {
    Preserve,
    Condense,
}

pub struct TokenizerOption {
    delimiters: (String, String),
    whitespace: WhitespaceStrategy,
}

impl Default for TokenizerOption {
    fn default() -> Self {
        Self {
            delimiters: ("{{".into(), "}}".into()),
            whitespace: WhitespaceStrategy::Condense,
        }
    }
}

pub struct Tokenizer<'a> {
    source: &'a str,
    current: usize,
    option: TokenizerOption,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            current: 0,
            option: Default::default(),
        }
    }
    pub fn with_option<'b>(&'b mut self, option: TokenizerOption) -> &'b mut Tokenizer<'a> {
        self.option = option;
        self
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!("TODO")
    }
}
