use super::common::{serialize_yaml, get_errors};
use insta::assert_snapshot;

macro_rules! assert_yaml {
    ($val: expr, $expr: expr) => {
        let name = insta::_macro_support::AutoName;
        let val = serialize_yaml($val);
        assert_snapshot!(name, val, $expr);
    };
}

#[test]
fn test_scan() {
    let cases = [
        // r#"a {{ b "#,
        // r#"<![CDATA["#,
        // r#"{{}}"#,
        // r#"{{test}}"#,
        // r#"<a test="value">...</a>"#,
        // r#"<a v-bind:['foo' + bar]="value">...</a>"#,
        // r#"<tag =value />"#,
        // r#"<a =123 />"#,
        // r#"<a ==123 />"#,
        // r#"<a b="" />"#,
        // r#"<a == />"#,
        // r#"<a wrong-attr>=123 />"#,
        // r#"<a></a < / attr attr=">" >"#,
        // r#"<a attr="1123"#,              // unclosed quote
        // r#"<a attr=""#,                  // unclosed without val
        // r#"<!-->"#,                      // abrupt closing
        // r#"<!--->"#,                     // abrupt closing
        // r#"<!---->"#,                    // ok
        // r#"<!-- nested <!--> text -->"#, // ok
        // r#"<p v-err=232/>"#,
    ];
    for case in cases {
        let val: Vec<_> = get_errors(case);
        assert_yaml!(val, case);
    }
}

#[test]
fn test_abrupt_closing_of_comment() {
    let cases = [
        r#"<template><!--></template>"#,
        r#"<template><!---></template>"#,
        r#"<template><!----></template>"#,
    ];
    for case in cases {
        let val: Vec<_> = get_errors(case);
        assert_yaml!(val, case);
    }
}
