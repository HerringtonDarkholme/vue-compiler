use crate::core::{runtime_helper::RuntimeHelper, PreambleHelper};

use super::{
    super::error::{CompilationError as Error, CompilationErrorKind as ErrorKind},
    super::parser::DirectiveArg,
    super::util::VStr,
    CoreDirConvRet, Directive, DirectiveConvertResult, DirectiveConverter, Element, ErrorHandler,
    JsExpr as Js,
};

// this module process v-on without arg and with arg.
pub fn convert_v_on<'a>(
    dir: Directive<'a>,
    _: &Element<'a>,
    eh: &dyn ErrorHandler,
) -> CoreDirConvRet<'a> {
    if let Some(error) = dir.check_empty_expr(ErrorKind::VOnNoExpression) {
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
            DirectiveArg::Static(s) => Js::StrLit(*VStr::raw(s).to_handler_key()),
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
        todo!()
    } else {
        todo!()
    };
    DirectiveConvertResult::Converted {
        value,
        need_runtime: false,
    }
}

pub const V_BIND: DirectiveConverter = ("on", convert_v_on);
