use compiler::converter::v_model::{
    convert_v_model as convert_v_model_core, convert_v_model_event,
};

use super::{CoreDirConvRet, Directive, DirectiveConverter, Element, ErrorHandler};
pub fn convert_v_model<'a>(
    dir: &mut Directive<'a>,
    e: &Element<'a>,
    eh: &dyn ErrorHandler,
) -> CoreDirConvRet<'a> {
    let mut converted = convert_v_model_core(dir, e, eh);
    convert_v_model_event(&mut converted);
    converted
}

pub const V_MODEL: DirectiveConverter = ("model", convert_v_model);
