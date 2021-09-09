use crate::core::{runtime_helper::RuntimeHelper, PreambleHelper};

use super::{
    error::CompilationErrorKind as ErrorKind, parser::DirectiveArg, tokenizer::AttributeValue,
    util::VStr, CoreDirConvRet, Directive, DirectiveConvertResult, DirectiveConverter, Element,
    ErrorHandler, JsExpr as Js,
};

// this module process v-on without arg and with arg.
pub fn convert_v_on<'a>(
    dir: Directive<'a>,
    _: &Element<'a>,
    eh: &dyn ErrorHandler,
) -> CoreDirConvRet<'a> {
    if let Some(error) = dir.check_empty_expr(ErrorKind::VOnNoExpression) {
        // no argument no expr, just return
        if dir.argument.is_none() {
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
            DirectiveArg::Static(s) => Js::StrLit(*VStr::raw(s).add_handler_key()),
            DirectiveArg::Dynamic(s) => {
                let e = Js::Simple(VStr::raw(s));
                Js::Compound(vec![
                    Js::Src(RuntimeHelper::TO_HANDLER_KEY.helper_str()),
                    Js::Src("("),
                    e,
                    Js::Src(")"),
                ])
            }
        };
        let exp = convert_v_on_expr(expression);
        let exp = add_modifiers(&event_name, exp, modifiers);
        Js::Props(vec![(event_name, exp)])
    } else {
        // bare v-on="" does not have mods
        let exp = expression.expect("v-on with no expr nor arg should be dropped.");
        let exp = Js::Simple(exp.content);
        Js::Compound(vec![
            Js::Src(RuntimeHelper::TO_HANDLERS.helper_str()),
            Js::Src("("),
            exp,
            Js::Src(")"),
        ])
    };
    DirectiveConvertResult::Converted {
        value,
        need_runtime: false,
    }
}

pub fn convert_v_on_expr(expr: Option<AttributeValue>) -> Js {
    todo!()
}

pub fn add_modifiers<'a>(evt_name: &Js<'a>, expr: Js<'a>, mods: Vec<&'a str>) -> Js<'a> {
    todo!()
}

pub const V_ON: DirectiveConverter = ("on", convert_v_on);
