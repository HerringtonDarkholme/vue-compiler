use rslint_parser::{
    self as rl,
    ast::{self, Expr, NameRef, ParameterList},
    parse_expr, AstNode, SyntaxKind, SyntaxNodeExt,
};
use std::cell::RefCell;

fn is_sole_child<N: AstNode>(n: &N, expect_len: usize) -> bool {
    use std::ops::Range;
    let r: Range<usize> = Range::from(n.syntax().trimmed_range());
    r.end - r.start == expect_len
}

pub fn parse_js_expr(text: &str) -> Option<Expr> {
    let parsed = parse_expr(text, 0);
    if !parsed.errors().is_empty() {
        return None;
    }
    // range should be equal after removing trailing trivia(comment/whitespace)
    // otherwise the text is not a single expression
    parsed
        .syntax()
        .try_to()
        .filter(|n: &Expr| is_sole_child(n, text.trim().len()))
}

// difference from descendants_with:
// 1. has enter and exit to enable scop analysis
// 2. enter/exit never stop walking. 「止まるんじゃねぇぞ…💃」
pub trait SyntaxWalker<T> {
    fn enter(&mut self, n: &rl::SyntaxNode) -> T;
    fn exit(&mut self, n: &rl::SyntaxNode, i: T);
    fn walk(&mut self, node: &rl::SyntaxNode) {
        let t = self.enter(node);
        for child in node.children() {
            self.walk(&child);
        }
        self.exit(node, t);
    }
}

const FN_KINDS: &[SyntaxKind] = &[
    SyntaxKind::ARROW_EXPR,
    SyntaxKind::FN_DECL,
    SyntaxKind::FN_EXPR,
    SyntaxKind::METHOD,
];

// just allocate if complex expressions are used
// dont have time to optimize it :(
struct FreeVarWalker<F: FnMut(NameRef)> {
    func: F,
    bound_vars: Vec<rl::SyntaxText>,
}

impl<F> SyntaxWalker<usize> for FreeVarWalker<F>
where
    F: FnMut(NameRef),
{
    fn enter(&mut self, node: &rl::SyntaxNode) -> usize {
        use SyntaxKind as SK;
        let kind = node.kind();
        if kind == SK::NAME_REF {
            self.emit_name_ref(node);
            0
        } else if kind == SK::BLOCK_STMT {
            self.track_block_var(&node.to())
        } else if FN_KINDS.contains(&kind) {
            self.track_param(node)
        } else {
            0
        }
    }
    fn exit(&mut self, node: &rl::SyntaxNode, c: usize) {
        self.untrack_var(c);
    }
}
impl<F> FreeVarWalker<F>
where
    F: FnMut(NameRef),
{
    fn new(func: F) -> Self {
        Self {
            func,
            bound_vars: vec![],
        }
    }
    fn emit_name_ref(&mut self, name_ref: &rl::SyntaxNode) {
        if self.bound_vars.contains(&name_ref.trimmed_text()) {
            return;
        }
        (self.func)(name_ref.to());
    }
    #[inline(never)]
    fn track_block_var(&mut self, node: &ast::BlockStmt) -> usize {
        use ast::Decl;
        let l = self.bound_vars.len();
        let decls = node.stmts().filter_map(|s| s.syntax().try_to::<Decl>());
        for decl in decls {
            collect_name(decl.syntax(), |n| {
                self.bound_vars.push(n.syntax().trimmed_text());
                false
            })
        }
        self.bound_vars.len() - l
    }
    #[inline(never)]
    fn track_param(&mut self, n: &rl::SyntaxNode) -> usize {
        debug_assert!(FN_KINDS.contains(&n.kind()));
        todo!()
    }
    fn untrack_var(&mut self, c: usize) {
        debug_assert!(self.bound_vars.len() >= c);
        if c > 0 {
            let new_len = self.bound_vars.len() - c;
            self.bound_vars.truncate(new_len);
        }
    }
}

// only visit free variable, not bound ones like identifiers
// declared in the scope/func param list
pub fn walk_free_variables<F>(root: Expr, func: F)
where
    F: FnMut(NameRef),
{
    let mut walker = FreeVarWalker::new(func);
    walker.walk(root.syntax())
}

