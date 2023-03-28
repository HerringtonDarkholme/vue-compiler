use compiler::BindingMetadata;

pub const CSS_VARS_HELPER: &str = "useCssVars";

/** <script setup> already gets the calls injected as part of the transform
 * this is only for single normal <script>
 */
pub fn gen_normal_script_css_vars_code(
    css_vars: &[&str],
    bindings: &BindingMetadata,
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

fn gen_css_vars_code(vars: &[&str], bindings: &BindingMetadata, id: &str, is_prod: bool) -> String {
    let vars_exp = gen_css_vars_from_list(vars, id, is_prod);
    todo!()
}

fn gen_css_vars_from_list(vars: &[&str], id: &str, is_prod: bool) -> String {
    todo!()
}

/*

  const varsExp = genCssVarsFromList(vars, id, isProd)
  const exp = createSimpleExpression(varsExp, false)
  const context = createTransformContext(createRoot([]), {
    prefixIdentifiers: true,
    inline: true,
    bindingMetadata: bindings.__isScriptSetup === false ? undefined : bindings
  })
  const transformed = processExpression(exp, context)
  const transformedString =
    transformed.type === NodeTypes.SIMPLE_EXPRESSION
      ? transformed.content
      : transformed.children
          .map(c => {
            return typeof c === 'string'
              ? c
              : (c as SimpleExpressionNode).content
          })
          .join('')

  return `_${CSS_VARS_HELPER}(_ctx => (${transformedString}))`
*/
