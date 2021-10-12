use super::{CoreDirConvRet, Directive, DirectiveConverter, Element, ErrorHandler};
pub fn convert_v_text<'a>(
    dir: &mut Directive<'a>,
    e: &Element<'a>,
    eh: &dyn ErrorHandler,
) -> CoreDirConvRet<'a> {
    todo!()
}
pub const V_TEXT: DirectiveConverter = ("text", convert_v_text);
