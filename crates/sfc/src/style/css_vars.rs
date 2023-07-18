use compiler::SFCInfo;
use compiler::{ExpressionProcessor, Js};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub const CSS_VARS_HELPER: &str = "useCssVars";

/** <script setup> already gets the calls injected as part of the transform
 * this is only for single normal <script>
 */
pub fn gen_normal_script_css_vars_code(
    css_vars: &[&str],
    sfc_info: &SFCInfo,
    id: &str,
    is_prod: bool,
    is_ssr: bool,
) -> String {
    let vars_code = gen_css_vars_code(css_vars, sfc_info, id, is_prod, is_ssr);
    format!(
        r#"
import {{ {CSS_VARS_HELPER} as _{CSS_VARS_HELPER} }} from 'vue'
const __injectCSSVars__ = () => {{
  {vars_code}
}}
const __setup__ = __default__.setup
__default__.setup = __setup__
  ? (props, ctx) => {{ __injectCSSVars__();return __setup__(props, ctx) }}
  : __injectCSSVars__
"#,
    )
}

fn gen_css_vars_code(
    vars: &[&str],
    sfc_info: &SFCInfo,
    id: &str,
    is_prod: bool,
    is_ssr: bool,
) -> String {
    let vars_exp = gen_css_vars_from_list(vars, id, is_prod, is_ssr);
    let exp = Js::simple(&vars_exp[..]);
    let exp = ExpressionProcessor::transform_expr(exp, sfc_info);
    let transformed_str = transform_str(exp);
    format!("_{CSS_VARS_HELPER}(_ctx => ({transformed_str}))")
}

// TODO: unifiy this with code writer
fn transform_str(expr: Js) -> String {
    match expr {
        Js::Src(s) | Js::Param(s) => s.into(),
        Js::Num(n) => n.to_string(),
        Js::StrLit(mut l) => l.be_js_str().into_string(),
        Js::Simple(e, _) => e.into_string(),
        Js::Compound(v) => v.into_iter().map(transform_str).collect(),
        Js::Symbol(_) | Js::Props(_) | Js::Array(_) | Js::Call(_, _) => unreachable!(),
        Js::FuncSimple { .. } => unreachable!(),
        Js::FuncCompound { .. } => unreachable!(),
    }
}

fn gen_css_vars_from_list(vars: &[&str], id: &str, is_prod: bool, is_ssr: bool) -> String {
    let prefix = if is_ssr { "--" } else { "" };
    let mut var_strings = vec![];
    for var in vars {
        let var_name = format!("{prefix}{}", gen_var_name(id, var, is_prod));
        let var_string = format!("\"{var_name}\": ({var})");
        var_strings.push(var_string);
    }
    format!("{{\n{}\n}}", var_strings.join(",\n  "))
}

fn gen_var_name(id: &str, var: &str, is_prod: bool) -> String {
    if is_prod {
        let mut hasher = DefaultHasher::new();
        (id.to_owned() + var).hash(&mut hasher);
        hasher.finish().to_string()
    } else {
        let escaped = var
            .chars()
            .map(|c| match c {
                ' ' | '!' | '"' | '#' | '$' | '%' | '&' | '\'' | '(' | ')' | '*' | '+' | ','
                | '-' | '.' | '/' | ':' | ';' | '<' | '=' | '>' | '?' | '@' | '[' | '\\' | ']'
                | '^' | '`' | '{' | '|' | '}' | '~' => format!("\\{}", c),
                _ => c.to_string(),
            })
            .collect::<String>();
        format!("{}-{}", id, escaped)
    }
}
