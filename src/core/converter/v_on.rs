use super::{Directive, DirectiveConvertResult, DirectiveConverter, Element, ErrorHandler};

// this module process v-on without arg and with arg.
pub fn convert_v_on<'a>(
    dir: Directive<'a>,
    _: &Element<'a>,
    _: &dyn ErrorHandler,
) -> DirectiveConvertResult<'a> {
    let Directive {
        expression,
        modifiers,
        argument,
        ..
    } = dir;
    let value = if let Some(arg) = argument {
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
