// v-once / v-memo
use super::{BaseConverter, BaseIR, CoreConverter, Directive, find_dir, Element};
use crate::error::CompilationErrorKind as ErrorKind;

pub fn pre_convert_memo<'a>(elem: &mut Element<'a>) -> Option<Directive<'a>> {
    let dir = find_dir(&mut *elem, "memo")?;
    let b = dir.take();
    Some(b)
}

pub fn pre_convert_once<'a>(elem: &mut Element<'a>) -> Option<Directive<'a>> {
    let dir = find_dir(&mut *elem, "once")?;
    let b = dir.take();
    Some(b)
}

pub fn convert_memo<'a>(bc: &BaseConverter, d: Directive<'a>, n: BaseIR<'a>) -> BaseIR<'a> {
    if let Some(error) = d.check_empty_expr(ErrorKind::VMemoNoExpression) {
        bc.emit_error(error);
        return n;
    }
    todo!()
}

pub fn convert_once<'a>(bc: &BaseConverter, d: Directive<'a>, n: BaseIR<'a>) -> BaseIR<'a> {
    todo!()
}

#[cfg(test)]
mod test {
    fn test_memo() {
        let cases = [
            "<template v-for='a in b'><p v-memo='a'/></template>",
            "<p v-for='a in b' v-memo='a'/>",
            "<p v-if='a' v-memo='a'/>",
            "<p v-memo='a'/>",
        ];
    }
    fn test_once() {
        let cases = [
            "<template v-for='a in b'><p v-once/></template>",
            "<p v-for='a in b' v-once/>",
            "<p v-if='a' v-once/>",
            "<p v-once/>",
        ];
    }
}
