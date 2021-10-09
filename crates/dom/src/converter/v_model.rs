use compiler::converter::v_model::{convert_v_model_event};

use super::{CoreDirConvRet, Directive, DirectiveConverter, Element, ErrorHandler};
pub fn convert_v_model<'a>(
    dir: &mut Directive<'a>,
    e: &Element<'a>,
    eh: &dyn ErrorHandler,
) -> CoreDirConvRet<'a> {
    convert_v_model_event(dir, e, eh)
}

pub const V_MODEL: DirectiveConverter = ("model", convert_v_model);
