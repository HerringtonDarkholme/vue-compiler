// v-once / v-memo
use super::{BaseConverter, BaseIR, CoreConverter, Directive, find_dir, Element};
use crate::ir::{IRNode, CacheIR, CacheKind, JsExpr as Js};
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
    let mut n = n;
    // non-component sub tree should be turned into a block
    if let IRNode::VNodeCall(vnode) = &mut n {
        if !vnode.is_component {
            vnode.is_block = true;
        }
    }
    let expr = d.expression.expect("v-memo should not be empty");
    IRNode::CacheNode(CacheIR {
        kind: CacheKind::Memo {
            in_v_for: false,
            expr: Js::simple(expr.content),
        },
        child: Box::new(n),
    })
}

pub fn convert_once<'a>(bc: &BaseConverter, d: Directive<'a>, n: BaseIR<'a>) -> BaseIR<'a> {
    // TODO: don't use cache if ancestor already in v-once/v-memo
    IRNode::CacheNode(CacheIR {
        kind: CacheKind::Once,
        child: Box::new(n),
    })
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
