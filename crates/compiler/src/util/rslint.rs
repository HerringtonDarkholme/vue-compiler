use rslint_parser::{ast::Expr, parse_expr, AstNode, SyntaxNodeExt};

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

// copied from parse_expr
pub fn parse_fn_param(text: &str) -> Option<Expr> {
    todo!()
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
        assert!(parse_js_expr(" a + b ").is_some());
        assert!(parse_js_expr("a **** b").is_none());
        assert!(parse_js_expr("a; ddd;").is_none());
        assert!(parse_js_expr("if (a) {b} else {c}").is_none());
    }
}
