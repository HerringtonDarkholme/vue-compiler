use super::common::{serialize_yaml, get_compiler};
use compiler::scanner::TokenSource;
use compiler::compiler::TemplateCompiler;
use crate::meta_macro;
use vue_compiler_core as compiler;

pub fn base_scan(s: &str) -> impl TokenSource {
    get_compiler().scan(s)
}

fn assert_scan(case: &str) -> String {
    let val: Vec<_> = base_scan(case).collect();
    serialize_yaml(val)
}

meta_macro!(assert_scan);

#[test]
fn test_scan() {
    assert_scan![[
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
    ]];
}

#[test]
fn test_scan_raw_text() {
    assert_scan![[
        r#"<style></style"#,
        r#"<style></styl"#,
        r#"<style></styles"#,
        r#"<style></style "#,
        r#"<style></style>"#,
        r#"<style>abc</style>"#,
    ]];
}
#[test]
fn test_scan_rc_data() {
    assert_scan![[
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
    ]];
}
