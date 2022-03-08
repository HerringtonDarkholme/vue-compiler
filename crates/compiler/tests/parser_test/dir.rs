use super::base_parse;
use crate::common::serialize_yaml;
use compiler::parser;
use insta::assert_snapshot;
use vue_compiler_core as compiler;

macro_rules! test_dir {
    ($case: expr) => {
        let mut elem = mock_element($case);
        let name = insta::_macro_support::AutoName;
        let dir = elem.properties.pop().unwrap();
        let val = serialize_yaml(dir);
        assert_snapshot!(name, val, $case);
    };
}

#[test]
fn test_custom_dir() {
    let cases = [
        r#"<p v-test="tt"/>"#,     // test, N/A,
        r#"<p v-test.add="tt"/>"#, // test, N/A, add
    ];
    for case in cases {
        test_dir!(case);
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
        test_dir!(case);
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
        test_dir!(case);
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
        test_dir!(case);
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
        test_dir!(case);
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
        test_dir!(case);
    }
}

pub fn mock_element(s: &str) -> parser::Element {
    let mut m = base_parse(s).children;
    m.pop().unwrap().into_element()
}
