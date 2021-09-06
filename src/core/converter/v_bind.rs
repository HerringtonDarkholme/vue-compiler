use super::{
    super::error::{CompilationError as Error, CompilationErrorKind as ErrorKind},
    super::parser::DirectiveArg,
    Directive, DirectiveConvertResult, DirectiveConverter, Element, ErrorHandler,
    JsExpression as Js,
};

// this module process v-bind without arg and with arg.
pub fn convert_v_bind<'a>(
    dir: Directive<'a>,
    _: &Element<'a>,
    eh: &dyn ErrorHandler,
) -> DirectiveConvertResult<'a> {
    let Directive {
        expression,
        modifiers,
        argument,
        head_loc,
        ..
    } = dir;
    let (expr_val, err_loc) = if let Some(e) = expression {
        (e.content, e.location)
    } else {
        ("".into(), head_loc)
    };
    let expr = if !expr_val.is_all_whitespace() {
        Js::Simple(todo!())
    } else {
        let error = Error::new(ErrorKind::VBindNoExpression).with_location(err_loc);
        eh.on_error(error);
        if argument.is_none() {
            return DirectiveConvertResult::Dropped;
        } else {
            Js::Simple("")
        }
    };
    let value = if let Some(arg) = argument {
        let arg = match arg {
            DirectiveArg::Static(s) => Js::Lit(s),
            DirectiveArg::Dynamic(s) => {
                Js::Compound(vec![Js::Lit("("), Js::Simple(s), Js::Lit(") || \"\"")])
            }
        };
        // TODO: handle .attr, .prop, .camel modifiers
        Js::Props(vec![(arg, expr)])
    } else {
        expr
    };
    DirectiveConvertResult::Converted {
        value,
        need_runtime: false,
    }
}

pub const V_BIND: DirectiveConverter = ("bind", convert_v_bind);
