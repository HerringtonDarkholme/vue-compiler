use crate::core::flags::RuntimeHelper;

use super::{
    super::error::{CompilationError as Error, CompilationErrorKind as ErrorKind},
    super::parser::DirectiveArg,
    super::util::{non_whitespace, VStr},
    CoreDirConvRet, Directive, DirectiveConvertResult, DirectiveConverter, Element, ErrorHandler,
    JsExpr as Js,
};

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
    let (expr_val, err_loc) = if let Some(e) = expression.take() {
        (e.content, e.location)
    } else {
        (VStr::raw(""), head_loc.clone())
    };
    let expr = if !expr_val.contains(non_whitespace) {
        Js::Simple(expr_val)
    } else {
        let error = Error::new(ErrorKind::VBindNoExpression).with_location(err_loc);
        eh.on_error(error);
        if argument.is_none() {
            return DirectiveConvertResult::Dropped;
        } else {
            // <p :test> returns {test: ""}
            Js::Simple(VStr::raw(""))
        }
    };
    let value = if let Some(arg) = argument {
        let mut arg = match arg {
            DirectiveArg::Static(s) => Js::StrLit(VStr::raw(s)),
            DirectiveArg::Dynamic(s) => {
                let e = Js::Simple(VStr::raw(s));
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
