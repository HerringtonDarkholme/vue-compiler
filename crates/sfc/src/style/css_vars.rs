use compiler::BindingMetadata;

pub const CSS_VARS_HELPER: &str = "useCssVars";

/** <script setup> already gets the calls injected as part of the transform
 * this is only for single normal <script>
 */
pub fn gen_normal_script_css_vars_code(
    css_vars: &[&str],
    bindings: BindingMetadata,
    id: &str,
    is_prod: bool,
) -> String {
    let vars_code = gen_css_vars_code(css_vars, bindings, id, is_prod);
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

fn gen_css_vars_code(vars: &[&str], bindings: BindingMetadata, id: &str, is_prod: bool) -> String {
    "TODO".into()
}
