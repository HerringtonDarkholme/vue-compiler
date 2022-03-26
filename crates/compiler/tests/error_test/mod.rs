use super::common::{serialize_yaml, get_errors};
use crate::meta_macro;

fn assert_error(case: &str) -> String {
    let val: Vec<_> = get_errors(case);
    serialize_yaml(val)
}
meta_macro!(assert_error);

#[test]
fn test_scan() {
    assert_error![[
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
    ]];
}

#[test]
fn test_abrupt_closing_of_comment() {
    assert_error![[
        r#"<template><!--></template>"#,
        r#"<template><!---></template>"#,
        r#"<template><!----></template>"#,
    ]];
}
