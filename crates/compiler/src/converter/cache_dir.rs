// v-once / v-memo
use super::{BaseConversion, BaseIR, CoreConversion, Directive, Element};
use crate::ir::{IRNode, CacheIR, CacheKind, JsExpr as Js};
use crate::error::CompilationErrorKind as ErrorKind;
use crate::util::find_dir_empty;

pub fn pre_convert_memo<'a>(elem: &mut Element<'a>) -> Option<Directive<'a>> {
    let dir = find_dir_empty(&mut *elem, "memo")?;
    let b = dir.take();
    Some(b)
}

pub fn pre_convert_once<'a>(elem: &mut Element<'a>) -> Option<Directive<'a>> {
    let dir = find_dir_empty(&mut *elem, "once")?;
    let b = dir.take();
    // don't use cache if ancestor already in v-once/v-memo
    let children = elem.children.iter_mut().filter_map(|c| c.get_element_mut());
    for child in children {
        pre_convert_once(child);
    }
    Some(b)
}

pub fn convert_memo<'a>(bc: &BaseConversion, d: Directive<'a>, n: BaseIR<'a>) -> BaseIR<'a> {
    if let Some(error) = d.check_empty_expr(ErrorKind::VMemoNoExpression) {
        bc.emit_error(error);
        return n;
    }
    let mut n = n;
    // non-component sub tree should be turned into a block
    if let IRNode::VNodeCall(vnode) = &mut n {
        if !vnode.is_component {
            vnode.is_block = true;
        }
    }
    let expr_raw = d.expression.expect("v-memo should not be empty");
    let expr = Js::simple(expr_raw.content);
    IRNode::CacheNode(CacheIR {
        kind: CacheKind::Memo(expr),
        child: Box::new(n),
    })
}

pub fn convert_once<'a>(_bc: &BaseConversion, _: Directive<'a>, n: BaseIR<'a>) -> BaseIR<'a> {
    IRNode::CacheNode(CacheIR {
        kind: CacheKind::Once,
        child: Box::new(n),
    })
}

#[cfg(test)]
mod test {
    // fn test_memo() {
    //     let cases = [
    //         "<p v-memo='a'/>",
    //     ];
    // }
    // fn test_memo_in_v_if() {
    //     let cases = [
    //         "<p v-if='a' v-memo='a'/>",
    //     ];
    // }
    // fn test_memo_in_v_for() {
    //     let cases = [
    //         "<template v-for='a in b'><p v-memo='a'/></template>",
    //         "<p v-for='a in b' v-memo='a'/>",
    //     ];
    // }
    // fn test_once() {
    //     let cases = [
    //         "<template v-for='a in b'><p v-once/></template>",
    //         "<p v-for='a in b' v-once/>",
    //         "<p v-if='a' v-once/>",
    //         "<p v-once/>",
    //     ];
    // }
}
