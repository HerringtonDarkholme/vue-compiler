use super::{
    find_dir, BaseConverter, BaseIR, ConvertInfo, CoreConverter, Directive, Element, IRNode,
};

/// Pre converts v-if or v-for like structural dir
/// The last argument is a continuation closure for base conversion.
// continuation is from continuation passing style.
// TODO: benchmark this monster function.
pub fn pre_convert_for<'a, T, C, K>(c: &C, mut e: Element<'a>, base_convert: K) -> IRNode<T>
where
    T: ConvertInfo,
    C: CoreConverter<'a, T> + ?Sized,
    K: FnOnce(Element<'a>) -> IRNode<T>,
{
    // convert v-for, v-if is converted elsewhere
    if let Some(dir) = find_dir(&mut e, "for") {
        let b = dir.take();
        let n = pre_convert_for(c, e, base_convert);
        c.convert_for(b, n)
    } else {
        base_convert(e)
    }
}

pub fn convert_for<'a>(bc: &BaseConverter, d: Directive<'a>, n: BaseIR<'a>) -> BaseIR<'a> {
    todo!()
}
