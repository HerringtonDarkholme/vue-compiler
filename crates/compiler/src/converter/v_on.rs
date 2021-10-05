use super::{
    CoreDirConvRet, Directive, DirectiveConvertResult, DirectiveConverter, Element, ErrorHandler,
};
use crate::{
    error::CompilationErrorKind as ErrorKind,
    flags::RuntimeHelper,
    ir::{HandlerType, JsExpr as Js},
    parser::DirectiveArg,
    scanner::AttributeValue,
    util::{is_simple_identifier, rslint, VStr},
};

// this module process v-on without arg and with arg.
pub fn convert_v_on<'a>(
    dir: &mut Directive<'a>,
    _: &Element<'a>,
    eh: &dyn ErrorHandler,
) -> CoreDirConvRet<'a> {
    if let Some(error) = dir.check_empty_expr(ErrorKind::VOnNoExpression) {
        // no argument no expr, just return
        if dir.argument.is_none() {
            eh.on_error(error);
            return DirectiveConvertResult::Dropped;
        }
        // allow @click.stop like
        if dir.modifiers.is_empty() {
            eh.on_error(error);
        }
    }
    let Directive {
        expression,
        modifiers,
        argument,
        ..
    } = dir;
    let value = if let Some(arg) = argument {
        let event_name = match arg {
            DirectiveArg::Static(s) => Js::StrLit(*VStr::raw(s).be_handler()),
            DirectiveArg::Dynamic(s) => {
                let e = Js::simple(*s);
                Js::Call(RuntimeHelper::ToHandlerKey, vec![e])
            }
        };
        let exp = convert_v_on_expr(expression.take());
        let exp = add_modifiers(&event_name, exp, modifiers);
        Js::Props(vec![(event_name, exp)])
    } else {
        // bare v-on="" does not have mods
        let exp = expression
            .take()
            .expect("v-on with no expr nor arg should be dropped.");
        let exp = Js::simple(exp.content);
        Js::Call(RuntimeHelper::ToHandlers, vec![exp])
    };
    DirectiveConvertResult::Converted {
        value,
        runtime: Err(false),
    }
}

pub fn convert_v_on_expr(expr: Option<AttributeValue>) -> Js {
    let val = match expr {
        Some(val) => val.content,
        None => return Js::Src("() => {}"),
    };
    let handler_type = if is_member_expression(val) {
        HandlerType::MemberExpr
    } else if is_fn_exp(val) {
        HandlerType::FuncExpr
    } else {
        HandlerType::InlineStmt
    };
    Js::func(val, handler_type)
}

fn is_fn_exp(expr: VStr) -> bool {
    todo!()
}

// cache handlers so that it's always the same handler being passed down.
// this avoids unnecessary re-renders when users use inline handlers on
// components.
pub fn cache_handlers() {
    todo!()
}

pub fn add_modifiers<'a>(evt_name: &Js<'a>, expr: Js<'a>, mods: &[&'a str]) -> Js<'a> {
    todo!()
}

pub fn is_member_expression(expr: VStr) -> bool {
    // TODO: looks like pattern can also work?
    if !expr.raw.starts_with(char::is_alphabetic) {
        return false;
    }
    is_simple_identifier(expr) || rslint::is_member_expression(&expr)
}

pub const V_ON: DirectiveConverter = ("on", convert_v_on);
