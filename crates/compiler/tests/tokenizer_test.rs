mod common;

use common::{serialize_yaml, SourceLocation, TestErrorHandler};
use compiler::tokenizer::{self, TextMode, TokenizeOption, Tokenizer};
use insta::{assert_snapshot, assert_yaml_snapshot};
use serde::Serialize;

#[derive(Serialize)]
pub struct Attribute {
    pub name: String,
    pub value: Option<AttributeValue>,
    pub name_loc: SourceLocation,
    pub location: SourceLocation,
}

#[derive(Serialize)]
pub struct AttributeValue {
    pub content: String,
    pub location: SourceLocation,
}

#[derive(Serialize)]
pub struct Tag {
    pub name: String,
    pub attributes: Vec<Attribute>,
    pub self_closing: bool,
}

#[derive(Serialize)]
pub enum Token {
    StartTag(Tag),
    EndTag(String),
    Text(String),
    Comment(String),
    Interpolation(String),
}

impl<'a> From<tokenizer::Token<'a>> for Token {
    fn from(t: tokenizer::Token<'a>) -> Self {
        use tokenizer::Token as T;
        match t {
            T::StartTag(s) => Self::StartTag(s.into()),
            T::EndTag(e) => Self::EndTag(e.into()),
            T::Text(t) => Self::Text(t.into_string()),
            T::Comment(c) => Self::Comment(c.into()),
            T::Interpolation(i) => Self::Interpolation(i.into()),
        }
    }
}

impl<'a> From<tokenizer::Tag<'a>> for Tag {
    fn from(t: tokenizer::Tag<'a>) -> Self {
        let attrs = t.attributes.into_iter().map(|a| Attribute {
            name: a.name.into(),
            value: a.value.map(|v| AttributeValue {
                content: v.content.into_string(),
                location: v.location.into(),
            }),
            name_loc: a.name_loc.into(),
            location: a.location.into(),
        });
        Tag {
            name: t.name.into(),
            attributes: attrs.collect(),
            self_closing: t.self_closing,
        }
    }
}

fn scan_with_opt(s: &str, opt: TokenizeOption) -> impl Iterator<Item = Token> + '_ {
    let tokenizer = Tokenizer::new(opt);
    let ctx = TestErrorHandler;
    tokenizer.scan(s, ctx).map(Token::from)
}

pub fn base_scan(s: &str) -> impl Iterator<Item = Token> + '_ {
    scan_with_opt(s, TokenizeOption::default())
}

fn assert_yaml<S: Serialize>(val: S, expr: &str) {
    let name = insta::_macro_support::AutoName;
    let val = serialize_yaml(val);
    assert_snapshot!(name, val, expr);
}

#[test]
fn should_scan() {
    let cases = [
        r#"<![CDATA["#,
        r#"{{}}"#,
        r#"{{test}}"#,
        r#"<a test="value">...</a>"#,
        r#"<a v-bind:['foo' + bar]="value">...</a>"#,
        r#"<tag =value />"#,
        r#"<a =123 />"#,
        r#"<a ==123 />"#,
        r#"<a b="" />"#,
        r#"<a == />"#,
        r#"<a wrong-attr>=123 />"#,
        r#"<a></a < / attr attr=">" >"#,
        r#"<a attr="1123"#,              // unclosed quote
        r#"<a attr=""#,                  // unclosed without val
        r#"<!-->"#,                      // abrupt closing
        r#"<!--->"#,                     // abrupt closing
        r#"<!---->"#,                    // ok
        r#"<!-- nested <!--> text -->"#, // ok
        r#"<p v-err=232/>"#,
    ];
    for case in cases {
        let val: Vec<_> = base_scan(case).collect();
        assert_yaml(val, case);
    }
}

#[test]
fn should_scan_raw_text() {
    let cases = [
        r#"<style></style"#,
        r#"<style></styl"#,
        r#"<style></styles"#,
        r#"<style></style "#,
        r#"<style></style>"#,
        r#"<style>abc</style>"#,
    ];
    for &case in cases.iter() {
        let opt = TokenizeOption {
            get_text_mode: |_| TextMode::RawText,
            ..Default::default()
        };
        let t: Vec<_> = scan_with_opt(case, opt).collect();
        assert_yaml(t, case);
    }
}
#[test]
fn should_scan_rc_data() {
    let cases = [
        r#"<textarea>   "#,
        r#"<textarea></textarea "#,
        r#"<textarea></textarea"#,
        r#"<textarea></textareas>"#,
        r#"<textarea><div/></textarea>"#,
        r#"<textarea><div/></textareas>"#,
        r#"<textarea>{{test}}</textarea>"#,
        r#"<textarea>{{'</textarea>'}}</textarea>"#,
        r#"<textarea>{{}}</textarea>"#,
        r#"<textarea>{{</textarea>"#,
        r#"<textarea>{{"#,
        r#"<textarea>{{ garbage  {{ }}</textarea>"#,
    ];
    for &case in cases.iter() {
        let opt = TokenizeOption {
            get_text_mode: |_| TextMode::RcData,
            ..Default::default()
        };
        let a: Vec<_> = scan_with_opt(case, opt).collect();
        assert_yaml(a, case);
    }
}
