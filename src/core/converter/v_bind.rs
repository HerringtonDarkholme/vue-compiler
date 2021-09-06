use super::{Directive, DirectiveConvertResult, Element};

// this module process v-bind without arg and with arg.
pub fn convert_v_bind<'a>(dir: &Directive<'a>, elem: &Element<'a>) -> DirectiveConvertResult<'a> {
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
    DirectiveConvertResult {
        value,
        need_runtime: false,
    }
}
