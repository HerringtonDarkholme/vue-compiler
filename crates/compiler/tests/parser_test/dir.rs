use super::base_parse;
use crate::common::serialize_yaml;
use compiler::parser;
use crate::meta_macro;
use vue_compiler_core as compiler;

fn assert_dir(case: &str) -> String {
    let mut elem = mock_element(case);
    let dir = elem.properties.pop().unwrap();
    serialize_yaml(dir)
}

meta_macro!(assert_dir);

#[test]
fn test_custom_dir() {
    assert_dir![[
        r#"<p v-test="tt"/>"#,     // test, N/A,
        r#"<p v-test.add="tt"/>"#, // test, N/A, add
    ]];
}

#[test]
fn test_bind_dir() {
    assert_dir![[
        r#"<p :="tt"/>"#,           // bind, N/A,
        r#"<p :^_^="tt"/>"#,        // bind, ^_^
        r#"<p :^_^.prop="tt"/>"#,   // bind, ^_^, prop
        r#"<p :_:.prop="tt"/>"#,    // bind, _:, prop
        r#"<p :[a.b].stop="tt"/>"#, // bind, [a.b], stop
        r#"<p :[]="tt"/>"#,         // bind, nothing
        r#"<p :[t]err="tt"/>"#,     // bind, nothing,
        r#"<p v-🖖:🤘.🤙/>"#, // unicode, VUE in hand sign
    ]];
}

#[test]
fn test_prop_dir() {
    assert_dir![[
        r#"<p .stop="tt"/>"#,          // bind, stop, prop
        r#"<p .^-^.attr="tt" />"#,     // bind, ^-^, attr|prop
        r#"<p .[dynamic]="tt" />"#,    // bind, dynamic, prop
        r#"<p v-t.[dynamic]="tt" />"#, // t, N/A, [dynamic]
    ]];
}

#[test]
fn test_on_dir() {
    assert_dir![[
        r#"<p @="tt"/>"#,        // on, N/A,
        r#"<p @::="tt"/>"#,      // on , :: ,
        r#"<p @_@="tt"/>"#,      // on , _@ ,
        r#"<p @_@.stop="tt"/>"#, // on, _@, stop
        r#"<p @.stop="tt"/>"#,   // on, N/A, stop
    ]];
}

#[test]
fn test_slot_dir() {
    assert_dir![[
        r#"<p #="tt"/>"#,         // slot, default,
        r#"<p #:)="tt"/>"#,       // slot, :),
        r#"<p #@_@="tt"/>"#,      // slot, @_@,
        r#"<p #.-.="tt"/>"#,      // slot, .-.,
        r#"<p v-slot@.@="tt"/>"#, // slot@, N/A, @
    ]];
}
#[test]
fn test_dir_parse_error() {
    assert_dir![[
        r#"<p v-="tt"/>"#,       // ERROR,
        r#"<p v-:="tt"/>"#,      // ERROR,
        r#"<p v-.="tt"/>"#,      // ERROR,
        r#"<p v-a:.="tt"/>"#,    // ERROR
        r#"<p v-a:b.="tt"/>"#,   // ERROR
        r#"<p v-slot.-="tt"/>"#, // ERROR: slot, N/A, -
    ]];
}

pub fn mock_element(s: &str) -> parser::Element {
    let mut m = base_parse(s).children;
    m.pop().unwrap().into_element()
}
