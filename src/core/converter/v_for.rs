use super::{
    super::error::CompilationErrorKind as ErrorKind, super::tokenizer::AttributeValue, find_dir,
    BaseConvertInfo, BaseConverter, BaseIR, ConvertInfo, CoreConverter, Directive, Element,
    ForNodeIR, ForParseResult, IRNode, JsExpr as Js,
};

/// Pre converts v-if or v-for like structural dir
/// The last argument is a continuation closure for base conversion.
// continuation is from continuation passing style.
// TODO: benchmark this monster function.
pub fn pre_convert_for<'a, T, C, K>(c: &C, mut e: Element<'a>, base_convert: K) -> IRNode<T>
where
    T: ConvertInfo,
    C: CoreConverter<'a, T> + ?Sized,
    K: FnOnce(Element<'a>) -> IRNode<T>,
{
    // convert v-for, v-if is converted elsewhere
    if let Some(dir) = find_dir(&mut e, "for") {
        let b = dir.take();
        debug_assert!(find_dir(&mut e, "for").is_none());
        let n = base_convert(e);
        c.convert_for(b, n)
    } else {
        base_convert(e)
    }
}

pub fn convert_for<'a>(bc: &BaseConverter, d: Directive<'a>, n: BaseIR<'a>) -> BaseIR<'a> {
    // on empty v-for expr error
    if let Some(error) = d.check_empty_expr(ErrorKind::VForNoExpression) {
        bc.emit_error(error);
        return n;
    }
    check_template_v_for_key();
    let expr = d.expression.expect("v-for must have expression");
    let (source, parse_result) = parse_for_expr(bc, expr);
    IRNode::For(ForNodeIR {
        source,
        parse_result,
        child: Box::new(n),
    })
}

fn parse_for_expr<'a>(
    bc: &BaseConverter,
    expr: AttributeValue<'a>,
) -> (Js<'a>, ForParseResult<BaseConvertInfo<'a>>) {
    todo!()
}

// check <template v-for> key placement
fn check_template_v_for_key() {}
