use super::{
    CoreDirConvRet, Directive, DirectiveConverter, Element, ErrorHandler, DirectiveConvertResult,
    JsExpr as Js,
};
use compiler::error::CompilationErrorKind;
use crate::extension::{DomError, dom_helper};

pub fn convert_v_show<'a>(
    dir: &mut Directive<'a>,
    e: &Element<'a>,
    eh: &dyn ErrorHandler,
) -> CoreDirConvRet<'a> {
    let error_kind = CompilationErrorKind::ExtendPoint(Box::new(DomError::VShowNoExpression));
    if let Some(err) = dir.check_empty_expr(error_kind) {
        eh.on_error(err);
    }
    DirectiveConvertResult::Converted {
        value: Js::Props(vec![]),
        runtime: Ok(dom_helper::V_SHOW),
    }
}
pub const V_SHOW: DirectiveConverter = ("show", convert_v_show);
