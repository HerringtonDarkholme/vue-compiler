use super::{
    BaseConvertInfo, BaseConverter, BaseIR, CompilationError, ConvertInfo, CoreConverter,
    Directive, Element,
};
use crate::{
    error::CompilationErrorKind as ErrorKind,
    flags::PatchFlag,
    ir::{ForNodeIR, ForParseResult, IRNode, JsExpr as Js},
    parser::ElementType,
    util::{find_dir_empty, find_prop, VStr},
};
use smallvec::SmallVec;

/// Pre converts v-if or v-for like structural dir
pub fn pre_convert_for<'a, T: ConvertInfo, C: CoreConverter<'a, T> + ?Sized>(
    bc: &C,
    elem: &mut Element<'a>,
) -> Option<Directive<'a>> {
    // convert v-for, v-if is converted elsewhere
    let dir = find_dir_empty(&mut *elem, "for")?;
    let b = dir.take();
    debug_assert!(find_dir_empty(&mut *elem, "for").is_none());
    check_template_v_for_key(bc, elem);
    Some(b)
}

pub fn convert_for<'a>(bc: &BaseConverter, d: Directive<'a>, n: BaseIR<'a>) -> BaseIR<'a> {
    // on empty v-for expr error
    if let Some(error) = d.check_empty_expr(ErrorKind::VForNoExpression) {
        bc.emit_error(error);
        return n;
    }
    let expr = d.expression.expect("v-for must have expression");
    let (source, parse_result) = match parse_for_expr(expr.content) {
        Some(parsed) => parsed,
        None => {
            let error = CompilationError::new(ErrorKind::VForMalformedExpression)
                .with_location(expr.location.clone());
            bc.emit_error(error);
            return n;
        }
    };
    let n = convert_memo_in_v_for(n, || parse_result.key.clone());
    IRNode::For(ForNodeIR {
        source,
        parse_result,
        child: Box::new(n),
        key: None,
        is_stable: false,
        fragment_flag: PatchFlag::empty(),
    })
}

type ParsedFor<'a> = (Js<'a>, ForParseResult<BaseConvertInfo<'a>>);

const PARENS: &[char] = &['(', ')'];
fn parse_for_expr(expr: VStr) -> Option<ParsedFor> {
    // split source and binding
    let (lhs, rhs) = expr
        .raw
        .split_once(" in ")
        .or_else(|| expr.raw.split_once(" of "))
        .map(|(l, r)| (l.trim_matches(PARENS), r.trim()))?;
    if rhs.is_empty() {
        return None;
    }
    // split iterator by ,
    let (val, key, idx) = split_v_for_iter(lhs);
    Some((
        Js::simple(rhs.trim()),
        ForParseResult {
            value: Js::Param(val),
            key: key.map(Js::Param),
            index: idx.map(Js::Param),
        },
    ))
}

const DESTRUCTING: &[char] = &['}', ']'];
fn split_v_for_iter(mut lhs: &str) -> (&str, Option<&str>, Option<&str>) {
    let mut split = SmallVec::<[&str; 3]>::new();
    while let Some((pre, post)) = lhs.rsplit_once(',') {
        if post.contains(DESTRUCTING) || split.len() == 2 {
            break;
        }
        lhs = pre;
        split.push(post.trim());
    }
    split.push(lhs.trim());
    split.reverse();
    match split.len() {
        2 => (split[0], Some(split[1]), None),
        3 => (split[0], Some(split[1]), Some(split[2])),
        _ => (split[0], None, None),
    }
}

fn convert_memo_in_v_for<'a, F>(n: BaseIR<'a>, get_key: F) -> BaseIR<'a>
where
    F: Fn() -> Option<Js<'a>>,
{
    use crate::ir::{
        CacheIR,
        CacheKind::{Memo, MemoInVFor},
    };
    use IRNode::CacheNode;
    let (expr, child) = match n {
        CacheNode(CacheIR {
            kind: Memo(expr),
            child,
        }) => (expr, child),
        _ => return n,
    };
    let kind = MemoInVFor {
        v_for_key: get_key(),
        expr,
    };
    CacheNode(CacheIR { kind, child })
}

// check <template v-for> key placement
fn check_template_v_for_key<'a, T: ConvertInfo, C: CoreConverter<'a, T> + ?Sized>(
    bc: &C,
    elem: &Element,
) {
    if elem.tag_type != ElementType::Template {
        return;
    }
    let first_wrong = elem
        .children
        .iter()
        .filter_map(|child| child.get_element())
        .find_map(|child| find_prop(child, "key"));
    if let Some(wrong) = first_wrong {
        let key_loc = wrong.get_ref().get_location().clone();
        let error =
            CompilationError::new(ErrorKind::VForTemplateKeyPlacement).with_location(key_loc);
        bc.emit_error(error);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cast;
    fn to_str(e: Js) -> &str {
        let v = cast!(e, Js::Param);
        v
    }
    fn check_equal(src: &str, expect: (&str, &str, Option<&str>, Option<&str>)) {
        let (src, ret) = parse_for_expr(VStr::raw(src)).expect("should parse");
        let source = cast!(src, Js::Simple);
        assert_eq!(source.into_string(), expect.0);
        assert_eq!(to_str(ret.value), expect.1);
        assert_eq!(ret.key.map(to_str), expect.2);
        assert_eq!(ret.index.map(to_str), expect.3);
    }
    #[test]
    fn test_parse_for_expr() {
        for &(src, expect) in &[
            ("a in [123]", ("[123]", "a", None, None)),
            ("   in [123]", ("[123]", "", None, None)),
            ("   a      in     [123]    ", ("[123]", "a", None, None)),
            ("a, b, c   in p ", ("p", "a", "b".into(), "c".into())),
            ("{a, b, c} in p ", ("p", "{a, b, c}", None, None)),
            ("{a, b}, c in p ", ("p", "{a, b}", "c".into(), None)),
            ("[a,] , b in p ", ("p", "[a,]", "b".into(), None)),
            ("a,b,c,d,e in p ", ("p", "a,b,c", "d".into(), "e".into())),
            ("(a,b) in p ", ("p", "a", "b".into(), None)),
            ("(a,b, c, d) in p ", ("p", "a,b", "c".into(), "d".into())),
            ("(,,,) in p ", ("p", ",", "".into(), "".into())),
            ("(,,) in p ", ("p", "", "".into(), "".into())),
        ] {
            check_equal(src, expect);
        }
    }

    #[test]
    fn test_parse_invalid_for() {
        for src in &["", "           in             "] {
            assert!(parse_for_expr(VStr::raw(src)).is_none());
        }
    }

    fn test_v_for_memo() {
        // TODO
    }
}