pub fn parse_fn_param(text: &str) -> Option<ParameterList> {
    let parsed = parse_param_impl(text);
    if !parsed.errors().is_empty() {
        return None;
    }
    parsed
        .syntax()
        .try_to()
        .filter(|p: &ParameterList| is_sole_child(p, text.len() + 2))
}
// TODO: thread local in Rust isn't that fast
thread_local! {
    static STR_CACHE: RefCell<String> = RefCell::new(String::with_capacity(50));
}
fn parse_param_impl(text: &str) -> rl::Parse<ParameterList> {
    use std::fmt::Write;
    STR_CACHE.with(|sc| {
        let mut s = sc.borrow_mut();
        s.clear();
        write!(s, "({})", text).unwrap();
        parse_param_real(&*s, 0)
    })
}

// copied from parse_expr
fn parse_param_real(text: &str, file_id: usize) -> rl::Parse<ParameterList> {
    let (tokens, mut errors) = rl::tokenize(text, file_id);
    let tok_source = rl::TokenSource::new(text, &tokens);
    let mut tree_sink = rl::LosslessTreeSink::new(text, &tokens);

    let mut parser = rl::Parser::new(tok_source, file_id, rl::Syntax::default());
    rl::syntax::decl::formal_parameters(&mut parser);
    let (events, p_diags) = parser.finish();
    errors.extend(p_diags);
    rl::process(&mut tree_sink, events, errors);
    let (green, parse_errors) = tree_sink.finish();
    rl::Parse::new(green, parse_errors)
}

const PATTERNS: &[SyntaxKind] = &[
    SyntaxKind::OBJECT_PATTERN,
    SyntaxKind::ARRAY_PATTERN,
    SyntaxKind::ASSIGN_PATTERN,
    SyntaxKind::REST_PATTERN,
    SyntaxKind::KEY_VALUE_PATTERN,
    SyntaxKind::SINGLE_PATTERN,
];
fn collect_name<F>(node: &rl::SyntaxNode, mut f: F)
where
    F: FnMut(ast::Name) -> bool,
{
    node.descendants_with(&mut |n| {
        if node.kind() == SyntaxKind::NAME {
            f(node.to())
        } else {
            PATTERNS.contains(&node.kind())
        }
    })
}

pub fn walk_fn_param<F>(list: ParameterList, f: F)
where
    F: FnMut(ast::Name) -> bool,
{
    collect_name(list.syntax(), f);
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cast;
    use rslint_parser::ast::{BinOp, IfStmt};

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

    fn walk_ident(s: &str) -> Vec<String> {
        let expr = parse_js_expr(s).unwrap();
        let mut ret = vec![];
        walk_free_variables(expr, |name_ref| {
            ret.push(name_ref.text());
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
            assert_eq!(walk_ident(src), expect);
        }
    }

    #[test]
    fn test_fn_param() {
        assert!(parse_fn_param("abc").is_some());
        assert!(parse_fn_param("a + b").is_none());
        assert!(parse_fn_param("a, b").is_some());
        assert!(parse_fn_param("{a, b = 123}").is_some());
        assert!(parse_fn_param("{a, ").is_none());
        assert!(parse_fn_param(" a={b: 3} ").is_some());
        assert!(parse_fn_param(" a={b: 3 ").is_none());
        assert!(parse_fn_param(" ").is_some());
    }

    fn walk_param(s: &str) -> Vec<String> {
        let expr = parse_fn_param(s).unwrap();
        let mut ret = vec![];
        walk_fn_param(expr, |name| {
            ret.push(name.text());
            true
        });
        ret
    }

    #[test]
    fn test_walk_fn_param() {
        let cases = [
            ("a, b", vec!["a", "b"]),
            ("a = (b) => {}", vec!["a"]),
            ("a=b", vec!["a"]),
            ("{a, b, c}", vec!["a", "b", "c"]),
            // ("{a=b}", vec!["a"]), // need https://github.com/rslint/rslint/issues/120
            ("[a, b, c]", vec!["a", "b", "c"]),
        ];
        for (src, expect) in cases {
            assert_eq!(walk_param(src), expect);
        }
    }
}
