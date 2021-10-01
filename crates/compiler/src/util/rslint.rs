use rslint_parser::{
    self as rl,
    ast::{self, Expr, NameRef, ParameterList},
    parse_expr, AstNode, SyntaxKind, SyntaxNodeExt,
};
use std::cell::RefCell;
use std::ops::Range;

fn is_sole_child<N: AstNode>(n: &N, expect_len: usize) -> bool {
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
    SyntaxKind::GETTER,
    SyntaxKind::SETTER,
];

// just allocate if complex expressions are used
// users should not abuse expression in template
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
        let len = self.bound_vars.len();
        let decls = node.stmts().filter_map(|s| s.syntax().try_to::<Decl>());
        let mut collect = |d: &rl::SyntaxNode| {
            collect_names(d, |n| {
                self.bound_vars.push(n.syntax().trimmed_text());
                false
            })
        };
        decls.for_each(|decl| match decl {
            Decl::VarDecl(v) => {
                v.declared().for_each(|d| collect(d.syntax()));
            }
            decl => collect(decl.syntax()),
        });
        self.bound_vars.len() - len
    }
    #[inline(never)]
    fn track_param(&mut self, node: &rl::SyntaxNode) -> usize {
        debug_assert!(FN_KINDS.contains(&node.kind()));
        let len = self.bound_vars.len();
        // arrow func has single param without parenthesis
        if node.kind() == SyntaxKind::ARROW_EXPR {
            let param = node.to::<ast::ArrowExpr>().params();
            if let Some(ast::ArrowExprParams::Name(n)) = param {
                self.bound_vars.push(n.syntax().trimmed_text());
            }
        }
        // function expression has name property
        else if node.kind() == SyntaxKind::FN_EXPR {
            let name = node.to::<ast::FnExpr>().name();
            if let Some(n) = name {
                self.bound_vars.push(n.syntax().trimmed_text());
            }
        }
        let list = node.children().find_map(|nd| nd.try_to::<ParameterList>());
        if let Some(list) = list {
            collect_names(list.syntax(), |nd| {
                self.bound_vars.push(nd.syntax().trimmed_text());
                false
            });
        }
        self.bound_vars.len() - len
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
    let parsed = if text.starts_with('(') {
        parse_param_impl(text, 0)
    } else {
        parse_param_normalized(text, 0)
    };
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
fn parse_param_normalized(text: &str, file_id: usize) -> rl::Parse<ParameterList> {
    use std::fmt::Write;
    STR_CACHE.with(|sc| {
        let mut s = sc.borrow_mut();
        s.clear();
        write!(s, "({})", text).unwrap();
        parse_param_impl(&*s, file_id)
    })
}

// copied from parse_expr
fn parse_param_impl(text: &str, file_id: usize) -> rl::Parse<ParameterList> {
    let (tokens, mut errors) = rl::tokenize(text, file_id);
    let tok_source = rl::TokenSource::new(text, &tokens);
    let mut tree_sink = rl::LosslessTreeSink::new(text, &tokens);

    // TODO: set is TS
    let syntax = rl::Syntax {
        file_kind: rl::FileKind::TypeScript,
        ..Default::default()
    };
    let mut parser = rl::Parser::new(tok_source, file_id, syntax);
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
    SyntaxKind::KEY_VALUE_PATTERN, // key value pattern requires special handle
    SyntaxKind::SINGLE_PATTERN,
];
fn collect_names<F>(node: &rl::SyntaxNode, mut f: F)
where
    F: FnMut(ast::Name) -> bool,
{
    node.descendants_with(&mut |d| collect_one_name(d, &mut f))
}
fn collect_one_name<F>(node: &rl::SyntaxNode, mut f: F) -> bool
where
    F: FnMut(ast::Name) -> bool,
{
    let kind = node.kind();
    if kind == SyntaxKind::NAME {
        let parent = match node.parent() {
            Some(prt) => prt,
            None => return f(node.to()),
        };
        // kv.name() also contains Name, we need skip
        if parent.kind() == SyntaxKind::KEY_VALUE_PATTERN {
            false
        } else {
            f(node.to())
        }
    } else {
        PATTERNS.contains(&kind)
    }
}

/// returns param and default argument's range in text
pub fn walk_param_and_default_arg<F>(list: ParameterList, mut f: F)
where
    F: FnMut(Range<usize>, bool),
{
    list.syntax().descendants_with(&mut |d| {
        if d.is::<Expr>() {
            f(Range::from(d.text_range()), false);
            false
        } else {
            collect_one_name(d, |name| {
                f(Range::from(name.range()), true);
                false
            })
        }
    })
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
            // object key shorthand
            // ("{a, b, c}", vec!["a", "b", "c"]), TODO
            // arrow
            ("() => {let a = 123}", vec![]),
            ("() => {let {a} = b;}", vec!["b"]),
            ("(c) => {let {a} = b;}", vec!["b"]),
            // nested
            ("(c) => { ((a) => {b})(); a; }", vec!["b", "a"]),
            // fn expr
            ("function (a) {}", vec![]),
            ("function test(a) {test; foo;}", vec!["foo"]),
            // method
            ("{test(a) {a; b}}", vec!["b"]),
            ("{test: a => {a; b}}", vec!["b"]),
            // getter, setter
            ("{get test(a) {a; b}}", vec!["b"]),
            ("{set test(a) {a; b}}", vec!["b"]),
            // keyword
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
        collect_names(expr.syntax(), |name| {
            ret.push(name.text());
            true
        });
        ret
    }

    #[test]
    fn test_walk_fn_param() {
        let cases = [
            ("a, b", vec!["a", "b"]),
            ("a = (b) => { var a = 123}", vec!["a"]),
            ("a=b", vec!["a"]),
            ("{a, b, c}", vec!["a", "b", "c"]),
            // ("{a=b}", vec!["a"]), // need https://github.com/rslint/rslint/issues/120
            ("[a, b, c]", vec!["a", "b", "c"]),
            // ts annotation
            ("a: A", vec!["a"]),
            // object destruct
            ("{a: b = c}", vec!["b"]),
            ("{a: b}", vec!["b"]),
            // array
            ("[a, b]", vec!["a", "b"]),
            ("[a=c, b]", vec!["a", "b"]),
        ];
        for (src, expect) in cases {
            assert_eq!(walk_param(src), expect);
        }
    }
}
