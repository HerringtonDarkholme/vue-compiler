use crate::error::CompilationErrorKind as ErrorKind;
use crate::flags::RuntimeHelper;

use super::{
    CoreDirConvRet, Directive, DirectiveConvertResult, DirectiveConverter, Element, ErrorHandler,
    JsExpr as Js,
};
use crate::parser::DirectiveArg;

// this module process v-bind without arg and with arg.
pub fn convert_v_bind<'a>(
    dir: &mut Directive<'a>,
    _: &Element<'a>,
    eh: &dyn ErrorHandler,
) -> CoreDirConvRet<'a> {
    let expr = if let Some(error) = dir.check_empty_expr(ErrorKind::VBindNoExpression) {
        eh.on_error(error);
        if dir.argument.is_none() {
            return DirectiveConvertResult::Dropped;
        } else {
            // <p :test> returns {test: ""}
            Js::str_lit("")
        }
    } else {
        let expr = dir
            .expression
            .take()
            .expect("dir without value should be dropped");
        Js::simple(expr.content)
    };
    let Directive {
        modifiers,
        argument,
        head_loc,
        ..
    } = dir;
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
