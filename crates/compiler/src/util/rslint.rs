use rslint_parser::{
    ast::{Expr, NameRef},
    parse_expr, AstNode, SyntaxKind, SyntaxNodeExt,
};

pub fn parse_js_expr(text: &str) -> Option<Expr> {
    use std::ops::Range;
    let parsed = parse_expr(text, 0);
    if !parsed.errors().is_empty() {
        return None;
    }
    // range should be equal after removing trailing trivia(comment/whitespace)
    // otherwise the text is not a single expression
    parsed.syntax().try_to().filter(|n: &Expr| {
        let r: Range<usize> = Range::from(n.syntax().trimmed_range());
        r.end - r.start == text.trim().len()
    })
}

fn walk_identifier<F>(root: Expr, mut func: F)
where
    F: FnMut(NameRef) -> bool,
{
    root.syntax().descendants_with(&mut |node| {
        if node.kind() != SyntaxKind::NAME_REF {
            return true;
        }
        // TODO: handle block declaration
        // TODO: handle fn param
        let name_ref = node.to::<NameRef>();
        func(name_ref)
    })
}

// copied from parse_expr
pub fn parse_fn_param(text: &str) -> Option<Expr> {
    todo!()
    // let (tokens, mut errors) = tokenize(text, file_id);
    // let tok_source = TokenSource::new(text, &tokens);
    // let mut parser = crate::Parser::new(tok_source, file_id, Syntax::default());
    // crate::syntax::expr::expr(&mut parser);
    // let (events, p_diags) = parser.finish();
    // errors.extend(p_diags);
    // let mut tree_sink = LosslessTreeSink::new(text, &tokens);
    // crate::process(&mut tree_sink, events, errors);
    // let (green, parse_errors) = tree_sink.finish();
    // Parse::new(green, parse_errors)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cast;
    use rslint_parser::ast::{AstNode, BinOp, IfStmt};

    #[test]
    #[should_panic]
    fn test_panic_wrong_cast() {
        let a = parse_expr("a + b", 0);
        let b = a.syntax().to::<IfStmt>();
    }
    #[test]
    fn test_no_panic() {
        let a = parse_js_expr("a + b").unwrap();
        let expr = cast!(a, Expr::BinExpr);
        let a = expr.lhs().unwrap();
        let b = expr.rhs().unwrap();
        assert_eq!(a.syntax().text(), "a");
        assert_eq!(expr.op().unwrap(), BinOp::Plus);
        assert_eq!(b.syntax().text(), "b");
    }

    #[test]
    fn test_syntax_range() {
        let s = "    a +     b";
        let a = parse_js_expr(s).unwrap();
        let expr = cast!(a, Expr::BinExpr);
        let a = expr.lhs().unwrap();
        let b = expr.rhs().unwrap();
        assert_eq!(&s[a.range()], "a");
        assert_eq!(expr.op().unwrap(), BinOp::Plus);
        assert_eq!(&s[b.range()], "b");
    }
    #[test]
    fn test_invalid_expr() {
        assert!(parse_js_expr("(a + b + c, d, e,f)").is_some());
        assert!(parse_js_expr("a..b").is_none());
        assert!(parse_js_expr("a // b").is_none());
        assert!(parse_js_expr("a b").is_none());
        assert!(parse_js_expr(" a + b ").is_some());
        assert!(parse_js_expr("a **** b").is_none());
        assert!(parse_js_expr("a; ddd;").is_none());
        assert!(parse_js_expr("if (a) {b} else {c}").is_none());
    }

    fn test_walk(s: &str) -> Vec<String> {
        let expr = parse_js_expr(s).unwrap();
        let mut ret = vec![];
        walk_identifier(expr, |name_ref| {
            ret.push(name_ref.text());
            true
        });
        ret
    }

    #[test]
    fn test_walk_identifier() {
        let cases = [
            ("a(b)", vec!["a", "b"]),
            ("a.call(b)", vec!["a", "b"]),
            ("a.b", vec!["a"]),
            ("a || b", vec!["a", "b"]),
            ("a + b(c.d)", vec!["a", "b", "c"]),
            ("a ? b : c", vec!["a", "b", "c"]),
            ("a(b + 1, {c: d})", vec!["a", "b", "d"]),
            ("a, a, a", vec!["a", "a", "a"]),
            ("() => {let a = 123}", vec![]),
            ("() => {let {a} = b;}", vec!["b"]),
            ("true, false, null, this", vec![]),
        ];
        for (src, expect) in cases {
            assert_eq!(test_walk(src), expect);
        }
    }
}
