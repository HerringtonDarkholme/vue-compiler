use crate::flags::StaticLevel;
use crate::{
    error::{CompilationError as Error, CompilationErrorKind as ErrorKind},
    parser::DirectiveArg,
    util::{is_simple_identifier, rslint, VStr},
};

use super::{
    v_bind::get_non_empty_expr, CoreDirConvRet, Directive, DirectiveConvertResult,
    DirectiveConverter, Element, ErrorHandler, JsExpr as Js,
};
pub fn convert_v_model<'a>(
    dir: &mut Directive<'a>,
    _: &Element<'a>,
    eh: &dyn ErrorHandler,
) -> CoreDirConvRet<'a> {
    let Directive {
        expression,
        modifiers,
        argument,
        head_loc,
        ..
    } = dir;
    let (expr_val, loc) = get_non_empty_expr(expression, head_loc);
    let val = match expr_val {
        Some(val) => val,
        None => {
            let error = Error::new(ErrorKind::VModelNoExpression).with_location(loc);
            eh.on_error(error);
            return DirectiveConvertResult::Dropped;
        }
    };
    if !is_member_expression(val) {
        let error = Error::new(ErrorKind::VModelMalformedExpression).with_location(loc);
        eh.on_error(error);
        return DirectiveConvertResult::Dropped;
    }
    // TODO: add scope variable check

    let prop_name = argument
        .take()
        .map(|arg| match arg {
            DirectiveArg::Static(s) => Js::str_lit(s),
            DirectiveArg::Dynamic(d) => Js::simple(d),
        })
        .unwrap_or_else(|| Js::str_lit("modelValue"));
    let mut props = vec![(prop_name, Js::Simple(val, StaticLevel::NotStatic))];
    // TODO process modifiers
    if !modifiers.is_empty() {
        props.push(modifiers_ir());
    }
    DirectiveConvertResult::Converted {
        value: Js::Props(props),
        runtime: Err(false),
    }
}

fn is_member_expression(expr: VStr) -> bool {
    // TODO: looks like pattern can also work?
    is_simple_identifier(expr) || rslint::is_member_expression(&expr)
}

fn modifiers_ir<'a>() -> (Js<'a>, Js<'a>) {
    todo!()
}

pub const V_MODEL: DirectiveConverter = ("model", convert_v_model);
