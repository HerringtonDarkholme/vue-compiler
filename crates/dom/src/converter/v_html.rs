use super::{
    CoreDirConvRet, Directive, DirectiveConverter, Element, ErrorHandler, DirectiveConvertResult,
    JsExpr as Js,
};
use compiler::error::{CompilationErrorKind, CompilationError};
use crate::extension::DomError;

pub fn convert_v_html<'a>(
    dir: &mut Directive<'a>,
    e: &Element<'a>,
    eh: &dyn ErrorHandler,
) -> CoreDirConvRet<'a> {
    let error_kind = CompilationErrorKind::extended(DomError::VHtmlNoExpression);
    if let Some(err) = dir.check_empty_expr(error_kind) {
        eh.on_error(err);
        return DirectiveConvertResult::Dropped;
    }
    if !e.children.is_empty() {
        let error = CompilationError::extended(DomError::VHtmlWithChildren)
            .with_location(dir.location.clone());
        eh.on_error(error);
        // TODO remove element children
    }
    let val = dir.expression.take().unwrap().content;
    let props = vec![(Js::str_lit("innerHTML"), Js::simple(val))];
    DirectiveConvertResult::Converted {
        value: Js::Props(props),
        runtime: Err(false),
    }
}
pub const V_HTML: DirectiveConverter = ("html", convert_v_html);
