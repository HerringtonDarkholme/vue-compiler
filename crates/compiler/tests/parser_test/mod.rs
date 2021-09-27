use super::common::{serialize_yaml, SourceLocation, TestErrorHandler};
use super::tokenizer_test::{base_scan, Attribute, AttributeValue};
use compiler::parser::{self, AstRoot, Element, ParseOption, Parser};
use insta::assert_snapshot;
use serde::Serialize;

#[derive(Serialize)]
pub enum DirectiveArg {
    // :static="val"
    Static(String),
    Dynamic(String), // :[dynamic]="val"
}
impl<'a> From<parser::DirectiveArg<'a>> for DirectiveArg {
    fn from(a: parser::DirectiveArg<'a>) -> Self {
        use parser::DirectiveArg as A;
        match a {
            A::Static(s) => Self::Static(s.into()),
            A::Dynamic(s) => Self::Dynamic(s.into()),
        }
    }
}

#[derive(Serialize)]
pub struct Directive {
    pub name: String,
    pub argument: Option<DirectiveArg>,
    pub modifiers: Vec<String>,
    pub expression: Option<AttributeValue>,
    pub head_loc: SourceLocation,
    pub location: SourceLocation,
}
#[derive(Serialize)]
pub enum ElemProp {
    Attr(Attribute),
    Dir(Directive),
}

impl<'a> From<parser::ElemProp<'a>> for ElemProp {
    fn from(p: parser::ElemProp<'a>) -> Self {
        use parser::ElemProp as P;
        match p {
            P::Attr(a) => Self::Attr(a.into()),
            P::Dir(d) => Self::Dir(d.into()),
        }
    }
}

impl<'a> From<parser::Directive<'a>> for Directive {
    fn from(d: parser::Directive<'a>) -> Self {
        Directive {
            name: d.name.into(),
            argument: d.argument.map(|a| a.into()),
            modifiers: d.modifiers.into_iter().map(String::from).collect(),
            expression: d.expression.map(|v| AttributeValue {
                content: v.content.into_string(),
                location: v.location.into(),
            }),
            head_loc: d.head_loc.into(),
            location: d.location.into(),
        }
    }
}

fn test_dir(case: &str) {
    let mut elem = mock_element(case);
    let name = insta::_macro_support::AutoName;
    let dir = elem.properties.pop().unwrap();
    let val = serialize_yaml(ElemProp::from(dir));
    assert_snapshot!(name, val, case);
}

#[test]
fn test_custom_dir() {
    let cases = [
        r#"<p v-test="tt"/>"#,     // test, N/A,
        r#"<p v-test.add="tt"/>"#, // test, N/A, add
    ];
    for case in cases {
        test_dir(case);
    }
}

#[test]
fn test_bind_dir() {
    let cases = [
        r#"<p :="tt"/>"#,           // bind, N/A,
        r#"<p :^_^="tt"/>"#,        // bind, ^_^
        r#"<p :^_^.prop="tt"/>"#,   // bind, ^_^, prop
        r#"<p :_:.prop="tt"/>"#,    // bind, _:, prop
        r#"<p :[a.b].stop="tt"/>"#, // bind, [a.b], stop
        r#"<p :[]="tt"/>"#,         // bind, nothing
        r#"<p :[t]err="tt"/>"#,     // bind, nothing,
        r#"<p v-🖖:🤘.🤙/>"#, // unicode, VUE in hand sign
    ];
    for case in cases {
        test_dir(case);
    }
}
#[test]
fn test_prop_dir() {
    let cases = [
        r#"<p .stop="tt"/>"#,          // bind, stop, prop
        r#"<p .^-^.attr="tt" />"#,     // bind, ^-^, attr|prop
        r#"<p .[dynamic]="tt" />"#,    // bind, dynamic, prop
        r#"<p v-t.[dynamic]="tt" />"#, // t, N/A, [dynamic]
    ];
    for case in cases {
        test_dir(case);
    }
}

#[test]
fn test_on_dir() {
    let cases = [
        r#"<p @="tt"/>"#,        // on, N/A,
        r#"<p @::="tt"/>"#,      // on , :: ,
        r#"<p @_@="tt"/>"#,      // on , _@ ,
        r#"<p @_@.stop="tt"/>"#, // on, _@, stop
        r#"<p @.stop="tt"/>"#,   // on, N/A, stop
    ];
    for case in cases {
        test_dir(case);
    }
}

#[test]
fn test_slot_dir() {
    let cases = [
        r#"<p #="tt"/>"#,         // slot, default,
        r#"<p #:)="tt"/>"#,       // slot, :),
        r#"<p #@_@="tt"/>"#,      // slot, @_@,
        r#"<p #.-.="tt"/>"#,      // slot, .-.,
        r#"<p v-slot@.@="tt"/>"#, // slot@, N/A, @
    ];
    for case in cases {
        test_dir(case);
    }
}
#[test]
fn test_dir_parse_error() {
    let cases = [
        r#"<p v-="tt"/>"#,       // ERROR,
        r#"<p v-:="tt"/>"#,      // ERROR,
        r#"<p v-.="tt"/>"#,      // ERROR,
        r#"<p v-a:.="tt"/>"#,    // ERROR
        r#"<p v-a:b.="tt"/>"#,   // ERROR
        r#"<p v-slot.-="tt"/>"#, // ERROR: slot, N/A, -
    ];
    for case in cases {
        test_dir(case);
    }
}

pub fn base_parse(s: &str) -> AstRoot {
    let tokens = base_scan(s);
    let parser = Parser::new(ParseOption::default());
    let eh = TestErrorHandler;
    parser.parse(tokens, eh)
}

pub fn mock_element(s: &str) -> Element {
    let mut m = base_parse(s).children;
    m.pop().unwrap().into_element()
}
