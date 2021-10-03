use crate::error::{CompilationError as Error, CompilationErrorKind as ErrorKind};
use crate::flags::RuntimeHelper;

use super::{
    CoreDirConvRet, Directive, DirectiveConvertResult, DirectiveConverter, Element, ErrorHandler,
    JsExpr as Js,
};
use crate::{
    parser::DirectiveArg,
    scanner::AttributeValue,
    util::{non_whitespace, VStr},
    SourceLocation,
};

/// Returns the expression string if it is non-empty, or the error location
pub fn get_non_empty_expr<'a>(
    expression: &mut Option<AttributeValue<'a>>,
    head_loc: &SourceLocation,
) -> (Option<VStr<'a>>, SourceLocation) {
    let (val, loc) = if let Some(e) = expression {
        (e.content, e.location.clone())
    } else {
        return (None, head_loc.clone());
    };
    if val.contains(non_whitespace) {
        (Some(val), loc)
    } else {
        (None, loc)
    }
}

// this module process v-bind without arg and with arg.
pub fn convert_v_bind<'a>(
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
    let (expr_val, err_loc) = get_non_empty_expr(expression, head_loc);
    let expr = match expr_val {
        Some(val) => Js::simple(val),
        None => {
            let error = Error::new(ErrorKind::VBindNoExpression).with_location(err_loc);
            eh.on_error(error);
            if argument.is_none() {
                return DirectiveConvertResult::Dropped;
            } else {
                // <p :test> returns {test: ""}
                Js::str_lit("")
            }
        }
    };
    let value = if let Some(arg) = argument {
        let mut arg = match arg {
            DirectiveArg::Static(s) => Js::str_lit(*s),
            DirectiveArg::Dynamic(s) => {
                let e = Js::simple(*s);
                Js::Compound(vec![Js::Src("("), e, Js::Src(") || ''")])
            }
        };
        // TODO: handle .attr, .prop, modifiers in DOM
        if modifiers.contains(&"camel") {
            arg = match arg {
                Js::StrLit(ref mut s) => {
                    s.camelize();
                    arg
                }
                a => Js::Call(RuntimeHelper::Camelize, vec![a]),
            }
        }
        Js::Props(vec![(arg, expr)])
    } else {
        expr
    };
    DirectiveConvertResult::Converted {
        value,
        runtime: Err(false),
    }
}

pub const V_BIND: DirectiveConverter = ("bind", convert_v_bind);

#[cfg(test)]
mod test {}
