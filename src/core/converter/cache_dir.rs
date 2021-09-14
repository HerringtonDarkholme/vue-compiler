// v-once / v-memo
use super::{BaseConverter as BC, BaseIR, Directive, Element};
use crate::core::util::find_dir;

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
