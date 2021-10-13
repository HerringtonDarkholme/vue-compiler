use super::{
    CoreDirConvRet, Directive, DirectiveConverter, Element, ErrorHandler, DirectiveConvertResult,
    JsExpr as Js,
};
use compiler::error::{CompilationError, CompilationErrorKind};
use compiler::flags::RuntimeHelper;
use crate::extension::DomError;

pub fn convert_v_text<'a>(
    dir: &mut Directive<'a>,
    e: &Element<'a>,
    eh: &dyn ErrorHandler,
) -> CoreDirConvRet<'a> {
    let error_kind = CompilationErrorKind::ExtendPoint(Box::new(DomError::VTextNoExpression));
    if let Some(err) = dir.check_empty_expr(error_kind) {
        eh.on_error(err);
        return DirectiveConvertResult::Dropped;
    }
    if !e.children.is_empty() {
        let error = CompilationError::extended(DomError::VTextWithChildren)
            .with_location(dir.location.clone());
        eh.on_error(error);
    }
    let exp = dir
        .expression
        .take()
        .expect("should not be empty after check")
        .content;
    let args = vec![Js::simple(exp)];
    let prop = (
        Js::str_lit("textContent"),
        Js::Call(RuntimeHelper::TO_DISPLAY_STRING, args),
    );
    DirectiveConvertResult::Converted {
        value: Js::Props(vec![prop]),
        runtime: Err(false),
    }
}
pub const V_TEXT: DirectiveConverter = ("text", convert_v_text);
