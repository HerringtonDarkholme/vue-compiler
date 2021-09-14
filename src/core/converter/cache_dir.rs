// v-once / v-memo
use super::{Directive, Element};
use crate::core::util::find_dir;

pub fn pre_convert_cache<'a>(elem: &mut Element<'a>) -> Option<Directive<'a>> {
    let dir = find_dir(&mut *elem, ["once", "memo"])?;
    let b = dir.take();
    Some(b)
}

pub fn convert_v_memo<'a>(elem: &mut Element<'a>) -> Option<Directive<'a>> {
    let dir = find_dir(&mut *elem, ["once", "memo"])?;
    let b = dir.take();
    Some(b)
}

pub fn convert_v_once<'a>(elem: &mut Element<'a>) -> Option<Directive<'a>> {
    let dir = find_dir(&mut *elem, ["once", "memo"])?;
    let b = dir.take();
    Some(b)
}
