use super::common::{serialize_yaml, get_compiler};
use compiler::scanner::TokenSource;
use compiler::compiler::TemplateCompiler;
use insta::assert_snapshot;
use vue_compiler_core as compiler;

pub fn base_scan(s: &str) -> impl TokenSource {
    get_compiler().scan(s)
}

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
        assert_yaml!(val, case);
    }
}

#[test]
fn test_scan_raw_text() {
    let cases = [
        r#"<style></style"#,
        r#"<style></styl"#,
        r#"<style></styles"#,
        r#"<style></style "#,
        r#"<style></style>"#,
        r#"<style>abc</style>"#,
    ];
    for &case in cases.iter() {
        let t: Vec<_> = base_scan(case).collect();
        assert_yaml!(t, case);
    }
}
#[test]
fn test_scan_rc_data() {
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
        let a: Vec<_> = base_scan(case).collect();
        assert_yaml!(a, case);
    }
}
