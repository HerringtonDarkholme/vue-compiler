use super::{
    find_dir, AstNode, BaseConvertInfo, BaseConverter, BaseIR, CompilationError, CoreConverter,
    Directive, ForNodeIR, ForParseResult, IRNode, JsExpr as Js,
};
use crate::core::error::CompilationErrorKind as ErrorKind;
use crate::core::util::VStr;

/// Pre converts v-if or v-for like structural dir
// TODO: benchmark this because we did check element twice
pub fn pre_convert_for<'a>(node: &mut AstNode<'a>) -> Option<Directive<'a>> {
    let e = node.get_element_mut()?;
    // convert v-for, v-if is converted elsewhere
    let dir = find_dir(&mut *e, "for")?;
    let b = dir.take();
    debug_assert!(find_dir(&mut *e, "for").is_none());
    Some(b)
}

pub fn convert_for<'a>(bc: &BaseConverter, d: Directive<'a>, n: BaseIR<'a>) -> BaseIR<'a> {
    // on empty v-for expr error
    if let Some(error) = d.check_empty_expr(ErrorKind::VForNoExpression) {
        bc.emit_error(error);
        return n;
    }
    check_template_v_for_key();
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
    IRNode::For(ForNodeIR {
        source,
        parse_result,
        child: Box::new(n),
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
        simple_var(rhs.trim()),
        ForParseResult {
            value: simple_var(val),
            key: key.map(simple_var),
            index: idx.map(simple_var),
        },
    ))
}
fn simple_var(v: &str) -> Js {
    Js::Simple(VStr::raw(v))
}

const DESTRUCTING: &[char] = &['}', ']'];
fn split_v_for_iter(mut lhs: &str) -> (&str, Option<&str>, Option<&str>) {
    let mut split = Vec::with_capacity(3);
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

// check <template v-for> key placement
fn check_template_v_for_key() {}

#[cfg(test)]
mod test {
    use super::*;
    fn to_str(e: Js) -> &str {
        if let Js::Simple(v) = e {
            v.raw
        } else {
            panic!("invalid js expression");
        }
    }
    fn check_equal(src: &str, expect: (&str, &str, Option<&str>, Option<&str>)) {
        let (src, ret) = parse_for_expr(VStr::raw(src)).expect("should parse");
        assert_eq!(to_str(src), expect.0);
        assert_eq!(to_str(ret.value), expect.1);
        assert_eq!(ret.key.map(to_str), expect.2);
        assert_eq!(ret.index.map(to_str), expect.3);
    }
    #[test]
    fn test_parse_for_expr() {
        for (src, expect) in vec![
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
        for src in vec!["", "           in             "] {
            assert!(parse_for_expr(VStr::raw(src)).is_none());
        }
    }
}
