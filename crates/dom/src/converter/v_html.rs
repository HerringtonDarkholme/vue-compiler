use super::{CoreDirConvRet, Directive, DirectiveConverter, Element, ErrorHandler};
pub fn convert_v_html<'a>(
    dir: &mut Directive<'a>,
    e: &Element<'a>,
    eh: &dyn ErrorHandler,
) -> CoreDirConvRet<'a> {
    todo!()
}
pub const V_HTML: DirectiveConverter = ("html", convert_v_html);
